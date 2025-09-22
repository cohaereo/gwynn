mod directory;
mod filetype;
mod icons;
mod io;
mod patchlist;
mod ui;

use std::{
    fs::{write, File},
    io::Cursor,
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::Context;
use binrw::{io::BufReader, BinReaderExt};
use directory::{Directory, FileEntry};
use eframe::egui::{self, Color32, FontId, RichText};
use filetype::FileType;
use gwynn_mpk::EntryHeader;
use gwynn_texture::{converter::TextureConverter, TextureHeader};
use icons::{
    ICON_BOX, ICON_CROSS, ICON_ECLIPSE, ICON_FILE_QUESTION, ICON_FILM, ICON_FOLDER, ICON_IMAGE,
    ICON_PERSON_STANDING, ICON_SETTINGS, ICON_SHAPES,
};
use io::ReaderExt;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use ui::loading_circle::LoadingCircle;

#[macro_use]
extern crate tracing;

fn main() -> anyhow::Result<()> {
    let filter = tracing_subscriber::filter::Targets::new()
        .with_default(tracing::Level::INFO)
        .with_target("wgpu_hal", tracing::level_filters::LevelFilter::OFF)
        .with_target("wgpu", tracing::Level::WARN);

    // Build a new subscriber with the `fmt` layer using the `Targets`
    // filter we constructed above.
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(filter)
        .init();

    let native_options = eframe::NativeOptions {
        window_builder: Some(Box::new(|viewport| viewport.with_inner_size((1600., 900.)))),
        ..Default::default()
    };

    eframe::run_native(
        "Gwynn",
        native_options,
        Box::new(|cc| Ok(Box::new(GwynnApp::new(cc).unwrap()))),
    )?;
    Ok(())
}

struct GwynnApp {
    root_dir: Directory,
    texture_converter: TextureConverter,
    preview_texture: Option<(String, egui::TextureId)>,
    wgpu_renderstate: eframe::egui_wgpu::RenderState,
}

impl GwynnApp {
    fn new(cc: &eframe::CreationContext<'_>) -> anyhow::Result<Self> {
        let mut fonts = egui::FontDefinitions::default();
        fonts.font_data.insert(
            "Inter-Medium".into(),
            Arc::new(egui::FontData::from_static(include_bytes!(
                "../assets/fonts/Inter-Medium.ttf"
            ))),
        );

        fonts.font_data.insert(
            "lucide".into(),
            Arc::new(egui::FontData::from_static(include_bytes!(
                "../assets/fonts/lucide.ttf"
            ))),
        );

        let proportional = fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default();

        proportional.insert(0, "lucide".to_owned());
        proportional.insert(1, "Inter-Medium".into());

        cc.egui_ctx.set_fonts(fonts);

        cc.egui_ctx
            .style_mut(|s| s.override_font_id = Some(FontId::proportional(14.0)));
        cc.egui_ctx.set_theme(egui::ThemePreference::Dark);

        info!("Indexing package directory");
        let mut root_dir = Directory::new_root();

        let dir = PathBuf::from(std::env::args().nth(1).context("No dir given")?);
        for info_path in glob::glob(&dir.join("Patch*.mpkinfo").to_string_lossy())
            .unwrap()
            .flatten()
        {
            let mut f = BufReader::new(File::open(&info_path)?);
            loop {
                let entry = match f.read_le::<EntryHeader>() {
                    Ok(o) => o,
                    Err(e) => {
                        if e.is_eof() {
                            break;
                        }

                        return Err(e.into());
                    }
                };

                if entry.flags & 1 != 0 {
                    // This is a directory entry, skip it.
                    continue;
                }

                let path = Path::new(&entry.path);
                root_dir.add_file(
                    FileEntry {
                        data_file: info_path
                            .with_extension("mpk")
                            .to_string_lossy()
                            .to_string(),
                        ftype: FileType::guess_from_path(&entry.path).unwrap_or_default(),
                        path: entry.path.clone(),
                        name: path.file_name().unwrap().to_string_lossy().to_string(),
                        info: entry.clone(),
                    },
                    path,
                );
            }
        }

        Ok(Self {
            root_dir,
            texture_converter: TextureConverter::new()?,
            preview_texture: None,
            wgpu_renderstate: cc.wgpu_render_state.as_ref().unwrap().clone(),
        })
    }

    pub fn show_directory(&self, ui: &mut egui::Ui, dir: &Directory) {
        ui.collapsing(
            RichText::new(format!("{ICON_FOLDER} {}", dir.name)).color(Color32::LIGHT_BLUE),
            |ui| {
                for dir in dir.subdirectories.values() {
                    self.show_directory(ui, dir);
                }

                for file in dir.files.values() {
                    let color = if file.ftype == FileType::Unknown {
                        egui::Color32::GRAY
                    } else {
                        Color32::WHITE
                    };

                    if ui
                        .selectable_label(
                            false,
                            RichText::new(format!("{} {}", file.ftype.icon(), file.name))
                                .color(color),
                        )
                        .clicked()
                    {
                        if let Err(e) = self.extract_file(file) {
                            error!("Failed to convert texture: {e}");
                        }
                    }
                }
            },
        )
        .header_response
        .context_menu(|ui| {
            if ui.button("Extract all files").clicked() {
                if let Err(e) = self.extract_directory(dir) {
                    error!("Failed to extract directory: {e}");
                }
            }
        });
    }

    pub fn extract_directory(&self, dir: &Directory) -> anyhow::Result<()> {
        for file in dir.files.values() {
            if let Err(e) = self.extract_file(file) {
                error!("Failed to extract file '{}': {e}", file.name);
            }
        }

        for subdir in dir.subdirectories.values() {
            if let Err(e) = self.extract_directory(subdir) {
                error!("Failed to extract subdirectory '{}': {e}", subdir.name);
            }
        }

        Ok(())
    }

    fn extract_file(&self, file: &FileEntry) -> anyhow::Result<()> {
        if file.ftype == FileType::Texture {
            return self.extract_texture(file);
        }

        if file.path.ends_with("etsb") {
            return self.extract_etsb(file);
        }

        self.extract_file_raw(file)
    }

    fn extract_etsb(&self, file: &FileEntry) -> anyhow::Result<()> {
        let mut data = File::open(&file.data_file)?;

        let mut buf = vec![0; file.info.length as usize];
        data.seek_read(&mut buf, file.info.offset)?;

        info!(
            "Extracting file '{}' with compression {:?}",
            &file.info.path,
            gwynn_mpk::compression::CompressionType::guess_from_slice(&buf),
        );

        let decompressed = gwynn_mpk::compression::decompress(&mut buf)?.to_vec();
        let value: serde_json::Value = rmp_serde::from_slice(&decompressed)?;

        let out_file = Path::new("dump")
            .join(&file.path)
            .with_extension("etsb.json");
        std::fs::create_dir_all(out_file.parent().unwrap())?;
        std::fs::write(&out_file, serde_json::to_string_pretty(&value)?)?;

        Ok(())
    }

    fn extract_file_raw(&self, file: &FileEntry) -> anyhow::Result<()> {
        // self.wgpu_renderstate.
        let mut data = File::open(&file.data_file)?;

        let mut buf = vec![0; file.info.length as usize];
        data.seek_read(&mut buf, file.info.offset)?;

        info!(
            "Extracting file '{}' with compression {:?}",
            &file.info.path,
            gwynn_mpk::compression::CompressionType::guess_from_slice(&buf),
        );

        let decompressed = gwynn_mpk::compression::decompress(&mut buf)?.to_vec();

        let out_file = Path::new("dump").join(&file.path);
        std::fs::create_dir_all(out_file.parent().unwrap())?;
        std::fs::write(&out_file, &decompressed)?;

        Ok(())
    }

    fn extract_texture(&self, file: &FileEntry) -> anyhow::Result<()> {
        let mut data = File::open(&file.data_file)?;

        let mut buf = vec![0; file.info.length as usize];
        data.seek_read(&mut buf, file.info.offset)?;

        info!(
            "Extracting texture '{}' with compression {:?}",
            &file.info.path,
            gwynn_mpk::compression::CompressionType::guess_from_slice(&buf),
        );
        let mut decompressed = gwynn_mpk::compression::decompress(&mut buf)?.to_vec();

        let out_file = Path::new("dump").join(&file.path);
        std::fs::create_dir_all(out_file.parent().unwrap())?;
        std::fs::write(&out_file, &decompressed)?;

        let mut c = Cursor::new(&decompressed);
        let _unk0: u32 = c.read_le()?;

        let texture_header: TextureHeader = c.read_le()?;
        let mip = texture_header.mips.last().unwrap();
        info!(
            "{:?} {}x{} / {}x{} - type={:?} lod_group={:?}",
            texture_header.format,
            texture_header.width,
            texture_header.height,
            mip.width,
            mip.height,
            texture_header.texture_type,
            texture_header.lod_group,
        );

        let texture_data =
            gwynn_mpk::compression::decompress(&mut decompressed[mip.data_offset.pos as usize..])?;

        let out_file = Path::new("textures").join(&file.path);
        // std::fs::write(out_file.with_extension("raw"), &texture_data)?;

        let image_data = match self
            .texture_converter
            .convert(&texture_data, &texture_header)
        {
            Ok(o) => o,
            Err(e) => {
                error!("Failed to convert texture: {e}");
                return Ok(());
            }
        };

        if let Some(output_image) = image::ImageBuffer::<image::Rgba<u8>, _>::from_raw(
            mip.width as u32,
            mip.height as u32,
            &image_data[..],
        ) {
            std::fs::create_dir_all(out_file.parent().unwrap())?;
            output_image.save(out_file.with_extension("png"))?;
        }

        Ok(())
    }
}

impl eframe::App for GwynnApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::left("file_selector")
            .min_width(512.0)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        for d in self.root_dir.subdirectories.values() {
                            self.show_directory(ui, d);
                        }
                    });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Hey");
        });

        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            // ui.horizontal(|ui| {
            //     LoadingCircle::new(8.0).ui(ui);
            //     ui.label("Loading entries...");
            // });
        });

        ctx.request_repaint_after_secs(0.025);
    }
}

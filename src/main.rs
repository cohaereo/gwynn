pub mod directory;
pub mod filetype;
pub mod icons;
pub mod ui;

use std::{
    fs::File,
    io::Cursor,
    os::windows::fs::FileExt,
    path::{Path, PathBuf},
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
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use ui::loading_circle::LoadingCircle;

#[macro_use]
extern crate tracing;

fn main() -> anyhow::Result<()> {
    let filter = tracing_subscriber::filter::Targets::new()
        .with_default(tracing::Level::INFO)
        .with_target("wgpu", tracing::Level::WARN);

    // Build a new subscriber with the `fmt` layer using the `Targets`
    // filter we constructed above.
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(filter)
        .init();

    let mut native_options = eframe::NativeOptions::default();
    native_options.window_builder =
        Some(Box::new(|viewport| viewport.with_inner_size((1600., 900.))));

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
}

impl GwynnApp {
    fn new(cc: &eframe::CreationContext<'_>) -> anyhow::Result<Self> {
        let mut fonts = egui::FontDefinitions::default();
        fonts.font_data.insert(
            "Inter-Medium".into(),
            egui::FontData::from_static(include_bytes!("../assets/fonts/Inter-Medium.ttf")),
        );

        fonts.font_data.insert(
            "lucide".into(),
            egui::FontData::from_static(include_bytes!("../assets/fonts/lucide.ttf")),
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
        })
    }

    pub fn show_directory(&self, ui: &mut egui::Ui, dir: &Directory) {
        ui.collapsing(
            RichText::new(format!("{ICON_FOLDER} {}", dir.name)).color(Color32::WHITE),
            |ui| {
                for file in dir.files.values() {
                    let color = if file.ftype == FileType::Unknown {
                        egui::Color32::from_rgb(255, 160, 160)
                    } else {
                        Color32::WHITE
                    };

                    if ui
                        .selectable_label(
                            false,
                            RichText::new(format!(
                                "{} {}",
                                icon_for_filetype(file.ftype),
                                file.name
                            ))
                            .color(color),
                        )
                        .clicked()
                    {
                        if file.ftype == FileType::Texture {
                            if let Err(e) = self.extract_texture(file) {
                                error!("Failed to convert texture: {e}");
                            }
                        }
                    }
                }
            },
        );
    }

    fn extract_texture(&self, file: &FileEntry) -> anyhow::Result<()> {
        let data = File::open(&file.data_file)?;

        let mut buf = vec![0; file.info.length as usize];
        data.seek_read(&mut buf, file.info.offset)?;

        info!(
            "Compression {:?} - {}",
            gwynn_mpk::compression::CompressionType::guess_from_slice(&buf),
            &file.info.path
        );
        let mut decompressed = gwynn_mpk::compression::decompress(&mut buf)?.to_vec();

        let mut c = Cursor::new(&decompressed);

        let texture_header: TextureHeader = c.read_le()?;
        let mip = texture_header.mips.last().unwrap();
        info!(
            "{:?} {}x{} / {}x{}",
            texture_header.format,
            texture_header.width,
            texture_header.height,
            mip.width,
            mip.height,
        );

        let texture_data =
            gwynn_mpk::compression::decompress(&mut decompressed[mip.data_offset.pos as usize..])?;

        let out_file = Path::new("textures").join(&file.name);

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
            output_image.save(out_file.with_extension("png"))?;
        }

        Ok(())
    }
}

impl eframe::App for GwynnApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                for d in self.root_dir.subdirectories.values() {
                    self.show_directory(ui, d);
                }
            });
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

fn icon_for_filetype(ftype: FileType) -> char {
    match ftype {
        FileType::Unknown => ICON_FILE_QUESTION,
        FileType::Animation => ICON_PERSON_STANDING,
        FileType::Mp4 => ICON_FILM,
        FileType::Texture => ICON_IMAGE,
        FileType::Prefab => ICON_SHAPES,
        FileType::UnknownMessiah => ICON_CROSS,
        FileType::Model => ICON_BOX,
        FileType::Material => ICON_ECLIPSE,
        FileType::Json => ICON_SETTINGS,
    }
}

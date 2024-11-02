pub mod icons;
pub mod ui;

use eframe::egui::{self, Color32, FontId, Widget};
use icons::ICON_IMAGE;
use ui::loading_circle::LoadingCircle;

fn main() -> anyhow::Result<()> {
    let mut native_options = eframe::NativeOptions::default();
    native_options.window_builder =
        Some(Box::new(|viewport| viewport.with_inner_size((1600., 900.))));

    eframe::run_native(
        "Gwynn",
        native_options,
        Box::new(|cc| Ok(Box::new(GwynnApp::new(cc)))),
    )?;
    Ok(())
}

#[derive(Default)]
struct GwynnApp {}

impl GwynnApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
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

        Self::default()
    }
}

impl eframe::App for GwynnApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            LoadingCircle::new(64.0).ui(ui);
            ui.heading(format!("{} HELLO", ICON_IMAGE));
            ui.label("Hello world!");
        });

        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                LoadingCircle::new(8.0).ui(ui);
                ui.label("Loading 321 entries");
            });
        });

        ctx.request_repaint_after_secs(0.025);
    }
}

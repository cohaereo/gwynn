use std::sync::Arc;

use crate::ui::loading_circle::LoadingCircle;

pub struct GwynnApp {}

impl GwynnApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> anyhow::Result<Self> {
        {
            let ctx = cc.egui_ctx.clone();
            subsecond::register_handler(Arc::new(move || ctx.request_repaint()));
        }

        info!(
            "Using wgpu backend: {:?}",
            cc.wgpu_render_state
                .as_ref()
                .unwrap()
                .adapter
                .get_info()
                .backend
        );

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
            .style_mut(|s| s.override_font_id = Some(egui::FontId::proportional(14.0)));
        cc.egui_ctx.set_theme(egui::ThemePreference::Dark);

        info!("App initialized");

        Ok(Self {
            // preview_texture: None,
        })
    }

    fn draw(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        subsecond::call(|| {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.heading("Hey");
            });

            egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    LoadingCircle::new(8.0).ui(ui);
                    ui.label("Loading entries...");
                });
            });

            ctx.request_repaint_after_secs(0.025);
        })
    }
}

impl eframe::App for GwynnApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        subsecond::call(|| {
            self.draw(ctx, _frame);
        });
    }
}

use egui::{Color32, Response, RichText};

pub trait UiExt {
    fn chip(&mut self, text: &str, color: Color32, text_color: Color32) -> Response;
}

impl UiExt for egui::Ui {
    fn chip(&mut self, text: &str, color: Color32, text_color: Color32) -> Response {
        self.label(
            RichText::from(format!(" {text} "))
                .background_color(color)
                .color(text_color),
        )
    }
}

use std::f32::consts::PI;

use eframe::egui::{self, Color32, Vec2};

pub struct LoadingCircle {
    radius: f32,
}

impl LoadingCircle {
    /// The amount of circles
    const AMOUNT: usize = 2;
    /// The time it takes for one circle to go from min to max radius
    const SPEED: f32 = 1.5;
    /// The time delay between each circle
    const DELAY: f32 = Self::SPEED / Self::AMOUNT as f32;

    pub fn new(radius: f32) -> Self {
        Self { radius }
    }

    pub fn ui(&self, ui: &mut egui::Ui) {
        let (_, painter) =
            ui.allocate_painter(Vec2::splat(self.radius * 2.1), egui::Sense::hover());
        let center = painter.clip_rect().center();

        let brightness = 196;
        let base_time = ui.input(|i| i.time) as f32;
        for i in 0..Self::AMOUNT {
            let time = (base_time + (i as f32) * Self::DELAY) / Self::SPEED;
            let scale = ease_out_sine(time.fract());

            painter.circle_filled(
                center,
                self.radius * scale,
                Color32::from_rgba_unmultiplied(
                    brightness,
                    brightness,
                    brightness,
                    (((1. - scale) * 222.0) as u8),
                ),
            );
        }
    }
}

fn ease_out_sine(x: f32) -> f32 {
    ((x * PI) / 2.).sin()
}

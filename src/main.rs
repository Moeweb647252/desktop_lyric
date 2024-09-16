use std::{fs::read_to_string, str::FromStr, sync::Arc};

use egui::{ecolor::HexColor, mutex::RwLock, Color32, RichText};
use serde::{Deserialize, Serialize};

mod lyric;
mod serve;
#[derive(Serialize, Deserialize)]
struct Vec2 {
    x: f32,
    y: f32,
}

#[derive(Serialize, Deserialize)]
struct Config {
    text_color: String,
    background_color: String,
    text_size: f32,
    default_size: Vec2,
    passthrough: bool,
    lyric_dir: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            text_color: "#ffffff".to_string(),
            background_color: "#000000".to_string(),
            text_size: 24.0,
            default_size: Vec2 { x: 600.0, y: 100.0 },
            passthrough: false,
            lyric_dir: "./".to_owned(),
        }
    }
}

fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let config: Config =
        serde_yaml::from_str(read_to_string("./config.yaml").unwrap().as_str()).unwrap();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_decorations(false) // Hide the OS-specific "chrome" around the window
            .with_inner_size([config.default_size.x, config.default_size.y])
            .with_min_inner_size([10.0, 10.0])
            .with_resizable(true)
            .with_transparent(true)
            .with_mouse_passthrough(config.passthrough),
        renderer: eframe::Renderer::Wgpu,
        ..Default::default()
    };
    eframe::run_native(
        "Desktop Lyric", // unused title
        options,
        Box::new(|_cc| {
            Ok(Box::new(MyApp {
                current_lyric: Arc::new(RwLock::new("".to_owned())),
                text_color: HexColor::from_str(&config.text_color.leak())
                    .unwrap()
                    .color(),
                background_color: HexColor::from_str(&config.background_color.leak())
                    .unwrap()
                    .color(),
            }))
        }),
    )
}

#[derive(Default)]
struct MyApp {
    current_lyric: Arc<RwLock<String>>,
    text_color: Color32,
    background_color: Color32,
}

impl eframe::App for MyApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        egui::Rgba::TRANSPARENT.to_array()
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default()
            .frame(egui::containers::Frame {
                fill: self.background_color,
                ..Default::default()
            })
            .show(ctx, |ui| {
                ui.heading(
                    RichText::new(self.current_lyric.read().as_str()).color(self.text_color),
                );
            })
            .response;
    }
}

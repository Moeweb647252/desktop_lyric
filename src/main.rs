use std::{fs::read_to_string, str::FromStr, sync::Arc};

use eframe::egui::{
    self, ecolor::HexColor, mutex::RwLock, Color32, Label, RichText, Sense, ViewportCommand,
};
use log::info;
use serde::{Deserialize, Serialize};

mod lyric;
mod serve;
#[derive(Serialize, Deserialize, Clone)]
struct Vec2 {
    x: f32,
    y: f32,
}

#[derive(Serialize, Deserialize, Clone)]
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
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    };
    eframe::run_native(
        "Desktop Lyric", // unused title
        options,
        Box::new(|cc| {
            setup_custom_fonts(&cc.egui_ctx);
            Ok(Box::new(MyApp {
                current_lyric: serve::serve(config.clone()),
                text_color: HexColor::from_str(&config.text_color.leak())
                    .unwrap()
                    .color(),
                background_color: HexColor::from_str(&config.background_color.leak())
                    .unwrap()
                    .color(),
                text_size: config.text_size,
                prev_lyric: "No lyric".to_owned(),
                drag_mode: true,
            }))
        }),
    )
}

struct MyApp {
    current_lyric: Arc<RwLock<String>>,
    text_color: Color32,
    background_color: Color32,
    text_size: f32,
    prev_lyric: String,
    drag_mode: bool,
}

impl eframe::App for MyApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        egui::Rgba::TRANSPARENT.to_array()
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let resp = egui::CentralPanel::default()
            .frame(egui::containers::Frame {
                fill: self.background_color,
                ..Default::default()
            })
            .show(ctx, |ui| {
                let cur_lyric = { self.current_lyric.read().clone() };
                let resp = ui.add(
                    Label::new(
                        RichText::new(&cur_lyric)
                            .color(self.text_color)
                            .size(self.text_size),
                    )
                    .extend(),
                );
                if cur_lyric != self.prev_lyric {
                    ctx.send_viewport_cmd(ViewportCommand::InnerSize(egui::Vec2::new(
                        resp.rect.max.x,
                        resp.rect.max.y,
                    )));
                }
            })
            .response
            .interact(Sense::click_and_drag());
        if resp.clicked_by(egui::PointerButton::Secondary) {
            self.drag_mode = !self.drag_mode;
            println!("Drag mode: {}", self.drag_mode)
        }

        if self.drag_mode {
            if resp.drag_started() {
                ctx.send_viewport_cmd(ViewportCommand::StartDrag);
            }
        } else {
            if resp.dragged() {
                if resp.drag_delta().y > 0.0 {
                    self.text_size += 1.0;
                } else if resp.drag_delta().y < 0.0 {
                    self.text_size -= 1.0;
                }
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(1000 / 50));
        ctx.request_repaint();
    }
}

fn setup_custom_fonts(ctx: &egui::Context) {
    // Start with the default fonts (we will be adding to them rather than replacing them).
    let mut fonts = egui::FontDefinitions::default();

    // Install my own font (maybe supporting non-latin characters).
    // .ttf and .otf files supported.
    fonts.font_data.insert(
        "my_font".to_owned(),
        egui::FontData::from_static(include_bytes!(
            "/usr/share/fonts/wenquanyi/wqy-zenhei/wqy-zenhei.ttc"
        )),
    );

    // Put my font first (highest priority) for proportional text:
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "my_font".to_owned());

    // Put my font as last fallback for monospace:
    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .push("my_font".to_owned());

    // Tell egui to use these fonts:
    ctx.set_fonts(fonts);
}

use std::{
    fs::{read, read_to_string},
    str::FromStr,
    sync::Arc,
};

use eframe::egui::{
    self, ecolor::HexColor, mutex::RwLock, Color32, FontData, Label, Margin, RichText, Rounding,
    Sense, ViewportCommand,
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
    font_path: String,
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
            setup_custom_fonts(&cc.egui_ctx, &config.font_path);
            Ok(Box::new(MyApp {
                current_lyric: serve::serve(config.clone()),
                text_color: HexColor::from_str(&config.text_color.leak())
                    .unwrap()
                    .color(),
                background_color: HexColor::from_str(&config.background_color.leak())
                    .unwrap()
                    .color(),
                text_size: config.text_size,
                prev_size: (config.default_size.x, config.default_size.y),
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
    prev_size: (f32, f32),
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
                rounding: Rounding::same(10.0),
                inner_margin: Margin::symmetric(10.0, 5.0),
                ..Default::default()
            })
            .show(ctx, |ui| {
                let cur_lyric = { self.current_lyric.read().clone() };
                let resp = ui.add(
                    Label::new(
                        RichText::new(format!("{}", &cur_lyric))
                            .color(self.text_color)
                            .size(self.text_size),
                    )
                    .extend(),
                );
                if self.prev_size != (resp.rect.max.x, resp.rect.max.y) {
                    self.prev_size = (resp.rect.max.x, resp.rect.max.y);
                    ctx.send_viewport_cmd(ViewportCommand::InnerSize(egui::Vec2::new(
                        resp.rect.max.x + 20.0,
                        resp.rect.max.y + 10.0,
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
    fn raw_input_hook(&mut self, _ctx: &egui::Context, _raw_input: &mut egui::RawInput) {
        use egui::Event::*;
        for i in _raw_input.events.iter() {
            match i {
                MouseWheel {
                    unit,
                    delta,
                    modifiers,
                } => {
                    if delta.y > 0.0 {
                        self.text_size += 1.0;
                    } else if delta.y < 0.0 {
                        self.text_size -= 1.0;
                    }
                }
                _ => {}
            }
        }
    }
}

fn setup_custom_fonts(ctx: &egui::Context, font_path: &str) {
    // Start with the default fonts (we will be adding to them rather than replacing them).
    let mut fonts = egui::FontDefinitions::default();
    let fontdata = if let Ok(font_data) = std::fs::read(font_path) {
        // .ttf and .otf files supported.
        FontData::from_owned(font_data)
    } else {
        FontData::from_static(include_bytes!("../assets/SetoFont-1.ttf"))
    };
    // Install my own font (maybe supporting non-latin characters).
    // .ttf and .otf files supported.
    fonts.font_data.insert("my_font".to_owned(), fontdata);
    // Put my font first (highest priority) for proportional text:
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "my_font".to_owned());

    // Put my font as last fallback for monospace:
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .push("my_font".to_owned());

    // Tell egui to use these fonts:
    ctx.set_fonts(fonts);
}

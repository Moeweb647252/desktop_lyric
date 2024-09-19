use std::sync::mpsc::{sync_channel, SyncSender};
#[allow(unused_assignments, dead_code)]
use std::{fs::read_to_string, str::FromStr, sync::Arc};

use eframe::egui::{
    self, ecolor::HexColor, mutex::RwLock, CentralPanel, Color32, ComboBox, FontData, Label,
    Margin, RichText, Rounding, Sense, Slider, ViewportBuilder, ViewportCommand, ViewportId,
};
use log::{debug, info};
use mpris::Player;
use serde::{Deserialize, Serialize};
use serve::{serve, Event};
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
    player_name: String,
    fuzzy: bool,
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

    let (tx, rx) = sync_channel(64);
    let (handle, lock) = serve(config.clone(), config.player_name.clone(), rx, config.fuzzy);
    eframe::run_native(
        "Desktop Lyric", // unused title
        options,
        Box::new(move |cc| {
            setup_custom_fonts(&cc.egui_ctx, &config.font_path);
            Ok(Box::new(MyApp {
                current_lyric: lock,
                text_color: HexColor::from_str(&config.text_color.leak())
                    .unwrap()
                    .color(),
                background_color: HexColor::from_str(&config.background_color.leak())
                    .unwrap()
                    .color(),
                text_size: config.text_size,
                drag_mode: true,
                settings_viewport_id: None,
                show_settings: false,
                serve_task_handle: handle,
                players: Vec::new(),
                player: "".to_string(),
                event_sender: tx,
                fuzzy: config.fuzzy,
            }))
        }),
    )
}

struct MyApp {
    current_lyric: Arc<RwLock<String>>,
    text_color: Color32,
    background_color: Color32,
    text_size: f32,
    drag_mode: bool,
    settings_viewport_id: Option<ViewportId>,
    show_settings: bool,
    serve_task_handle: std::thread::JoinHandle<()>,
    players: Vec<Player>,
    player: String,
    event_sender: SyncSender<Event>,
    fuzzy: bool,
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
                let screen_rect = ctx.input(|v| v.screen_rect().max);
                if (resp.rect.max.x, resp.rect.max.y)
                    != (screen_rect.x.round() - 20.0, screen_rect.y.round() - 10.0)
                {
                    debug!(
                        "Container size: {:?}, Screen size: {:?}",
                        resp.rect.max, screen_rect
                    );
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
            info!("Drag mode: {}", self.drag_mode)
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
        if self.show_settings {
            let vp_id = if let Some(id) = self.settings_viewport_id {
                id
            } else {
                let id = ViewportId::from_hash_of("Settings");
                self.settings_viewport_id = Some(id);
                id
            };
            ctx.show_viewport_immediate(vp_id, ViewportBuilder::default(), |ctx, v| {
                ctx.input(|v| {
                    if v.viewport().close_requested() {
                        self.show_settings = false;
                    }
                });
                CentralPanel::default().show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Text Size");
                        ui.add(
                            Slider::new(&mut self.text_size, 10.0..=100.0)
                                .text_color(self.text_color),
                        );
                    });
                    if self.players.is_empty() {
                        self.players = mpris::PlayerFinder::new().unwrap().find_all().unwrap();
                    }

                    let mut p = self.player.clone();
                    ui.horizontal(|ui| {
                        ui.label("Player");
                        ComboBox::from_label("")
                            .selected_text(&self.player)
                            .show_ui(ui, |ui| {
                                for player in self.players.iter() {
                                    ui.selectable_value(
                                        &mut p,
                                        player.bus_name_player_name_part().to_owned(),
                                        player.bus_name_player_name_part(),
                                    );
                                }
                            });
                        ui.button("Refresh").clicked().then(|| {
                            self.players = mpris::PlayerFinder::new().unwrap().find_all().unwrap()
                        });
                    });
                    if p != self.player {
                        info!("Player changed to {}", p);
                        self.event_sender.send(Event::ChangePlayer(p.clone())).ok();
                        self.player = p;
                    }

                    ui.horizontal(|ui| {
                        ui.label("Fuzzy match");
                        ui.checkbox(&mut self.fuzzy, "")
                            .changed()
                            .then(|| self.event_sender.send(Event::ToggleFuzzy).ok());
                    })
                });
            });
        }
        std::thread::sleep(std::time::Duration::from_millis(1000 / 50));
        ctx.request_repaint();
    }
    fn raw_input_hook(&mut self, ctx: &egui::Context, raw_input: &mut egui::RawInput) {
        use egui::Event::*;
        for i in raw_input.events.iter() {
            match i {
                MouseWheel { delta, .. } => {
                    if delta.y > 0.0 {
                        self.text_size += 1.0;
                    } else if delta.y < 0.0 {
                        self.text_size -= 1.0;
                    }
                }
                Key {
                    key,
                    pressed,
                    modifiers,
                    ..
                } => {
                    if !*pressed {
                        if *modifiers == egui::Modifiers::NONE {
                            match key.name() {
                                "S" => {
                                    self.show_settings = true;
                                }
                                "P" => {
                                    println!("{:?}", ctx.input(|v| v.screen_rect));
                                }
                                _ => {}
                            }
                        }
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
        FontData::from_static(include_bytes!("../assets/XiaolaiSC-Regular.ttf"))
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

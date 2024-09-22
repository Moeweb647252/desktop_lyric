use std::{
    path::PathBuf,
    sync::{mpsc::SyncSender, Arc},
};

use eframe::{
    egui::{self, mutex::RwLock, ViewportId},
    App,
};
use mpris::Player;

use crate::{config::Config, serve::Event};

mod lyric;
mod run;
mod settings;

pub struct DesktopLyricApp {
    pub current_lyric: Arc<RwLock<String>>,
    pub drag_mode: bool,
    pub settings_viewport_id: Option<ViewportId>,
    pub show_settings: bool,
    pub players: Vec<Player>,
    pub event_sender: SyncSender<Event>,
    pub config: Config,
    pub config_path: PathBuf,
}

impl App for DesktopLyricApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        egui::Rgba::TRANSPARENT.to_array()
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.lyric_ui(&ctx);
        if self.show_settings {
            self.settings_ui(&ctx);
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
                        self.config.text_size += 1.0;
                    } else if delta.y < 0.0 {
                        self.config.text_size -= 1.0;
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

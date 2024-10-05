use std::fs::write;

use eframe::egui::{CentralPanel, ComboBox, Context, Slider, ViewportBuilder, ViewportId};
use log::info;

use crate::{font::setup_custom_fonts, serve::Event};

use super::DesktopLyricApp;

impl DesktopLyricApp {
    pub fn settings_ui(&mut self, ctx: &Context) {
        let vp_id = if let Some(id) = self.settings_viewport_id {
            id
        } else {
            let id = ViewportId::from_hash_of("Settings");
            self.settings_viewport_id = Some(id);
            id
        };
        ctx.show_viewport_immediate(vp_id, ViewportBuilder::default(), |ctx, _v| {
            ctx.input(|v| {
                if v.viewport().close_requested() {
                    self.show_settings = false;
                }
            });
            CentralPanel::default().show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Text Size");
                    ui.add(
                        Slider::new(&mut self.config.text_size, 10.0..=100.0)
                            .text_color(self.config.text_color.color()),
                    );
                });
                if self.players.is_empty() {
                    self.players = mpris::PlayerFinder::new().unwrap().find_all().unwrap();
                }

                let mut player_name = self.config.player_name.clone();
                ui.horizontal(|ui| {
                    ui.label("Player");
                    ComboBox::from_id_source("player_combo_box")
                        .selected_text(&self.config.player_name)
                        .show_ui(ui, |ui| {
                            for player in self.players.iter() {
                                ui.selectable_value(
                                    &mut player_name,
                                    player.bus_name_player_name_part().to_owned(),
                                    player.bus_name_player_name_part(),
                                );
                            }
                        });
                    ui.button("Refresh").clicked().then(|| {
                        self.players = mpris::PlayerFinder::new().unwrap().find_all().unwrap()
                    });
                });
                if player_name != self.config.player_name {
                    info!("Player changed to {}", player_name);
                    self.event_sender
                        .send(Event::ChangePlayer(player_name.clone()))
                        .ok();
                    self.config.player_name = player_name;
                }
                ui.horizontal(|ui| {
                    ui.label("Fuzzy match");
                    ui.checkbox(&mut self.config.fuzzy, "")
                        .changed()
                        .then(|| self.event_sender.send(Event::ToggleFuzzy).ok());
                });
                let mut font_name = self.config.font_name.clone().unwrap_or("".to_owned());
                ui.horizontal(|ui| {
                    ui.label("Font");
                    ComboBox::from_id_source("font_combo_box")
                        .selected_text(&self.config.font_name.clone().unwrap_or("".to_owned()))
                        .show_ui(ui, |ui| {
                            for font in font_loader::system_fonts::query_all() {
                                ui.selectable_value(&mut font_name, font.clone(), font.as_str());
                            }
                        });
                });
                if let Some(config_font_name) = &self.config.font_name {
                    if font_name != config_font_name.as_str() {
                        info!("Font changed to {}", font_name);
                        self.config.font_name = Some(font_name.clone());
                        setup_custom_fonts(ctx, &self.config);
                    }
                } else {
                    self.config.font_name = Some(font_name.clone());
                }
                ui.horizontal(|ui| {
                    ui.label("Auto resize");
                    ui.checkbox(&mut self.config.auto_resize, "")
                });
                ui.horizontal(|ui| {
                    ui.label("Spotify access token");
                    let mut buf = self
                        .config
                        .spotify_access_token
                        .clone()
                        .unwrap_or("".to_string());
                    ui.text_edit_singleline(&mut buf);
                    self.config.spotify_access_token = Some(buf);
                });
                ui.horizontal(|ui| {
                    ui.label("Spotify client token");
                    let mut buf = self
                        .config
                        .spotify_client_token
                        .clone()
                        .unwrap_or("".to_string());
                    ui.text_edit_singleline(&mut buf);
                    self.config.spotify_client_token = Some(buf);
                });
                if ui.button("Save").clicked() {
                    if let Ok(data) = serde_yaml::to_string(&self.config) {
                        write(&self.config_path, data.as_bytes()).ok();
                    }
                }
            });
        });
    }
}

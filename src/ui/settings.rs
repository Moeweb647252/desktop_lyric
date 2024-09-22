use eframe::egui::{CentralPanel, ComboBox, Context, Slider, ViewportBuilder, ViewportId};
use log::info;

use crate::serve::Event;

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
                    ComboBox::from_label("")
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
            });
        });
    }
}

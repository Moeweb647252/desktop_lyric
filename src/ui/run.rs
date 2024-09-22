use std::sync::mpsc::sync_channel;

use eframe::egui::ViewportBuilder;

use crate::{config::Config, serve::serve, utils::setup_custom_fonts};

use super::DesktopLyricApp;

impl DesktopLyricApp {
    pub fn run(config: Config) -> eframe::Result {
        let options = eframe::NativeOptions {
            viewport: ViewportBuilder::default()
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
        let (_handle, lock) = serve(config.clone(), rx);
        eframe::run_native(
            "Desktop Lyric", // unused title
            options,
            Box::new(move |cc| {
                setup_custom_fonts(&cc.egui_ctx, &config);
                Ok(Box::new(DesktopLyricApp {
                    config,
                    current_lyric: lock,
                    drag_mode: true,
                    settings_viewport_id: None,
                    show_settings: false,
                    players: Vec::new(),
                    event_sender: tx,
                }))
            }),
        )
    }
}

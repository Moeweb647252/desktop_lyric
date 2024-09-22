use eframe::egui::{self, FontData};
use font_loader::system_fonts::FontPropertyBuilder;

use crate::config::Config;

fn load_font(config: &Config) -> Option<FontData> {
    if let Some(font_path) = config.font_path.clone() {
        if let Ok(font_data) = std::fs::read(font_path) {
            // .ttf and .otf files supported.
            return Some(FontData::from_owned(font_data));
        }
    } else if let Some(font_name) = config.font_name.clone() {
        if let Some(font) = font_loader::system_fonts::get(
            &FontPropertyBuilder::new()
                .family(font_name.as_str())
                .build(),
        ) {
            return Some(FontData::from_owned(font.0));
        }
    } else {
        if let Some(font) = font_loader::system_fonts::get(&FontPropertyBuilder::new().build()) {
            return Some(FontData::from_owned(font.0));
        } else {
            return None;
        }
    }
    None
}

pub fn setup_custom_fonts(ctx: &egui::Context, config: &Config) {
    let mut fonts = egui::FontDefinitions::default();
    let fontdata = load_font(config).expect("Cannot find font");
    fonts.font_data.insert("default".to_owned(), fontdata);
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "default".to_owned());

    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .push("default".to_owned());

    ctx.set_fonts(fonts);
}

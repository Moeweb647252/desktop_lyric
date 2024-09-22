use eframe::egui::ecolor::HexColor;
use log::info;
use serde::de::Visitor;
use serde::{Deserialize, Serialize, Serializer};
use std::fs::read_to_string;
use std::str::FromStr;

static DEFAULT_CONFIG: &'static str = include_str!("../config.yaml");
struct HexColorVisitor;

impl Visitor<'_> for HexColorVisitor {
    type Value = HexColor;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a hex color string")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(HexColor::from_str(v)
            .map_err(|_| serde::de::Error::custom(format!("Can not identify {} as HexColor", v)))?)
    }
}

fn serialize_hex_color<S>(color: &HexColor, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_str(color.to_string().as_str())
}

fn deserialize_hex_color<'de, D>(deserializer: D) -> Result<HexColor, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Ok(deserializer.deserialize_str(HexColorVisitor)?)
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    #[serde(
        serialize_with = "serialize_hex_color",
        deserialize_with = "deserialize_hex_color"
    )]
    pub text_color: HexColor,
    #[serde(
        serialize_with = "serialize_hex_color",
        deserialize_with = "deserialize_hex_color"
    )]
    pub background_color: HexColor,
    pub text_size: f32,
    pub default_size: Vec2,
    pub passthrough: bool,
    pub lyric_dir: String,
    pub font_path: Option<String>,
    pub font_name: Option<String>,
    pub player_name: String,
    pub fuzzy: bool,
}

impl Config {
    pub fn from_file(path: &str) -> Self {
        let content = read_to_string(path).unwrap();
        serde_yaml::from_str(&content).unwrap()
    }

    pub fn init() -> Self {
        if let Some(home_dir) = dirs::home_dir() {
            let path = home_dir
                .join(".config")
                .join("desktop_lyric")
                .join("config.yaml");
            if !path.exists() {
                std::fs::create_dir_all(&path.parent().unwrap()).unwrap();
                std::fs::write(path, DEFAULT_CONFIG.as_bytes()).unwrap();
                info!("Using default config file");
                serde_yaml::from_str(DEFAULT_CONFIG).unwrap()
            } else {
                info!("Using config file: {}", path.to_string_lossy());
                serde_yaml::from_str(read_to_string(path).unwrap().as_str()).unwrap()
            }
        } else {
            info!("Using default config file");
            serde_yaml::from_str(DEFAULT_CONFIG).unwrap()
        }
    }
}

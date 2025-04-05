use eframe::egui::ecolor::HexColor;
use log::info;
use serde::de::Visitor;
use serde::{Deserialize, Serialize, Serializer};
use std::fs::read_to_string;
use std::path::PathBuf;
use std::str::FromStr;

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
#[serde(default)]
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
    pub auto_resize: bool,
    pub spotify_access_token: Option<String>,
    pub spotify_client_token: Option<String>,
}

impl Config {
    pub fn from_file(path: &str) -> Self {
        let content = read_to_string(path).unwrap();
        serde_yaml::from_str(&content).unwrap()
    }

    pub fn init() -> (Self, PathBuf) {
        let path = dirs::config_dir()
            .map(|c| c.join("desktop_lyric").join("config.yaml"))
            .unwrap_or(PathBuf::from("./config.yaml"));

        let config;

        if let Some(_config) = read_to_string(&path)
            .ok()
            .and_then(|s| serde_yaml::from_str::<'_, Config>(&*s).ok())
        {
            info!("Using config file: {}", path.to_string_lossy());
            config = _config;
        } else {
            info!("Falling back to default config");
            config = Config::default();
        }

        std::fs::create_dir_all(&path.parent().unwrap()).unwrap();
        let _ = std::fs::write(&path, serde_yaml::to_string(&config).unwrap().as_str());
        (config, path)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            text_color: HexColor::from_str("#ffffff60").unwrap(),
            background_color: HexColor::from_str("#00000060").unwrap(),
            text_size: 50.,
            default_size: Vec2 { x: 700., y: 10. },
            passthrough: false,
            lyric_dir: "~/Music".into(),
            font_path: Some("".into()),
            player_name: "deadbeef".into(),
            fuzzy: false,
            auto_resize: false,
            font_name: None,
            spotify_access_token: None,
            spotify_client_token: None,
        }
    }
}

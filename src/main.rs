#![allow(dead_code)]

use clap::Parser;
use config::Config;
use log::info;
use ui::DesktopLyricApp;

mod config;
mod fetch;
mod font;
mod fuo;
mod lyric;
mod serve;
mod ui;

#[derive(clap::Parser)]
struct Args {
    #[arg(help = "Config file path", short = 'c')]
    config: Option<String>,
}

fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let args = Args::parse();
    let (config, config_path) = if let Some(path) = args.config {
        info!("Using config file: {}", path);
        (Config::from_file(path.as_str()), path.into())
    } else {
        Config::init()
    };
    DesktopLyricApp::run(config, config_path)
}

use std::ffi::OsStr;
use std::fs::read_to_string;
use std::path::PathBuf;
use std::sync::mpsc::Receiver;
use std::{sync::Arc, thread};

use crate::lyric::Lyric;
use crate::Config;
use eframe::egui::mutex::RwLock;
use eframe::egui::TextBuffer;
use log::{error, info};
use mpris::{Metadata, PlayerFinder};
use simsearch::SimSearch;
use std::thread::{sleep, JoinHandle};
use std::time::{Duration, Instant};

pub enum Event {
    Stop,
    ChangePlayer(String),
    ToggleFuzzy,
}

macro_rules! unwarp_or_break {
    ($e:expr, $t:tt) => {
        match $e {
            Ok(v) => v,
            Err(e) => {
                eprintln!("Error: {}", e);
                break $tt;
            }
        }
    };
}

macro_rules! unwarp_or_continue {
    ($e:expr, $t:tt) => {
        match $e {
            Ok(v) => v,
            Err(e) => {
                error!("{}", e);
                sleep(Duration::from_secs(1));
                continue $t;
            }
        }
    };
}

pub fn serve(
    config: Config,
    mut player_name: String,
    event_receiver: Receiver<Event>,
    mut fuzzy: bool,
) -> (JoinHandle<()>, Arc<RwLock<String>>) {
    let _lock = Arc::new(RwLock::new("No lyric".to_owned()));
    let lock = _lock.clone();
    (
        thread::spawn(move || 'dbus: loop {
            let finder = unwarp_or_continue!(PlayerFinder::new(), 'dbus);
            'player: loop {
                let mut engine = SimSearch::new();
                let players = finder.find_all().unwrap();
                for i in &players {
                    let name = i.bus_name_player_name_part().to_owned();
                    engine.insert(i.identity(), &name);
                }
                let player = unwarp_or_continue!(
                    finder.find_by_name(
                        unwarp_or_continue!(
                            engine.search(
                                player_name.as_str()
                            )
                            .get(0).ok_or("No player"), 'player)), 'player);
                info!("Selected player: {}", player.bus_name_player_name_part());
                let metadata = unwarp_or_continue!(player.get_metadata(), 'player);
                info!(
                    "Playing song: {}",
                    unwarp_or_continue!(metadata.title().ok_or("Song doesn't have a title"), 'player)
                );
                let lrc = find_lyric(&metadata, &config.lyric_dir, fuzzy);
                //dbg!(&lrc);
                let mut count = 0;
                let mut position = unwarp_or_continue!(player.get_position(), 'player);
                let mut instant = Instant::now();
                loop {
                    if count > 50 {
                        //println!("Timeout");
                        let new_metadata = unwarp_or_continue!(player.get_metadata(), 'player);
                        if new_metadata.title() != metadata.title()
                            && new_metadata.artists() != metadata.artists()
                        {
                            continue 'player;
                        }
                        count = 0;
                    }
                    if count > 10 {
                        if let Some(event) = event_receiver.try_recv().ok() {
                            match event {
                                Event::Stop => {
                                    break 'dbus;
                                }
                                Event::ChangePlayer(name) => {
                                    player_name = name;
                                    break 'player;
                                }
                                Event::ToggleFuzzy => {
                                    fuzzy = !fuzzy;
                                    break 'player;
                                }
                            }
                        }

                        position = unwarp_or_continue!(player.get_position(), 'player);
                        instant = Instant::now();
                    }
                    let pos = (position + instant.elapsed()).as_millis() as u64;
                    let line = lrc
                        .lines
                        .iter()
                        .filter(|v| v.begin <= pos)
                        .map(|v| v.content.to_owned())
                        .collect::<Vec<String>>();
                    if let Some(line) = line.last() {
                        {
                            if count % 10 == 0 {
                                //dbg!(pos);
                                //dbg!(line);
                            }
                            if !line.is_empty() {
                                (*lock.write()) = line.to_owned();
                            }
                        }
                    }
                    count += 1;
                    sleep(Duration::from_millis(20));
                }
            }
        }),
        _lock,
    )
}

fn find_lyric(metadata: &Metadata, lyric_dir: &str, fuzzy: bool) -> Lyric {
    if let Some(url) = metadata.url() {
        let path = PathBuf::from(url.replace("file://", ""));
        if let Some(Some(file_stem)) = path.file_stem().map(|v| v.to_str()) {
            if fuzzy {
                let mut engine: SimSearch<PathBuf> = SimSearch::new();
                std::fs::read_dir(lyric_dir)
                    .unwrap()
                    .map(|v| v.unwrap())
                    .filter(|v| v.path().file_stem().is_some())
                    .filter(|v| v.path().extension() == Some(OsStr::new("lrc")))
                    .for_each(|v| {
                        engine.insert(
                            v.path(),
                            v.path()
                                .clone()
                                .file_stem()
                                .unwrap()
                                .to_string_lossy()
                                .as_str(),
                        )
                    });
                let res = engine.search(file_stem);
                if let Some(path) = res.get(0) {
                    if let Ok(content) = read_to_string(path) {
                        return Lyric::from_str(&content);
                    }
                }
            } else {
                if let Ok(content) = read_to_string(format!("{}/{}.lrc", lyric_dir, file_stem)) {
                    return Lyric::from_str(&content);
                }
            }
        }
    }
    if let Some(title) = metadata.title() {
        let mut artist = String::new();
        if let Some(artists) = metadata.artists() {
            if let Some(_artist) = artists.first() {
                artist = _artist.to_owned().to_owned()
            }
        }
        let resp = Err("fetch lyric not implemented");
        match resp {
            Ok(lyrics) => return Lyric::from_str(lyrics),
            Err(e) => {
                log::error!("Error: {}", e);
            }
        };
    }
    Lyric::from_str("")
}

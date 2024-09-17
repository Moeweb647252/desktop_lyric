use std::fs::read_to_string;
use std::{sync::Arc, thread};

use crate::lyric::Lyric;
use crate::Config;
use eframe::egui::mutex::RwLock;
use log::info;
use mpris::PlayerFinder;
use std::thread::sleep;
use std::time::{Duration, Instant};

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
                eprintln!("Error: {}", e);
                sleep(Duration::from_secs(1));
                continue $t;
            }
        }
    };
}

pub fn serve(config: Config) -> Arc<RwLock<String>> {
    let _lock = Arc::new(RwLock::new("No lyric".to_owned()));
    let lock = _lock.clone();
    thread::spawn(move || {
        'dbus: loop {
            let finder = unwarp_or_continue!(PlayerFinder::new(), 'dbus);
            'player: loop {
                let player = unwarp_or_continue!(finder.find_active(), 'player);
                let metadata = unwarp_or_continue!(player.get_metadata(), 'player);
                println!("{:?}", metadata);
                let lrc_file = if let Some(url) = metadata.url() {
                    std::fs::read_dir(&config.lyric_dir)
                        .unwrap()
                        .map(|v| v.unwrap())
                        .map(|v| (v.file_name().to_string_lossy().to_string(), v))
                        .filter(|v| v.0.ends_with(".lrc"))
                        .find(|v| {
                            v.0.contains(
                                url.to_string()
                                    .replace("file://", "")
                                    .split('.')
                                    .collect::<Vec<&str>>()[0]
                                    .split('/')
                                    .collect::<Vec<&str>>()
                                    .last()
                                    .unwrap(),
                            )
                        })
                } else {
                    None
                };
                let lrc = if let Some(lrc_file) = lrc_file {
                    Lyric::from_str(&read_to_string(&lrc_file.1.path()).unwrap())
                } else {
                    if let Some(title) = metadata.title() {
                        let artist = if let Some(artists) = metadata.artists() {
                            if let Some(artist) = artists.first() {
                                artist.to_owned()
                            } else {
                                ""
                            }
                        } else {
                            ""
                        };
                        let resp = Err("fetch lyric not implemented");
                        match resp {
                            Ok(lyrics) => Lyric::from_str(lyrics),
                            Err(e) => {
                                log::error!("Error: {}", e);
                                Lyric::from_str("")
                            }
                        }
                    } else {
                        Lyric::from_str("")
                    }
                };
                dbg!(&lrc);
                let mut count = 0;
                let mut position = unwarp_or_continue!(player.get_position(), 'player);
                let mut instant = Instant::now();
                loop {
                    if count > 50 {
                        let new_metadata = unwarp_or_continue!(player.get_metadata(), 'player);
                        if new_metadata.title() != metadata.title()
                            && new_metadata.artists() != metadata.artists()
                        {
                            info!("Song changed: {:?}", new_metadata.title());
                            continue 'player;
                        }
                        count = 0;
                    }
                    if count > 10 {
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
        }
    });
    _lock
}

use std::{sync::Arc, thread};

use egui::mutex::RwLock;
use mpris::PlayerFinder;

use crate::Config;

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
                continue $t;
            }
        }
    };
}

fn serve(config: Config) -> Arc<RwLock<String>> {
    let _lock = Arc::new(RwLock::new("".to_owned()));
    let lock = _lock.clone();
    thread::spawn(move || 'dbus: loop {
        let finder = unwarp_or_continue!(PlayerFinder::new(), 'dbus);
        'player: loop {
            let player = unwarp_or_continue!(finder.find_active(), 'player);
            let mut metadata = unwarp_or_continue!(player.get_metadata(), 'player);
            'song: loop {
                if unwarp_or_continue!(player.get_metadata(), 'player)
                    .as_hashmap()
                    .eq(&metadata.as_hashmap())
                {
                    metadata = unwarp_or_continue!(player.get_metadata(), 'player);
                    for i in std::fs::read_dir(&config.lyric_dir).unwrap() {
                        let i 
                    }
                }
            }
        }
    });
    _lock
}

pub struct Lyric {
    pub lines: Vec<LyricLine>,
}

pub struct LyricLine {
    pub begin: f64,
    pub end: u64,
    pub content: String,
}

impl Lyric {
    pub fn from_str(lrc: &str) {
        let lrc = lrc.to_owned();
    }
}

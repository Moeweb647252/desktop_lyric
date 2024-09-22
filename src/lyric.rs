#[derive(Debug)]
pub struct Lyric {
    pub lines: Vec<LyricLine>,
}

#[derive(Debug)]
pub struct LyricLine {
    pub begin: u64,
    pub end: u64,
    pub content: String,
}

impl Lyric {
    pub fn from_str(lrc: &str) -> Self {
        let lrc = lrc.replace("&apos;", "'");
        let mut res = Vec::new();
        let lrc = amll_lyric::lrc::parse_lrc(lrc.as_str());
        for i in lrc {
            res.push(LyricLine {
                begin: i.start_time,
                end: i.end_time,
                content: i
                    .words
                    .iter()
                    .map(|v| v.word.to_string())
                    .collect::<Vec<String>>()
                    .join(" "),
            })
        }
        Self { lines: res }
    }
}

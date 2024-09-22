use std::{
    any::Any,
    io::{Read, Write},
    net::TcpStream,
    process::Command,
};

use serde::de::Error;

pub struct FuoClient;

impl FuoClient {
    pub fn status(&mut self) -> String {
        String::from_utf8_lossy(&Command::new("fuo").arg("status").output().unwrap().stdout)
            .to_string()
    }

    pub fn lyric(&mut self) -> Option<String> {
        let status = self.status();
        let lines = status.lines();
        let line: String = lines.filter(|v| v.contains("song:")).collect();
        let song = &line[line.chars().position(|v| v == ':')? + 1..]
            .trim()
            .split('\t')
            .nth(0)?;
        dbg!(song);
        Some(
            String::from_utf8_lossy(
                &Command::new("fuo")
                    .arg("show")
                    .arg(format!("{}/lyric", song))
                    .output()
                    .unwrap()
                    .stdout,
            )
            .to_string(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_status() {
        let mut client = FuoClient;
        let status = client.status();
        println!("{}", status);
        let lyric = client.lyric();
        println!("{:?}", lyric);
    }
}

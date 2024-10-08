use std::time::{Duration, SystemTime};

use log::debug;
use serde_json::Value;

use crate::lyric::{Lyric, LyricLine};

struct ClientToken {
    expire: Duration,
    fetch_time: SystemTime,
    token: String,
}

pub fn fetch_spotify_lyric(token: &str, client_token: &str, id: Option<String>) -> Option<Lyric> {
    debug!("track id: {:?}", id);
    let id = if let Some(id) = id {
        id
    } else {
        fetch_spotify_cur_trackid(token)?
    };
    if let Value::Object(obj) = serde_json::from_str(ureq::get(
        format!("https://spclient.wg.spotify.com/color-lyrics/v2/track/{}?format=json&vocalRemoval=false&market=from_token", id)
        .as_str())
        .set("authorization", token)
        .set("client-token",client_token)
        .set("app-platform", "WebPlayer").call().ok()?.into_string().ok()?.as_str()).ok()? {
        if let Value::Array(lines) = obj.get("lyrics")?.as_object()?.get("lines")? {
            return Some(Lyric{
                lines: lines.iter().map(|v| {
                    LyricLine{
                        content: v.get("words").unwrap().as_str().unwrap().to_string(),
                        begin: v.get("startTimeMs").unwrap().as_str().unwrap().parse().unwrap(),
                        end: v.get("endTimeMs").unwrap().as_str().unwrap().parse().unwrap(),
                    }
                }).collect()
            }) ;
        }
    }
    None
}

pub fn fetch_spotify_cur_trackid(token: &str) -> Option<String> {
    if let Value::Object(obj) = serde_json::from_str(
        &ureq::get("https://api.spotify.com/v1/me/player/currently-playing")
            .set("authorization", token)
            .call()
            .ok()?
            .into_string()
            .ok()?,
    )
    .ok()?
    {
        if let Value::Object(item) = obj.get("item")? {
            if let Value::String(id) = item.get("id")? {
                return Some(id.to_owned());
            }
        }
    }
    None
}

pub fn fetch_spotify_client_token() -> Option<String> {
    if let Value::Object(obj) = serde_json::from_str(
        &ureq::get("https://clienttoken.spotify.com/v1/clienttoken")
            .send_string(include_str!("../assets/client.json"))
            .ok()?
            .into_string()
            .ok()?,
    )
    .ok()?
    {
        if let Value::String(token) = obj.get("accessToken")? {
            return Some(token.to_owned());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_fetch_spotify_lyric() {
        let token = "Bearer BQCLZPuRBUCJ2iexfHHwWVDwiegJB2dJrJ81mI5IJ8O6krLKWpLrzPdcO3-5omlmpcQmKZLBUCr3EDnkyqKbIf9G9Y7CZX9DU9NbjBaOmSAZ_-pT1rjc5GzgUa2tsRb3JDwDmNpigMxrOwQPKd1yjBgnVSamsxcOvWoISH1F4htIzRkIrP3wxDVar6JOV3ouR2CdOb2M1s6c6Eln21OdU5eCseKD8IyeDLE4T7GTeAen-Hs8wgJVWGutNBtLsxOtjVg6MXnU8EGg77tQqYhk3BmEvUp2bevFg-h7sL4GwDGPI8ouxkmcADMZl-wbk251ZEUGOP8ScORx9BLrU0C8upUp-j5a_6tzLP343ug";
        let client_token = "AABN2xRCJyE/dFUZATsFD8vNSUxxVcREklxUyfw2Whw9cGwEszAmQdT7y5cWNvsbHJqcnMzrxCrQFqKRX9QvyYT0SexOtOepLsbsjUYjB+EIhBwREeiHO4EDKjqO/BnHj+e12imTxwN+stQEo5nsIkIGSoaacMpJyHjwl2mlohAPKNbdEPGI8uSXRUYGDndH1ppJ9aveTYbjx82SdJucqIyeBOX4t1kWCJh7/dOz4RskAz2fqew49AErQdQZeP1F1EeXLUoPZgQTpnfzK88IrE3EuKfP2MC9ixnbRogs/EHP";
        let lyric = fetch_spotify_lyric(token, client_token, None);
        println!("{:?}", lyric);
    }
}

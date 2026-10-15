use std::thread;
use std::time::Duration;

use crate::Error;

pub const MAXIMUM_DOWNLOAD_BYTES: u64 = 256 * 1024 * 1024;
const CONTENT_LENGTH: &str = "content-length";
const ATTEMPTS: u32 = 3;
const PAUSE: Duration = Duration::from_secs(2);
const NOT_FOUND: u16 = 404;
const TOO_MANY_REQUESTS: u16 = 429;
const FIRST_SERVER_ERROR: u16 = 500;

pub enum Downloaded {
    Bytes(Vec<u8>),
    NotFound,
    TooLarge,
}

pub struct Request<'a> {
    pub agent: &'a ureq::Agent,
    pub url: &'a str,
    pub limit: u64,
}

pub fn download(request: Request) -> Result<Downloaded, Error> {
    let Request { agent, url, limit } = request;
    let mut attempt = 1;
    loop {
        let result = agent.get(url).call().and_then(|mut response| {
            let declared = response
                .headers()
                .get(CONTENT_LENGTH)
                .and_then(|value| value.to_str().ok()?.parse::<u64>().ok());
            if declared.is_some_and(|length| length > limit) {
                return Ok(None);
            }
            response
                .body_mut()
                .with_config()
                .limit(limit)
                .read_to_vec()
                .map(Some)
        });
        match result {
            Ok(Some(bytes)) => return Ok(Downloaded::Bytes(bytes)),
            Ok(None) | Err(ureq::Error::BodyExceedsLimit(_)) => return Ok(Downloaded::TooLarge),
            Err(ureq::Error::StatusCode(NOT_FOUND)) => return Ok(Downloaded::NotFound),
            Err(error) if attempt < ATTEMPTS && is_temporary(&error) => {
                thread::sleep(PAUSE * attempt);
                attempt += 1;
            }
            Err(error) => return Err(Error::new(format!("cannot download {url}: {error}"))),
        }
    }
}

fn is_temporary(error: &ureq::Error) -> bool {
    match error {
        ureq::Error::StatusCode(code) => *code == TOO_MANY_REQUESTS || *code >= FIRST_SERVER_ERROR,
        ureq::Error::Timeout(_)
        | ureq::Error::Io(_)
        | ureq::Error::ConnectionFailed
        | ureq::Error::BodyStalled => true,
        _ => false,
    }
}

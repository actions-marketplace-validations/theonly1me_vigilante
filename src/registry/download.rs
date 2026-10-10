use std::thread;
use std::time::Duration;

use crate::Error;

const MAXIMUM_DOWNLOAD_BYTES: u64 = 256 * 1024 * 1024;
const ATTEMPTS: u32 = 3;
const PAUSE: Duration = Duration::from_secs(2);
const NOT_FOUND: u16 = 404;
const TOO_MANY_REQUESTS: u16 = 429;
const FIRST_SERVER_ERROR: u16 = 500;

pub enum Downloaded {
    Bytes(Vec<u8>),
    NotFound,
}

pub struct Request<'a> {
    pub agent: &'a ureq::Agent,
    pub url: &'a str,
}

pub fn download(request: Request) -> Result<Downloaded, Error> {
    let Request { agent, url } = request;
    let mut attempt = 1;
    loop {
        let result = agent.get(url).call().and_then(|mut response| {
            response
                .body_mut()
                .with_config()
                .limit(MAXIMUM_DOWNLOAD_BYTES)
                .read_to_vec()
        });
        match result {
            Ok(bytes) => return Ok(Downloaded::Bytes(bytes)),
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

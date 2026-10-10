use std::fs;

use serde_json::Value;
use sha2::{Digest, Sha256};

use super::Registry;
use super::download::{Downloaded, Request, download};
use crate::Error;

impl Registry {
    pub fn get_json(&self, url: &str) -> Result<Option<Value>, Error> {
        let Some(bytes) = self.get_cached(url)? else {
            return Ok(None);
        };
        serde_json::from_slice(&bytes)
            .map(Some)
            .map_err(|error| Error::new(format!("{url} is not JSON: {error}")))
    }

    pub fn get_text(&self, url: &str) -> Result<Option<String>, Error> {
        Ok(self
            .get_cached(url)?
            .map(|bytes| String::from_utf8_lossy(&bytes).into_owned()))
    }

    fn get_cached(&self, url: &str) -> Result<Option<Vec<u8>>, Error> {
        let digest: String = Sha256::digest(url.as_bytes())
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect();
        let cached = self.cache.document_file(&digest);
        if let Ok(bytes) = fs::read(&cached) {
            return Ok(Some(bytes));
        }
        let Downloaded::Bytes(bytes) = download(Request {
            agent: &self.agent,
            url,
            limit: self.limit,
        })?
        else {
            return Ok(None);
        };
        if let Some(parent) = cached.parent()
            && fs::create_dir_all(parent).is_ok()
        {
            let staging = cached.with_extension(format!("{}.part", std::process::id()));
            if fs::write(&staging, &bytes).is_ok() {
                let _ = fs::rename(&staging, &cached);
            }
        }
        Ok(Some(bytes))
    }
}

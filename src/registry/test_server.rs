use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;

pub struct Serving {
    pub files: BTreeMap<String, Vec<u8>>,
    pub failures: usize,
}

pub struct Symlink<'a> {
    pub link: &'a str,
    pub target: &'a str,
}

pub struct TestServer {
    pub base_url: String,
    requests: Arc<AtomicUsize>,
}

impl TestServer {
    pub fn serve(files: BTreeMap<String, Vec<u8>>) -> Self {
        Self::serve_with(Serving { files, failures: 0 })
    }

    pub fn serve_with(serving: Serving) -> Self {
        let Serving { files, failures } = serving;
        let listener = TcpListener::bind("127.0.0.1:0").expect("listener");
        let base_url = format!("http://{}", listener.local_addr().expect("address"));
        let requests = Arc::new(AtomicUsize::new(0));
        let counter = Arc::clone(&requests);
        thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(mut stream) = stream else { continue };
                let seen = counter.fetch_add(1, Ordering::SeqCst);
                let mut reader = BufReader::new(stream.try_clone().expect("clone"));
                let mut request_line = String::new();
                reader.read_line(&mut request_line).expect("request line");
                let mut header = String::new();
                while reader.read_line(&mut header).is_ok_and(|read| read > 2) {
                    header.clear();
                }
                let path = request_line.split_whitespace().nth(1).unwrap_or("/");
                let (status, body) = match files.get(path) {
                    _ if seen < failures => ("503 Service Unavailable", Vec::new()),
                    Some(body) => ("200 OK", body.clone()),
                    None => ("404 Not Found", Vec::new()),
                };
                let head = format!(
                    "HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    body.len()
                );
                let _ = stream.write_all(head.as_bytes());
                let _ = stream.write_all(&body);
            }
        });
        Self { base_url, requests }
    }

    pub fn request_count(&self) -> usize {
        self.requests.load(Ordering::SeqCst)
    }
}

pub fn tar_gzip(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast());
    let mut builder = tar::Builder::new(encoder);
    for (path, contents) in entries {
        let mut header = tar::Header::new_gnu();
        let name = header.as_gnu_mut().expect("gnu header");
        name.name[..path.len()].copy_from_slice(path.as_bytes());
        header.set_size(contents.len() as u64);
        header.set_mode(0o644);
        header.set_entry_type(tar::EntryType::Regular);
        header.set_cksum();
        builder.append(&header, *contents).expect("append");
    }
    builder.into_inner().expect("tar").finish().expect("gzip")
}

pub fn tar_with_symlink(symlink: Symlink) -> Vec<u8> {
    let Symlink { link, target } = symlink;
    let encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast());
    let mut builder = tar::Builder::new(encoder);
    let mut header = tar::Header::new_gnu();
    header.set_entry_type(tar::EntryType::Symlink);
    header.set_size(0);
    builder
        .append_link(&mut header, link, target)
        .expect("symlink");
    builder.into_inner().expect("tar").finish().expect("gzip")
}

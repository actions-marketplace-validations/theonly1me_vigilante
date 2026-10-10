use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::thread;

pub fn serve_files(files: BTreeMap<String, Vec<u8>>) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener");
    let base_url = format!("http://{}", listener.local_addr().expect("address"));
    thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { continue };
            let mut reader = BufReader::new(stream.try_clone().expect("clone"));
            let mut request_line = String::new();
            reader.read_line(&mut request_line).expect("request line");
            let mut header = String::new();
            while reader.read_line(&mut header).is_ok_and(|read| read > 2) {
                header.clear();
            }
            let path = request_line.split_whitespace().nth(1).unwrap_or("/");
            let (status, body) = match files.get(path) {
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
    base_url
}

pub fn allow_config(base_url: &str) -> String {
    format!("[registries]\nallow = [\"{base_url}\"]\n")
}

#![allow(clippy::unwrap_used)]
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicUsize, Ordering},
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

// In-process WebDAV fixture. Only the explicitly bound loopback URL is accepted.
pub(crate) struct AssetServer {
    pub url: String,
    pub settings: Arc<Mutex<Option<Vec<u8>>>>,
    pub requests: Arc<AtomicUsize>,
    task: tokio::task::JoinHandle<()>,
}

impl Drop for AssetServer {
    fn drop(&mut self) {
        self.task.abort();
    }
}

pub(crate) async fn asset_server() -> AssetServer {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let stored = Arc::new(Mutex::new(None::<Vec<u8>>));
    let remote = stored.clone();
    let requests = Arc::new(AtomicUsize::new(0));
    let count = requests.clone();
    let task = tokio::spawn(async move {
        loop {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut request = Vec::new();
            let mut chunk = [0; 4096];
            let (end, length) = loop {
                let count = socket.read(&mut chunk).await.unwrap();
                assert_ne!(count, 0);
                request.extend_from_slice(&chunk[..count]);
                if let Some(end) = request.windows(4).position(|p| p == b"\r\n\r\n") {
                    let length = String::from_utf8_lossy(&request[..end])
                        .lines()
                        .find_map(|line| {
                            let (name, value) = line.split_once(':')?;
                            name.eq_ignore_ascii_case("content-length")
                                .then(|| value.trim().parse::<usize>().unwrap())
                        })
                        .unwrap_or(0);
                    break (end + 4, length);
                }
            };
            while request.len() < end + length {
                let count = socket.read(&mut chunk).await.unwrap();
                assert_ne!(count, 0);
                request.extend_from_slice(&chunk[..count]);
            }
            count.fetch_add(1, Ordering::Relaxed);
            let first = String::from_utf8_lossy(&request[..end]);
            let target = first.split_whitespace().nth(1).unwrap();
            let (status, body) = if first.starts_with("MKCOL ") {
                assert!(target.trim_end_matches('/').ends_with("/claude"));
                ("201 Created", Vec::new())
            } else if first.starts_with("PUT ") {
                assert!(target.ends_with("/claude/settings.json"));
                *remote.lock().unwrap() = Some(request[end..end + length].to_vec());
                ("201 Created", Vec::new())
            } else if first.starts_with("GET ") && target.ends_with("/claude/settings.json") {
                match remote.lock().unwrap().clone() {
                    Some(bytes) => ("200 OK", bytes),
                    None => ("404 Not Found", Vec::new()),
                }
            } else {
                assert!(first.starts_with("GET ") || first.starts_with("PROPFIND "));
                ("404 Not Found", Vec::new())
            };
            let header = format!(
                "HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body.len()
            );
            socket.write_all(header.as_bytes()).await.unwrap();
            socket.write_all(&body).await.unwrap();
            socket.shutdown().await.unwrap();
        }
    });
    AssetServer {
        url: format!("http://{address}/"),
        settings: stored,
        requests,
        task,
    }
}

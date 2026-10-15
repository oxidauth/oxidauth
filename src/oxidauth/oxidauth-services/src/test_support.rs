use std::{
    io::{BufRead, BufReader, Read, Write},
    net::TcpListener,
    sync::{Arc, Mutex},
};

use url::Url;

/// One-shot devserver answering every request with `response_body`, capturing each
/// request body. Stands in for the customer webhook — the half of a TOTP login that
/// oxidauth does not own, and the reason both the login leg and the registration
/// leg need their mail path exercised against a real socket.
pub(crate) fn spawn_webhook(response_body: &'static str) -> (Url, Arc<Mutex<Vec<String>>>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind webhook");
    let addr = listener
        .local_addr()
        .expect("addr");
    let bodies: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(vec![]));
    let thread_bodies = bodies.clone();

    std::thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            let mut writer = stream
                .try_clone()
                .expect("clone stream");
            let mut reader = BufReader::new(stream);
            let mut content_length = 0usize;

            loop {
                let mut line = String::new();
                if reader
                    .read_line(&mut line)
                    .unwrap_or(0)
                    == 0
                    || line.trim().is_empty()
                {
                    break;
                }
                if line
                    .to_ascii_lowercase()
                    .starts_with("content-length:")
                {
                    content_length = line[15..]
                        .trim()
                        .parse()
                        .unwrap_or(0);
                }
            }

            let mut body = vec![0u8; content_length];
            if reader
                .read_exact(&mut body)
                .is_err()
            {
                break;
            }
            thread_bodies
                .lock()
                .expect("bodies")
                .push(String::from_utf8_lossy(&body).to_string());

            let header = format!(
                "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: \
                 {}\r\nconnection: close\r\n\r\n",
                response_body.len()
            );
            if writer
                .write_all(header.as_bytes())
                .and_then(|_| writer.write_all(response_body.as_bytes()))
                .is_err()
            {
                break;
            }
            let _ = writer.flush();
        }
    });

    (
        Url::parse(&format!("http://{addr}/send")).expect("webhook url"),
        bodies,
    )
}

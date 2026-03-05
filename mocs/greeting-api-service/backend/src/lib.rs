use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};

pub const DEFAULT_PORT: u16 = 4318;
pub const GREETING_PATH: &str = "/api/v1/greeting";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Greeting {
    pub title: String,
    pub message: String,
}

impl Greeting {
    pub fn demo() -> Self {
        Self {
            title: "Hello from blocks".to_string(),
            message: "The moc model can carry a real backend-to-frontend slice.".to_string(),
        }
    }

    pub fn to_json(&self) -> String {
        serde_json::json!({
            "title": self.title,
            "message": self.message,
        })
        .to_string()
    }
}

pub fn bind_listener(port: u16) -> io::Result<TcpListener> {
    TcpListener::bind(("127.0.0.1", port))
}

pub fn serve(
    listener: TcpListener,
    greeting: Greeting,
    max_requests: Option<usize>,
) -> io::Result<()> {
    let mut handled_requests = 0usize;

    for stream in listener.incoming() {
        let stream = stream?;
        handle_connection(stream, &greeting)?;
        handled_requests += 1;

        if let Some(limit) = max_requests {
            if handled_requests >= limit {
                break;
            }
        }
    }

    Ok(())
}

fn handle_connection(mut stream: TcpStream, greeting: &Greeting) -> io::Result<()> {
    let request = read_http_request(&mut stream)?;
    let request_line = request.lines().next().unwrap_or_default();
    let response = build_response(request_line, greeting);
    stream.write_all(response.as_bytes())?;
    stream.flush()
}

fn read_http_request(stream: &mut TcpStream) -> io::Result<String> {
    let mut buffer = [0u8; 4096];
    let mut request = Vec::new();

    loop {
        let read = stream.read(&mut buffer)?;
        if read == 0 {
            break;
        }

        request.extend_from_slice(&buffer[..read]);

        if request.windows(4).any(|window| window == b"\r\n\r\n") || request.len() >= 4096 {
            break;
        }
    }

    Ok(String::from_utf8_lossy(&request).into_owned())
}

fn build_response(request_line: &str, greeting: &Greeting) -> String {
    let (status, content_type, body) = match parse_request_line(request_line) {
        Some(("GET", GREETING_PATH)) => (
            "200 OK",
            "application/json; charset=utf-8",
            greeting.to_json(),
        ),
        Some(("GET", _)) => (
            "404 Not Found",
            "application/json; charset=utf-8",
            "{\"error\":\"not found\"}".to_string(),
        ),
        Some((_, _)) => (
            "405 Method Not Allowed",
            "application/json; charset=utf-8",
            "{\"error\":\"method not allowed\"}".to_string(),
        ),
        None => (
            "400 Bad Request",
            "application/json; charset=utf-8",
            "{\"error\":\"bad request\"}".to_string(),
        ),
    };

    format!(
        "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nAccess-Control-Allow-Origin: *\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    )
}

fn parse_request_line(request_line: &str) -> Option<(&str, &str)> {
    let mut parts = request_line.split_whitespace();
    let method = parts.next()?;
    let path = parts.next()?;
    Some((method, path))
}

#[cfg(test)]
mod tests {
    use std::io::{Read, Write};
    use std::net::TcpStream;
    use std::thread;

    use serde_json::Value;

    use super::{GREETING_PATH, Greeting, bind_listener, serve};

    #[test]
    fn serves_greeting_contract_on_ephemeral_port() {
        let listener = bind_listener(0).expect("listener should bind");
        let address = listener.local_addr().expect("local addr should resolve");
        let server = thread::spawn(move || serve(listener, Greeting::demo(), Some(1)));

        let response = send_request(address.port(), GREETING_PATH);
        let (status_line, body) = split_http_response(&response);
        let json: Value = serde_json::from_str(body).expect("body should be valid json");

        assert_eq!(status_line, "HTTP/1.1 200 OK");
        assert_eq!(json["title"], "Hello from blocks");
        assert_eq!(
            json["message"],
            "The moc model can carry a real backend-to-frontend slice."
        );

        server
            .join()
            .expect("server thread should join")
            .expect("server should exit cleanly");
    }

    #[test]
    fn rejects_unknown_routes() {
        let listener = bind_listener(0).expect("listener should bind");
        let address = listener.local_addr().expect("local addr should resolve");
        let server = thread::spawn(move || serve(listener, Greeting::demo(), Some(1)));

        let response = send_request(address.port(), "/missing");
        let (status_line, body) = split_http_response(&response);

        assert_eq!(status_line, "HTTP/1.1 404 Not Found");
        assert_eq!(body, "{\"error\":\"not found\"}");

        server
            .join()
            .expect("server thread should join")
            .expect("server should exit cleanly");
    }

    fn send_request(port: u16, path: &str) -> String {
        let mut stream =
            TcpStream::connect(("127.0.0.1", port)).expect("client should connect to server");
        let request =
            format!("GET {path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n\r\n");
        stream
            .write_all(request.as_bytes())
            .expect("request should be written");

        let mut response = String::new();
        stream
            .read_to_string(&mut response)
            .expect("response should be readable");
        response
    }

    fn split_http_response(response: &str) -> (&str, &str) {
        let (head, body) = response
            .split_once("\r\n\r\n")
            .expect("response should contain headers and body");
        let status_line = head.lines().next().expect("status line should exist");
        (status_line, body)
    }
}

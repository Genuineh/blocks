use std::io::{Read, Write};
use std::net::TcpStream;

use blocks_runtime::BlockExecutionError;
use serde_json::{Value, json};

pub fn run(input: &Value) -> Result<Value, BlockExecutionError> {
    let url = input
        .get("url")
        .and_then(Value::as_str)
        .ok_or_else(|| BlockExecutionError::new("missing string field: url"))?;

    let (host, port, path) = parse_plain_http_url(url)?;
    let mut stream = TcpStream::connect((host.as_str(), port)).map_err(|error| {
        BlockExecutionError::new(format!("failed to connect to {host}:{port}: {error}"))
    })?;

    let request = format!("GET {path} HTTP/1.1\r\nHost: {host}\r\nConnection: close\r\n\r\n");
    stream.write_all(request.as_bytes()).map_err(|error| {
        BlockExecutionError::new(format!("failed to write request to {host}:{port}: {error}"))
    })?;

    let mut buffer = Vec::new();
    stream.read_to_end(&mut buffer).map_err(|error| {
        BlockExecutionError::new(format!(
            "failed to read response from {host}:{port}: {error}"
        ))
    })?;

    let response = String::from_utf8_lossy(&buffer);
    let (head, body) = response
        .split_once("\r\n\r\n")
        .ok_or_else(|| BlockExecutionError::new("invalid HTTP response"))?;
    let status = head
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|value| value.parse::<u16>().ok())
        .ok_or_else(|| BlockExecutionError::new("invalid HTTP status line"))?;

    Ok(json!({
        "status": status,
        "body": body,
    }))
}

fn parse_plain_http_url(url: &str) -> Result<(String, u16, String), BlockExecutionError> {
    let rest = url.strip_prefix("http://").ok_or_else(|| {
        BlockExecutionError::new("only plain http:// URLs are supported in the current MVP runner")
    })?;

    let (host_port, path) = match rest.split_once('/') {
        Some((host_port, path)) => (host_port, format!("/{path}")),
        None => (rest, "/".to_string()),
    };

    if host_port.is_empty() {
        return Err(BlockExecutionError::new("missing host in url"));
    }

    let (host, port) = match host_port.split_once(':') {
        Some((host, port)) => {
            let port = port
                .parse::<u16>()
                .map_err(|_| BlockExecutionError::new("invalid port in url"))?;
            (host.to_string(), port)
        }
        None => (host_port.to_string(), 80),
    };

    if host.is_empty() {
        return Err(BlockExecutionError::new("missing host in url"));
    }

    Ok((host, port, path))
}

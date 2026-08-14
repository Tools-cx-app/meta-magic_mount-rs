use std::{
    io::{Read, Write},
    net::{TcpStream, ToSocketAddrs},
    path::Path,
    time::Duration,
};

use api::{ApiConfig, ConnectionInfo};

pub fn load_config(path: &Path) -> anyhow::Result<ApiConfig> {
    let connection: ConnectionInfo = serde_json::from_slice(&std::fs::read(path)?)?;
    anyhow::ensure!(connection.port != 0, "invalid daemon port");
    anyhow::ensure!(
        !connection.token.is_empty()
            && connection
                .token
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.')),
        "invalid daemon token"
    );

    let address = ("127.0.0.1", connection.port)
        .to_socket_addrs()?
        .next()
        .ok_or_else(|| anyhow::anyhow!("failed to resolve daemon address"))?;
    let mut stream = TcpStream::connect_timeout(&address, Duration::from_secs(2))?;
    stream.set_read_timeout(Some(Duration::from_secs(2)))?;
    stream.set_write_timeout(Some(Duration::from_secs(2)))?;
    write!(
        stream,
        "GET /api/v1/config HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer {}\r\nConnection: close\r\n\r\n",
        connection.token
    )?;
    stream.shutdown(std::net::Shutdown::Write)?;

    let mut response = Vec::new();
    stream.read_to_end(&mut response)?;
    let header_end = response
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .ok_or_else(|| anyhow::anyhow!("invalid daemon response"))?;
    let (headers, body) = response.split_at(header_end + 4);
    let headers = std::str::from_utf8(headers)?;
    anyhow::ensure!(
        headers.starts_with("HTTP/1.1 200 "),
        "daemon request failed"
    );
    let length = headers
        .lines()
        .find_map(|line| line.strip_prefix("Content-Length: "))
        .ok_or_else(|| anyhow::anyhow!("daemon response has no content length"))?
        .parse::<usize>()?;
    anyhow::ensure!(body.len() == length, "invalid daemon response length");
    Ok(serde_json::from_slice(body)?)
}

#[cfg(test)]
mod tests {
    use std::io::{Read, Write};

    use super::load_config;

    #[test]
    fn loads_config_from_authenticated_daemon() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let thread = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut request = Vec::new();
            let mut buffer = [0; 256];
            while !request.windows(4).any(|window| window == b"\r\n\r\n") {
                let size = stream.read(&mut buffer).unwrap();
                assert_ne!(size, 0);
                request.extend_from_slice(&buffer[..size]);
            }
            let request = String::from_utf8(request).unwrap();
            assert!(request.contains("GET /api/v1/config HTTP/1.1"));
            assert!(request.contains("Authorization: Bearer secret\r\n"));
            let body = r#"{"mountsource":"KSU","umount":false,"partitions":[],"ignoreList":[],"customMounts":[]}"#;
            write!(
                stream,
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}",
                body.len(),
                body
            )
            .unwrap();
        });
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("daemon.json");
        std::fs::write(
            &path,
            format!(r#"{{"port":{},"token":"secret"}}"#, address.port()),
        )
        .unwrap();
        let config = load_config(&path).unwrap();
        assert_eq!(config.mountsource, "KSU");
        thread.join().unwrap();
    }
}

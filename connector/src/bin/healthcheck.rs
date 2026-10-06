#![forbid(unsafe_code)]

//! This is a minimal health check client for use in container health checks, since the runtime
//! image has no shell.
//!
//! Exits successfully if `GET /health` on the local server returns `200 OK`.

use std::{
    env,
    io::{Read, Write},
    net::{SocketAddr, TcpStream},
    process::ExitCode,
    time::Duration,
};

const TIMEOUT: Duration = Duration::from_secs(2);

fn check() -> anyhow::Result<()> {
    let port = env::var("PORT")?.parse::<u16>()?;
    let addr = SocketAddr::from(([127, 0, 0, 1], port));

    let mut stream = TcpStream::connect_timeout(&addr, TIMEOUT)?;
    stream.set_read_timeout(Some(TIMEOUT))?;
    stream.set_write_timeout(Some(TIMEOUT))?;

    stream.write_all(b"GET /health HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")?;

    let mut response = String::new();
    stream.read_to_string(&mut response)?;

    let status_line = response.lines().next().unwrap_or_default();
    match status_line.split_whitespace().nth(1) {
        Some("200") => Ok(()),
        _ => Err(anyhow::anyhow!("Unexpected response: {}", status_line)),
    }
}

fn main() -> ExitCode {
    match check() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("Health check failed: {}", err);
            ExitCode::FAILURE
        }
    }
}

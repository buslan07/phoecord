//! Outgoing connections of the proxy. With `upstream` set in `config.toml` (`socks5://host:port` or
//! `http://host:port`), every connection PhoeCord opens to the internet, intercepted or tunnelled,
//! goes through that proxy; unset means a direct connection. Meant for VPN and zapret users.

use anyhow::{Context, Result, bail};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

const TIMEOUT: Duration = Duration::from_secs(15);

/// A parsed `upstream` value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Upstream {
    Socks5 { addr: String },
    Http { addr: String },
}

impl Upstream {
    /// `socks5://127.0.0.1:1080` (`socks5h://` too: names are always resolved by the proxy) or
    /// `http://host:port`.
    pub fn parse(s: &str) -> Result<Upstream> {
        let s = s.trim();
        let (scheme, rest) = s.split_once("://").with_context(|| {
            format!("upstream {s:?} must look like socks5://host:port or http://host:port")
        })?;
        let addr = rest.trim_end_matches('/');
        if addr.is_empty() || addr.contains('@') || addr.contains('/') {
            bail!("upstream {s:?}: expected host:port (credentials are not supported)");
        }
        let (host, port) = addr
            .rsplit_once(':')
            .with_context(|| format!("upstream {s:?} needs a port"))?;
        if host.is_empty() || port.parse::<u16>().is_err() {
            bail!("upstream {s:?} needs a host and a port from 1 to 65535");
        }
        match scheme.to_ascii_lowercase().as_str() {
            "socks5" | "socks5h" => Ok(Upstream::Socks5 { addr: addr.into() }),
            "http" => Ok(Upstream::Http { addr: addr.into() }),
            other => bail!("upstream scheme {other:?} is not supported (use socks5 or http)"),
        }
    }
}

/// Open a TCP connection to `host:port`, through `upstream` when there is one.
pub async fn connect(upstream: Option<&Upstream>, host: &str, port: u16) -> Result<TcpStream> {
    let Some(up) = upstream else {
        return Ok(TcpStream::connect((host, port)).await?);
    };
    let (Upstream::Socks5 { addr } | Upstream::Http { addr }) = up;
    let mut s = tokio::time::timeout(TIMEOUT, TcpStream::connect(addr.as_str()))
        .await
        .map_err(|_| anyhow::anyhow!("upstream proxy {addr} did not answer in time"))?
        .with_context(|| {
            format!("cannot reach the upstream proxy {addr}; check `upstream` in config.toml")
        })?;
    let handshake = async {
        match up {
            Upstream::Socks5 { .. } => socks5(&mut s, host, port).await,
            Upstream::Http { .. } => http_connect(&mut s, host, port).await,
        }
    };
    tokio::time::timeout(TIMEOUT, handshake)
        .await
        .map_err(|_| {
            anyhow::anyhow!("upstream proxy {addr} stalled while connecting to {host}:{port}")
        })?
        .with_context(|| format!("upstream proxy {addr}, target {host}:{port}"))?;
    Ok(s)
}

async fn socks5(s: &mut TcpStream, host: &str, port: u16) -> Result<()> {
    s.write_all(&[5, 1, 0]).await?; // version 5, one method: no authentication
    let mut r = [0u8; 2];
    s.read_exact(&mut r).await.context("not a SOCKS5 server")?;
    if r[0] != 5 {
        bail!("not a SOCKS5 server");
    }
    if r[1] != 0 {
        bail!("the SOCKS5 server wants authentication, which is not supported");
    }
    let name = host.as_bytes();
    if name.is_empty() || name.len() > 255 {
        bail!("bad target host name");
    }
    let mut req = vec![5, 1, 0, 3, name.len() as u8];
    req.extend_from_slice(name);
    req.extend_from_slice(&port.to_be_bytes());
    s.write_all(&req).await?;
    let mut h = [0u8; 4];
    s.read_exact(&mut h).await?;
    if h[1] != 0 {
        bail!("the SOCKS5 server refused: {}", socks_reply(h[1]));
    }
    let skip = match h[3] {
        1 => 4,
        4 => 16,
        3 => {
            let mut l = [0u8; 1];
            s.read_exact(&mut l).await?;
            l[0] as usize
        }
        _ => bail!("bad SOCKS5 reply"),
    } + 2;
    let mut rest = vec![0u8; skip];
    s.read_exact(&mut rest).await?;
    Ok(())
}

fn socks_reply(code: u8) -> &'static str {
    match code {
        1 => "general failure",
        2 => "connection not allowed",
        3 => "network unreachable",
        4 => "host unreachable",
        5 => "connection refused",
        6 => "TTL expired",
        7 => "command not supported",
        8 => "address type not supported",
        _ => "unknown error",
    }
}

async fn http_connect(s: &mut TcpStream, host: &str, port: u16) -> Result<()> {
    s.write_all(
        format!("CONNECT {host}:{port} HTTP/1.1\r\nHost: {host}:{port}\r\n\r\n").as_bytes(),
    )
    .await?;
    let mut head = Vec::new();
    let mut b = [0u8; 1];
    while !head.ends_with(b"\r\n\r\n") {
        if head.len() > 8192 || s.read(&mut b).await? == 0 {
            bail!("the HTTP proxy closed the connection or sent a bad reply");
        }
        head.push(b[0]);
    }
    let line = String::from_utf8_lossy(&head);
    let line = line.lines().next().unwrap_or_default();
    if line.split_whitespace().nth(1) != Some("200") {
        bail!("the HTTP proxy refused: {line}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::net::TcpListener;

    #[test]
    fn parses_and_rejects() {
        assert_eq!(
            Upstream::parse("socks5://127.0.0.1:1080").unwrap(),
            Upstream::Socks5 {
                addr: "127.0.0.1:1080".into()
            }
        );
        assert_eq!(
            Upstream::parse("http://proxy.local:8080/").unwrap(),
            Upstream::Http {
                addr: "proxy.local:8080".into()
            }
        );
        for bad in [
            "",
            "127.0.0.1:1080",
            "ftp://h:1",
            "socks5://h",
            "socks5://:1",
            "socks5://h:x",
            "http://u:p@h:1",
        ] {
            assert!(Upstream::parse(bad).is_err(), "{bad:?} should fail");
        }
    }

    /// A fake SOCKS5 server that records the requested target and then echoes.
    async fn fake_socks(reply: u8) -> (String, tokio::task::JoinHandle<String>) {
        let l = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = l.local_addr().unwrap().to_string();
        let h = tokio::spawn(async move {
            let (mut s, _) = l.accept().await.unwrap();
            let mut g = [0u8; 3];
            s.read_exact(&mut g).await.unwrap();
            assert_eq!(g, [5, 1, 0]);
            s.write_all(&[5, 0]).await.unwrap();
            let mut h = [0u8; 5];
            s.read_exact(&mut h).await.unwrap();
            assert_eq!(&h[..4], &[5, 1, 0, 3]);
            let mut name = vec![0u8; h[4] as usize + 2];
            s.read_exact(&mut name).await.unwrap();
            let port = u16::from_be_bytes([name[name.len() - 2], name[name.len() - 1]]);
            let host = String::from_utf8(name[..name.len() - 2].to_vec()).unwrap();
            s.write_all(&[5, reply, 0, 1, 0, 0, 0, 0, 0, 0])
                .await
                .unwrap();
            if reply == 0 {
                let mut buf = [0u8; 16];
                let n = s.read(&mut buf).await.unwrap();
                s.write_all(&buf[..n]).await.unwrap();
            }
            format!("{host}:{port}")
        });
        (addr, h)
    }

    #[tokio::test]
    async fn goes_through_a_socks5_server() {
        let (addr, target) = fake_socks(0).await;
        let up = Upstream::parse(&format!("socks5://{addr}")).unwrap();
        let mut s = connect(Some(&up), "discord.com", 443).await.unwrap();
        s.write_all(b"ping").await.unwrap();
        let mut got = [0u8; 4];
        s.read_exact(&mut got).await.unwrap();
        assert_eq!(&got, b"ping");
        assert_eq!(target.await.unwrap(), "discord.com:443");
    }

    #[tokio::test]
    async fn a_refusing_socks5_server_gives_a_clear_error() {
        let (addr, _t) = fake_socks(5).await;
        let up = Upstream::parse(&format!("socks5://{addr}")).unwrap();
        let e = format!(
            "{:#}",
            connect(Some(&up), "discord.com", 443).await.unwrap_err()
        );
        assert!(e.contains("connection refused"), "{e}");
    }

    #[tokio::test]
    async fn a_dead_upstream_gives_a_clear_error() {
        let l = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = l.local_addr().unwrap();
        drop(l);
        let up = Upstream::parse(&format!("http://{addr}")).unwrap();
        let e = format!(
            "{:#}",
            connect(Some(&up), "discord.com", 443).await.unwrap_err()
        );
        assert!(e.contains("cannot reach the upstream proxy"), "{e}");
    }

    #[tokio::test]
    async fn goes_through_an_http_proxy() {
        let l = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = l.local_addr().unwrap();
        let t = tokio::spawn(async move {
            let (mut s, _) = l.accept().await.unwrap();
            let mut buf = [0u8; 512];
            let n = s.read(&mut buf).await.unwrap();
            s.write_all(b"HTTP/1.1 200 Connection established\r\n\r\n")
                .await
                .unwrap();
            String::from_utf8_lossy(&buf[..n])
                .lines()
                .next()
                .unwrap()
                .to_string()
        });
        let up = Upstream::parse(&format!("http://{addr}")).unwrap();
        connect(Some(&up), "discord.com", 443).await.unwrap();
        assert_eq!(t.await.unwrap(), "CONNECT discord.com:443 HTTP/1.1");
    }
}

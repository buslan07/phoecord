//! The local HTTP proxy. Clients send `CONNECT host:443`; Discord's API and gateway hosts are opened up
//! with a certificate from the local CA, everything else (CDN, media, voice, other sites) is tunnelled
//! through byte for byte.

use crate::Proxy;
use crate::gateway;
use crate::rest::{self, Body, Upstream, empty, full};
use anyhow::Result;
use hyper::body::Incoming;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Method, Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;

pub async fn bind(addr: &str) -> Result<TcpListener> {
    Ok(TcpListener::bind(addr).await?)
}

/// `GET` this path on the listener to learn that a PhoeCord proxy (and which version) is there. Like
/// everything the listener serves, it answers loopback peers only.
pub const IDENTITY_PATH: &str = "/__phoecord";

/// What [`IDENTITY_PATH`] answers: `phoecord <version>`.
pub fn identity_body() -> String {
    format!("phoecord {}\n", env!("CARGO_PKG_VERSION"))
}

/// Only this computer may use the proxy.
fn allowed(peer: &SocketAddr) -> bool {
    peer.ip().is_loopback()
}

pub async fn serve(app: Arc<Proxy>, listener: TcpListener) -> Result<()> {
    loop {
        let (sock, peer) = listener.accept().await?;
        if !allowed(&peer) {
            tracing::warn!("refused a connection from {peer}: PhoeCord only serves this computer");
            continue;
        }
        sock.set_nodelay(true).ok();
        let app = app.clone();
        tokio::spawn(async move {
            let svc = service_fn(move |req| front(app.clone(), req, peer));
            if let Err(e) = http1::Builder::new()
                .serve_connection(TokioIo::new(sock), svc)
                .with_upgrades()
                .await
            {
                tracing::debug!("proxy connection: {e}");
            }
        });
    }
}

async fn front(
    app: Arc<Proxy>,
    req: Request<Incoming>,
    _peer: SocketAddr,
) -> Result<Response<Body>, Infallible> {
    if req.method() == Method::GET
        && req.uri().authority().is_none()
        && req.uri().path() == IDENTITY_PATH
    {
        return Ok(Response::new(full(identity_body())));
    }
    if req.method() != Method::CONNECT {
        let mut r = Response::new(full(
            "PhoeCord is an HTTPS proxy: use it as the client's proxy (CONNECT).\n",
        ));
        *r.status_mut() = StatusCode::METHOD_NOT_ALLOWED;
        return Ok(r);
    }
    let Some(auth) = req.uri().authority().cloned() else {
        let mut r = Response::new(empty());
        *r.status_mut() = StatusCode::BAD_REQUEST;
        return Ok(r);
    };
    let host = auth.host().to_string();
    let port = auth.port_u16().unwrap_or(443);
    let intercept = port == 443 && app.cfg().intercepts(&host);
    tokio::spawn(async move {
        let upgraded = match hyper::upgrade::on(req).await {
            Ok(u) => u,
            Err(e) => return tracing::debug!("CONNECT upgrade failed: {e}"),
        };
        let io = TokioIo::new(upgraded);
        let result = if intercept {
            mitm(app, io, host.clone()).await
        } else {
            match app.upstream_route() {
                Ok(route) => tunnel(io, &host, port, route).await,
                Err(e) => Err(e),
            }
        };
        if let Err(e) = result {
            tracing::debug!("{host}:{port}: {e:#}");
        }
    });
    Ok(Response::new(empty()))
}

async fn tunnel(
    mut io: TokioIo<hyper::upgrade::Upgraded>,
    host: &str,
    port: u16,
    route: Option<crate::egress::Upstream>,
) -> Result<()> {
    let mut up = crate::egress::connect(route.as_ref(), host, port).await?;
    up.set_nodelay(true).ok();
    tokio::io::copy_bidirectional(&mut io, &mut up).await?;
    Ok(())
}

async fn mitm(app: Arc<Proxy>, io: TokioIo<hyper::upgrade::Upgraded>, host: String) -> Result<()> {
    let tls = tokio_rustls::TlsAcceptor::from(app.server_tls.clone())
        .accept(io)
        .await?;
    let upstream = Arc::new(Upstream::new(
        host.clone(),
        app.upstream_tls.clone(),
        app.upstream_route()?,
    ));
    let svc = service_fn(move |req: Request<Incoming>| {
        let (app, upstream, host) = (app.clone(), upstream.clone(), host.clone());
        async move {
            if gateway::is_upgrade(&req) {
                Ok::<_, Infallible>(gateway::upgrade(app, req, host).await.unwrap_or_else(|e| {
                    tracing::warn!("gateway: {e:#}");
                    let mut r = Response::new(full("PhoeCord could not reach the gateway"));
                    *r.status_mut() = StatusCode::BAD_GATEWAY;
                    r
                }))
            } else {
                rest::handle(app, upstream, req).await
            }
        }
    });
    http1::Builder::new()
        .serve_connection(TokioIo::new(tls), svc)
        .with_upgrades()
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, Paths};
    use phoecord_plugin_runtime::Chain;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpStream;

    #[test]
    fn only_loopback_peers_are_served() {
        assert!(allowed(&"127.0.0.1:5".parse().unwrap()));
        assert!(allowed(&"[::1]:5".parse().unwrap()));
        assert!(!allowed(&"192.168.1.20:5".parse().unwrap()));
        assert!(!allowed(&"8.8.8.8:443".parse().unwrap()));
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn non_discord_hosts_are_tunnelled_byte_for_byte() {
        // A local echo server stands in for "some other website".
        let echo = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let echo_port = echo.local_addr().unwrap().port();
        tokio::spawn(async move {
            let (mut s, _) = echo.accept().await.unwrap();
            let mut buf = [0u8; 4096];
            loop {
                match s.read(&mut buf).await {
                    Ok(0) | Err(_) => break,
                    Ok(n) => s.write_all(&buf[..n]).await.unwrap(),
                }
            }
        });

        let dir = std::env::temp_dir().join(format!("phoecord-test-server-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let paths = Paths {
            config: dir.join("config.toml"),
            data: dir.join("data"),
        };
        let proxy = Proxy::new(paths, Config::default(), Chain::default(), false).unwrap();
        let listener = bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(serve(proxy, listener));

        let mut c = TcpStream::connect(addr).await.unwrap();
        c.write_all(
            format!(
                "CONNECT 127.0.0.1:{echo_port} HTTP/1.1\r\nHost: 127.0.0.1:{echo_port}\r\n\r\n"
            )
            .as_bytes(),
        )
        .await
        .unwrap();
        let mut head = Vec::new();
        let mut b = [0u8; 1];
        while !head.ends_with(b"\r\n\r\n") {
            let n = tokio::time::timeout(std::time::Duration::from_secs(5), c.read(&mut b))
                .await
                .expect("CONNECT answered")
                .unwrap();
            assert_eq!(n, 1, "proxy closed early");
            head.push(b[0]);
        }
        assert!(
            head.starts_with(b"HTTP/1.1 200"),
            "{}",
            String::from_utf8_lossy(&head)
        );

        let payload: Vec<u8> = (0..=255u8).cycle().take(10_000).collect();
        c.write_all(&payload).await.unwrap();
        let mut got = vec![0u8; payload.len()];
        tokio::time::timeout(std::time::Duration::from_secs(5), c.read_exact(&mut got))
            .await
            .expect("echo came back")
            .unwrap();
        assert_eq!(got, payload);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn plain_requests_are_refused() {
        let dir =
            std::env::temp_dir().join(format!("phoecord-test-server2-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let paths = Paths {
            config: dir.join("config.toml"),
            data: dir.join("data"),
        };
        let proxy = Proxy::new(paths, Config::default(), Chain::default(), false).unwrap();
        let listener = bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(serve(proxy, listener));
        let mut c = TcpStream::connect(addr).await.unwrap();
        c.write_all(b"GET / HTTP/1.1\r\nHost: x\r\n\r\n")
            .await
            .unwrap();
        let mut buf = [0u8; 256];
        let n = tokio::time::timeout(std::time::Duration::from_secs(5), c.read(&mut buf))
            .await
            .expect("proxy answered")
            .unwrap();
        let out = String::from_utf8_lossy(&buf[..n]).into_owned();
        assert!(out.starts_with("HTTP/1.1 405"), "{out}");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn the_listener_says_who_it_is() {
        let dir =
            std::env::temp_dir().join(format!("phoecord-test-server3-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let paths = Paths {
            config: dir.join("config.toml"),
            data: dir.join("data"),
        };
        let proxy = Proxy::new(paths, Config::default(), Chain::default(), false).unwrap();
        let listener = bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(serve(proxy, listener));
        let mut c = TcpStream::connect(addr).await.unwrap();
        c.write_all(
            format!("GET {IDENTITY_PATH} HTTP/1.1\r\nHost: x\r\nConnection: close\r\n\r\n")
                .as_bytes(),
        )
        .await
        .unwrap();
        let mut out = String::new();
        tokio::time::timeout(
            std::time::Duration::from_secs(5),
            c.read_to_string(&mut out),
        )
        .await
        .expect("proxy answered")
        .unwrap();
        assert!(out.starts_with("HTTP/1.1 200"), "{out}");
        assert!(out.ends_with(&identity_body()), "{out}");
    }
}

//! `phoecord selftest`: run the proxy on a spare port and talk to the real Discord through it, without
//! logging in. Checks that interception works, that the CA chain is accepted, and that every gateway
//! encoding and compression survives a decode → re-encode round trip.

use crate::tf;
use anyhow::{Context, Result, bail};
use futures_util::StreamExt;
use http_body_util::BodyExt;
use hyper_util::rt::TokioIo;
use phoecord_i18n::Localizer;
use phoecord_plugin_runtime::Chain;
use phoecord_proxy::Proxy;
use phoecord_proxy::codec::{Codec, Inflater};
use phoecord_proxy::config::{Config, Paths};
use rustls::pki_types::ServerName;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio_rustls::client::TlsStream;
use tokio_tungstenite::tungstenite::Message;

const COMBOS: &[&str] = &[
    "encoding=json&compress=zlib-stream",
    "encoding=json&compress=zstd-stream",
    "encoding=etf&compress=zlib-stream",
    "encoding=etf&compress=zstd-stream",
    "encoding=json",
    "encoding=etf",
];

/// Open a CONNECT tunnel through the proxy and do TLS trusting ONLY the local CA, so the check fails
/// unless PhoeCord really intercepted the connection.
async fn via_proxy(
    proxy: &str,
    host: &str,
    only_ca: Arc<rustls::ClientConfig>,
    l10n: &Localizer,
) -> Result<TlsStream<TcpStream>> {
    let mut s = TcpStream::connect(proxy).await?;
    s.write_all(format!("CONNECT {host}:443 HTTP/1.1\r\nHost: {host}:443\r\n\r\n").as_bytes())
        .await?;
    let mut head = Vec::new();
    let mut b = [0u8; 1];
    while !head.ends_with(b"\r\n\r\n") {
        if s.read(&mut b).await? == 0 {
            bail!(l10n.t("cli-selftest-err-connect-closed"));
        }
        head.push(b[0]);
    }
    if !head.starts_with(b"HTTP/1.1 200") {
        bail!(tf(
            l10n,
            "cli-selftest-err-proxy-answered",
            &[(
                "line",
                String::from_utf8_lossy(&head).lines().next().unwrap_or("")
            )]
        ));
    }
    Ok(tokio_rustls::TlsConnector::from(only_ca)
        .connect(ServerName::try_from(host.to_string())?, s)
        .await?)
}

pub async fn run(paths: Paths, l10n: &Localizer) -> Result<()> {
    // Honour `upstream` from the user's config, so the check also covers a VPN/zapret setup.
    let cfg = Config {
        upstream: Config::load(&paths.config).ok().and_then(|c| c.upstream),
        ..Config::default()
    };
    let app = Proxy::new(paths, cfg, Chain::new(phoecord_plugins::builtin()), true)?;
    let listener = phoecord_proxy::server::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?.to_string();
    tokio::spawn(phoecord_proxy::server::serve(app.clone(), listener));

    let mut roots = rustls::RootCertStore::empty();
    roots.add(app.ca.ca_der.clone())?;
    let mut cc = rustls::ClientConfig::builder_with_provider(Arc::new(
        rustls::crypto::ring::default_provider(),
    ))
    .with_safe_default_protocol_versions()?
    .with_root_certificates(roots)
    .with_no_client_auth();
    cc.alpn_protocols = vec![b"http/1.1".to_vec()];
    let only_ca = Arc::new(cc);
    let mut failed = 0;

    // REST through the proxy.
    let rest = async {
        let tls = via_proxy(&addr, "discord.com", only_ca.clone(), l10n).await?;
        let (mut tx, conn) = hyper::client::conn::http1::handshake(TokioIo::new(tls)).await?;
        tokio::spawn(conn);
        let req = hyper::Request::get("/api/v10/gateway")
            .header("host", "discord.com")
            .header("user-agent", "PhoeCord selftest")
            .body(phoecord_proxy::rest::empty())?;
        let r = tx.send_request(req).await?;
        let status = r.status();
        let body = r.into_body().collect().await?.to_bytes();
        if !status.is_success() || !body.windows(6).any(|w| w == b"wss://") {
            bail!(tf(
                l10n,
                "cli-selftest-err-http",
                &[
                    ("status", &status.to_string()),
                    ("body", &String::from_utf8_lossy(&body))
                ]
            ));
        }
        anyhow::Ok(())
    };
    match tokio::time::timeout(Duration::from_secs(15), rest).await {
        Ok(Ok(())) => println!("{}", l10n.t("cli-selftest-rest-ok")),
        Ok(Err(e)) => {
            failed += 1;
            println!(
                "{}",
                tf(
                    l10n,
                    "cli-selftest-rest-fail",
                    &[("reason", &format!("{e:#}"))]
                )
            )
        }
        Err(_) => {
            failed += 1;
            println!("{}", l10n.t("cli-selftest-rest-timeout"))
        }
    }

    // Gateway: every encoding × compression, decoded by PhoeCord, re-encoded, decoded again here.
    for q in COMBOS {
        let one = async {
            let tls = via_proxy(&addr, "gateway.discord.gg", only_ca.clone(), l10n).await?;
            let url = format!("wss://gateway.discord.gg/?v=10&{q}");
            let (mut ws, _) = tokio_tungstenite::client_async(url, tls)
                .await
                .with_context(|| l10n.t("cli-selftest-err-handshake"))?;
            let codec = Codec::from_query(q)?;
            let mut inflater = Inflater::new(codec.compression)?;
            loop {
                let data = match ws
                    .next()
                    .await
                    .with_context(|| l10n.t("cli-selftest-err-closed-before-hello"))??
                {
                    Message::Binary(b) => b.to_vec(),
                    Message::Text(t) => t.as_bytes().to_vec(),
                    _ => continue,
                };
                if let Some(plain) = inflater.push(&data)? {
                    let v = codec.decode(&plain)?;
                    let op = v.get("op").and_then(|o| o.as_i64());
                    let hb = v.at(&["d", "heartbeat_interval"]).and_then(|h| h.as_i64());
                    let _ = ws.close(None).await;
                    return match (op, hb) {
                        (Some(10), Some(ms)) => anyhow::Ok(ms),
                        _ => bail!(tf(
                            l10n,
                            "cli-selftest-err-expected-hello",
                            &[("op", &format!("{op:?}"))]
                        )),
                    };
                }
            }
        };
        match tokio::time::timeout(Duration::from_secs(15), one).await {
            Ok(Ok(ms)) => println!(
                "{}",
                tf(
                    l10n,
                    "cli-selftest-gateway-ok",
                    &[("combo", q), ("ms", &ms.to_string())]
                )
            ),
            Ok(Err(e)) => {
                failed += 1;
                println!(
                    "{}",
                    tf(
                        l10n,
                        "cli-selftest-gateway-fail",
                        &[("combo", q), ("reason", &format!("{e:#}"))]
                    )
                )
            }
            Err(_) => {
                failed += 1;
                println!(
                    "{}",
                    tf(l10n, "cli-selftest-gateway-timeout", &[("combo", q)])
                )
            }
        }
    }
    let bad = app.stats.undecodable.load(Ordering::Relaxed);
    if bad > 0 {
        failed += 1;
        println!(
            "{}",
            tf(
                l10n,
                "cli-selftest-undecodable",
                &[("count", &bad.to_string())]
            )
        );
    }
    println!(
        "  {}",
        tf(
            l10n,
            "cli-selftest-decoded",
            &[(
                "count",
                &app.stats.decoded.load(Ordering::Relaxed).to_string()
            )]
        )
    );
    if failed > 0 {
        bail!(tf(
            l10n,
            "cli-selftest-failed",
            &[("count", &failed.to_string())]
        ));
    }
    println!("{}", l10n.t("cli-selftest-passed"));
    Ok(())
}

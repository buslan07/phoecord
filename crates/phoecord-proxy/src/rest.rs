//! HTTPS requests to Discord's API (everything that isn't the gateway websocket).

use crate::{Proxy, RequestOutcome};
use anyhow::{Context, Result};
use bytes::Bytes;
use http_body_util::{BodyExt, Empty, Full, combinators::BoxBody};
use hyper::body::Incoming;
use hyper::client::conn::http1::SendRequest;
use hyper::header::{self, HeaderMap, HeaderValue};
use hyper::{Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use phoecord_core::Val;
use phoecord_core::route::route_of;
use rustls::pki_types::ServerName;
use std::convert::Infallible;
use std::io::Read;
use std::sync::Arc;

pub type BoxError = Box<dyn std::error::Error + Send + Sync>;
pub type Body = BoxBody<Bytes, BoxError>;

pub fn full(b: impl Into<Bytes>) -> Body {
    Full::new(b.into()).map_err(|e| match e {}).boxed()
}

pub fn empty() -> Body {
    Empty::new().map_err(|e| match e {}).boxed()
}

pub fn incoming(b: Incoming) -> Body {
    b.map_err(|e| Box::new(e) as BoxError).boxed()
}

const HOP_BY_HOP: &[&str] = &[
    "connection",
    "keep-alive",
    "proxy-connection",
    "proxy-authenticate",
    "proxy-authorization",
    "te",
    "trailer",
    "transfer-encoding",
    "upgrade",
];

fn strip_hop_by_hop(h: &mut HeaderMap) {
    for name in HOP_BY_HOP {
        h.remove(*name);
    }
}

/// The `payload_json` part of a `multipart/form-data` body: how the client sends slash command
/// interactions. `None` when the content type or the part is missing, or the part is not JSON.
pub fn multipart_payload_json(content_type: &str, body: &[u8]) -> Option<Val> {
    let ct = content_type.trim();
    if !ct.to_ascii_lowercase().starts_with("multipart/form-data") {
        return None;
    }
    let boundary = ct.split(';').skip(1).find_map(|p| {
        let (k, v) = p.trim().split_once('=')?;
        k.eq_ignore_ascii_case("boundary")
            .then(|| v.trim_matches('"').to_string())
    })?;
    let delim = format!("--{boundary}");
    let text = String::from_utf8_lossy(body);
    text.split(delim.as_str()).find_map(|part| {
        let (head, data) = part.split_once("\r\n\r\n")?;
        if !head.contains("name=\"payload_json\"") {
            return None;
        }
        let json = data.strip_suffix("\r\n").unwrap_or(data);
        serde_json::from_str(json).ok().map(Val::from_json)
    })
}

fn is_json(h: &HeaderMap) -> bool {
    h.get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v.starts_with("application/json"))
}

/// One kept-alive HTTPS connection to the real host, opened on first use and re-opened when it drops.
pub struct Upstream {
    host: String,
    tls: Arc<rustls::ClientConfig>,
    route: Option<crate::egress::Upstream>,
    sender: tokio::sync::Mutex<Option<SendRequest<Body>>>,
}

impl Upstream {
    pub fn new(
        host: String,
        tls: Arc<rustls::ClientConfig>,
        route: Option<crate::egress::Upstream>,
    ) -> Upstream {
        Upstream {
            host,
            tls,
            route,
            sender: tokio::sync::Mutex::new(None),
        }
    }

    async fn connect(&self) -> Result<SendRequest<Body>> {
        let tcp = crate::egress::connect(self.route.as_ref(), &self.host, 443)
            .await
            .with_context(|| format!("connecting to {}", self.host))?;
        tcp.set_nodelay(true).ok();
        let name = ServerName::try_from(self.host.clone())?;
        let tls = tokio_rustls::TlsConnector::from(self.tls.clone())
            .connect(name, tcp)
            .await?;
        let (sender, conn) = hyper::client::conn::http1::handshake(TokioIo::new(tls)).await?;
        tokio::spawn(async move {
            if let Err(e) = conn.await {
                tracing::debug!("upstream connection closed: {e}");
            }
        });
        Ok(sender)
    }

    async fn send(&self, req: Request<Body>) -> Result<Response<Incoming>> {
        let mut guard = self.sender.lock().await;
        if guard.as_ref().is_none_or(|s| s.is_closed()) {
            *guard = Some(self.connect().await?);
        }
        let s = guard.as_mut().unwrap();
        if s.ready().await.is_err() {
            *guard = Some(self.connect().await?);
        }
        Ok(guard.as_mut().unwrap().send_request(req).await?)
    }
}

/// A response made here instead of by Discord: a plugin's `Reply` (JSON body optional).
fn local_reply(status: u16, body: Option<Val>) -> Response<Body> {
    let mut r = match body.map(|v| serde_json::to_vec(&v.to_json())) {
        Some(Ok(b)) => {
            let mut r = Response::new(full(b));
            r.headers_mut().insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static("application/json"),
            );
            r
        }
        _ => Response::new(empty()),
    };
    *r.status_mut() = StatusCode::from_u16(status).unwrap_or(StatusCode::NO_CONTENT);
    r
}

pub async fn handle(
    app: Arc<Proxy>,
    up: Arc<Upstream>,
    req: Request<Incoming>,
) -> Result<Response<Body>, Infallible> {
    let what = format!("{} {}", req.method(), req.uri().path());
    match forward(app, up, req).await {
        Ok(r) => Ok(r),
        Err(e) => {
            tracing::warn!("{what}: {e:#}");
            let mut r = Response::new(full("PhoeCord could not reach Discord"));
            *r.status_mut() = StatusCode::BAD_GATEWAY;
            Ok(r)
        }
    }
}

async fn forward(
    app: Arc<Proxy>,
    up: Arc<Upstream>,
    req: Request<Incoming>,
) -> Result<Response<Body>> {
    let (mut parts, body) = req.into_parts();
    let method = parts.method.as_str().to_string();
    let route = route_of(parts.uri.path());

    let mut kept: Option<Bytes> = None;
    let mut stream = Some(body);
    let mut json = None;
    let mut multipart = false;
    if app.wants_request(&method, &route) {
        let ct = parts
            .headers
            .get(header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();
        if is_json(&parts.headers) {
            let b = stream.take().unwrap().collect().await?.to_bytes();
            json = serde_json::from_slice(&b).ok().map(Val::from_json);
            kept = Some(b);
        } else if method == "POST" && route == "/interactions" && ct.starts_with("multipart/") {
            // Slash command interactions: plugins read `payload_json`; the body is never edited.
            let b = stream.take().unwrap().collect().await?.to_bytes();
            json = multipart_payload_json(&ct, &b);
            multipart = true;
            kept = Some(b);
        }
    }

    let changed = match app.on_request(&method, &route, &mut json) {
        RequestOutcome::Reply { status, body } => return Ok(local_reply(status, body)),
        RequestOutcome::Forward { changed } => changed,
    };

    let out: Body = match (changed && !multipart, json, kept, stream) {
        (true, Some(v), _, _) => {
            let b = serde_json::to_vec(&v.to_json())?;
            parts
                .headers
                .insert(header::CONTENT_LENGTH, HeaderValue::from(b.len()));
            full(b)
        }
        (_, _, Some(b), _) => full(b),
        (_, _, None, Some(s)) => incoming(s),
        _ => empty(),
    };

    strip_hop_by_hop(&mut parts.headers);
    let pq = parts
        .uri
        .path_and_query()
        .map(|p| p.as_str().to_string())
        .unwrap_or_else(|| "/".into());
    parts.uri = pq.parse()?;
    let wants_response = app.wants_response(&method, &route);
    if wants_response {
        parts
            .headers
            .insert(header::ACCEPT_ENCODING, HeaderValue::from_static("gzip"));
    }

    let resp = up.send(Request::from_parts(parts, out)).await?;
    let (mut rp, rb) = resp.into_parts();
    let status = rp.status.as_u16();
    strip_hop_by_hop(&mut rp.headers);

    if wants_response && rp.status.is_success() && is_json(&rp.headers) {
        let raw = rb.collect().await?.to_bytes();
        let gz = rp
            .headers
            .get(header::CONTENT_ENCODING)
            .is_some_and(|v| v.as_bytes().eq_ignore_ascii_case(b"gzip"));
        let plain = if gz {
            let mut out = Vec::new();
            flate2::read::GzDecoder::new(&raw[..]).read_to_end(&mut out)?;
            Bytes::from(out)
        } else {
            raw
        };
        let mut plain = plain;
        if let Ok(j) = serde_json::from_slice(&plain) {
            let mut v = Val::from_json(j);
            if app.on_response(&method, &route, status, &mut v) {
                plain = Bytes::from(serde_json::to_vec(&v.to_json())?);
            }
        }
        rp.headers.remove(header::CONTENT_ENCODING);
        rp.headers
            .insert(header::CONTENT_LENGTH, HeaderValue::from(plain.len()));
        return Ok(Response::from_parts(rp, full(plain)));
    }
    Ok(Response::from_parts(rp, incoming(rb)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_payload_json_from_multipart() {
        let body = "--xyz\r\nContent-Disposition: form-data; name=\"payload_json\"\r\n\r\n{\"type\":2,\"nonce\":\"1\"}\r\n--xyz--\r\n";
        let v = multipart_payload_json("multipart/form-data; boundary=xyz", body.as_bytes())
            .expect("payload_json");
        assert_eq!(v.str_at(&["nonce"]), Some("1"));
        assert!(multipart_payload_json("application/json", body.as_bytes()).is_none());
        assert!(
            multipart_payload_json("multipart/form-data; boundary=zzz", body.as_bytes()).is_none()
        );
        let other =
            "--xyz\r\nContent-Disposition: form-data; name=\"file\"\r\n\r\nabc\r\n--xyz--\r\n";
        assert!(
            multipart_payload_json("multipart/form-data; boundary=xyz", other.as_bytes()).is_none()
        );
    }
}

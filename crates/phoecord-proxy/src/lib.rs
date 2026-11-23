//! PhoeCord network layer: the local MITM proxy between the client and Discord.
//!
//! [`Proxy`] owns the live config, the account state, the plugin [`Chain`] and the TLS setup.
//! [`server`] accepts `CONNECT`, [`rest`] and [`gateway`] run Discord's traffic through the chain.

pub mod account;
pub mod ca;
pub mod codec;
pub mod config;
pub mod egress;
pub mod gateway;
pub mod host;
pub mod rest;
pub mod server;
pub mod trust;

use anyhow::Result;
use ca::Authority;
use config::{Config, Paths};
use host::{ProxyHost, Shared};
use phoecord_core::{EventKind, Val};
use phoecord_plugin_runtime::package::PackageSet;
use phoecord_plugin_runtime::{Chain, Outcome};
use std::sync::atomic::AtomicU64;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, SystemTime};
use tokio::sync::{broadcast, watch};

#[derive(Default)]
pub struct Stats {
    pub decoded: AtomicU64,
    pub undecodable: AtomicU64,
}

/// What the chain decided about a REST request.
#[derive(Debug, PartialEq)]
pub enum RequestOutcome {
    /// Forward it; `changed` says the body must be re-serialized.
    Forward { changed: bool },
    /// Answer locally, the request never reaches Discord.
    Reply { status: u16, body: Option<Val> },
}

/// What the chain decided about a server → client gateway payload.
#[derive(Debug)]
pub struct DispatchResult {
    pub payload: Val,
    pub extra: Vec<Val>,
    pub changed: bool,
    pub drop: bool,
    /// Client → server payloads plugins queued with `Host::emit_send`.
    pub sends: Vec<(i64, Val)>,
}

/// What the chain decided about a client → server gateway payload.
#[derive(Debug)]
pub struct SendResult {
    pub changed: bool,
    pub drop: bool,
    pub sends: Vec<(i64, Val)>,
}

pub struct Proxy {
    pub paths: Paths,
    config: RwLock<Arc<Config>>,
    pub shared: Shared,
    pub chain: Chain,
    /// Keeps the chain's dynamic segment in step with `data/plugins/` (WASM packages).
    packages: Mutex<Option<PackageSet>>,
    pub ca: Arc<Authority>,
    pub server_tls: Arc<rustls::ServerConfig>,
    pub upstream_tls: Arc<rustls::ClientConfig>,
    /// Bumped on every config reload.
    pub reloads: watch::Sender<u64>,
    /// Server → client payloads made locally (`Host::emit_dispatch` from a REST event); every
    /// gateway connection sends what it receives to its client.
    pub locals: broadcast::Sender<Val>,
    /// Decode (and re-encode) the gateway even when no plugin needs it (self-test).
    pub force_decode: bool,
    pub stats: Stats,
}

pub fn upstream_tls(
    extra_root: Option<rustls::pki_types::CertificateDer<'static>>,
) -> Arc<rustls::ClientConfig> {
    let mut roots = rustls::RootCertStore::empty();
    roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    if let Some(r) = extra_root {
        let _ = roots.add(r);
    }
    let mut c = rustls::ClientConfig::builder_with_provider(Arc::new(
        rustls::crypto::ring::default_provider(),
    ))
    .with_safe_default_protocol_versions()
    .expect("TLS versions")
    .with_root_certificates(roots)
    .with_no_client_auth();
    c.alpn_protocols = vec![b"http/1.1".to_vec()];
    Arc::new(c)
}

impl Proxy {
    pub fn new(paths: Paths, cfg: Config, chain: Chain, force_decode: bool) -> Result<Arc<Proxy>> {
        let ca = Arc::new(Authority::load_or_create(&paths.data)?);
        let server_tls = ca.server_config();
        let mut shared = Shared::new(paths.data.join("plugins"));
        shared.config_path = paths.config.clone();
        let proxy = Arc::new(Proxy {
            paths,
            config: RwLock::new(Arc::new(Config::default())),
            shared,
            chain,
            packages: Mutex::new(None),
            ca,
            server_tls,
            upstream_tls: upstream_tls(None),
            reloads: watch::channel(0).0,
            locals: broadcast::channel(64).0,
            force_decode,
            stats: Stats::default(),
        });
        proxy.apply(cfg);
        Ok(proxy)
    }

    pub fn cfg(&self) -> Arc<Config> {
        self.config.read().unwrap().clone()
    }

    /// The `upstream` proxy from the live config; `None` means connect directly.
    pub fn upstream_route(&self) -> anyhow::Result<Option<egress::Upstream>> {
        self.cfg()
            .upstream
            .as_deref()
            .filter(|s| !s.trim().is_empty())
            .map(egress::Upstream::parse)
            .transpose()
    }

    /// Make `cfg` the live config and switch the chain to the plugins it enables.
    pub fn apply(&self, cfg: Config) {
        self.set_enabled_from(&cfg);
        *self.config.write().unwrap() = Arc::new(cfg);
    }

    fn set_enabled_from(&self, cfg: &Config) {
        let enabled: Vec<String> = self
            .chain
            .ids()
            .into_iter()
            .filter(|id| cfg.is_enabled(id))
            .collect();
        self.chain.set_enabled(enabled);
    }

    /// Follow `data/plugins/` with `packages`: load what is new now and on every later
    /// [`Proxy::sync_packages`].
    pub fn watch_packages(&self, packages: PackageSet) {
        *self.packages.lock().unwrap() = Some(packages);
        self.sync_packages();
    }

    /// Load new packages and unload removed ones, then switch the chain to what the config
    /// enables. Does nothing when no packages are watched or nothing changed. Compiling a WASM
    /// package takes a while, so call this off the async executor's threads.
    pub fn sync_packages(&self) {
        let logs = match self.packages.lock().unwrap().as_mut() {
            Some(set) => set.sync(&self.chain),
            None => return,
        };
        if logs.is_empty() {
            return;
        }
        for line in &logs {
            tracing::info!("{line}");
        }
        self.set_enabled_from(&self.cfg());
        // Gateway tasks replay their state through the chain when this changes.
        self.reloads.send_modify(|n| *n += 1);
    }

    /// Re-read config.toml whenever it changes (checked every 2 s, together with the `plugins/` folder). A broken file keeps the old config.
    pub fn spawn_reloader(self: &Arc<Self>) {
        let proxy = self.clone();
        tokio::spawn(async move {
            let mtime = |p: &std::path::Path| std::fs::metadata(p).and_then(|m| m.modified()).ok();
            let mut last: Option<SystemTime> = mtime(&proxy.paths.config);
            loop {
                tokio::time::sleep(Duration::from_secs(2)).await;
                let p = proxy.clone();
                let _ = tokio::task::spawn_blocking(move || p.sync_packages()).await;
                let now = mtime(&proxy.paths.config);
                if now == last {
                    continue;
                }
                last = now;
                match Config::load(&proxy.paths.config) {
                    Ok(c) => {
                        proxy.apply(c);
                        proxy.reloads.send_modify(|n| *n += 1);
                        tracing::info!("config reloaded");
                    }
                    Err(e) => tracing::warn!("config not reloaded: {e:#}"),
                }
            }
        });
    }

    pub fn gateway_active(&self) -> bool {
        self.force_decode || self.chain.wants_gateway()
    }

    pub fn wants_request(&self, method: &str, route: &str) -> bool {
        self.chain.wants(&EventKind::RestRequest { method, route })
    }

    pub fn wants_response(&self, method: &str, route: &str) -> bool {
        self.chain.wants(&EventKind::RestResponse { method, route })
    }

    pub fn on_request(&self, method: &str, route: &str, body: &mut Option<Val>) -> RequestOutcome {
        let cfg = self.cfg();
        self.shared
            .account
            .lock()
            .unwrap()
            .record_request(method, route, body.as_ref());
        let mut host = ProxyHost::new(&self.shared, &cfg);
        let outcome = self
            .chain
            .rest_request(&mut host, method, route, body.as_mut());
        self.publish(host.locals);
        match outcome {
            Outcome::Untouched => RequestOutcome::Forward { changed: false },
            Outcome::Changed => RequestOutcome::Forward { changed: true },
            Outcome::Replied { by, status, body } => {
                tracing::debug!("{by}: answered {method} {route} locally ({status})");
                RequestOutcome::Reply { status, body }
            }
            Outcome::Dropped { by } => {
                tracing::debug!("{by}: dropped {method} {route}");
                RequestOutcome::Reply {
                    status: 204,
                    body: None,
                }
            }
        }
    }

    /// A REST answer: plugins may rewrite `body`; returns whether they did.
    pub fn on_response(&self, method: &str, route: &str, status: u16, body: &mut Val) -> bool {
        let cfg = self.cfg();
        let mut host = ProxyHost::new(&self.shared, &cfg);
        let outcome = self
            .chain
            .rest_response(&mut host, method, route, status, body);
        self.publish(host.locals);
        outcome == Outcome::Changed
    }

    /// Hand local server → client payloads to the gateway connections (none open: dropped).
    fn publish(&self, locals: Vec<Val>) {
        for v in locals {
            let _ = self.locals.send(v);
        }
    }

    /// A server → client payload: learn from it, then let plugins change it.
    pub fn on_dispatch(&self, mut payload: Val) -> DispatchResult {
        let cfg = self.cfg();
        let mut extra = Vec::new();
        let name = if payload.get("op").and_then(Val::as_i64) == Some(0) {
            payload.str_at(&["t"]).map(str::to_string)
        } else {
            None
        };
        let Some(name) = name else {
            return DispatchResult {
                payload,
                extra,
                changed: false,
                drop: false,
                sends: Vec::new(),
            };
        };
        if let Some(d) = payload.get("d") {
            self.shared.account.lock().unwrap().track(&name, d);
        }
        let mut host = ProxyHost::new(&self.shared, &cfg);
        let outcome = self
            .chain
            .dispatch(&mut host, &name, &mut payload, &mut extra);
        let (changed, drop) = match outcome {
            Outcome::Untouched => (false, false),
            Outcome::Changed => (true, false),
            Outcome::Dropped { .. } | Outcome::Replied { .. } => (false, true),
        };
        extra.append(&mut host.locals);
        DispatchResult {
            payload,
            extra,
            changed,
            drop,
            sends: host.sends,
        }
    }

    /// A client → server payload.
    pub fn on_send(&self, payload: &mut Val) -> SendResult {
        let cfg = self.cfg();
        let Some(op) = payload.get("op").and_then(Val::as_i64) else {
            return SendResult {
                changed: false,
                drop: false,
                sends: Vec::new(),
            };
        };
        let mut host = ProxyHost::new(&self.shared, &cfg);
        let outcome = self.chain.send(&mut host, op, payload);
        self.publish(std::mem::take(&mut host.locals));
        SendResult {
            changed: outcome == Outcome::Changed,
            drop: matches!(outcome, Outcome::Dropped { .. } | Outcome::Replied { .. }),
            sends: host.sends,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use phoecord_core::plugin::API_VERSION;
    use phoecord_core::{Event, Host, Layer, Manifest, Plugin, Verdict};

    /// Rewrites MESSAGE_CREATE content and queues a presence re-send.
    struct Rewrite(Manifest);

    impl Plugin for Rewrite {
        fn manifest(&self) -> &Manifest {
            &self.0
        }
        fn wants(&self, e: &EventKind) -> bool {
            matches!(
                e,
                EventKind::Dispatch {
                    name: "MESSAGE_CREATE"
                }
            )
        }
        fn wants_gateway(&self) -> bool {
            true
        }
        fn on_event(&self, host: &mut dyn Host, e: Event<'_>) -> Verdict {
            if let Event::Dispatch { payload, .. } = e
                && let Some(d) = payload.get_mut("d")
            {
                d.set("content", Val::Str("rewritten".into()));
                host.emit_send(3, Val::map());
                return Verdict::Changed;
            }
            Verdict::Pass
        }
    }

    fn test_proxy(on: bool) -> Arc<Proxy> {
        static N: AtomicU64 = AtomicU64::new(0);
        let n = N.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let dir =
            std::env::temp_dir().join(format!("phoecord-test-proxy-{}-{n}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let paths = Paths {
            config: dir.join("config.toml"),
            data: dir.join("data"),
        };
        let chain = Chain::new(vec![
            Box::new(phoecord_plugins::no_track::NoTrack::new()),
            Box::new(Rewrite(Manifest {
                id: "rewrite".into(),
                name: "Rewrite".into(),
                version: "0".into(),
                description: String::new(),
                authors: vec![],
                vencord: None,
                layers: vec![Layer::Network],
                api: API_VERSION,
            })),
        ]);
        let mut cfg = Config::default();
        cfg.set_enabled("rewrite", on).unwrap();
        Proxy::new(paths, cfg, chain, false).unwrap()
    }

    fn msg() -> Val {
        Val::from_json(
            serde_json::from_str(
                r#"{"op":0,"t":"MESSAGE_CREATE","s":1,"d":{"content":"hi","channel_id":"5"}}"#,
            )
            .unwrap(),
        )
    }

    #[test]
    fn no_track_answers_science_locally() {
        let p = test_proxy(false);
        assert!(p.wants_request("POST", "/science"));
        assert!(!p.wants_request("GET", "/users/@me"));
        let mut body = None;
        assert_eq!(
            p.on_request("POST", "/science", &mut body),
            RequestOutcome::Reply {
                status: 204,
                body: None
            }
        );
        assert_eq!(
            p.on_request("GET", "/users/@me", &mut body),
            RequestOutcome::Forward { changed: false }
        );
    }

    #[test]
    fn dispatch_goes_through_the_chain_only_when_enabled() {
        let off = test_proxy(false);
        assert!(!off.gateway_active());
        let r = off.on_dispatch(msg());
        assert!(!r.changed && r.sends.is_empty());

        let on = test_proxy(true);
        assert!(on.gateway_active());
        let r = on.on_dispatch(msg());
        assert!(r.changed && !r.drop);
        assert_eq!(r.payload.str_at(&["d", "content"]), Some("rewritten"));
        assert_eq!(r.sends.len(), 1);
        // Account learned the channel's guild-less message without choking.
        assert!(on.shared.account.lock().unwrap().user_id.is_none());
    }

    #[test]
    fn reload_switches_plugins_and_keeps_the_old_config_on_error() {
        let p = test_proxy(false);
        assert_eq!(p.chain.enabled_ids(), vec!["no_track".to_string()]);
        let mut cfg = (*p.cfg()).clone();
        cfg.set_enabled("rewrite", true).unwrap();
        p.apply(cfg);
        assert_eq!(p.chain.enabled_ids().len(), 2);
        std::fs::create_dir_all(p.paths.config.parent().unwrap()).unwrap();
        std::fs::write(&p.paths.config, "listen = [").unwrap();
        assert!(Config::load(&p.paths.config).is_err());
        assert_eq!(
            p.chain.enabled_ids().len(),
            2,
            "a broken file changes nothing"
        );
    }
}

//! `config.toml`: listen address, which hosts to intercept and one free table per plugin.
//! Reloaded live by [`crate::Proxy::spawn_reloader`].

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Plugins that are on until the user switches them off. Everything else is off by default.
pub const DEFAULT_ON: &[&str] = &["no_track"];

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Where the local proxy listens. Keep it on 127.0.0.1.
    pub listen: String,
    /// Hosts whose HTTPS traffic PhoeCord opens up (`*.` and `-*` wildcards allowed). Everything else
    /// is tunnelled through untouched, e.g. the CDN, media and voice.
    pub intercept: Vec<String>,
    /// Send every outgoing connection through this proxy (`socks5://host:port` or `http://host:port`),
    /// for VPN and zapret users. Direct when unset; applies to new connections without a restart.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub upstream: Option<String>,
    /// UI language (`en`, `ru`); the OS language when unset.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    /// One table per plugin id; `enabled` is a key inside it, the rest are the plugin's own settings.
    pub plugins: BTreeMap<String, toml::Table>,
    /// Anything else in the file is kept as it is when the config is saved.
    #[serde(flatten)]
    pub extra: toml::Table,
}

impl Default for Config {
    fn default() -> Self {
        let mut plugins = BTreeMap::new();
        for id in DEFAULT_ON {
            let mut t = toml::Table::new();
            t.insert("enabled".into(), true.into());
            plugins.insert((*id).to_string(), t);
        }
        Config {
            listen: "127.0.0.1:18463".into(),
            intercept: vec![
                "discord.com".into(),
                "canary.discord.com".into(),
                "ptb.discord.com".into(),
                "discordapp.com".into(),
                "gateway.discord.gg".into(),
                "gateway-*.discord.gg".into(),
            ],
            language: None,
            upstream: None,
            plugins,
            extra: toml::Table::new(),
        }
    }
}

pub struct Paths {
    pub config: PathBuf,
    pub data: PathBuf,
}

impl Paths {
    /// `PHOECORD_CONFIG` / `PHOECORD_DATA`, else the OS config and data directories.
    pub fn resolve() -> Paths {
        let config = std::env::var_os("PHOECORD_CONFIG")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                dirs::config_dir()
                    .unwrap_or_else(|| PathBuf::from("."))
                    .join("phoecord")
                    .join("config.toml")
            });
        let data = std::env::var_os("PHOECORD_DATA")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                dirs::data_dir()
                    .unwrap_or_else(|| PathBuf::from("."))
                    .join("phoecord")
            });
        Paths { config, data }
    }

    /// `data/plugins/<id>/`
    pub fn plugin_dir(&self, id: &str) -> PathBuf {
        self.data.join("plugins").join(id)
    }
}

impl Config {
    /// Load the config, writing the defaults first if there is none.
    pub fn load_or_init(path: &Path) -> Result<Config> {
        if !path.exists() {
            let c = Config::default();
            c.save(path)?;
            return Ok(c);
        }
        Config::load(path)
    }

    pub fn load(path: &Path) -> Result<Config> {
        let text =
            std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
        toml::from_str(&text).with_context(|| format!("parsing {}", path.display()))
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let body = toml::to_string_pretty(self)?;
        let text = format!(
            "# PhoeCord config. Changes apply live (no restart), except `listen`.\n\
             # Plugins: `phoecord plugins`; switch with `phoecord enable NAME` / `phoecord disable NAME`.\n\n{body}"
        );
        let tmp = path.with_extension("toml.tmp");
        std::fs::write(&tmp, text)?;
        std::fs::rename(&tmp, path)?;
        Ok(())
    }

    /// Whether a plugin runs: its `enabled` key, else the default (only [`DEFAULT_ON`] are on).
    pub fn is_enabled(&self, id: &str) -> bool {
        self.plugins
            .get(id)
            .and_then(|t| t.get("enabled"))
            .and_then(|v| v.as_bool())
            .unwrap_or_else(|| DEFAULT_ON.contains(&id))
    }

    pub fn set_enabled(&mut self, id: &str, on: bool) -> Result<()> {
        if id.is_empty() || !id.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            bail!("invalid plugin id {id:?}");
        }
        self.plugins
            .entry(id.to_string())
            .or_default()
            .insert("enabled".into(), on.into());
        Ok(())
    }

    pub fn intercepts(&self, host: &str) -> bool {
        let host = host.trim_end_matches('.').to_ascii_lowercase();
        self.intercept.iter().any(|p| host_matches(p, &host))
    }
}

/// `discord.com`, `*.discord.com` (subdomains only) or `gateway-*.discord.gg` (one label wildcard).
pub fn host_matches(pattern: &str, host: &str) -> bool {
    let p = pattern.to_ascii_lowercase();
    if let Some(suffix) = p.strip_prefix("*.") {
        return host.ends_with(&format!(".{suffix}"));
    }
    if let Some((pre, post)) = p.split_once('*') {
        return host.len() > pre.len() + post.len()
            && host.starts_with(pre)
            && host.ends_with(post)
            && !host[pre.len()..host.len() - post.len()].contains('.');
    }
    p == host
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("phoecord-test-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn host_patterns() {
        assert!(host_matches("discord.com", "discord.com"));
        assert!(!host_matches("discord.com", "cdn.discord.com"));
        assert!(host_matches("*.discord.com", "canary.discord.com"));
        assert!(!host_matches("*.discord.com", "discord.com"));
        assert!(host_matches(
            "gateway-*.discord.gg",
            "gateway-us-east1-c.discord.gg"
        ));
        assert!(!host_matches(
            "gateway-*.discord.gg",
            "gateway-x.evil.discord.gg"
        ));
        let c = Config::default();
        assert!(c.intercepts("Gateway.Discord.gg."));
        assert!(!c.intercepts("cdn.discordapp.com"));
        assert!(!c.intercepts("media.discordapp.net"));
    }

    #[test]
    fn defaults_and_toggles_roundtrip() {
        let mut c = Config::default();
        assert!(c.is_enabled("no_track"));
        assert!(!c.is_enabled("message_logger"));
        c.set_enabled("message_logger", true).unwrap();
        assert!(c.set_enabled("../x", true).is_err());
        let text = toml::to_string_pretty(&c).unwrap();
        let back: Config = toml::from_str(&text).unwrap();
        assert!(back.is_enabled("message_logger") && back.is_enabled("no_track"));
        // A config that leaves things out gets the defaults.
        let sparse: Config = toml::from_str("[plugins.unindent]\nenabled = true\n").unwrap();
        assert!(sparse.is_enabled("no_track") && sparse.is_enabled("unindent"));
        assert_eq!(sparse.listen, Config::default().listen);
        // An explicit `enabled = false` beats the default.
        let off: Config = toml::from_str("[plugins.no_track]\nenabled = false\n").unwrap();
        assert!(!off.is_enabled("no_track"));
    }

    #[test]
    fn save_keeps_unknown_sections_and_plugin_settings() {
        let dir = temp("cfg-keep");
        let path = dir.join("config.toml");
        std::fs::write(
            &path,
            "future = 7\nlanguage = \"ru\"\n[plugins.fake_nitro]\nemoji_size = 64\n[other]\nx = 1\n",
        )
        .unwrap();
        let mut c = Config::load_or_init(&path).unwrap();
        assert_eq!(c.language.as_deref(), Some("ru"));
        c.set_enabled("fake_nitro", true).unwrap();
        c.save(&path).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(text.contains("future = 7"), "{text}");
        assert!(text.contains("[other]"), "{text}");
        assert!(text.contains("emoji_size = 64"), "{text}");
        let back = Config::load(&path).unwrap();
        assert!(back.is_enabled("fake_nitro"));
        assert_eq!(
            back.plugins["fake_nitro"]["emoji_size"].as_integer(),
            Some(64)
        );
    }

    #[test]
    fn broken_file_is_an_error_and_missing_file_gets_defaults() {
        let dir = temp("cfg-broken");
        let path = dir.join("config.toml");
        let c = Config::load_or_init(&path).unwrap();
        assert!(path.exists() && c.is_enabled("no_track"));
        std::fs::write(&path, "listen = [").unwrap();
        assert!(Config::load(&path).is_err());
    }
}

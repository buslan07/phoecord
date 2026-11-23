//! Import a Vencord user's plugin choices (task 1.2.9). Reads the `settings.json` of Vencord, Vesktop
//! or Equicord and works out which PhoeCord plugins match the Vencord plugins that are switched on.
//! Only names are mapped (`Manifest::vencord`); plugin settings are left alone.

use anyhow::{Context, Result, bail};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// Where the Vencord family keeps `settings.json`, relative to the OS config directory.
const FOLDERS: &[(&str, &str)] = &[
    ("Vencord", "Vencord"),
    ("Vesktop", "vesktop"),
    ("Equicord", "Equicord"),
    ("Vesktop", "Vesktop"),
];

/// Candidate `settings.json` files that exist, as `(client name, path)`. `config` is the OS config
/// directory (`%APPDATA%`, `~/.config`, `~/Library/Application Support`); `home` adds the Flatpak
/// location of Vesktop.
pub fn find(config: &Path, home: Option<&Path>) -> Vec<(&'static str, PathBuf)> {
    let mut out = Vec::new();
    for (name, dir) in FOLDERS {
        let p = config.join(dir).join("settings").join("settings.json");
        if p.is_file() && !out.iter().any(|(_, q)| q == &p) {
            out.push((*name, p));
        }
    }
    if let Some(home) = home {
        let p = home.join(".var/app/dev.vencord.Vesktop/config/vesktop/settings/settings.json");
        if p.is_file() {
            out.push(("Vesktop", p));
        }
    }
    out
}

/// Names of the Vencord plugins switched on in a `settings.json` text.
pub fn enabled_plugins(text: &str) -> Result<BTreeSet<String>> {
    let v: serde_json::Value =
        serde_json::from_str(text).context("settings.json is not valid JSON")?;
    let Some(plugins) = v.get("plugins").and_then(|p| p.as_object()) else {
        bail!("settings.json has no \"plugins\" section");
    };
    Ok(plugins
        .iter()
        .filter(|(_, s)| s.get("enabled").and_then(|e| e.as_bool()) == Some(true))
        .map(|(name, _)| name.clone())
        .collect())
}

/// What an import would do.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Plan {
    /// PhoeCord plugin ids to switch on.
    pub enable: Vec<String>,
    /// Of those, the ones that were already on.
    pub already_on: Vec<String>,
    /// Vencord plugins that are on but have no PhoeCord counterpart.
    pub unmatched: Vec<String>,
}

/// Match the enabled Vencord plugins against `(id, Manifest::vencord, currently on)` rows.
pub fn plan(enabled: &BTreeSet<String>, known: &[(String, Option<String>, bool)]) -> Plan {
    let mut p = Plan::default();
    let mut matched = BTreeSet::new();
    for (id, vencord, on) in known {
        let Some(v) = vencord else { continue };
        if enabled.contains(v) {
            matched.insert(v.clone());
            if *on {
                p.already_on.push(id.clone());
            } else {
                p.enable.push(id.clone());
            }
        }
    }
    p.unmatched = enabled.difference(&matched).cloned().collect();
    p
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_enabled_plugins_only() {
        let t = r#"{"plugins":{"NoTrack":{"enabled":true},"ClearURLs":{"enabled":false},
            "FakeNitro":{"enabled":true,"x":1},"Settings":{"someKey":1}}}"#;
        let e = enabled_plugins(t).unwrap();
        assert_eq!(e.into_iter().collect::<Vec<_>>(), ["FakeNitro", "NoTrack"]);
    }

    #[test]
    fn bad_files_give_errors() {
        assert!(enabled_plugins("not json").is_err());
        assert!(enabled_plugins("{}").is_err());
    }

    #[test]
    fn plans_by_vencord_name() {
        let enabled: BTreeSet<String> = ["NoTrack", "FakeNitro", "Whatever"]
            .map(String::from)
            .into();
        let known = vec![
            ("no_track".to_string(), Some("NoTrack".to_string()), true),
            (
                "fake_nitro".to_string(),
                Some("FakeNitro".to_string()),
                false,
            ),
            (
                "clear_urls".to_string(),
                Some("ClearURLs".to_string()),
                false,
            ),
            ("message_logger".to_string(), None, false),
        ];
        let p = plan(&enabled, &known);
        assert_eq!(p.enable, ["fake_nitro"]);
        assert_eq!(p.already_on, ["no_track"]);
        assert_eq!(p.unmatched, ["Whatever"]);
    }

    #[test]
    fn finds_settings_files() {
        let dir =
            std::env::temp_dir().join(format!("phoecord-test-vencord-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let f = dir.join("vesktop/settings");
        std::fs::create_dir_all(&f).unwrap();
        std::fs::write(f.join("settings.json"), "{}").unwrap();
        let found = find(&dir, None);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].0, "Vesktop");
        let _ = std::fs::remove_dir_all(&dir);
    }
}

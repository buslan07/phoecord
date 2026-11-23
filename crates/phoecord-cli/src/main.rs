//! `phoecord`: command line front end. Thin: every command calls library code.
//!
//! Every user-facing string comes from the `.ftl` bundles of `phoecord-i18n` through one `Localizer`.

mod help;
mod log;
mod selftest;

use anyhow::{Context, Result};
use clap::{CommandFactory, FromArgMatches, Parser, Subcommand};
use phoecord_i18n::{Args, Localizer, Resolved, resolve_language};
use phoecord_installer::launch::Occupant;
use phoecord_installer::log::Step;
use phoecord_installer::shortcut::Outcome;
use phoecord_installer::uninstall::Kept;
use phoecord_plugin_runtime::Chain;
use phoecord_plugin_runtime::package::PackageSet;
use phoecord_plugin_runtime::wasm::WasmRuntime;
use phoecord_proxy::config::{Config, Paths};
use phoecord_proxy::{Proxy, ca, server, trust};
use std::time::{Duration, Instant};

/// Help texts are set at run time by `help::localize`: clap wants `&'static str` here.
#[derive(Parser)]
#[command(
    name = "phoecord",
    version,
    disable_help_flag = true,
    disable_version_flag = true,
    disable_help_subcommand = true
)]
struct Cli {
    #[arg(long, global = true, value_name = "CODE")]
    lang: Option<String>,
    #[command(subcommand)]
    cmd: Option<Cmd>,
}

/// The Discord channels `--channel` accepts.
#[derive(Clone, Copy, clap::ValueEnum)]
enum ChannelArg {
    Stable,
    Ptb,
    Canary,
}

impl From<ChannelArg> for phoecord_installer::Channel {
    fn from(c: ChannelArg) -> Self {
        match c {
            ChannelArg::Stable => Self::Stable,
            ChannelArg::Ptb => Self::Ptb,
            ChannelArg::Canary => Self::Canary,
        }
    }
}

/// What `launch` was asked to start: an explicit command, or a channel (none: the only client).
struct Launch {
    command: Vec<String>,
    channel: Option<phoecord_installer::Channel>,
}

#[derive(Subcommand)]
enum Cmd {
    Run,
    Launch {
        #[arg(
            long,
            value_enum,
            conflicts_with = "command",
            hide_possible_values = true
        )]
        channel: Option<ChannelArg>,
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        command: Vec<String>,
    },
    Setup {
        #[arg(long)]
        accept_terms: bool,
        #[arg(long, value_enum, hide_possible_values = true)]
        channel: Option<ChannelArg>,
    },
    Uninstall,
    Trust,
    Untrust,
    Plugins,
    Enable {
        name: String,
    },
    Disable {
        name: String,
    },
    Toggle {
        name: String,
    },
    Log {
        #[arg(short, default_value_t = 20)]
        n: usize,
    },
    ImportVencord {
        #[arg(long, value_name = "FILE")]
        path: Option<std::path::PathBuf>,
    },
    Selftest,
    Paths,
}

/// A message with `{ $name }` variables, from `(name, value)` pairs.
fn tf(l: &Localizer, key: &str, vars: &[(&str, &str)]) -> String {
    let mut a = Args::new();
    for (k, v) in vars {
        a.set(*k, v.to_string());
    }
    l.t_args(key, &a)
}

fn chain() -> Chain {
    Chain::new(phoecord_plugins::builtin())
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_env("PHOECORD_LOG")
                .unwrap_or_else(|_| "phoecord_proxy=info,phoecord=info".into()),
        )
        .with_target(false)
        .init();
    let paths = Paths::resolve();
    // The language is needed before clap runs, so that `--help` is already translated.
    let raw: Vec<String> = std::env::args().collect();
    let resolved = language(&paths, help::lang_from_args(&raw).as_deref());
    let l10n = Localizer::new(&resolved.lang);
    for w in &resolved.warnings {
        eprintln!("{}: {w}", l10n.t("cli-warning-prefix"));
    }
    let matches = help::localize(Cli::command(), &l10n).get_matches();
    let cli = Cli::from_arg_matches(&matches).unwrap_or_else(|e| e.exit());
    if let Err(e) = dispatch(cli, paths, &resolved, &l10n) {
        eprintln!(
            "{}: {}",
            l10n.t("cli-error-prefix"),
            describe_error(&e, &l10n)
        );
        std::process::exit(1);
    }
}

/// The error chain as one line; a few known errors get their localized text.
fn describe_error(e: &anyhow::Error, l10n: &Localizer) -> String {
    if let Some(c) = e.downcast_ref::<phoecord_installer::launch::ChooseError>() {
        use phoecord_installer::launch::ChooseError;
        return match c {
            ChooseError::NotInstalled(ch) => tf(
                l10n,
                "cli-err-channel-not-installed",
                &[("channel", ch.name())],
            ),
            ChooseError::Several(list) => {
                let list: Vec<String> = list
                    .iter()
                    .map(|(ch, root)| format!("  --channel {:<6} {}", ch.name(), root.display()))
                    .collect();
                tf(
                    l10n,
                    "cli-err-several-clients",
                    &[(
                        "list",
                        &list.join(
                            "
",
                        ),
                    )],
                )
            }
        };
    }
    if e.downcast_ref::<phoecord_installer::launch::NoDiscord>()
        .is_some()
    {
        let key = if cfg!(windows) {
            "cli-err-no-discord-windows"
        } else if cfg!(target_os = "macos") {
            "cli-err-no-discord-macos"
        } else {
            "cli-err-no-discord-linux"
        };
        return l10n.t(key);
    }
    format!("{e:#}")
}

fn dispatch(cli: Cli, paths: Paths, resolved: &Resolved, l10n: &Localizer) -> Result<()> {
    match cli.cmd.unwrap_or(Cmd::Run) {
        Cmd::Run => runtime()?.block_on(run(paths, None, l10n)),
        Cmd::Launch { command, channel } => {
            let launch = Launch {
                command,
                channel: channel.map(Into::into),
            };
            runtime()?.block_on(run(paths, Some(launch), l10n))
        }
        Cmd::Setup {
            accept_terms,
            channel,
        } => setup(
            &paths,
            l10n,
            &resolved.lang,
            accept_terms,
            channel.map(Into::into),
        ),
        Cmd::Uninstall => uninstall(l10n),
        Cmd::Trust => {
            let ca = ca::Authority::load_or_create(&paths.data)?;
            trust::install(&ca.ca_pem_path)
        }
        Cmd::Untrust => trust::uninstall(),
        Cmd::Plugins => plugins(&paths, l10n),
        Cmd::Enable { name } => switch(&paths, &name, Some(true), l10n),
        Cmd::Disable { name } => switch(&paths, &name, Some(false), l10n),
        Cmd::Toggle { name } => switch(&paths, &name, None, l10n),
        Cmd::Log { n } => log::run(&paths, n, l10n),
        Cmd::ImportVencord { path } => import_vencord(&paths, path, l10n),
        Cmd::Selftest => runtime()?.block_on(selftest::run(paths, l10n)),
        Cmd::Paths => {
            for (key, p) in [
                ("cli-paths-config", paths.config.clone()),
                ("cli-paths-ca", paths.data.join("ca.pem")),
                ("cli-paths-plugins", paths.data.join("plugins")),
            ] {
                println!("{}", paths_line(&l10n.t(key), &p.display().to_string()));
            }
            let source = l10n.t(&format!("lang-source-{}", resolved.source.id()));
            println!(
                "{}",
                paths_line(
                    &l10n.t("cli-paths-language"),
                    &format!("{} ({source})", l10n.language())
                )
            );
            Ok(())
        }
    }
}

fn paths_line(label: &str, value: &str) -> String {
    let pad = 10usize.saturating_sub(label.chars().count());
    format!("{label}{}{value}", " ".repeat(pad))
}

/// `--lang`, then `language` in config.toml (read without creating the file), then the OS.
fn language(paths: &Paths, flag: Option<&str>) -> Resolved {
    let cfg = if paths.config.exists() {
        Config::load(&paths.config).ok().and_then(|c| c.language)
    } else {
        None
    };
    resolve_language(
        flag,
        cfg.as_deref(),
        phoecord_i18n::detect_os_language().as_deref(),
    )
}

fn runtime() -> Result<tokio::runtime::Runtime> {
    Ok(tokio::runtime::Runtime::new()?)
}

async fn run(paths: Paths, launch: Option<Launch>, l10n: &Localizer) -> Result<()> {
    let cfg = Config::load_or_init(&paths.config)?;
    let listen = cfg.listen.clone();
    if let Some(l) = &launch {
        match phoecord_installer::launch::probe_proxy(&listen) {
            Occupant::PhoeCord(version) => {
                return reuse_proxy(l, &listen, &version, l10n);
            }
            Occupant::Other => {
                anyhow::bail!(tf(l10n, "cli-err-port-taken", &[("listen", &listen)]))
            }
            Occupant::Nobody => {}
        }
    }
    // Pick the client before anything starts: a wrong or missing choice must change nothing.
    let client_cmd = match &launch {
        Some(l) => Some(phoecord_installer::launch::command(
            &l.command,
            l.channel,
            &phoecord_installer::discover::discover(),
            &listen,
        )?),
        None => None,
    };
    let proxy = Proxy::new(paths, cfg, chain(), false)?;
    // WASM plugin packages in `<data>/plugins/<id>/`: loaded now, and again on every folder scan.
    match WasmRuntime::new() {
        Ok(rt) => proxy.watch_packages(PackageSet::new(rt, proxy.paths.data.join("plugins"))),
        Err(e) => tracing::warn!("WASM plugins are off: {e:#}"),
    }
    proxy.spawn_reloader();
    let listener = server::bind(&listen)
        .await
        .with_context(|| tf(l10n, "cli-err-listen", &[("listen", &listen)]))?;
    tracing::info!(
        "{}",
        tf(
            l10n,
            "cli-listening",
            &[
                ("listen", &listen),
                ("plugins", &proxy.chain.enabled_ids().join(", "))
            ]
        )
    );
    let server = tokio::spawn(server::serve(proxy.clone(), listener));

    if let Some(cmd) = client_cmd {
        tracing::info!(
            "{}",
            tf(
                l10n,
                "cli-starting",
                &[("program", &format!("{:?}", cmd.get_program()))]
            )
        );
        let started = Instant::now();
        let mut child = tokio::process::Command::from(cmd)
            .spawn()
            .with_context(|| l10n.t("cli-err-start-discord"))?;
        tokio::select! {
            status = child.wait() => {
                let status = status?.to_string();
                // Launchers (Update.exe, flatpak) hand off and exit at once: keep serving then.
                if started.elapsed() < Duration::from_secs(15) {
                    tracing::info!("{}", tf(l10n, "cli-launcher-exited", &[("status", &status)]));
                    let _ = tokio::signal::ctrl_c().await;
                } else {
                    tracing::info!("{}", tf(l10n, "cli-client-exited", &[("status", &status)]));
                }
            }
            _ = tokio::signal::ctrl_c() => {}
        }
        return Ok(());
    }
    tokio::select! {
        r = server => r??,
        _ = tokio::signal::ctrl_c() => {}
    }
    Ok(())
}

/// A PhoeCord proxy already serves `listen` (autostart, another `launch`): only start the client.
fn reuse_proxy(launch: &Launch, listen: &str, version: &str, l10n: &Localizer) -> Result<()> {
    let mut cmd = phoecord_installer::launch::command(
        &launch.command,
        launch.channel,
        &phoecord_installer::discover::discover(),
        listen,
    )?;
    println!(
        "{}",
        tf(
            l10n,
            "cli-proxy-reused",
            &[("listen", listen), ("version", version)]
        )
    );
    cmd.spawn()
        .with_context(|| l10n.t("cli-err-start-discord"))?;
    Ok(())
}

fn config_line(l10n: &Localizer, paths: &Paths) -> String {
    tf(
        l10n,
        "cli-config-line",
        &[("path", &paths.config.display().to_string())],
    )
}

/// Show the terms and ask for `yes` (also `да` in Russian) unless this version was accepted already.
fn ask_terms(l10n: &Localizer, lang: &str, flag: bool) -> Result<()> {
    use phoecord_installer::terms;
    let env = phoecord_installer::system::Env::system()?;
    if !terms::needs_acceptance(&env) {
        return Ok(());
    }
    if !flag {
        println!("{}\n", terms::text(lang));
        print!("{} ", l10n.t("terms-prompt"));
        std::io::Write::flush(&mut std::io::stdout())?;
        let mut answer = String::new();
        std::io::stdin().read_line(&mut answer)?;
        let a = answer.trim().to_lowercase();
        if !(a == "yes" || (lang == "ru" && a == "да")) {
            anyhow::bail!(l10n.t("terms-declined"));
        }
    }
    terms::accept(&env, terms::TERMS_VERSION)
}

fn setup(
    paths: &Paths,
    l10n: &Localizer,
    lang: &str,
    accept_terms: bool,
    want: Option<phoecord_installer::Channel>,
) -> Result<()> {
    // Same rules as `launch`: `--channel` must be installed (checked before anything changes); with
    // no flag the only installed client gets the shortcut, and with none or several setup goes on
    // without one and says so, instead of refusing to install the proxy.
    let clients = phoecord_installer::discover::discover();
    let channel = match phoecord_installer::launch::choose(&clients, want) {
        Ok(c) => Some(c.channel),
        Err(e) if want.is_some() => return Err(e),
        Err(e) => {
            println!(
                "{}",
                tf(
                    l10n,
                    "cli-shortcut-skipped",
                    &[("why", &describe_error(&e, l10n))]
                )
            );
            None
        }
    };
    ask_terms(l10n, lang, accept_terms)?;
    Config::load_or_init(&paths.config)?;
    println!("{}", config_line(l10n, paths));
    // CA, trust and autostart, each step recorded in `install.log` so `uninstall` can undo it.
    let env = phoecord_installer::system::Env::system()?;
    let report = phoecord_installer::install::install_with(
        &env,
        &phoecord_installer::system::RealSystem,
        channel,
    )?;
    println!(
        "{}",
        tf(
            l10n,
            "cli-ca-line",
            &[
                ("path", &report.ca_pem.display().to_string()),
                ("hosts", &ca::CONSTRAINED_TO.join(", "))
            ]
        )
    );
    match &report.shortcut {
        Outcome::Created { name, path } => println!(
            "{}",
            tf(
                l10n,
                "cli-shortcut-created",
                &[("name", name), ("path", &path.display().to_string())]
            )
        ),
        Outcome::Unsupported(_) => println!(
            "{}",
            tf(
                l10n,
                "cli-shortcut-unsupported",
                &[("channel", channel.map_or("", |c| c.name()))]
            )
        ),
        Outcome::NotRequested => {}
    }
    let mut a = Args::new();
    a.set("count", report.added);
    println!("{}", l10n.t_args("cli-setup-steps", &a));
    println!("\n{}", l10n.t("cli-setup-done"));
    Ok(())
}

/// What an install step was, in words.
fn describe_step(l10n: &Localizer, step: &Step) -> String {
    match step {
        Step::Dir { path } => tf(
            l10n,
            "cli-step-dir",
            &[("path", &path.display().to_string())],
        ),
        Step::File { path } => tf(
            l10n,
            "cli-step-file",
            &[("path", &path.display().to_string())],
        ),
        Step::CaTrusted { ca_pem } => tf(
            l10n,
            "cli-step-ca-trusted",
            &[("path", &ca_pem.display().to_string())],
        ),
        Step::RunKey { name, .. } => tf(l10n, "cli-step-run-key", &[("name", name)]),
        Step::LaunchAgent { label, .. } => tf(l10n, "cli-step-launch-agent", &[("label", label)]),
        Step::SystemdUnit { unit } => tf(l10n, "cli-step-systemd-unit", &[("unit", unit)]),
    }
}

fn describe_kept(l10n: &Localizer, why: Kept) -> String {
    l10n.t(match why {
        Kept::AlreadyGone => "cli-kept-already-gone",
        Kept::NotEmpty => "cli-kept-not-empty",
        Kept::HoldsLog => "cli-kept-holds-log",
    })
}

fn uninstall(l10n: &Localizer) -> Result<()> {
    let env = phoecord_installer::system::Env::system()?;
    let r = phoecord_installer::uninstall::uninstall_all(
        &env,
        &phoecord_installer::system::RealSystem,
    )?;
    for pid in &r.stopped {
        println!(
            "{}",
            tf(l10n, "cli-uninstall-stopped", &[("pid", &pid.to_string())])
        );
    }
    if let Some(why) = &r.stop_failed {
        eprintln!("{}", tf(l10n, "cli-uninstall-stop-failed", &[("why", why)]));
    }
    for step in &r.removed {
        let what = describe_step(l10n, step);
        println!("{}", tf(l10n, "cli-uninstall-removed", &[("what", &what)]));
    }
    for (step, why) in &r.kept {
        let what = describe_step(l10n, step);
        let why = describe_kept(l10n, *why);
        println!(
            "{}",
            tf(l10n, "cli-kept-line", &[("what", &what), ("why", &why)])
        );
    }
    for (step, why) in &r.failed {
        let what = describe_step(l10n, step);
        eprintln!(
            "{}",
            tf(
                l10n,
                "cli-uninstall-failed",
                &[("what", &what), ("why", why)]
            )
        );
    }
    if !r.is_clean() {
        anyhow::bail!(l10n.t("cli-uninstall-incomplete"));
    }
    println!("\n{}", l10n.t("cli-uninstall-done"));
    Ok(())
}

/// The description of a plugin: translated when the bundles have `plugin-desc-<id>`, else the
/// manifest's own text (plugins from packages bring their own).
fn plugin_description(l10n: &Localizer, id: &str, fallback: &str) -> String {
    l10n.try_t(&format!("plugin-desc-{id}"))
        .unwrap_or_else(|| fallback.to_string())
}

fn plugins(paths: &Paths, l10n: &Localizer) -> Result<()> {
    let cfg = Config::load_or_init(&paths.config)?;
    let (on_text, off_text) = (l10n.t("cli-plugin-on"), l10n.t("cli-plugin-off"));
    let width = on_text.chars().count().max(off_text.chars().count());
    for m in chain().manifests() {
        let state = if cfg.is_enabled(&m.id) {
            &on_text
        } else {
            &off_text
        };
        let state = format!("{state:<width$}");
        let vencord = m
            .vencord
            .as_deref()
            .map(|v| format!("  {}", tf(l10n, "cli-plugins-vencord", &[("name", v)])))
            .unwrap_or_default();
        println!(
            "{state}  {:<22} {}{vencord}",
            m.id,
            plugin_description(l10n, &m.id, &m.description)
        );
    }
    println!("\n{}", config_line(l10n, paths));
    Ok(())
}

fn import_vencord(paths: &Paths, path: Option<std::path::PathBuf>, l10n: &Localizer) -> Result<()> {
    use phoecord_installer::vencord;
    let file = match path {
        Some(p) => p,
        None => {
            let config = dirs::config_dir().unwrap_or_default();
            let home = dirs::home_dir();
            let found = vencord::find(&config, home.as_deref());
            let Some((name, p)) = found.into_iter().next() else {
                anyhow::bail!(l10n.t("cli-import-none"));
            };
            println!(
                "{}",
                tf(
                    l10n,
                    "cli-import-found",
                    &[("name", name), ("path", &p.display().to_string())]
                )
            );
            p
        }
    };
    let text = std::fs::read_to_string(&file).with_context(|| {
        tf(
            l10n,
            "cli-import-read",
            &[("path", &file.display().to_string())],
        )
    })?;
    let enabled = vencord::enabled_plugins(&text)?;
    let mut cfg = Config::load_or_init(&paths.config)?;
    let known: Vec<_> = chain()
        .manifests()
        .iter()
        .map(|m| (m.id.clone(), m.vencord.clone(), cfg.is_enabled(&m.id)))
        .collect();
    let plan = vencord::plan(&enabled, &known);
    for id in &plan.enable {
        cfg.set_enabled(id, true)?;
        println!("{}", tf(l10n, "cli-plugin-state-on", &[("name", id)]));
    }
    if !plan.enable.is_empty() {
        cfg.save(&paths.config)?;
    }
    println!(
        "\n{}",
        tf(
            l10n,
            "cli-import-summary",
            &[
                ("enabled", &plan.enable.len().to_string()),
                ("already", &plan.already_on.len().to_string()),
                ("unmatched", &plan.unmatched.len().to_string()),
            ]
        )
    );
    Ok(())
}

fn switch(paths: &Paths, name: &str, to: Option<bool>, l10n: &Localizer) -> Result<()> {
    let chain = chain();
    if !chain.ids().iter().any(|id| id == name) {
        anyhow::bail!(tf(l10n, "cli-no-such-plugin", &[("name", name)]));
    }
    let mut cfg = Config::load_or_init(&paths.config)?;
    let was = cfg.is_enabled(name);
    let on = to.unwrap_or(!was);
    cfg.set_enabled(name, on)?;
    cfg.save(&paths.config)?;
    let key = if on {
        "cli-plugin-state-on"
    } else {
        "cli-plugin-state-off"
    };
    println!("{}", tf(l10n, key, &[("name", name)]));
    if on && !was && chain.uses_gateway(name) {
        // Gateway plugins only see a connection that started after they were switched on.
        println!("{}", l10n.t("cli-gateway-note"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every `"cli-…"`, `"cmd-…"` and `"plugin-desc-…"` literal in the CLI sources.
    fn requested_keys() -> Vec<String> {
        let mut keys = Vec::new();
        for src in [
            include_str!("main.rs"),
            include_str!("log.rs"),
            include_str!("selftest.rs"),
            include_str!("help.rs"),
        ] {
            for part in src.split('"').skip(1).step_by(2) {
                let ok = part.starts_with("cli-")
                    && !part.ends_with('-')
                    && part
                        .chars()
                        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
                if ok {
                    keys.push(part.to_string());
                }
            }
        }
        keys
    }

    #[test]
    fn every_requested_key_exists_in_english_and_russian() {
        let keys = requested_keys();
        assert!(keys.len() > 40, "scan found only {}", keys.len());
        let (en, ru) = (Localizer::new("en"), Localizer::new("ru"));
        for k in keys {
            assert!(en.try_t(&k).is_some(), "key {k} is missing in en");
            assert!(ru.try_t(&k).is_some(), "key {k} is missing in ru");
        }
        for id in ["fake_nitro", "message_logger", "commands"] {
            assert!(en.try_t(&format!("plugin-desc-{id}")).is_some());
        }
        for source in ["flag", "config", "os", "default"] {
            assert!(en.try_t(&format!("lang-source-{source}")).is_some());
        }
    }

    #[test]
    fn every_builtin_plugin_has_a_description_key() {
        let en = Localizer::new("en");
        for m in chain().manifests() {
            assert!(
                en.try_t(&format!("plugin-desc-{}", m.id)).is_some(),
                "plugin-desc-{} is missing",
                m.id
            );
        }
    }
}

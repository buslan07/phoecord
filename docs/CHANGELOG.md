# Changelog

All notable changes to the project ([Keep a Changelog](https://keepachangelog.com/en/1.1.0/), [SemVer](https://semver.org/)). Every PR adds its line under "Unreleased"; a release moves those lines under its version and date.

## [Unreleased]

### Added

- 1.1.11: `upstream = "socks5://host:port"` or `"http://host:port"` in `config.toml` sends every outgoing connection of the proxy (intercepted REST and Gateway, and tunnelled) through a SOCKS5 or HTTP-CONNECT proxy, for VPN and zapret users; a dead or refusing upstream gives a clear error; `phoecord selftest` honours it.
- 1.2.9: `phoecord import-vencord [--path FILE]` finds the settings of Vencord, Vesktop or Equicord and switches on the PhoeCord plugins whose `Manifest::vencord` matches an enabled Vencord plugin (English and Russian).
- Stage 1.1 core refactor: the proxy, REST and Gateway run through Plugin API v1 and `Chain`; `ProxyHost` provides scoped settings, KV, account state and notifications. The `phoecord` CLI now has `run`, `launch`, `setup`, `trust`, `untrust`, `plugins`, `enable`, `disable`, `toggle`, `paths` and `selftest`.
- Cargo workspace of 8 crates: `core`, `proxy`, `plugin-runtime`, `plugin-sdk`, `plugins`, `loader`, `installer`, `cli`.
- Plugin API v1 (`phoecord-core::plugin`): `Manifest`, `Event`, `EventKind`, `Verdict`, `Host`.
- Plugin chain `phoecord-plugin-runtime::Chain`: the first `Drop`/`Reply` stops the chain.
- AOB signature scanner in `phoecord-loader` (`??` masks, ambiguous matches rejected).
- Docs: README, ARCHITECTURE, ROADMAP, CONTRIBUTING, SECURITY, AGENTS; `docs/` index, code map, governance, runbook, secrets, support, project map; templates for plans, incidents, releases, release checks and rulings; the Phase 1 plan.
- `scripts/check.sh` / `check.ps1` (all checks), `scripts/gen_tasks.py` (`TASKS.md`), `scripts/gen_decisions.py` (`docs/DECISIONS.md`).
- Issue and PR templates; CI on Linux, Windows and macOS plus an MSRV job.
- Localization planned: English first, Russian next (ROADMAP 1.7).

- `phoecord-i18n`: Fluent localizer with English and Russian bundles, fallback ru → en → key, OS language detection (stage 1.7.1).
- `scripts/check_i18n.py`: fails when a Russian translation is missing a key or a variable; part of `check.sh` and CI (stage 1.7.5).

- `phoecord-loader` builds as a proxy `version.dll`: lazy export forwarding to the system DLL, a `DllMain` that only starts a worker, client build detection (file version + `.text` SHA-256), `signatures/*.toml` with unique AOB matches and RIP resolution, inline detours with removal on unload (stages 1.5.1-1.5.4).
- `silent_typing` plugin: your typing indicator is never sent (stage 1.2.1).
- `clear_urls` plugin: strips tracking parameters (`utm_*`, `fbclid`, `gclid`, `si` on YouTube and Spotify, …) from links in messages you send or edit; extra rules in `[plugins.clear_urls] extra_rules` (stage 1.2.7).

- `docs/research/gpui.md`: GPUI map, a release + LTO reference build (`research/gpui-reference`), anchors that survive in a stripped binary, and ranked hook candidates (palette, settings tabs, message row) for Phase 2 (stage 1.6).

- Plugins `game_activity_toggle`, `relationship_notifier` and `fake_nitro` (emoji part), all off by default (stages 1.2.3-1.2.5).
- `message_logger` plugin and `phoecord log`: deleted and edited messages stay visible and are written to `messages.jsonl` (stage 1.2.6).
- `/phoecord plugins` and `/phoecord toggle <id>` slash commands, answered locally (stage 1.2.8).
- Plugin API (minor, `API_VERSION` stays 1): `RestResponse.body` is mutable; new `Host` methods `emit_dispatch`, `plugin_enabled`, `set_plugin_enabled`, `append_data`.
- Installer library: client discovery on 3 OSes, CA + autostart install with an install log, full rollback (stages 1.4.1, 1.4.2, 1.4.4; not wired to the CLI yet).
- WASM plugin runtime: `wit/phoecord.wit`, wasmtime 38 backend behind the same `Chain`, 5 ms / 32 MiB limits, plugin packages (`plugin.toml` + `plugin.wasm`, `events` and `rights`) with reload (stages 1.3.1-1.3.5).
- WASM plugins: a slow event costs a strike (3 in a minute disable the plugin) instead of disabling it at 5 ms; new rights `emit_dispatch`, `plugins`, `data`; plugin packages copied into or removed from `plugins/` load and unload without a restart, and `phoecord run` now loads them (stage 1.3.4).
- Issues are the place for every problem; leaks of personal data or secrets go privately to `nandak070` on Discord (`SECURITY.md`). The README shows contributors automatically.
- `phoecord setup` now runs the installer library (CA, trust and autostart, logged in `install.log`), and `phoecord uninstall` undoes it (stages 1.4.2, 1.4.4).
- Language setting: `phoecord --lang ru`, `language` in `config.toml`, or the OS language; `setup`, `uninstall` and `paths` already speak Russian, the rest of the CLI follows in 1.7.3 (stage 1.7.2).
- The whole CLI speaks Russian, `--help` included (stage 1.7.3).
- Terms of use (`TERMS.md`, `TERMS.ru.md`, version 1): `phoecord setup` shows them and installs nothing until you type `yes` (`да`); a new version asks again (stage 1.4.7).
- Plugins `i_love_spam`, `no_blocked_messages` and `no_reply_mention`, off by default (stage 1.2.10); ImageLink and NoPendingCount are UI-only and move to Phase 2.
- Plugins `text_replace`, `send_timestamps`, `anonymise_file_names` and `silent_message_toggle` with `/phoecord silent`, off by default (stage 1.2.11); BetterGifAltText is UI-only and moves to Phase 2.
- The first third-party plugin example: `clear_urls` as a WASM package in `examples/plugins/clear_urls/`, loaded without a restart (stage 1.3.8).
- `phoecord launch --channel stable|ptb|canary` (no silent default to Stable); `launch` reuses a running proxy; `setup --channel` adds a Start-menu shortcut that starts that Discord through PhoeCord; `uninstall` stops a running proxy and removes PhoeCord's own CA files (stage 1.4.8).

### Removed

- Docs: `PROJECT_MAP.md`, `SUPPORT.md`, `SECRETS.md` (merged into `GOVERNANCE.md`), `TASKS.md` and `scripts/gen_tasks.py`, `incidents/`, `reports/`, release, incident and plan templates. Task status now lives only in `docs/phase-1/README.md`.

### Ported from the v0 proxy

- `val`, `etf`, REST routes → `phoecord-core`.
- `codec`, `ca`, `trust` → `phoecord-proxy`. The CA is renamed "PhoeCord local CA".
- The `no_track` plugin → `phoecord-plugins`, on the new Plugin API.

### Changed

- License: MIT → **GPL-3.0-or-later**, like Vencord. `NOTICE` holds the copyright and exceptions; the plugin SDK, `wit/` and templates stay MIT; the v0 code's MIT notice moved to `LICENSES/MIT-v0.txt`. Vencord code may now be adapted with attribution.

- All documentation is English-only.
- `docs/PORTING_PLUGINS.md`: step-by-step guide to porting any plugin (layer choice, Vencord → PhoeCord concept map, native and WASM, tests, worked example).
- `docs/PLUGINS.md`: every Vencord plugin and the owner's userplugins mapped to a layer and a task; Phase 1 tasks 1.2.10–1.2.19 for Vencord parity on the network.
- Decided: WASM plugin runtime, egui installer GUI, no code signing (like Vencord).

### Fixed

- `trust.rs`: an unused `PathBuf` import broke `-D warnings` builds on Windows and macOS.

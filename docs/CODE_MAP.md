# Code map: which file does what

Checked against code: `a965f75`, 23.11.2026. Whoever re-checks this document (or a section of it) against the code updates this line (rule in `AGENTS.md`).

A reference to the repository: which file holds what, why it exists and what is easy to break in it. Also every config key, environment variable and on-disk file the app uses. This is a living document and describes the `main` branch.

> **Mandatory for people and AI agents.** Added, renamed, moved or removed a crate, module, file, config key or environment variable: update this file in the same commit. A change after which this map is wrong is not done. The rule is in [`AGENTS.md`](AGENTS.md).

Related: [`ARCHITECTURE.md`](ARCHITECTURE.md) explains how the layers work; [`PLUGINS.md`](PLUGINS.md) lists every plugin and its status.

**How to read.** `phoecord-core` is pure data and contracts, no I/O. `phoecord-proxy` talks to the network. `phoecord-plugins` holds plugin logic only. `phoecord-loader` is the only crate allowed to use `unsafe`.

---

## 1. Repository root

| Path | What it is |
|:---|:---|
| `Cargo.toml` | Workspace: members `crates/*`, default member `phoecord-cli`, shared `[workspace.package]` (edition 2024, MSRV 1.88, GPL-3.0-or-later), shared dependency versions, lints, release profile |
| `Cargo.lock` | Locked versions; committed because the workspace ships binaries |
| `README.md` | Product overview |
| `CONTRIBUTING.md`, `SECURITY.md`, `CODE_OF_CONDUCT.md` | Process, security policy, conduct |
| `LICENSE`, `NOTICE`, `LICENSES/` | GPL-3.0 text; copyright and exceptions; MIT notices for the v0 code (`MIT-v0.txt`) and the plugin SDK (`MIT-sdk.txt`) |
| `assets/logo.svg` | Logo used in the README |
| `.github/workflows/ci.yml` | CI: rustfmt, generated docs freshness, check/clippy/test on Linux, Windows, macOS, MSRV 1.88 |
| `.github/ISSUE_TEMPLATE/` | Bug, feature and plugin forms; blank issues disabled |
| `.github/PULL_REQUEST_TEMPLATE.md` | PR form with the living-docs checklist |
| `.github/dependabot.yml` | Weekly cargo updates (grouped), monthly GitHub Actions updates; Dependabot opens the `dependabot/*` branches |
| `.editorconfig`, `.gitattributes`, `.gitignore` | LF everywhere, 4-space Rust, 2-space YAML/TOML; secrets and dumps ignored |
| `signatures/` | AOB signature tables per client build (`README.md` documents the format; no Discord signatures yet) |
| `research/` | Out-of-workspace research projects: `gpui-reference` (GPUI app built like a shipping client) and `aob-probe` (checks AOB anchors against a binary); excluded from the workspace |
| `.github/CODEOWNERS`, `.github/workflows/assign.yml` | Every PR is assigned to Nanda070 and requests Nanda070's review |
| `scripts/` | §4 |
| `docs/` | §5 |

## 2. Crates

| Crate | Path | Kind | Depends on (ours) |
|:---|:---|:---|:---|
| `phoecord-core` | `crates/phoecord-core/` | lib | none |
| `phoecord-proxy` | `crates/phoecord-proxy/` | lib | core |
| `phoecord-plugin-runtime` | `crates/phoecord-plugin-runtime/` | lib | core |
| `phoecord-plugin-sdk` | `crates/phoecord-plugin-sdk/` | lib (guest side) | none, by design |
| `phoecord-plugins` | `crates/phoecord-plugins/` | lib | core |
| `phoecord-loader` | `crates/phoecord-loader/` | cdylib + rlib | none |
| `phoecord-installer` | `crates/phoecord-installer/` | lib | core |
| `phoecord-cli` | `crates/phoecord-cli/` | bin `phoecord` | core, proxy, plugin-runtime, plugins |
| `phoecord-i18n` | `crates/phoecord-i18n/` | lib | none |

## 3. Files by crate

### `phoecord-core`

| File | What | Easy to break |
|:---|:---|:---|
| `src/lib.rs` | Re-exports `Val` and the Plugin API types | Anything re-exported here is public API for plugins |
| `src/val.rs` | `Val`: one value type for JSON and ETF; keeps atoms, big ints, binaries, so ETF round-trips unchanged | Changing a variant changes how every Gateway frame re-encodes |
| `src/etf.rs` | Erlang External Term Format decode/encode (the desktop client's Gateway encoding) | Compressed terms (tag 80) need `flate2` |
| `src/route.rs` | `route_of` (`/api/v9/x` → `/x`), `message_route_channel` | Plugins match on the normalized route |
| `src/plugin.rs` | Plugin API v1: `API_VERSION`, `Layer`, `Manifest`, `Event`, `EventKind`, `Verdict`, `Host`, `Plugin` | **The contract.** Changes need an `api` PR and the owner's review |

### `phoecord-proxy`

| File | What | Easy to break |
|:---|:---|:---|
| `src/lib.rs` | Module list and status | |
| `src/config.rs`, `src/account.rs` | Live plugin configuration and account state available to plugins | Config reload must re-enable the chain and preserve the last live state |
| `src/host.rs` | `ProxyHost`: scoped settings, KV, account, rate-limited notifications and REST/fetch hooks | Host state must be bound to the plugin currently running in `Chain` |
| `GET /__phoecord` (in `src/server.rs`) | Identity answer (`phoecord <version>`) for loopback peers, so `launch` can reuse a running proxy | |
| `src/server.rs`, `src/rest.rs`, `src/gateway.rs` | Proxy loop, REST and Gateway traffic routed through `Chain` | Keep REST decisions and Gateway encode/compress round-trips lossless |
| `src/codec.rs` | `Codec` from the Gateway query (`encoding`, `compress`), `Inflater` / `Deflater` for zlib-stream and zstd-stream | Compression context is per connection: once a frame changes, everything after must use our context |
| `src/ca.rs` | `Authority`: local CA load-or-create, name constraint (`CONSTRAINED_TO`), leaf certificates per SNI, rustls `ServerConfig` | `CA_NAME` is what `untrust` looks for; tests check the constraint blocks other domains |
| `src/trust.rs` | Install/remove the CA: Windows `certutil -user`, macOS keychain, Linux NSS (incl. Flatpak, Snap) | Platform code behind `cfg`; build on all 3 OSes before merging |

`Host::rest` and `Host::fetch` are stubs: they answer "not wired" (the rate limiter is in place, no real request is made yet).

### `phoecord-plugin-runtime`

| File | What | Easy to break |
|:---|:---|:---|
| `src/lib.rs` | `Chain` (built-in segment, then a dynamic segment swapped by `replace_dynamic`; first `Drop`/`Reply` stops), `Outcome`, `NullHost` for tests | The stop rule is part of the API semantics; the lock is never held while a plugin runs |
| `src/wasm.rs` | `WasmRuntime` (engine + epoch clock), `WasmLimits`, `WasmPlugin`, `Right`, `Setup`, `tree` ↔ `Val` | Guest trees are untrusted: keep the bounds, depth and single-use checks |
| `src/package.rs` | `plugin.toml` format and rights table (documented at the top), `PackagePlugin` with reload, `load_dir`, `PackageSet::sync` (load new folders, unload removed ones) | A broken new version must leave the old one running |
| `tests/guest/` | A separate workspace: the test guest, built for `wasm32-wasip2` by the tests (CI installs the target) | Slow-event tests use a zero soft limit so they do not depend on machine speed |
| `tests/clear_urls_wasm.rs` | Builds `examples/plugins/clear_urls`, loads it as a package (also into a running chain) and checks it behaves like the native plugin | Needs the `wasm32-wasip2` target |
| `benches/overhead.rs` | Per-event cost of a WASM plugin vs a native one on a MESSAGE_CREATE payload (`cargo bench`) | Input for benchmark task 1.1.10 |
| `wit/phoecord.wit` (repo root) | The `plugin` world and `host` interface; mirrors `phoecord-core::plugin` | Change together with `plugin.rs` |

### `phoecord-plugin-sdk`

| File | What |
|:---|:---|
| `src/lib.rs` | `API_VERSION` only; bindings and the `#[phoecord::plugin]` macro are task 1.3.6. MIT-licensed (`license = "MIT"`), unlike the rest of the workspace |

### `phoecord-plugins`

| File | What | Easy to break |
|:---|:---|:---|
| `src/lib.rs` | `builtin()`: every built-in plugin in default order; uniqueness test for ids | Order is run order |
| `src/no_track.rs` | NoTrack: answers analytics and crash-report routes locally with 204 | `wants` must stay as narrow as `on_event` |
| `src/clear_urls.rs` | ClearURLs: strips tracking query parameters from links in sent and edited messages; `extra_rules` setting | Only `content` of JSON bodies; messages with attachments (multipart) are not cleaned yet |
| `src/silent_typing.rs` | SilentTyping: answers `POST /channels/{id}/typing` locally with 204 | Only that exact route; `wants` must stay as narrow as `on_event` |
| `src/i_love_spam.rs` | iLoveSpam: clears the likely-spammer bit (`1<<20`) from user `flags` / `public_flags` in messages, history and member events | Keep every other bit |
| `src/no_blocked_messages.rs` | NoBlockedMessages: learns blocked (type 2) and ignored users, drops their live messages; settings `apply_to_ignored_users` (true), `filter_history` (false) | Filtering history makes pages shorter than `limit`, which can stop the client loading older messages; that is why it is off by default |
| `src/no_reply_mention.rs` | NoReplyMention: sets `allowed_mentions.replied_user = false` on replies per Vencord's rules; settings `user_list`, `role_list`, `should_ping_listed`; remembers authors of 5000 messages | An unknown replied-to message counts as "not listed" |
| `src/text_replace.rs` | TextReplace: string and regex rules on outgoing `content` (send and edit), JavaScript replacement syntax emulated; uses `regex` | JS-only regex syntax makes a rule invalid; invalid rules are skipped and logged once |
| `src/send_timestamps.rs` | SendTimestamps: `` `12:34` `` → `<t:unix:t>`; local offset from the `time` crate, `utc_offset_minutes` fallback on Unix | DST: the next occurrence is computed per calendar day |
| `src/anonymise_file_names.rs` | AnonymiseFileNames: renames `files[].filename` in `POST /channels/{id}/attachments` and the matching `attachments[].filename` in the message | Files inside a multipart message body are not renamed (the proxy never rewrites multipart) |
| `src/silent_message_toggle.rs` | SilentMessageToggle: `flags \|= 1<<12` on outgoing messages; `/phoecord silent [on\|off]` | The toggle is in memory and starts off |
| `src/testutil.rs` | Test helpers shared by the plugin tests | |
| `src/game_activity_toggle.rs` | GameActivityToggle: removes "Playing" activities from op 3 presence updates | The proxy re-sends the last presence on toggle |
| `src/relationship_notifier.rs` | RelationshipNotifier: `notify` on unfriend, server and group removal; your own actions in the last 60 s are ignored | Own-action memory must match the REST routes |
| `src/fake_nitro.rs` | FakeNitro (emoji): rewrites unusable emoji in outgoing messages as CDN links; `premium_type` 2 in `READY` only for tier 0 | Learns the real tier from `READY`; needs Ctrl+R after enabling |
| `src/message_logger.rs` | MessageLogger: LRU cache of messages, deletes turned into marked updates, edit history, `messages.jsonl` via `Host::append_data` | Ids may be JSON strings or ETF ints |
| `src/commands.rs` | `/phoecord plugins` and `toggle`: injects the command into search/index answers, answers `POST /interactions` locally; `Subcommand` trait | Command shapes not yet checked on a live client |

### `phoecord-loader`

| File | What | Easy to break |
|:---|:---|:---|
| `src/lib.rs` | Module list and design notes | |
| `src/aob.rs` | `Pattern`: parse `48 8B ?? …`, `find_all`, `find_unique` (ambiguous = miss) | All-wildcard patterns are rejected on purpose |
| `build.rs`, `exports.txt` | Generates one assembly forwarding stub per `version.dll` export and `/EXPORT` link args (MSVC x86_64) | A Windows test checks `exports.txt` against the real DLL |
| `src/proxy_dll.rs` | `FORWARD_TABLE`, `resolve_export`: loads the real DLL from `GetSystemDirectoryW` on first call | Never load by name: it would resolve to this DLL |
| `src/entry.rs` | `DllMain` (only `DisableThreadLibraryCalls` + `CreateThread`), the worker: log, detect build, load signatures | No work under the loader lock; panics are caught |
| `src/log.rs` | Loader log in `%LOCALAPPDATA%\PhoeCord\loader.log` | |
| `src/pe.rs`, `src/version.rs` | PE parsing, file version resource, `.text` SHA-256 | Pure, tested on every OS |
| `src/signatures.rs` | `SignatureSet::load_dir`, `select`, `resolve_all` (`find_unique` + `rip`) | Format documented in `signatures/README.md` |
| `src/hook.rs` | Inline detours (`retour`), installed with other threads suspended, `uninstall_all` | Hooks must be removed before `FreeLibrary` |
| `examples/host.rs` | Throwaway exe that loads `version.dll` from its folder for manual checks | |

### `phoecord-installer`

| File | What |
|:---|:---|
| `src/lib.rs` | `Channel` (Stable, PTB, Canary, Phoenix), module list |
| `src/discover.rs` | `discover()`: Stable/PTB/Canary on Windows, macOS, Linux (PATH, Flatpak, Snap); `discover_in(&Roots)` for tests |
| `src/system.rs` | `Os`, `Env`, the `System` trait (trust store, Run key, launchctl, systemd); `RealSystem` is the only code that touches the machine |
| `src/ca.rs`, `src/autostart.rs` | Install steps: local CA + trust; Run key / LaunchAgent / systemd user unit |
| `src/shortcut.rs` | Start-menu shortcut "Discord … (PhoeCord)" that runs `phoecord launch --channel …`: Windows `.lnk` (PowerShell + WScript.Shell, every value quoted), Linux `.desktop` | Created and removed through `System`, logged in `install.log` |
| `src/launch.rs` | `choose` (the client for `--channel`, or the only one installed; several without `--channel` is an error), `client_command`, `probe_proxy` (is PhoeCord already on the port: `GET /__phoecord`) | `launch` never defaults silently to Stable |
| `src/terms.rs` | `TERMS_VERSION`, both terms texts (`include_str!` of `TERMS.md` / `TERMS.ru.md`), `needs_acceptance`, `accept` (stored in the data folder and logged so `uninstall` removes it) | Bump `TERMS_VERSION` whenever the text changes, so users are asked again |
| `src/system.rs` additions | `stop_proxies` (only `run`/`launch` processes of PhoeCord's own binaries, never the calling process; `select_proxy_pids` is tested), `shortcut_create`, `Env.start_menu`, `Env.applications_dir` | |
| `src/install.rs`, `src/uninstall.rs` | `install()` (idempotent) and `uninstall_all()` (undoes the log newest-first, `UninstallReport`) |
| `src/log.rs` | `install.log` (JSON lines in the data folder): every step the installer made, so rollback removes only what it created |

### `phoecord-i18n`

| File | What | Easy to break |
|:---|:---|:---|
| `src/lib.rs` | `Localizer` (`new`, `t`, `t_args`, `language`), `normalize`, `detect_os_language`, `SUPPORTED` | Bundles are `include_str!`-ed: a new language needs a `source()` arm |
| `i18n/en/main.ftl`, `i18n/ru/main.ftl` | Shared strings; English is the source | Every `en` key needs a `ru` twin with the same variables (`check_i18n.py`) |

### `phoecord-cli`

| File | What | Easy to break |
|:---|:---|:---|
| `src/main.rs` | Commands: `run`, `launch`, `setup` (installer `install()`), `uninstall` (installer `uninstall_all()`), `trust`, `untrust`, `plugins`, `enable`, `disable`, `toggle`, `log`, `selftest`, `paths`; `run` loads WASM packages | `plugins`/`enable` know built-ins only, not WASM packages; every user-facing string comes from `crates/phoecord-i18n/i18n/{en,ru}/main.ftl`; `setup` asks to accept `TERMS.md` first (`--accept-terms` skips the prompt) |
| `src/help.rs` | `--help` built at runtime from the `Localizer` (clap needs static strings, so the help is rebuilt per language); `lang_from_args` reads `--lang` before clap runs | A new subcommand or flag needs `cmd-*` / `cli-arg-*` keys in both `.ftl` files |
| `src/log.rs`, `src/selftest.rs` | `phoecord log` output, `phoecord selftest` | |

## 4. Scripts

| Script | What |
|:---|:---|
| `scripts/check.sh`, `scripts/check.ps1` | All checks in order, stop at first failure (see `AGENTS.md`) |
| `scripts/check_i18n.py` | Every key of `crates/*/i18n/en/*.ftl` exists in the `ru` file with the same variables; exit 1 otherwise |
| `examples/plugins/clear_urls/` | The first third-party plugin example: `clear_urls` as a WASM package (`plugin.toml` id `clear_urls_wasm`, `wit-bindgen`), its own cargo workspace, excluded from the main one; README shows build and install |
| `scripts/dev/New-TestVm.ps1` | Creates a throwaway Hyper-V VM (Gen 2, Secure Boot, vTPM, Default Switch) for the manual checks 1.4.6 and 1.5.7, so they never touch your own Discord; run elevated |
| `scripts/gen_decisions.py` | Builds `docs/DECISIONS.md` from `Ruling:` lines of `docs/<plan>/ai/ledger.md`; `--check` exits 1 if stale |

## 5. Docs

See [`README.md`](README.md) in this folder. Project-level documents live here too: [`ARCHITECTURE.md`](ARCHITECTURE.md), [`ROADMAP.md`](ROADMAP.md), [`AGENTS.md`](AGENTS.md) + [`CLAUDE.md`](CLAUDE.md) (rules for people and AI agents). [`PORTING_PLUGINS.md`](PORTING_PLUGINS.md) is the porting guide; [`PLUGINS.md`](PLUGINS.md) is the plugin catalogue (every Vencord plugin and userplugin mapped to a layer and a task).

## 6. Config, environment and files on disk

Planned in 1.1; the table is filled as each item lands.

| Name | Kind | Default | Used by |
|:---|:---|:---|:---|
| `listen` | `config.toml` | `127.0.0.1:18463` | proxy |
| `upstream` | `config.toml` | unset (direct) | proxy (`phoecord-proxy::egress`): `socks5://host:port` or `http://host:port`; applies to new connections without a restart |
| `intercept` | `config.toml` | Discord API and Gateway hosts | proxy |
| `language` | `config.toml` | OS language, fallback `en` | all user-facing text; order: `--lang` flag, then this key, then the OS language, then `en` (`phoecord_i18n::resolve_language`); `phoecord paths` shows the result and its source |
| `--lang <code>` | CLI flag | unset | overrides `language` for one command |
| `[plugins.<id>]` | `config.toml` | per plugin | `Host::setting` |
| `PHOECORD_CONFIG`, `PHOECORD_DATA` | env | OS config/data dirs | paths |
| `PHOECORD_LOG` | env | `phoecord=info` | logging |
| `PHOECORD_NO_NOTIFY` | env | unset | `Host::notify` |
| `ca.pem`, `ca.key` | data folder | created on first run | `phoecord-proxy::ca` |
| `plugins/<id>/kv.json` | data folder | empty map | `Host::kv_*` |
| `plugins/message_logger/messages.jsonl` | data folder | created when `write_log` is on | `message_logger`, `phoecord log` |
| `install.log` | data folder | created by the installer | `phoecord-installer` rollback |

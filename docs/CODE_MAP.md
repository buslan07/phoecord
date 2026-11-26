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
| `catalog/` | The plugin catalog (1.3.11): `plugins.json` (the index, empty until a community plugin has a hashed download), `plugins.schema.json`, `README.md` (how to add an entry) |
| `README.md` | Product overview |
| `CONTRIBUTING.md`, `SECURITY.md`, `CODE_OF_CONDUCT.md` | Process, security policy, conduct |
| `LICENSE`, `NOTICE`, `LICENSES/` | GPL-3.0 text; copyright and exceptions; MIT notices for the v0 code (`MIT-v0.txt`) and the plugin SDK (`MIT-sdk.txt`) |
| `assets/logo.svg` | Logo used in the README |
| `.github/workflows/ci.yml` | CI: rustfmt, generated docs freshness, check/clippy/test on Linux, Windows, macOS, MSRV 1.88 |
| `.github/workflows/package.yml` | Native installers (2.4.5): the NSIS setup `.exe` on Windows (checked to carry `loader\version.dll`), `.app` in a `.dmg` on macOS arm64 and Intel, `.deb` (cargo-packager) and `.rpm` (cargo-generate-rpm) on Ubuntu 22.04, both test-installed (Ubuntu runner, Fedora container); `SHA256SUMS`, workflow artifacts; on PRs touching the installer, by hand, and called by `release.yml` |
| `.github/workflows/release.yml` | Releases on a `v*` tag: checks the tag against `Cargo.toml`, builds and archives the CLI (and `version.dll` on Windows) for 4 targets, reuses `package.yml` for installers, writes `SHA256SUMS`, publishes the GitHub release (created as a draft, published once every file is uploaded, so the updater never sees half a release; prerelease for `-` tags); manual run is a dry run |
| `.github/ISSUE_TEMPLATE/` | Bug, feature and plugin forms; blank issues disabled |
| `.github/PULL_REQUEST_TEMPLATE.md` | PR form with the living-docs checklist |
| `.github/dependabot.yml` | Weekly cargo updates (grouped), monthly GitHub Actions updates; Dependabot opens the `dependabot/*` branches |
| `.editorconfig`, `.gitattributes`, `.gitignore` | LF everywhere, 4-space Rust, 2-space YAML/TOML; secrets and dumps ignored |
| `signatures/` | AOB signature tables per client build (`README.md` documents the format; no Discord signatures yet) |
| `tests/fixtures/` | Decoded, scrubbed Gateway events for tests (task 2.1.10); `README.md` documents recording and scrubbing (`scripts/scrub_fixture.py`); no fixtures committed yet |
| `research/` | Out-of-workspace research projects: `gpui-reference` (GPUI app built like a shipping client) and `aob-probe` (checks AOB anchors against a binary); excluded from the workspace |
| `.github/CODEOWNERS`, `.github/workflows/assign.yml` | Every PR is assigned to Nanda070 and requests Nanda070's review |
| `scripts/` | §4 |
| `docs/` | §5 |

## 2. Crates

| Crate | Path | Kind | Depends on (ours) |
|:---|:---|:---|:---|
| `phoecord-core` | `crates/phoecord-core/` | lib | none |
| `phoecord-transport` | `crates/phoecord-transport/` | lib | core, plugin-runtime, i18n |
| `phoecord-proxy` | `crates/phoecord-proxy/` | lib | core, transport, plugin-runtime, loader (IPC types and pipe only, without the `dll` feature) |
| `phoecord-plugin-runtime` | `crates/phoecord-plugin-runtime/` | lib | core |
| `phoecord-plugin-sdk` | `crates/phoecord-plugin-sdk/` | lib (guest side) | plugin-sdk-macros, `wit-bindgen`; no host crates, by design |
| `phoecord-plugin-sdk-macros` | `crates/phoecord-plugin-sdk-macros/` | proc-macro (guest side) | none |
| `phoecord-plugins` | `crates/phoecord-plugins/` | lib | core |
| `phoecord-loader` | `crates/phoecord-loader/` | cdylib + rlib | none |
| `phoecord-installer` | `crates/phoecord-installer/` | lib | core |
| `phoecord-cli` | `crates/phoecord-cli/` | bin `phoecord` | catalog, core, proxy, transport, plugin-runtime, plugins |
| `phoecord-i18n` | `crates/phoecord-i18n/` | lib | none |
| `phoecord-catalog` | `crates/phoecord-catalog/` | lib | plugin-runtime, proxy |
| `phoecord-installer-gui` | `crates/phoecord-installer-gui/` | bin `phoecord-installer` | catalog, core, i18n, installer, plugin-runtime, plugins, proxy |

## 3. Files by crate

### `phoecord-core`

| File | What | Easy to break |
|:---|:---|:---|
| `src/lib.rs` | Re-exports `Val` and the Plugin API types | Anything re-exported here is public API for plugins |
| `src/val.rs` | `Val`: one value type for JSON and ETF; keeps atoms, big ints, binaries, so ETF round-trips unchanged | Changing a variant changes how every Gateway frame re-encodes |
| `src/etf.rs` | Erlang External Term Format decode/encode (the desktop client's Gateway encoding) | Compressed terms (tag 80) need `flate2` |
| `src/route.rs` | `route_of` (`/api/v9/x` → `/x`), `message_route_channel`, `split_query` and `query_pairs` (for `Host::request_query`) | Plugins match on the normalized route |
| `src/plugin.rs` | Plugin API v1 (frozen, 2.4.3): `API_VERSION`, `TIMER_EVENT`, `LOCAL_DELETE_EVENT`, `Layer`, `Manifest`, `MenuEntry`, `Event`, `UiEvent`, `EventKind`, `Verdict`, `Host`, `ChainHost`, `HostResponse`, `FilePart`, `Plugin` | **The contract.** Changes need an `api` PR and the owner's review; only additive changes (ARCHITECTURE.md, "Plugin API v1") |

### `phoecord-transport`

The transport boundary (task 1.1.12): everything between "an event arrived" and "here is what to do with it", with no sockets, TLS or HTTP. A transport (the proxy today, the loader's in-process hooks later) calls `Engine` and supplies a `Network`.

| File | What | Easy to break |
|:---|:---|:---|
| `src/lib.rs` | `Engine`: owns `Chain`, `Shared` and the live `Config`; `on_request`, `on_response`, `on_dispatch`, `on_send`, `chain_queued`, `fire_timer`, `apply`, `sync_packages`; result types `RequestOutcome`, `DispatchResult`, `SendResult`, `Effects` | No network or async types here: the loader must be able to link it. Keep the order of effects (locals, then frames) |
| `src/host.rs` | `ProxyHost` and `Shared` (KV, rate limits, notifications, account, timers, strings), the `Network` trait a transport implements for `Host::rest` / `Host::fetch`, `NoNetwork` | Host state must be bound to the plugin currently running in `Chain`; the `upstream` string is passed through unparsed |
| `src/config.rs`, `src/account.rs` | Live plugin configuration and account state available to plugins | Config reload must re-enable the chain and preserve the last live state |
| `src/dev.rs` | `DevBus` (1.3.9): log lines and per-plugin run records for an attached console, and the last event of each kind for `replay` (nothing copied while nobody is attached); `impl Engine`: `plugin_list`, `toggle_plugin`, `reload_plugin`, `plugin_settings`, `plugin_kv`, `replay` (dry run) | Payloads are cloned only when `DevBus::active()`; replay discards queued sends and locals |
| `src/timers.rs` | `Timers`: the store of plugin timers from `Host::schedule` (50 ms min, 24 h max, 64 per plugin, memory only), `timer_payload` | The clock is the transport's: it calls `next_due` / `take_due` and `Engine::fire_timer` |

### `phoecord-proxy`

| File | What | Easy to break |
|:---|:---|:---|
| `src/lib.rs` | `Proxy`: TLS setup, reload task, the live-gateway registry and the `locals` broadcast around an `Engine` (`Proxy` derefs to it); delivers `Effects` to the connections; re-exports `config`, `account`, `host` from `phoecord-transport` | The proxy calls the chain only through `Engine` |
| `benches/chain.rs` | Criterion bench: decode → `Proxy::on_dispatch` with every built-in plugin → encode, for a ~5 MB READY, a MESSAGE_CREATE burst and a REST body; prints p50/p99. Report: `docs/chain-bench.md` | Generated payloads only, never real traffic; `-- --test` is the smoke run |
| `GET /__phoecord` (in `src/server.rs`) | Identity answer (`phoecord <version>`) for loopback peers, so `launch` can reuse a running proxy | |
| `src/server.rs`, `src/rest.rs`, `src/gateway.rs` | Proxy loop, REST and Gateway traffic routed through `Chain` | Keep REST decisions and Gateway encode/compress round-trips lossless |
| `src/multipart.rs` | `Form::parse` / `payload_json` / `rebuild` (2.4.2 a): message sends and edits with files; plugins see `payload_json`, a change rebuilds the body with the same boundary and renames `files[n]` from `attachments[id == n].filename` | Byte-level, file data never touched; anything unparsed is forwarded as it came; bodies over 64 MiB stream untouched (`rest::MAX_MULTIPART`); the in-process transport (2.2.7) must mirror it |
| `src/codec.rs` | `Codec` from the Gateway query (`encoding`, `compress`), `Inflater` / `Deflater` for zlib-stream and zstd-stream | Compression context is per connection: once a frame changes, everything after must use our context |
| `src/ca.rs` | `Authority`: local CA load-or-create, name constraint (`CONSTRAINED_TO`), leaf certificates per SNI, rustls `ServerConfig` | `CA_NAME` is what `untrust` looks for; tests check the constraint blocks other domains |
| `src/trust.rs` | Install/remove the CA: Windows `certutil -user`, macOS keychain, Linux NSS (incl. Flatpak, Snap) | Platform code behind `cfg`; build on all 3 OSes before merging |
| `src/process.rs` | `command` / `hide_console`: child processes that open no console window on Windows (`CREATE_NO_WINDOW`) | Every child of `phoecord` and the window goes through it: started without a console, a console program would flash its own window |

| `src/hostnet.rs` | `Net` (the proxy's `Network` for the engine): the client's session headers kept in memory only (30 min TTL, dropped on 401), `Host::rest` to the client's own Discord API host with route checks, per-plugin 1/s + global 5/s caps and Discord's rate-limit buckets; `Host::rest_files` (multipart upload, 10 files, 25 MB); `Host::fetch` to allow-listed HTTPS hosts (5 MB cap, no redirects, no credentials, raw `bytes` for non-JSON answers); a small dedicated runtime so plugin calls never block the proxy | Never log or persist the session headers; `Debug` is redacted |
| `src/timers.rs` | The `drive` task: sleeps until the earliest plugin timer and fires it through `Proxy::fire_timer` (store: `phoecord-transport::timers`) | Holds only a `Weak<Proxy>` |
| `src/ipc.rs` | The proxy's end of the loader IPC (1.5.5): `spawn` (pipe server thread on Windows, no-op elsewhere), `serve_connection` (any `BufRead`/`Write`), `ConfigSettings` (`[loader]` in `config.toml`) | Settings are only the `[loader]` table, scalar values; garbage drops the connection; at most 8 connections |
| `src/dev.rs` | The dev console endpoint (1.3.9) under `/__phoecord/dev` on the proxy listener: `new_token` + `write_token` (`dev-token` in the data folder, new per start, written only by `phoecord run` through `Proxy::publish_dev_token`), `route` (loopback peer, then bearer token, then `GET /stream` NDJSON and `POST /command`) | Both checks stay in `route`; a wrong peer is refused before the body is read |
| `tests/dev_console.rs` | Real listener, raw HTTP: token and peer rejection, event stream, toggle, reload, replay, settings and KV | |
| `src/live.rs` | `LiveGateways`: registry of bridged gateway connections; sends queued outside a frame go to the newest one at once | A `LiveGuard` unregisters on drop |
| `tests/hostnet.rs` | Loopback fake Discord and outside servers: capture/reuse, refusals, rate limits, allow-list, caps, a `Chain` test | |

### `phoecord-plugin-runtime`

| File | What | Easy to break |
|:---|:---|:---|
| `src/lib.rs` | `Chain` (takes a `ChainHost`; `set_observer` for the dev console, `run_to` for one plugin, `ui` / `ui_to` for UI events, `swap_dynamic`; built-in segment, then a dynamic segment swapped by `replace_dynamic`; first `Drop`/`Reply` stops), `Outcome`, `NullHost` for tests | The stop rule is part of the API semantics; the lock is never held while a plugin runs |
| `src/wasm.rs` | `WasmRuntime` (engine + epoch clock), `WasmLimits`, `WasmPlugin`, `Right`, `Setup`, `tree` ↔ `Val` | Guest trees are untrusted: keep the bounds, depth and single-use checks |
| `src/package.rs` | `plugin.toml` format (`ui:<name>` event filters, `[[menu]]` entries, `[[setting]]` specs) and rights table (documented at the top), `PackagePlugin` with reload, `load_dir`, `PackageSet::sync` (load new folders, unload removed ones, reload changed ones at once through `reload_if_changed`), `PackageSet::reload` (forced reload of one package, the old one stays if the new one fails); `i18n/<lang>.ftl` of a package go to `PluginStrings` on load and reload (`PackageSet::with_strings`) | A broken new version must leave the old one running; the `.ftl` files are part of the reload stamp |
| `src/watch.rs` | `DebouncedWatcher` (`notify`, trailing-edge debounce, `is_healthy`), `is_package_file`; used by the proxy for `plugins/` and by `phoecord dev watch` for sources | A burst is cut after 2 s; callbacks run on a background thread |
| `tests/watch.rs`, `tests/watch_latency.rs` | Change/add/remove/debounce and a notified `PackageSet::sync`; the latency measurement (`--ignored`, release) | The latency test builds the clear_urls example |
| `tests/guest/` | A separate workspace: the test guest, built for `wasm32-wasip2` by the tests (CI installs the target) | Slow-event tests use a zero soft limit so they do not depend on machine speed |
| `tests/compat-guest/` | A separate workspace: a guest built against `wit/frozen/`, standing in for plugins already built against v1 | Never point it at `wit/phoecord.wit` |
| `tests/frozen_wit.rs` | The v1 freeze: `interface types` and the world equal `wit/frozen/`, host functions only grow, the compat guest loads and gets every event, the SDK's `API_VERSION` equals core's | A failure means a WIT change would break built plugins |
| `tests/strings.rs` | `Host::t` for packages: a string from the package's own `.ftl`, English fallback, hot reload of an edited `.ftl`, unload | Uses the guest's `STRINGS` dispatch |
| `tests/clear_urls_wasm.rs` | Builds `examples/plugins/clear_urls`, loads it as a package (also into a running chain) and checks it behaves like the native plugin | Needs the `wasm32-wasip2` target |
| `tests/hello_plugin_wasm.rs` | Builds `examples/hello-plugin`, loads it and checks that it logs each `MESSAGE_CREATE` without the text | Needs the `wasm32-wasip2` target |
| `tests/hello_plugin_ts_wasm.rs` | Builds `examples/hello-plugin-ts` with npm, loads it and checks the same logs as the Rust example | `#[ignore]`: needs Node.js; the `typescript-template` CI job runs it in release |
| `benches/overhead.rs` | Per-event cost of a WASM plugin vs a native one on a MESSAGE_CREATE payload (`cargo bench`) | Input for benchmark task 1.1.10 |
| `wit/phoecord.wit` (repo root) | The `plugin` world and `host` interface; mirrors `phoecord-core::plugin` | Change together with `plugin.rs`; frozen at v1: only new host functions, UI event names and dispatch names |
| `wit/frozen/phoecord-1.0.0.wit` (repo root) | The frozen v1 contract that `tests/compat-guest/` builds against | Never edit |

### `phoecord-plugin-sdk`

| File | What | Easy to break |
|:---|:---|:---|
| `src/lib.rs` | Bindings from `wit/phoecord.wit`, the guest `Plugin` trait, `Outcome` and `Tree` helpers, `__export!`. MIT-licensed (`license = "MIT"`), unlike the rest of the workspace | Must build without host crates; the WIT path is relative to the crate |
| `../phoecord-plugin-sdk-macros/src/lib.rs` | `#[phoecord::plugin]`: appends `::phoecord::__export!(Type)` to the `impl` | The SDK must be the dependency named `phoecord` |

### `phoecord-plugins`

| File | What | Easy to break |
|:---|:---|:---|
| `src/lib.rs` | `builtin()`: every built-in plugin in default order; uniqueness test for ids; `with_settings` puts `settings::specs` into each manifest (test: every spec sound, labels in en and ru); `I18N` (the plugins' `.ftl` bundles) and `register_strings` | Order is run order; a new `i18n/<lang>/<id>.ftl` must be listed in `I18N` (`check_i18n.py` checks) |
| `src/settings.rs` | `specs(id)`: the settings tab's `SettingSpec`s per built-in plugin (2.4.2 e), `KNOWN` | A spec's `default` must equal the plugin's own fallback |
| `i18n/{en,ru}/<plugin id>.ftl` | The plugin's own strings behind `Host::t`: `commands` (the `/phoecord` answers), `relationship_notifier`, `better_sessions`, `vc_narrator` | Every `en` key needs a `ru` twin with the same variables; keys of other plugins' subcommand answers go to `commands.ftl` |
| `src/no_track.rs` | NoTrack: answers analytics and crash-report routes locally with 204 | `wants` must stay as narrow as `on_event` |
| `src/clear_urls.rs` | ClearURLs: strips tracking query parameters from links in sent and edited messages; `extra_rules` setting | Only `content` of the body; a message with files is cleaned through its `payload_json` (2.4.2 a) |
| `src/silent_typing.rs` | SilentTyping: answers `POST /channels/{id}/typing` locally with 204 | Only that exact route; `wants` must stay as narrow as `on_event` |
| `src/unindent.rs` | Unindent: removes the indentation shared by every line of fenced code blocks in sent and edited messages | The opening line (language) and text outside blocks must stay byte for byte |
| `src/dearrow.rs` | Dearrow: YouTube embed titles and thumbnails from `sponsor.ajay.app` in new and loaded messages | The event waits for lookups: keep `MAX_LOOKUPS` small; a failed lookup pauses them for `PAUSE` |
| `src/fix_images_quality.rs` | FixImagesQuality: with `originalImagesInChat`, image attachments are served from `cdn.discordapp.com` | Only image attachments on `media.discordapp.net/attachments/` |
| `src/gif_provider_switcher.rs` | GifProviderSwitcher: answers the GIF picker's REST calls from Tenor or Giphy (setting `provider`; own keys `tenorApiKey` / `giphyApiKey` as a fallback) | Never log the URL: it holds the search text; a failed provider is paused for 5 minutes and the call goes to Discord |
| `src/custom_commands.rs` | CustomCommands: `/phoecord tags` manages tags in the kv store (`tagsList`), `/phoecord tag` fills `{{arg=default}}` and sends or shows one | Argument parsing mirrors Vencord's regex `{{(.+?)}}`, quirks included |
| `src/friend_invites.rs` | FriendInvites: create, list and revoke friend invites via `/users/@me/invites`; `iso_unix` for ISO 8601 times | The create body (`{}`) is unverified on a live account |
| `src/petpet.rs` | petpet: Vencord's ten hand frames (fetched once) over an avatar or Discord-hosted picture, encoded as a GIF with `image`, uploaded with `rest_files` | Runs inside the proxy: resolution capped at 512 px; `url` only on Discord's image hosts (manifest `fetch`) |
| `src/server_info.rs` | ServerInfo data part: `/phoecord serverinfo` from the guild (`with_counts`), its channels and your member object | Friends/blocked tabs not ported (one call per user) |
| `src/spotify_share_commands.rs` | SpotifyShareCommands: remembers your Spotify activity from presence sends (op 3) and `SESSIONS_REPLACE`; `/phoecord track`, `album`, `artist` send the link | Activity field names unverified on a live client; the presence is never changed |
| `src/unsuppress_embeds.rs` | UnsuppressEmbeds data part: `/phoecord unsuppress` reads a message's flags and flips `SUPPRESS_EMBEDS` with `PATCH` | Keep the other flag bits |
| `src/view_raw.rs` | ViewRaw data part: `/phoecord viewraw` shows a message, channel, guild, role or user as 4-space JSON (`sortKeys`, `stripSensitiveData`) | Answers are cut at 2000 characters; code fences inside are broken with a zero-width space |
| `src/review_db.rs` | ReviewDB data part: `/phoecord reviews` reads `manti.vendicated.dev/api/reviewdb/users/{id}/reviews` (50 a page), hides blocked authors | Read-only: writes need an `Authorization` header `fetch_with` refuses |
| `src/translate.rs` | Translate data part: Google translation (Vencord's endpoint, key and language table) for `/phoecord translate` and `autoTranslate` on sent messages | DeepL / Kagi fall back to Google (header rules) |
| `src/voice_messages.rs` | VoiceMessages data part: Ogg Opus parsing (duration, packet sizes), Vencord-shaped waveform, upload with flag `1 << 13` | The waveform approximates loudness from Opus bytes per ms |
| `src/i_love_spam.rs` | iLoveSpam: clears the likely-spammer bit (`1<<20`) from user `flags` / `public_flags` in messages, history and member events | Keep every other bit |
| `src/no_blocked_messages.rs` | NoBlockedMessages: learns blocked (type 2) and ignored users, drops their live messages; settings `apply_to_ignored_users` (true), `filter_history` (false) | Filtering history makes pages shorter than `limit`, which can stop the client loading older messages; that is why it is off by default |
| `src/no_reply_mention.rs` | NoReplyMention: sets `allowed_mentions.replied_user = false` on replies per Vencord's rules; settings `user_list`, `role_list`, `should_ping_listed`; remembers authors of 5000 messages | An unknown replied-to message counts as "not listed" |
| `src/no_profile_themes.rs` | NoProfileThemes: sets `theme_colors` to null in `user_profile` and `guild_member_profile` of `GET /users/{id}/profile` for every user except the signed-in one | Does nothing until `self_id` is known; also exports `profile_route` |
| `src/fake_profile_themes.rs` | FakeProfileThemes: decodes `[#rrggbb,#rrggbb]` hidden as tag characters (`U+E0000` + ASCII) in `user_profile.bio` into `theme_colors` and sets `premium_type` 2 (as Vencord); `nitroFirst` setting; `encode` for the future settings tab | Reads only the user bio, not a per-server bio; runs before `no_profile_themes` |
| `src/text_replace.rs` | TextReplace: string and regex rules on outgoing `content` (send and edit), JavaScript replacement syntax emulated; uses `regex` | JS-only regex syntax makes a rule invalid; invalid rules are skipped and logged once |
| `src/send_timestamps.rs` | SendTimestamps: `` `12:34` `` → `<t:unix:t>`; local offset from the `time` crate, `utc_offset_minutes` fallback on Unix | DST: the next occurrence is computed per calendar day |
| `src/anonymise_file_names.rs` | AnonymiseFileNames: renames `files[].filename` in `POST /channels/{id}/attachments` and the matching `attachments[].filename` in the message | Files inside a multipart message body are not renamed (the proxy never rewrites multipart) |
| `src/silent_message_toggle.rs` | SilentMessageToggle: `flags \|= 1<<12` on outgoing messages; `/phoecord silent [on\|off]` | The toggle is in memory and starts off |
| `src/valid_user.rs` | ValidUser: adds unknown mentioned users to `mentions` via `GET /users/{id}`; per-message and per-minute lookup budgets | Off by default |
| `src/valid_reply.rs` | ValidReply: fills a missing `referenced_message` from the history page or `GET /channels/{id}/messages?around=` | Off by default |
| `src/message_link_embeds.rs` | MessageLinkEmbeds: attaches a rich embed per linked message; `/phoecord linkembeds` | Off by default |
| `src/new_guild_settings.rs` | NewGuildSettings: one `PATCH /users/@me/guilds/{id}/settings` for a server joined after READY; `/phoecord newguildsettings` | Writes to Discord: once per server, tracked in kv |
| `src/override_forum_defaults.rs` | OverrideForumDefaults: rewrites `default_forum_layout` / `default_sort_order` in gateway and REST payloads | Off by default |
| `src/pause_invites_forever.rs` | PauseInvitesForever: `/phoecord pauseinvites` adds/removes `INVITES_DISABLED` in guild `features` | Writes to Discord only on the command |
| `src/read_all_notifications.rs` | ReadAllNotificationsButton: tracks read states; `/phoecord readall` acks up to 100 channels in one `ack-bulk` | Only acks channels that already have a read state |
| `src/platform_indicators.rs` | PlatformIndicators: `client_status` per user from presences; `/phoecord platforms` | Read-only: always returns `Pass` |
| `src/typing_indicator.rs` | TypingIndicator: TYPING_START per channel with 10 s expiry; `/phoecord typing` | Read-only: always returns `Pass` |
| `src/user_voice_show.rs` | UserVoiceShow: voice states by server and channel; `/phoecord voice` | Read-only: always returns `Pass` |
| `src/vc_narrator.rs` | VcNarrator: `notify` on joins, leaves, moves, mute/deafen in your voice channel; Vencord message templates | Read-only: always returns `Pass` |
| `src/member_count.rs` | MemberCount: total, online and voice counts per server; `/phoecord members` | Read-only: always returns `Pass` |
| `src/better_sessions.rs` | BetterSessions: `GET /auth/sessions` on READY, command and an interval checked on traffic; known ids in kv; `/phoecord sessions` | Read-only: always returns `Pass` |
| `src/implicit_relationships.rs` | ImplicitRelationships: `GET /users/@me/affinities/users` minus relationships; `/phoecord implicit` | Read-only: always returns `Pass` |
| `src/mutual_group_dms.rs` | MutualGroupDMs: group DMs (type 3) and recipients from READY and CHANNEL_*; `/phoecord mutualgdms` | Read-only: always returns `Pass` |
| `src/who_reacted.rs` | WhoReacted: `/phoecord reactions` lists reactions or up to 100 reactors via REST | Read-only: always returns `Pass` |
| `src/message_latency.rs` | MessageLatency: nonce vs id snowflake delay per MESSAGE_CREATE (Vencord's rules); `/phoecord latency` | Read-only: always returns `Pass` |
| `src/show_hidden_channels.rs` | ShowHiddenChannels: computes VIEW_CHANNEL from roles and overwrites; `/phoecord hidden`; optional READY rewrite | `rewrite_payload` and its READY rewrite stay off by default (ban-prone) |
| `src/show_hidden_things.rs` | ShowHiddenThings: member timeouts, paused invites and DMs, community features; `/phoecord hiddenthings` | Read-only: always returns `Pass` |
| `src/experiments.rs` | Experiments: experiment assignments from READY and `GET /experiments`; `/phoecord experiments`; optional STAFF flag | `enableIsStaff` stays off by default (ban-prone) |
| `src/fake_deafen.rs` | FakeDeafen (owner's userplugin): sets `self_mute`/`self_deaf` on outbound op 4 while `/phoecord fakedeafen` is on; the toggle re-sends your voice state at once | Rewrites outbound op 4 only |
| `src/follow_user.rs` | FollowUser (owner's userplugin): follows `followUserId` or `/phoecord follow` across voice channels with op 4 sends | Sends op 4 at once; permissions unknown to plugins |
| `src/fake_deafen_detector.rs` | FakeDeafenDetector (owner's userplugin): mute/deafen claims from voice states, flags `SPEAKING` while claimed silent; `/phoecord flagged` | No `SPEAKING` feed in the proxy yet: detection is inert |
| `src/anti_rickroll.rs` | AntiRickroll (owner's userplugin, MIT): known hosts/video IDs + custom rules over messages and history pages; `/phoecord rickrolls` | Read-only, no outside calls |
| `src/local_dev_badges.rs` | localDevBadges (owner's userplugin): adds STAFF / Early Verified Bot Developer / Active Developer badges and flags to your own user in READY, USER_UPDATE, `/users/@me` and your profile | Inbound rewrite for self only |
| `src/voice_chat_utilities.rs` | VoiceChatUtilities (owner's userplugin): `/phoecord vcu` PATCHes every member of a voice channel (disconnect, mute, deafen, move), one per timer step with the script's `waitAfter` / `waitSeconds` | Retries a member after a local refusal or a 429 |
| `src/rmcord.rs` | rmcord (owner's userplugin, GPL-3.0): the script's search-and-delete engine as a timer state machine; `/phoecord rmcord` (handled from its own `POST /interactions` so timers and REST run as rmcord) | A local refusal counts as a 429 |
| `src/silent_delete.rs` | SilentDelete (owner's userplugin): placeholder post, `deleteDelay`, delete; `/phoecord silentdelete`, `/phoecord silentpurge` (handled from its own `POST /interactions`) | Any error ends the job, as in the script |
| `src/quest_master/` | QuestMaster (owner's userplugin): `mod.rs` the timer-driven quest engine (request queue, enroll, video progress, heartbeats, claim) and `/phoecord quests`; `data.rs` quest parsing and helpers; `tests.rs` | Game/stream emulated on the wire (presence + desktop heartbeats, unverified live); ACHIEVEMENT via `fetch_with` |
| `src/state.rs` | Shared gateway-state trackers for data parts: `Shared<T>`, `Guilds`, `VoiceStates`/`VoiceChange`, bounded `Users` and `Ring<T>` | Trackers keep ids, names and counts only |
| `src/testutil.rs` | Test helpers shared by the plugin tests: `Cfg`, recording `Rec` with scripted `rest`, `j` | |
| `src/game_activity_toggle.rs` | GameActivityToggle: removes "Playing" activities from op 3 presence updates | The proxy re-sends the last presence on toggle |
| `src/presence.rs` | The presence in identify (op 2) and op 3; `Owned` (one activity of a plugin's own, the client's presence to resend); `asset_id` (Rich Presence image ids via `external-assets` / app assets, cached); `drop_empty` | A resent presence passes the chain again: remove your old copy before adding |
| `src/auto_dnd_while_playing.rs` | AutoDNDWhilePlaying: `statusToSet` on presences with a "Playing" activity, never over invisible | Must sit before GameActivityToggle and IgnoreActivities in `builtin()` |
| `src/custom_idle.rs` | CustomIdle: the client's idle presence (`afk: true`) shown as active (`idleTimeout = 0`) or held by a timer; `remainInIdle` and `/phoecord exitidle` | Under 10 minutes is impossible: the proxy cannot see input |
| `src/custom_rpc.rs` | CustomRPC: Vencord's `createActivity` added to every presence; `/phoecord rpc` resends | Asset routes unverified live; a missing asset is cached until `/phoecord rpc` |
| `src/ignore_activities.rs` | IgnoreActivities: Vencord's `isActivityNotIgnored` on every presence; `/phoecord activities`, `/phoecord ignoreactivity` (kv `ignoredActivities`) | Last in the presence part of the chain, so it can hide CustomRPC too, as in Vencord |
| `src/music_rich_presence.rs` | MusicRichPresence: polls Last.fm / ListenBrainz every 16 s with a timer, builds Vencord's activity, resends on change | Only the manifest's hosts: no self-hosted ListenBrainz |
| `src/relationship_notifier.rs` | RelationshipNotifier: `notify` on unfriend, server and group removal; your own actions in the last 60 s are ignored | Own-action memory must match the REST routes |
| `src/fake_nitro.rs` | FakeNitro (emoji): rewrites unusable emoji in outgoing messages as CDN links; `premium_type` 2 in `READY` only for tier 0 | Learns the real tier from `READY`; needs Ctrl+R after enabling |
| `src/message_logger.rs` | MessageLogger: LRU cache of messages, deletes turned into marked updates, edit history, `messages.jsonl` via `Host::append_data` | Ids may be JSON strings or ETF ints |
| `src/commands.rs` | `/phoecord plugins` and `toggle`: injects the command into search/index answers, answers `POST /interactions` locally; `Subcommand` trait; `invocation` / `kept_answer` for plugins that do a command's work as themselves; `snowflake_arg`, `message_ref` | Command shapes not yet checked on a live client |

### `phoecord-loader`

| File | What | Easy to break |
|:---|:---|:---|
| `src/lib.rs` | Module list and design notes | |
| `src/aob.rs` | `Pattern`: parse `48 8B ?? …`, `find_all`, `find_unique` (ambiguous = miss) | All-wildcard patterns are rejected on purpose |
| `build.rs`, `exports.txt` | Generates one assembly forwarding stub per `version.dll` export and `/EXPORT` link args (MSVC x86_64) | A Windows test checks `exports.txt` against the real DLL |
| `src/proxy_dll.rs` | `FORWARD_TABLE`, `resolve_export`: loads the real DLL from `GetSystemDirectoryW` on first call | Never load by name: it would resolve to this DLL |
| `src/entry.rs` | `DllMain` (only `DisableThreadLibraryCalls` + `CreateThread`, and no worker at all when the command line has `--type=`), the worker: log (including the transport `phoecord launch` passed, 2.2.8), detect the build from the mapped image (`GetModuleHandleW(null)`, logs the time in ms), load signatures | No work under the loader lock; panics are caught; child processes must still forward every export |
| `src/log.rs` | Loader log in `%LOCALAPPDATA%\PhoeCord\loader.log` | |
| `src/ipc.rs` | Loader ↔ proxy protocol (1.5.5): `Msg` / `Envelope` (`hello`, `status`, `settings.get/set/value`, `ui.event`), `encode_line` / `decode_line` / `read_line` with the 64 KiB cap, `pipe_name`, `CONNECT_DELAYS_MS`, `describe_connection` (the proxy's log line) | No OS code, used by the proxy too: a change is a protocol change, bump `PROTOCOL_VERSION` |
| `src/pipe.rs` | Windows named pipe: `PipeServer` (user-only DACL from the user's SID, `PIPE_REJECT_REMOTE_CLIENTS`, first instance), `connect`, `send_all`, `current_user_sid` | All the IPC `unsafe` lives here, each block with `// SAFETY:`; no work under the loader lock (only the worker calls it) |
| `src/pe.rs`, `src/version.rs` | PE parsing (file or mapped image: `Pe::parse_mapped`, `Pe::image_size`, `Pe::image_base` from `ImageBase`), file version resource, `.text` SHA-256 (`detect` for a file, `detect_mapped` for memory, same hash), `is_child_process`, `ClientKind::of` (Phoenix or Electron, by the `DiscordPhoenix` install folder) | Pure, tested on every OS; the hash must stay equal for file and image, `signatures/*.toml` depend on it |
| `src/signatures.rs` | `SignatureSet::load_dir`, `select`, `resolve_all` (`find_unique` + `rip`), `folder_hash` (SHA-256 over every `*.toml` name and contents in the loader's read order; half of the Phoenix check cache key, 2.2.8) | Format documented in `signatures/README.md` |
| `src/check.rs` | `check(exe_bytes, dir)` (2.2.6): the exe on disk against the signature table of its build; `transport_plan(exe_bytes, dir)` (2.2.8): the hook locators of `found` on the exe on disk, as a `Plan` (unknown build without a table); `locate_all(exe_bytes)` / `Report::locators` (2.2.6 part 2): every typed locator of `locators` on the exe, `Located { rvas, how }`, `Report::locators_passed` | `Report::passed` is about the signatures only; the locators are Phoenix's, so the caller adds `locators_passed` for a Phoenix exe |
| `src/hooks/found.rs` | Compiled as `crate::found` with and without the `dll` feature (`#[path]` in `lib.rs`; `hooks::found` re-exports it), so `phoecord launch` runs the same locators on disk. `Found<T>` typed locators over `signatures/*.toml` (`http_send`, `http_poll`, `rust_alloc` + `rust_dealloc`, `gateway_registry`, `gateway_send`), `HookPlan`: each miss (missing, none, several, outside the image) turns off only its hook | "Unique or off"; one allocator shim alone is no allocator |
| `src/hooks/guard.rs` | `Breaker`: `catch_unwind(AssertUnwindSafe)` around every hook entry; a panic logs and trips the hook, a bad input only falls back | The fallback is always "pass the traffic through" |
| `src/hooks/http.rs`, `src/hooks/gateway.rs` | Bridge to `Engine::on_request` / `on_response` and `on_dispatch` (client JSON in, verdict out; changed bodies only without an allocator → pass) | Detour ABI shims (2.1.5-2.1.7) are not here yet |
| `src/hooks/abi.rs` | Reads method, URL and body of the client's REST request through a `Memory` that may refuse reads (`read_head`, `read_body`, `decode_method`); layout of Phoenix 0.1.0.2208 | Platform-independent, tested on byte buffers |
| `src/hooks/detour.rs` | The `send` inline detour: naked entry, `on_send`, `ProcessMemory` (`ReadProcessMemory`), `SendSink`, `prepare` | Windows x64 only; read-only (2.2.2 part 1) |
| `src/hooks/mod.rs` | `HookRuntime` (the three guarded entry points), `install` / `runtime` | Needs the `dll` feature (pulls `phoecord-transport`) |
| `src/theme/mod.rs` | `Theme::from_toml` / `load` (`name`, `[colors]` → `Hsla`, bad entries skipped as `Problem`s), `ThemeWatcher::poll` (mtime + length, every `POLL_INTERVAL` = 2 s; `Loaded` / `Removed`) | Field names are free-form until Phoenix's palette is known (2.1); no hook yet |
| `src/theme/parse.rs` | `parse_color`: hex, `rgb(a)`, `hsl(a)` → `Hsla` (standard RGB → HSL) | Pure, tested on every OS |
| `src/theme/import.rs` | `import(css)` (2.3.2): a Vencord CSS theme's colour variables → palette fields (`MAP`), `var()` resolved in the file, `Imported { colors, unsupported, bad }`, `to_toml` | `MAP` holds PhoeCord's placeholder field names; it moves with Phoenix's palette (2.3.1) |
| `themes/default.toml`, `themes/light.toml`, `themes/high-contrast.toml` | The built-in themes (2.3.1, 2.3.2); default: Discord's published surface colours under placeholder field names | Not yet a no-op default — Phoenix's real palette field names are unknown; parsed in a loader test (`include_str!`) so a typo is caught here |
| `src/ui/mod.rs` | Module root for the UI hooks of stage 2.3 | Nothing wired to a real Phoenix hook; see `src/ui/message_logger.rs` |
| `src/ui/message_logger.rs` | `Accessory::for_message` (2.3.4): `MessageState { deleted, history }` → `Deleted` / `Edited { history }` / `None` | Pure; not called from a hook — `MessageRow::render`'s address is not in `phoecord-recon` |
| `src/locators/mod.rs` | `Found<T>`, `LocatorError`, `Located<T>`; `exactly_one`, `winner`; `Image` (`.text`, `.rdata`, `.pdata`, `image_base` by RVA: `from_pe`, `find_bytes`, `find_str`, `find_ptr`, `function_of`) | Pure, mock sections in tests; never fall back to an offset from another build |
| `src/locators/index.rs` | `CallIndex::build`: `call rel32` to function starts and RIP-relative `lea` / `mov` into `.text` / `.rdata`; `calls_to`, `refs_to`, `callers`, `loaders` | A byte scan, not a decoder: targets are checked, and locators should intersect several facts |
| `src/locators/http_transport.rs` | `find` (2.2.5 part 2): `ReqwestTransport::send`'s entry by masked AOB, with the struct size/alignment the recon notes read off the vtable | One build only (`0.1.0.2208`); the vtable address itself is not re-derived (two identical copies in this build) |
| `src/locators/allocator.rs` | `find`: `__rust_alloc` and `__rust_dealloc` together by masked AOB on each shim's own prologue | Both or neither — one shim alone is no allocator (same rule as `hooks::found::Allocator`) |
| `src/locators/gateway.rs` | `find`: the Gateway registry's builder/query function by masked AOB (no fixed table address exists, per 2.1.6) | What a 2.2.3 detour hooks, not a static address |
| `src/plan.rs` | `Plan::resolve` / `from_hooks` / `unknown_build`: from a `Probe` per `Target` (`probe` turns a `Located<T>` into one) to a `Route` per `EntryPoint` and on/off per `UiFeature`, `report` for the log; `all_in_process` (the whole-launch condition of 2.2.8) and `net_report` | Pure; dependencies between entry points live only in `requires`; the 2.3.7 scenarios are tested in `tests/degradation.rs` |
| `src/transport.rs` | `InProcessTransport` (`request`, `response`, `dispatch`: plan gate, `HookRuntime`, delivery), `Delivery` (queue for the dispatch detour, `FrameSink`, `Dropped` counts), `Clock` (plugin timers on one thread) | Never `Clock::stop` under the loader lock; queued payloads reach the client only through `dispatch` |
| `src/hook.rs` | Inline detours (`retour`), installed with other threads suspended, `uninstall_all` | Hooks must be removed before `FreeLibrary` |
| `src/choice.rs` | Phoenix transport choice shared by launcher and loader (2.2.8): `Mode` (`in-process` / `proxy`), the environment the launcher sets (`ENV_MODE`, `ENV_CACHE`, `ENV_SIGNATURES_HASH`) and `Launched::from_env`, `CheckCache` (`phoenix-check.json`: verdicts keyed by `.text` hash + signature folder hash, newest 8 builds, an `ExeStamp` so an unchanged exe is not re-hashed), `mark_failed` | Two seams not called yet: `Launched::from_env` (2.2.7 decides whether to install the network hooks from it) and `mark_failed` (written from the loader when a hook fails in an in-process session); a broken cache file reads as empty |
| `src/inject.rs` | `spawn_suspended`, `spawn_suspended_with_env` (this process's environment plus extra variables, `CREATE_UNICODE_ENVIRONMENT`), `queue_load_library`, `resume` (2.1.7): `CreateProcess` suspended, `QueueUserAPC` to `LoadLibraryW` on the target's own main thread, `ResumeThread` — how Phoenix is reached, since it imports no DLL this loader can sideload | The remote allocation for the DLL path is never freed (freeing before the APC runs would be a use-after-free in the target); `default-features = false` is all `phoecord-cli` needs |
| `examples/host.rs` | Throwaway exe that loads `version.dll` from its folder for manual checks | |

### `phoecord-installer`

| File | What |
|:---|:---|
| `src/lib.rs` | `Channel` (Stable, PTB, Canary, Phoenix), module list |
| `src/discover.rs` | `discover()`: Stable/PTB/Canary on Windows, macOS, Linux (PATH, Flatpak, Snap); Desktop Phoenix on Windows only (`%LOCALAPPDATA%\DiscordPhoenix\app\Discord.exe`, `Channel::Phoenix`, version from the executable; Linux and macOS locations are not known, so nothing is looked for there); `discover_with(extra)` adds the folders from `clients`, one entry per real folder (canonical paths, so a junction or symlink to a listed folder is not listed twice); `is_listed` for Choose folder; `clients_at(dir)` checks a folder the user picked (for Phoenix: `DiscordPhoenix`, its `app` folder or the folder above); `discover_in(&Roots)` for tests |
| `src/pe_version.rs` | `file_version(path)`: the `FileVersion` of a Windows executable from its `VS_VERSIONINFO` resource, read by seeking (headers, resource directory, version block; the 500 MB Phoenix `Discord.exe` is never mapped or run); `None` for anything that is not a PE with a version, never panics | Phoenix's folder has no `app-x.y.z` to take the version from |
| `src/system.rs` | `Os`, `Env`, the `System` trait (trust store, Run key, launchctl, systemd); `RealSystem` is the only code that touches the machine |
| `src/ca.rs`, `src/autostart.rs` | Install steps: local CA + trust; Run key / LaunchAgent / systemd user unit |
| `src/ca.rs` (installer) | `ensure_ca`: trusts the CA when the fingerprint stored in `install.log` differs or `System::is_trusted` says no; `is_untrusted` drives the `run`/`launch` warning | A failing check must not re-prompt on every `setup` |
| `src/shortcut.rs` | Start-menu shortcut "Discord … (PhoeCord)" (Phoenix: "Discord Phoenix (PhoeCord)") that runs `phoecord launch --channel …`: Windows `.lnk` (PowerShell + WScript.Shell, every value quoted), Linux `.desktop` | Created and removed through `System`, logged in `install.log` |
| `src/launch.rs` | `choose` (the client for `--channel`, or the only one installed; several without `--channel` is an error; Phoenix only when named: it is never counted as "the only one"), `client_command` (Phoenix starts its `Discord.exe` directly from `app\`, no `Update.exe`, plain launch with `--proxy-server`: used only when there is no loader to inject; a macOS `.app` client starts through `open -n -a ... --env ... --args`), `probe_proxy` (is PhoeCord already on the port: `GET /__phoecord`) | `launch` never defaults silently to Stable or to Phoenix |
| `src/phoenix.rs` | Phoenix transport choice (2.2.8): `choose(&Inputs, &mut dyn Probe)` → `Choice { mode, why, missing, loader, signatures_hash }` (forced mode, not Windows, no loader, no signatures, check failed, marked failed, cached, checked); `DiskProbe` reads `Discord.exe` once (`.text` hash, `transport_plan`); `injected` (exe, args, environment and DLL of the injected start: proxy flags in proxy mode, mode + cache + signature hash in-process); `spawn` (`phoecord_loader::inject`); `remove_cache` | Every decision is tested with fake locator results; `auto` never fails a launch, it falls back to the proxy; forced `in-process` without Windows or the loader is `Unavailable` |
| `src/vencord.rs` | `phoecord import-vencord`: finds Vencord / Vesktop / Equicord `settings.json` (or `--path`), maps enabled plugins by `Manifest::vencord` | Reads only; never writes Vencord's files; one file is listed once on case-insensitive file systems |
| `src/signatures.rs` | Signature hot-fix (2.2.10): checks `*.toml` files, writes them into the loader's folder by rename, logs the ones it created, lists what is there | The folder is under `%LOCALAPPDATA%`, not PhoeCord's data folder; all files are checked before the first write |
| `src/terms.rs` | `TERMS_VERSION`, both terms texts (`include_str!` of `TERMS.md` / `TERMS.ru.md`), `needs_acceptance`, `accept` (stored in the data folder and logged so `uninstall` removes it) | Bump `TERMS_VERSION` whenever the text changes, so users are asked again |
| `src/system.rs` additions | `stop_proxies` (only `run`/`launch` processes of PhoeCord's own binaries, never the calling process; `select_proxy_pids` is tested), `shortcut_create`, `Env.start_menu`, `Env.applications_dir` | |
| `src/install.rs`, `src/uninstall.rs` | `install()` (idempotent), `install_with_progress` (reports each `Phase` start/end for the window) and `uninstall_all()` (undoes the log newest-first, `UninstallReport`; also removes `phoenix-check.json`, which `launch` writes unlogged) |
| `src/log.rs` | `install.log` (JSON lines in the data folder): every step the installer made, so rollback removes only what it created |
| `src/update.rs` | PhoeCord's own automatic updates from GitHub Releases (2.4.5): `parse_releases` (no drafts, prereleases or `-` tags), `detect` (`Kind`: NSIS, Mac `.app`, `.deb`, `.rpm`, portable), `asset_name`, `parse_sums`, `Updater` (`check` once a day: download + SHA-256 against the release's `SHA256SUMS`; `ready` hashes again; `settle` records an applied update; `updates/state.json`), `plan` / `execute` (silent NSIS setup, `.app` swap from the `.dmg`, `pkexec dpkg`/`rpm`), `auto_enabled` / `set_auto` (`[update] auto`); network only through the `Fetch` trait | Every refusal (missing asset, missing sums, unlisted, mismatch) installs nothing; asset names must match `scripts/package/name-files.sh`; NSIS flags must not start with `P`, `R` or `S` (`GetOptions` matches prefixes) |
| `src/uninstall.rs`, `src/autostart.rs` additions | `stop_proxies` (`phoecord stop`: the proxies of every binary in the log), `autostart::is_installed` | |
| `src/loader.rs` | Loader DLL step (1.5.6, opt-in `setup --loader`, Windows only): `install_loader` records `Step::LoaderWanted` and copies `version.dll` (`default_source`: next to the `phoecord` binary, else in its `loader\` folder, where the Windows setup puts it, `DLL_DIR`) into the newest `app-x.y.z` (`discover::newest_app_dir`); `ensure_loader` re-applies it on `run`/`launch` after Discord updates; every copy is logged as `Step::LoaderCopy` with its SHA-256 | A `version.dll` whose hash is not recorded for that file is foreign: never overwritten, never removed by `uninstall` (`Kept::NotOurs`); `ensure_loader` never fails its caller |

### `phoecord-catalog`

Plugin packages and the community catalog (1.3.11), shared by the CLI and the installer window.

| File | What | Easy to break |
|:---|:---|:---|
| `src/store.rs` | Plugin packages: stage a `.zip` / folder / `plugin.toml` into `plugins-staging/`, check it, `commit` into `plugins/<id>` and switch it on, `remove`, HTTPS-only `download` (32 MB cap), `verify_package` (the WASM load check) | Zip paths outside the archive are refused; only `plugin.toml`, `plugin.wasm`, README, LICENSE and `i18n/*.ftl` are copied |
| `src/index.rs` | `Index` / `Entry` of `catalog/plugins.json`: parsing, validation (`parse_strict` is the CI check), `Mark` (`.catalog.json` next to a catalog-installed package), `updates`, `is_newer` | A hand-installed package has no mark and is never offered a catalog update |
| `src/catalog.rs` | `fetch_index`, `download_entry` (SHA-256 must equal the entry's), `stage_entry` (the staged package must have the entry's id and version and no right the entry lacks); URL from `PHOECORD_CATALOG_URL` or the `main` branch on GitHub; tests run a local HTTP fixture | Hash check comes before unpacking; `stage_entry` discards the staging folder on any mismatch |
| `src/net.rs` | One size-capped GET: HTTPS only, plain `http://` only to 127.0.0.1 / `[::1]` / `localhost` (no redirects then) | `user@host` tricks must not pass `is_loopback_http` |

### `phoecord-installer-gui`

The installer window (stage 1.4.3). Every change to the machine goes through `phoecord-installer`, the same code as `phoecord setup` / `uninstall`.

| File | What | Easy to break |
|:---|:---|:---|
| `src/main.rs` | Window options, icon, `--dry-run` / `--demo` / `--demo-empty` / `--open <path>`; installs a downloaded PhoeCord update before the window opens (`update::at_start`) | `windows_subsystem = "windows"`: no console on Windows |
| `src/app.rs` | `App`: header with tabs (adapts to narrow windows), tab transition, dialogs (scrim + card), toasts, drop overlay, messages from jobs | Dialog card is an `Area` anchored at the centre; its scroll area sets `min_scrolled_height` or it shrinks to half the window |
| `src/update.rs` | The window's side of PhoeCord's updates: `at_start` (say what was updated, install a verified download and exit when the setup or the new window takes over), `check_in_background` (sends `Msg::Update`), `Note`; the Settings tab has the switch and the status line | Nothing runs in `--dry-run` |
| `src/backend.rs` | `Mode` (real or dry-run sandbox with a simulated trust store), `read_status` (clients, install log, trust, proxy probe), background jobs (install, uninstall, launch, plugin jobs); a Phoenix `launch` that exits 0 with no proxy listening started in-process (2.2.8) and counts as started | `Env.proxy_exe` is the `phoecord` binary next to the window (or `PHOECORD_BIN`), never the window itself |
| `src/packages.rs` | Re-exports `phoecord_catalog::store` (the package code moved there in 1.3.11) | none |
| `src/tabs/catalog.rs` | The Catalog list of the Plugins tab (1.3.11): reads the index, rows with Install / Update, an "updates available" pill; installs go through `catalog::stage_entry` and the normal rights dialog | The index is fetched only when the list is opened, or when a package installed from the catalog exists (to offer updates) |
| `src/theme.rs` | The one palette, spacing, radii, type scale, Onest fonts + Phosphor icons, the app icon as window icon and in-app mark | Contrast is tested; add colours here, never inline |
| `src/splash.rs` | Opening animation (~1.4 s): the flame rises and catches (flash, two shock rings, embers), the name comes in letter by letter, then the window opens out of the flame as an iris and comes in (`reveal`); click or key skips, off with reduced motion | Own clock capped per frame, so slow first frames do not eat it; drawn last in `update`, above everything |
| `src/onboarding.rs` | First start: welcome, terms, pick a Discord, done (starts the install) | Shown on the first start of each app version: the `onboarded` file in the data folder holds the version that finished it; the terms step is skipped when already accepted |
| `src/motion.rs` | Interruptible tweens with custom easing, durations, reduce-motion switch | Tweens request repaints only while they move |
| `src/widgets.rs` | Buttons, segmented tabs, switch, pill, banner, field, spinner, check mark | Every widget reports itself to accesskit and shows a focus ring |
| `src/terms.rs` | Draws `TERMS.md` from its Markdown (headings, bullets, bold, code, links as text) | |
| `src/tabs/*.rs` | Install, Plugins (+ dialogs), Settings, About, terms screen | Strings only through `gui-*` keys (tested in `text.rs`) |
| `assets/installer/` | NSIS header and sidebar, DMG background | Written by `scripts/package/make_art.py` |
| `assets/installer/nsis/installer.nsi` | PhoeCord's NSIS template (`template` in `Cargo.toml`): cargo-packager 0.11.8's, plus `loader\version.dll`, `phoecord stop` before copying, `phoecord uninstall` on removal (not with `/UPDATE`), `/LAUNCHPROXY` | Every change is marked `PhoeCord:`; compare with upstream when `PACKAGER_VERSION` changes |
| `assets/linux/phoecord-installer.desktop` | Menu entry of the `.rpm` (the `.deb` gets cargo-packager's own) | |
| `assets/icon/` | App icon: `icon.svg` (source), `icon-256.png` (window and in-app mark), `icon-1024.png`, `icon.ico` (Windows), `icon.icns` (macOS) | Regenerate the rest from the SVG when it changes (`python scripts/package/make_art.py`, which also draws `assets/installer/`) |
| `assets/installer/nsis/Russian.nsh` | Russian strings for cargo-packager's own NSIS pages (`custom-language-file` in `Cargo.toml`) | UTF-8 with BOM; keys must match cargo-packager's `English.nsh` for the pinned version |
| `build.rs` (here and in `phoecord-cli`) | Embeds `assets/icon/icon.ico` and version info in the Windows `.exe` files (`winresource`) | Taskbar, Explorer and Settings > Apps read the icon from the exe; `with_icon` covers only the title bar |
| `assets/fonts/Onest-*.ttf` | Onest 400/500/600, static instances of the variable font | SIL OFL 1.1, `LICENSES/OFL-onest.txt` |

### `phoecord-i18n`

| File | What | Easy to break |
|:---|:---|:---|
| `src/lib.rs` | `Localizer` (`new`, `t`, `t_args`, `language`), `normalize`, `detect_os_language`, `resolve_language`, `SUPPORTED`; `PluginStrings` (per plugin and language bundles behind `Host::t`) | Bundles are `include_str!`-ed: a new language needs a `source()` arm |
| `i18n/en/main.ftl`, `i18n/ru/main.ftl` | Shared strings; English is the source | Every `en` key needs a `ru` twin with the same variables (`check_i18n.py`) |

### `phoecord-cli`

| File | What | Easy to break |
|:---|:---|:---|
| `src/upgrade.rs` | `phoecord upgrade` (check and download now), `phoecord stop`, and `at_run_start`: at the start of `run` (autostart only, never `launch`) says what was updated, installs a downloaded update and exits when the setup takes over, then checks in the background | `run` returns right away when an update takes over |
| `src/main.rs` | Commands: `run`, `launch`, `setup` (installer `install()`), `uninstall` (installer `uninstall_all()`), `stop`, `upgrade`, `trust`, `untrust`, `plugins`, `enable`, `disable`, `toggle`, `log`, `selftest`, `paths` (config, CA, plugins, the Phoenix folder and build from `discover` or "not found", language); `run` loads WASM packages; `--channel` accepts `phoenix` (`launch`, `setup`); `launch --channel phoenix` picks the transport (`phoenix::choose`, `--transport auto\|proxy\|in-process` over `[phoenix] transport`), prints it and injects the loader; in-process starts no proxy at all | `plugins`/`enable` know built-ins only, not WASM packages; every user-facing string comes from `crates/phoecord-i18n/i18n/{en,ru}/main.ftl` (Phoenix: `cli-paths-phoenix`, `cli-paths-phoenix-missing`, `cli-err-phoenix-not-installed`); `setup` asks to accept `TERMS.md` first (`--accept-terms` skips the prompt); `setup --loader` installs the loader DLL, and `run`/`launch` call `ensure_loader` first |
| `src/help.rs` | `--help` built at runtime from the `Localizer` (clap needs static strings, so the help is rebuilt per language); `lang_from_args` reads `--lang` before clap runs | A new subcommand or flag needs `cmd-*` / `cli-arg-*` keys in both `.ftl` files |
| `src/dev/mod.rs`, `src/dev/watch.rs`, `src/dev/inspect_pe.rs` | `phoecord dev ...` (one file per tool); `dev inspect-pe <exe> [--functions]` (2.1.2): read-only PE report with the `object` crate: imported and delay-loaded DLLs (sideload candidates marked), `DllCharacteristics`, Load Configuration `DependentLoadFlags`, TLS callbacks, CLR flag, section layout (virtual address and raw offset); `dev watch <source>`: `cargo build --target wasm32-wasip2 --release`, atomic copy into `plugins/<id>/`, rebuild on source changes | The proxy side is `Proxy::start_package_watcher` (`phoecord-proxy/src/lib.rs`); `help.rs` localizes second-level commands by `cmd-<parent>-<child>` / `cli-arg-<parent>-<child>-<arg id>` |
| `src/dev/inject.rs` | `dev inject <exe> [--dll <path>] [-- args...]` (2.1.7 prototype): calls `phoecord_loader::inject`, reports the pid and elapsed ms. Windows only | A developer tool; `phoecord launch --channel phoenix` injects the same way (2.2.8) |

| `src/catalog.rs` | `phoecord install <id\|https link\|path>`, `update [id]`, `remove <id>`, `catalog` (1.3.11): stage through `phoecord-catalog`, print what the plugin asks for, ask (`--yes` skips; no terminal and no `--yes` means no) | `uninstall` stays the undo of `setup`, so removing a plugin is `remove`; an install from a link or a path has no hash and says so |
| `src/log.rs`, `src/selftest.rs` | `phoecord log` output, `phoecord selftest` | |
| `src/check.rs` | `phoecord check <exe> [--signatures <dir>]` (2.2.6): prints the build, a table of signature name, status and RVA, then one row per typed locator (what it found and how, or why not) | Exit code 1 on any signature miss, on a locator miss when the exe is Phoenix (`DiscordPhoenix` in its path), a skipped file, an empty or missing table; the default folder is `signatures` under the current directory, not the loader's `%LOCALAPPDATA%` one |
| `src/signatures.rs` | `phoecord signatures install <file or folder>` and `signatures list` (2.2.10) | Windows only: the folder comes from `phoecord_loader::signatures::default_dir` |
| `src/theme.rs` | `phoecord theme import <css> [--out <file>] [--name <name>]` (2.3.2): prints or writes the `theme.toml`, lists unsupported and skipped variables on stderr | Exit code 1 when no colour carries over |
| `src/dev/mod.rs`, `src/dev/console.rs` | `phoecord dev console` (1.3.9): attaches with the token from `dev-token`, prints the stream (`--filter`, `--plugin`), runs `toggle` / `reload` / `settings` / `kv` / `replay` typed on stdin. Other `dev` subcommands get their own file here | Strings through `cli-dev-*` keys; nested help keys are `cmd-dev-<sub>` and `cli-arg-dev-<sub>-<arg>` (`help.rs`) |

## 4. Scripts

| Script | What |
|:---|:---|
| `scripts/check.sh`, `scripts/check.ps1` | All checks in order, stop at first failure (see `AGENTS.md`) |
| `scripts/check_i18n.py` | Every key of `crates/*/i18n/en/*.ftl` exists in the `ru` file with the same variables; exit 1 otherwise |
| `scripts/check_builds.py` | Runs `phoecord check` on every client exe in a folder outside the repository (2.2.6, 2.2.9); exit 1 on any failure, with fewer than `--min-builds` exes, or with `--phoenix` on an exe outside a `DiscordPhoenix` folder | Not wired into `check.ps1` or CI; the builds are never committed |
| `scripts/scrub_fixture.py` | Rewrites a decoded Gateway event into a `tests/fixtures/` entry with no real data (task 2.1.10): every string/snowflake replaced, keys/nesting/types and the envelope (`op`/`t`/`s`/`type`) kept; `--map` for a consistent id across files; `--self-test` | Not wired into `check.ps1`; run by hand when adding a fixture |
| `examples/plugins/clear_urls/` | The first third-party plugin example: `clear_urls` as a WASM package (`plugin.toml` id `clear_urls_wasm`, `wit-bindgen`), its own cargo workspace, excluded from the main one; README shows build and install |
| `examples/hello-plugin/` | The smallest SDK plugin (`hello_plugin`): logs `MESSAGE_CREATE` ids; its own cargo workspace, built by the `sdk-example` CI job | `Cargo.lock` is committed and CI builds with `--locked` |
| `examples/hello-plugin-ts/` | The TypeScript template (`hello_plugin_ts`): the same plugin; `build.mjs` (esbuild + `componentize-js` with WASI features off), `src/tree.ts` (value tree helpers) | `package-lock.json` is committed, versions pinned; `types/`, `dist/`, `plugin.wasm` are build output |
| `scripts/update_contributors.py` | Rewrites the README contributors block (avatars between `<!-- contributors:* -->` markers) from GitHub's contributors list, bots left out; `--check` exits 1 if stale. Run after merging a new contributor's PR |
| `scripts/package/make_art.py`, `scripts/package/name-files.sh`, `scripts/package/README.md` | Renders the app icons and installer pictures from `icon.svg`; renames the packaged `.exe`/`.msi` to `PhoeCord_<version>_…`; how to build the installers locally (`cargo packager`, config in `crates/phoecord-installer-gui/Cargo.toml`) |
| `scripts/dev/New-TestVm.ps1` | Creates a throwaway Hyper-V VM (Gen 2, Secure Boot, vTPM, Default Switch) for the manual checks 1.4.6 and 1.5.7, so they never touch your own Discord; run elevated |
| `scripts/gen_decisions.py` | Builds `docs/DECISIONS.md` from `Ruling:` lines of `docs/<plan>/ai/ledger.md`; `--check` exits 1 if stale |

## 5. Docs

See [`README.md`](README.md) in this folder. Project-level documents live here too: [`ARCHITECTURE.md`](ARCHITECTURE.md), [`ROADMAP.md`](ROADMAP.md), [`AGENTS.md`](AGENTS.md) + [`CLAUDE.md`](CLAUDE.md) (rules for people and AI agents). [`PORTING_PLUGINS.md`](PORTING_PLUGINS.md) is the porting guide; [`plugins/`](plugins/README.md) is the developer documentation for third-party WASM plugins; [`PLUGINS.md`](PLUGINS.md) is the plugin catalogue (every Vencord plugin and userplugin mapped to a layer and a task).

| Path | What |
|:---|:---|
| `docs/phase-1/` | Phase 1 plan, task table, agent rules, spec, implementation plan and ledger (closed 26.11.2026, kept as history) |
| `docs/plugins/` | Developer docs for third-party WASM plugin authors (2.4.4): `README.md` (index), `quick-start.md` (Rust and TypeScript), `events.md` (events, verdicts, `events` filters, `PHOECORD_TIMER`, `PHOECORD_LOCAL_DELETE`), `rights.md` (rights, limits, what the user sees), `publishing.md` (package layout, `plugin.toml` fields, versioning and the v1 freeze, catalog, licensing), `localization.md` (`i18n/<lang>.ftl`, `Host::t`) |
| `docs/phase-2/` | Phase 2 plan (Phoenix Canary → v1.0): `README.md` (task table), `AGENTS.md`, `ai/spec.md`, `ai/plan.md`, `ai/ledger.md` |
| `docs/re/` | Reverse-engineering notes on the Phoenix client: `gpui-lifecycle.md` (GPUI init chain, main window, injection points) |
| `docs/research/` | Research documents: `phoenix-desktop.md` (sunbird study), `gpui.md` (GPUI map), `phoenix-recon.md` (findings for 2.1.2, 2.1.5, 2.1.6; method and findings for 2.1.3, 2.1.4, 2.3.7; 2.1.9 completes it) |

## 6. Config, environment and files on disk

Planned in 1.1; the table is filled as each item lands.

| Name | Kind | Default | Used by |
|:---|:---|:---|:---|
| `listen` | `config.toml` | `127.0.0.1:18463` | proxy |
| `upstream` | `config.toml` | unset (direct) | proxy (`phoecord-proxy::egress`): `socks5://host:port` or `http://host:port`; applies to new connections without a restart |
| `intercept` | `config.toml` | Discord API and Gateway hosts | proxy |
| `[update] auto` | `config.toml` | `true` | `phoecord-installer::update::auto_enabled`: PhoeCord's own daily update check and install (2.4.5); the window's Settings switch writes it |
| `PHOECORD_UPDATE_URL` | env | this repository's GitHub releases API | `phoecord-installer::update::releases_url`: a mirror for manual tests; plain `http://` only to this computer |
| `PHOECORD_CATALOG_URL` | env | the `main` branch's `catalog/plugins.json` on raw.githubusercontent.com | `phoecord-catalog::catalog::index_url`: a mirror or a test server; plain `http://` only to this computer |
| `language` | `config.toml` | OS language, fallback `en` | all user-facing text; order: `--lang` flag, then this key, then the OS language, then `en` (`phoecord_i18n::resolve_language`); `phoecord paths` shows the result and its source |
| `clients` | `config.toml` | empty | extra folders with a Discord client installed outside the default places, added by "Choose folder…" in the installer window; `launch`, `setup` and the window look in them too (`discover_with`) |
| `--lang <code>` | CLI flag | unset | overrides `language` for one command |
| `[phoenix] transport` | `config.toml` | `auto` | `phoecord launch --channel phoenix` (2.2.8): `auto` picks per launch from the cached check, `proxy` / `in-process` force one (debugging) |
| `--transport auto\|proxy\|in-process` | CLI flag of `launch` | unset | overrides `[phoenix] transport` for one launch; `in-process` is refused for any client but Phoenix |
| `PHOECORD_TRANSPORT`, `PHOECORD_CHECK_CACHE`, `PHOECORD_SIGNATURES_HASH` | env of the injected Phoenix | set by `launch` | `phoecord_loader::choice::Launched`: the launch's transport, the cache file and signature folder hash for the loader's "failed" mark |
| `[plugins.<id>]` | `config.toml` | per plugin | `Host::setting` |
| `PHOECORD_CONFIG`, `PHOECORD_DATA` | env | OS config/data dirs | paths |
| `PHOECORD_LOG` | env | `phoecord=info` | logging |
| `PHOECORD_NO_NOTIFY` | env | unset | `Host::notify` |
| `ca.pem`, `ca.key` | data folder | created on first run | `phoecord-proxy::ca` |
| `dev-token` | data folder | written at every proxy start | `phoecord-proxy::dev`, read by `phoecord dev console` |
| `plugins/<id>/kv.json` | data folder | empty map | `Host::kv_*` |
| `plugins/message_logger/messages.jsonl` | data folder | created when `write_log` is on | `message_logger`, `phoecord log` |
| `install.log` | data folder | created by the installer | `phoecord-installer` rollback |
| `updates/` | data folder | created by the first update check | `phoecord-installer::update`: `state.json` (last check, pending and applied update, last error) and the downloaded installer until it is installed |
| `phoenix-check.json` | data folder | written by `launch --channel phoenix` | `phoecord_loader::choice::CheckCache`: the transport verdict per Phoenix build and signature folder, and "failed" marks; removed by `uninstall` |
| `plugins-staging/` | data folder | created while a plugin waits for the user's yes in the installer window, removed after | `phoecord-catalog::store` |
| `launch.log` | data folder | written each time the installer window starts Discord | `phoecord-installer-gui::backend`: `phoecord launch` output, shown when it stops |
| `PHOECORD_BIN` | env | the `phoecord` binary next to the window | `phoecord-installer-gui`: which binary autostart and shortcuts run |
| `PHOECORD_GUI_SANDBOX` | env | `<temp>/phoecord-gui-dry-run` | `phoecord-installer --dry-run`: the sandbox folder |

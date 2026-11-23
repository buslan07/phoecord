# Phase 1. Before the Phoenix Canary public beta: plan for people

The work plan until Phoenix Canary ships, written 23.11.2026 at Nanda's request. Goal: PhoeCord fully works on today's Discord through the network layer, plugins install without a rebuild, the app speaks English and Russian, and the installer, the loader and our knowledge of GPUI are ready for the day Phoenix Canary comes out.

> This is the plan for people. The spec, the step-by-step tasks and the ledger are in [`ai/`](ai/). If this file and [`ai/spec.md`](ai/spec.md) disagree, the spec wins.

## Status

23.11.2026, `main` `ea73482`. Stage 1.0 is closed: plan, docs structure and English-only docs merged (PR #4), Dependabot updates folded into one PR. Phase 0 is closed: workspace of 8 crates, Plugin API v1, plugin chain, AOB scanner; the Gateway codecs, the local CA, certificate trust and `no_track` are ported; CI is green on Linux, Windows and macOS. Next: stage 1.1, starting with 1.1.1.

## What we get

- **A user** installs PhoeCord with one button, starts Discord and gets every Vencord plugin that works on the network (69 of them, fully or in their data part, including the owner's userplugins; see [`docs/PLUGINS.md`](../PLUGINS.md)), on today's client, without Vencord, in English or Russian. Uninstalling is one button too and leaves nothing behind.
- **A plugin author** writes a plugin in Rust or TypeScript, builds a `.wasm`, drops it into the `plugins/` folder, and it runs without rebuilding PhoeCord, in a sandbox, with the rights its manifest asks for.
- **The team**, when Phoenix Canary ships, has: a working loader (`version.dll`) proven on today's client; a signature format; a GPUI reference build for reverse engineering; a report on which GPUI functions to look for.

## Decisions

- Remove every mention of the original upstream project from the repository; the copyright lines and the MIT text in `LICENSE` stay because the license requires them (Nanda 23.11.2026: "remove all mentions of [the upstream project]", translated).
- The repository is public, `nanda070/phoecord` (Nanda 23.11.2026: "Public", translated).
- The copyright holder is Cheterin Group (Nanda 23.11.2026: "Copyright in LICENSE", translated).
- The in-client UI (PhoeCord settings tab, UI plugins, themes) is built only for Phoenix; on today's Electron client PhoeCord stays network-only and is controlled from the CLI and the installer (Nanda 23.11.2026: "UI only for Phoenix", translated).
- Hybrid architecture: the network is the base, the UI layer is targeted; dynamic plugins through an external runtime (Nanda 23.11.2026, project brief).
- Plugin runtime: WASM (wasmtime + WIT), not Lua or QuickJS (Nanda 23.11.2026: "I don't know, decide yourself what's better", translated; decided by Claude, see `docs/DECISIONS.md`).
- Installer GUI on egui, not GPUI (same delegation).
- No code signing, now or at 1.0, like Vencord (Nanda 23.11.2026: "we build the project without signing", translated).
- Plugin scope: everything Vencord has, plus the owner's userplugins, is the priority; `no_track` is the only plugin on by default (Nanda 23.11.2026: "priority is everything in Vencord" and "also the ones in my userplugins", translated). QuestMaster and TrustMeBro are excluded (see `docs/PLUGINS.md`).
- License: GPL-3.0-or-later, like Vencord; the plugin SDK stays MIT so third-party plugins may use any license (Nanda 23.11.2026: "make ours GPL 3.0 too", translated).
- Git history stays as it is; no force-push to `main` (Nanda 23.11.2026: "keep the history", translated).
- Docs, plans and templates follow the VALORANT repository's structure (Nanda 23.11.2026: "like VALORANT", translated).
- All documentation is English-only; the app must support Russian later (Nanda 23.11.2026: "translate all docs to English, no Russian at all, though the program should support Russian in the future", translated). Localization becomes stage 1.7.

**Not doing in Phase 1:** reverse engineering Phoenix and its signatures (no binary yet, that is Phase 2); UI hooks on today's Electron client (Vencord covers its UI; on Electron the loader only proves the mechanics); the loader on macOS and Linux; a third-party plugin catalogue; languages other than English and Russian.

## Open questions

none

## Who does what

| | Nanda | Contributors |
|:---|:---|:---|
| Area | core, Plugin API, proxy, CA and trust, loader, installer, runtime, releases, review and merging | plugins, SDK examples, tests and fixtures, translations, docs, research |
| Tasks | 1.0, 1.1 (except 1.1.9), 1.2.3–1.2.6, 1.2.8, 1.3 (except 1.3.6, 1.3.7), 1.4 (except 1.4.3), 1.5, 1.6, 1.7.1–1.7.5 | 1.1.9, 1.2.1 (matvee, done), 1.2.2 and 1.2.7 (matvee), 1.4.3 (Kennurken), 1.2.9, 1.3.6, 1.3.7 |

Plugin API changes (`phoecord-core::plugin`) are made by Nanda only: contributor plugins are written against an API that is already merged.

## Tasks

Details, files and "done when" in [`ai/plan.md`](ai/plan.md). **This table is the only place task status is tracked**; `ROADMAP.md` and `ai/plan.md` carry no checkboxes.

| Task | What | Who | Status |
|:--|:--|:--|:--|
| **1.0.1** 🔴 | Merge the Phase 1 plan, the docs structure and the English-only docs | Nanda | ✅ 23.11, `main` `ea73482` (PR #4) |
| **1.0.2** 🟡 | Dependabot: close #1, merge #2 and #3 after green CI | Nanda | ✅ 23.11, folded into one PR |
| **1.1.1** 🔴 | Proxy `Host`: settings, KV, `self_id`, `notify` | Nanda | ✅ |
| **1.1.2** 🔴 | `config.toml` with `[plugins.<id>]` sections and live reload | Nanda | ✅ |
| **1.1.3** 🔴 | Account state behind `Host` | Nanda | ✅ |
| **1.1.4** 🔴 | Proxy: `CONNECT`, loopback only, MITM by `intercept` | Nanda | ✅ |
| **1.1.5** 🔴 | REST through `Chain` | Nanda | ✅ |
| **1.1.6** 🔴 | Gateway through `Chain` | Nanda | ✅ |
| **1.1.7** 🔴 | CLI: `run`, `launch`, `setup`, `trust`, `untrust`, `plugins`, `enable`, `disable`, `paths` | Nanda | ✅ |
| **1.1.8** 🟡 | `selftest` against the real Discord | Nanda | ✅ |
| **1.1.9** 🟡 | Gateway fixtures without personal data | anyone | ⏳ |
| **1.1.10** 🟡 | Chain benchmark: ≤ 1 ms p99 | Nanda | ⏳ |
| **1.1.11** 🟡 | Upstream proxy (SOCKS5 / HTTP) for outgoing connections, for VPN and zapret users | anyone | ⏳ |
| **1.1.12** 🟡 | Transport boundary: the proxy as one backend of the plugin chain, ready for an in-process backend | Nanda | ⏳ |
| **1.2.1** 🔴 | `silent_typing` | matvee | ✅ |
| **1.2.2** 🟡 | `unindent` | matvee | ⏳ |
| **1.2.3** 🟡 | `game_activity_toggle` | Nanda | ✅ 23.11, PR #21 |
| **1.2.4** 🟡 | `relationship_notifier` | Nanda | ✅ 23.11, PR #21 |
| **1.2.5** 🔴 | `fake_nitro` (emoji) | Nanda | ✅ 23.11, PR #21 |
| **1.2.6** 🔴 | `message_logger` and `phoecord log` | Nanda | ✅ 23.11, PR #22 |
| **1.2.7** 🔴 | `clear_urls` (new) | matvee | ✅ 23.11, PR #42 |
| **1.2.8** 🟡 | `/phoecord` slash commands (new) | Nanda | ✅ 23.11, PR #22 |
| **1.2.9** 🟢 | `phoecord import-vencord` | anyone | ⏳ |
| **1.2.10** 🟡 | Message filters: iLoveSpam, NoBlockedMessages, NoReplyMention (ImageLink and NoPendingCount are UI-only, moved to Phase 2) | Nanda | ✅ 23.11, PR #45 |
| **1.2.11** 🟡 | Outgoing text: TextReplace, SendTimestamps, AnonymiseFileNames, SilentMessageToggle (`/phoecord silent`); BetterGifAltText is UI-only, moved to Phase 2 | Nanda | ✅ 23.11, PR #49 |
| **1.2.12** 🟡 | Commands: CustomCommands, FriendInvites, petpet, SpotifyShareCommands, ServerInfo, ViewRaw, UnsuppressEmbeds | buslan07 | ⏳ |
| **1.2.13** 🟡 | Presence: CustomRPC, MusicRichPresence, IgnoreActivities, AutoDNDWhilePlaying, CustomIdle | anyone | ⏳ |
| **1.2.14** 🟡 | Profiles: NoProfileThemes, FakeProfileThemes, USRBG, Decor (data part) | buslan07 | ⏳ |
| **1.2.15** 🟡 | External services: Dearrow, GifProviderSwitcher, FixImagesQuality | anyone | ⏳ |
| **1.2.16** 🟡 | Data parts of UI plugins: ValidUser, ValidReply, NewGuildSettings, OverrideForumDefaults, MessageLinkEmbeds, PlatformIndicators, TypingIndicator, MemberCount, MutualGroupDMs, WhoReacted, and the other 🌐 + 🎨 rows of `docs/PLUGINS.md` marked 1.2.16 | Nanda | ⏳ |
| **1.2.17** 🟡 | Translate, ReviewDB, VoiceMessages (data part), XSOverlay | anyone | ⏳ |
| **1.2.18** 🟢 | ShowHiddenChannels, ShowHiddenThings, Experiments (data part) | Nanda | ⏳ |
| **1.2.19** 🟡 | Owner's userplugins: SilentDelete, FakeDeafen, FakeDeafenDetector, localDevBadges, FollowUser, VoiceChatUtilities, rmcord, AntiRickroll (network parts) | Nanda | ⏳ |
| **1.3.1** 🔴 | `wit/phoecord.wit` | Nanda | ✅ 23.11, PR #24 |
| **1.3.2** 🔴 | WASM backend for `Chain` on wasmtime | Nanda | ✅ 23.11, PR #24 |
| **1.3.3** 🔴 | Limits: time, memory, disable on panic | Nanda | ✅ 23.11, PR #24 |
| **1.3.4** 🔴 | Packages `plugin.toml` + `plugin.wasm`, hot loading | Nanda | ✅ 23.11, PRs #24 and #28 (new packages load on the next 2 s scan, removed ones unload) |
| **1.3.5** 🔴 | Rights from the manifest | Nanda | ✅ 23.11, PR #24 |
| **1.3.6** 🟡 | SDK: bindings, macro, `examples/hello-plugin` | anyone | ⏳ |
| **1.3.7** 🟡 | TypeScript plugin template | anyone | ⏳ |
| **1.3.8** 🔴 | `clear_urls` as a WASM plugin | Nanda | ✅ 23.11, PR #48 (`examples/plugins/clear_urls`, 79 KB, loads without a restart) |
| **1.4.1** 🔴 | Client discovery on 3 OSes | Nanda | ✅ 23.11, PR #23 (library; real machines not checked) |
| **1.4.2** 🔴 | CA and proxy autostart | Nanda | ✅ 23.11, PR #23; `phoecord setup` uses it since PR #33 |
| **1.4.3** 🟡 | Installer GUI: dark red theme, tabs, motion, as easy as BetterDiscord/Vencord; terms screen; Plugins tab (install from file or link, rights shown first, on/off, remove) | Kennurken | ⏳ |
| **1.4.4** 🔴 | Full rollback | Nanda | ✅ 23.11, PR #23; `phoecord uninstall` since PR #33 |
| **1.4.5** 🟡 | Releases: builds, `SHA256SUMS`, no signing | Nanda | ⏳ |
| **1.4.6** 🔴 | Manual check on real Windows, macOS and Linux machines: `setup` → reboot → proxy runs and Discord connects; `uninstall` leaves no file, trust entry or autostart entry | Nanda | 🔄 Windows 23.11: setup, repeat setup, client through the proxy, proxy as autostart would run it, uninstall checked; real reboot not done; 5 bugs found (1.4.8); macOS and Linux not checked |
| **1.4.7** 🔴 | Terms of use (`TERMS.md`, English and Russian) that must be accepted before install, in the GUI and in `phoecord setup` | Nanda | ✅ 23.11, PR #38; text approved by the owner (the GUI screen is part of 1.4.3) |
| **1.4.8** 🔴 | Fixes from the Windows check (1.4.6): `uninstall` removes a CA made by any command and stops a running proxy; `launch --channel stable/ptb/canary` (no silent default to Stable); `launch` reuses a running proxy; a Start-menu shortcut that starts the client through the proxy, so autostart is useful | Nanda | ✅ 24.11, PR #52 (all five fixed with tests; the Windows re-check is part of 1.4.6) |
| **1.5.1** 🔴 | `version.dll`: export forwarding | Nanda | ✅ 23.11, PR #10 |
| **1.5.2** 🔴 | `DllMain` and the worker thread | Nanda | ✅ 23.11, PR #10 |
| **1.5.3** 🔴 | Client version detection and `signatures/*.toml` | Nanda | ✅ 23.11, PR #10 |
| **1.5.4** 🔴 | Inline detours and hook removal | Nanda | ✅ 23.11, PR #10 |
| **1.5.5** 🟡 | Loader ↔ proxy IPC | Nanda | ⏳ |
| **1.5.6** 🟡 | Reinstall after a Discord update | Nanda | ⏳ |
| **1.5.7** 🔴 | Verify on 3 client versions | Nanda | ⏳ |
| **1.6.1** 🔴 | A map of GPUI | Nanda | ✅ 23.11, PR #12 |
| **1.6.2** 🔴 | GPUI reference build | Nanda | ✅ 23.11, PR #12 |
| **1.6.3** 🟡 | What of GPUI is visible in a release binary | Nanda | ✅ 23.11, PR #12 |
| **1.6.4** 🟡 | Themes in Zed and what to expect from Phoenix | Nanda | ✅ 23.11, PR #12 |
| **1.6.5** 🔴 | Report `docs/research/gpui.md` | Nanda | ✅ 23.11, PR #12 |
| **1.7.1** 🔴 | `phoecord-i18n` on Fluent, fallback to English | Nanda | ✅ 23.11, PR #9 |
| **1.7.2** 🔴 | `language` setting, OS default | Nanda | ✅ 23.11, PR #34 |
| **1.7.3** 🔴 | Russian translation of CLI, notifications, installer | Nanda | ✅ 23.11, PR #39 (CLI and installer messages; plugin notifications are 1.7.4) |
| **1.7.4** 🟡 | `Host::t()` and per-plugin `.ftl` | Nanda | ⏳ |
| **1.7.5** 🟡 | CI check: every `en` key exists in `ru` | Nanda | ✅ 23.11, PR #9 |

Order: 1.0 → 1.1 → (1.2 and 1.3 in parallel) → 1.4. Stages 1.5 and 1.6 do not depend on the network and run in parallel with 1.2–1.4. Stage 1.7 starts after 1.1.7 (once there are CLI strings to translate) and must land before 1.4.3.

## Risks

- **Phoenix Canary ships before the phase ends.** Drop 1.4.3 and 1.2.9, bring 1.5 and 1.6 to a minimum, and start Phase 2 (2.1 reconnaissance) in parallel.
- **Phoenix ignores the proxy or pins certificates.** The network layer then depends on the loader (ARCHITECTURE.md, "Phoenix and the proxy", option 3); 1.5 becomes required for every plugin, not just UI ones.
- **Antivirus flags `version.dll` as malware.** Open source, reproducible builds with `SHA256SUMS`, submitting samples to Microsoft and major vendors before release.
- **Discord changes the Gateway protocol.** Caught by `selftest` (1.1.8) and fixtures; the fix goes into `phoecord-proxy::codec`, plugins stay untouched.
- **WASM is slow on the hot path.** Run the 1.1.10 benchmark on a WASM chain too; if p99 > 1 ms, `message_logger` and `fake_nitro` stay native.
- **Account bans.** Client modification breaks Discord's Terms, and heavy automation (bulk requests) is the easiest thing for Discord to spot; bulk plugins go through the `Host::rest` rate limiter.
- **Russian translation drifts behind English.** The 1.7.5 CI check fails on missing keys; a string without a translation falls back to English rather than showing a key.

## Living docs and publications

- [`docs/CHANGELOG.md`](../CHANGELOG.md): an "Unreleased" line in every PR; versions `0.2`–`0.6` at the end of stages.
- [`ARCHITECTURE.md`](../ARCHITECTURE.md): on Plugin API changes, the runtime decision and how we reach Phoenix.
- [`docs/CODE_MAP.md`](../CODE_MAP.md): every new file, config key and env variable.
- [`docs/PLUGINS.md`](../PLUGINS.md): the plugin's "Where" cell.
- README: the "Plugins" table and the pre-alpha note at the end of each stage (keep it compact; details belong in `docs/PLUGINS.md`).
- GitHub Releases: one per stage starting at `0.2`.

## How to start

Read [`AGENTS.md`](AGENTS.md), then [`ai/spec.md`](ai/spec.md) and your task in [`ai/plan.md`](ai/plan.md). Branch from a fresh `main` with a prefix from [`CONTRIBUTING.md`](../../CONTRIBUTING.md). Open for contributors: 1.1.9 (fixtures), 1.2.9 (`import-vencord`), 1.2.10–1.2.15 and 1.2.17 (Vencord network plugins), 1.3.6 (SDK), 1.3.7 (TypeScript template). Pick one, write a `claimed` line in the ledger and set yourself as "Who" in the table.

## Where things are

| What | Where |
|:--|:--|
| Plugin API | `crates/phoecord-core/src/plugin.rs` |
| Plugin chain | `crates/phoecord-plugin-runtime/src/lib.rs` |
| Gateway codecs | `crates/phoecord-proxy/src/codec.rs` |
| Local CA | `crates/phoecord-proxy/src/ca.rs` |
| Certificate trust | `crates/phoecord-proxy/src/trust.rs` |
| Built-in plugins | `crates/phoecord-plugins/src/` |
| Plugin template | `docs/_template/code/plugin.rs`, `docs/_template/code/plugin.toml` |
| AOB scanner | `crates/phoecord-loader/src/aob.rs` |
| Installer | `crates/phoecord-installer/src/` (`discover`, `install`, `uninstall`, `system`) |
| WASM runtime | `crates/phoecord-plugin-runtime/src/wasm.rs`, `package.rs`, `wit/phoecord.wit` |
| CLI | `crates/phoecord-cli/src/main.rs` |

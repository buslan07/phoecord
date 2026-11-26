# Phase 2. Phoenix Canary release: plan for people

The work plan from Phoenix Canary to v1.0, written 25.11.2026 at Nanda's request. Goal: PhoeCord works inside Desktop Phoenix through an in-process transport, with a theme engine, in-client settings, and full Vencord parity — network and visual plugins, panels and themes.

> This is the plan for people. The spec, the step-by-step tasks and the ledger are in [`ai/`](ai/). If this file and [`ai/spec.md`](ai/spec.md) disagree, the spec wins.

## Status

26.11.2026. Done: 2.1.1, 2.1.2, 2.1.5–2.1.7, 2.1.9, 2.2.1, 2.2.11 and 2.4.3. The Plugin API is frozen at v1 (2.4.3, PR #114): `#[non_exhaustive]` types with `new` constructors, `ChainHost`, UI events as `Event::Ui` (one generic WIT case), `Manifest.menu`, and `wit/frozen/` guarding the WIT; WASM packages built for 0.5 must be rebuilt. The recon report is complete (2.1.9, PR #117). Part 1 of the signature hot-fix is in `main` (2.2.10, PR #116: `phoecord signatures install` / `list`). 2.2.8 is fully specified ahead of time: the transport is chosen per launch, in-process with no proxy when every signature is found, the check cached by the client's `.text` hash and the signature folder's hash (rulings, 2026-11-26). ken and matveee have access to the private recon notes (2.2.2/2.2.3 and 2.2.5). In progress: ken on 2.2.2 (part 1: read-only `send` detour), 2.2.4 (part 1: `gateway_send` locator) and 2.2.7 (all or nothing per launch); 2.1.3, 2.1.4, 2.1.8, 2.3.3 and the live runs wait for a Windows PC with Phoenix; 2.2.3 and 2.4.1 are blocked (payload type, UI hooks); matvee on 2.3.1, with 2.2.5 part 2 now in review (branch `feat/locators-targets`): typed locators for `ReqwestTransport::send`, the allocator shims and the Gateway registry builder, checked against Phoenix `0.1.0.2208` only (`http_poll`'s vtable slot stays open, no usable anchor yet); buslan on 2.2.6 and 2.2.10 part 2. Next for Nanda: 2.4.5 (installer 1.0, started) and 2.2.8 (the Phoenix side of it), 2.4.4 (developer docs), 2.4.2 (the rest of the v1.0 scope), then 2.4.6 (security audit, once the detours land) and 2.4.7 (release). ken also has 2.4.1 (top 10 plugins on Phoenix); 2.4.8 is closed without the macOS check. Follow-up: `hooks::found::Found<T>` and `locators::Found<T>` should become one type (2.2.2 with 2.2.5). Phase 1 is closed (26.11.2026): 1.1.9 is now 2.1.10, the USRBG and Decor part of 1.2.14 is 2.3.8, and FixImagesQuality's default mode from 1.2.15 is 2.3.9.

## What we get

- **A user** installs PhoeCord on Windows, starts Discord Phoenix and gets every Vencord plugin — network and visual — with custom themes, an in-client settings tab, and no extra process or CA trust required. On today's Electron client, the network layer works as before. Uninstalling is one button.
- **A plugin author** writes a WASM plugin that uses both network events and Phoenix UI hooks (message accessories, context menu entries, settings pages), following the developer docs.
- **The team** ships v1.0 with a frozen Plugin API, a security audit, signed releases and a process for updating signatures within 48 hours of a new Phoenix build.

## Decisions

Decisions from Phase 1 that carry forward (see `docs/phase-1/README.md` for quotes and dates):

- The in-client UI (settings tab, visual plugins, themes) is built only for Phoenix; on today's Electron client PhoeCord stays network-only.
- Hybrid transport: proxy today, in-process on Phoenix; the proxy as the fallback.
- Plugin runtime: WASM (wasmtime + WIT).
- No code signing (like Vencord).
- GPL-3.0-or-later; plugin SDK MIT.
- Full Vencord parity including visual plugins and panels.
- Vencord source reused with attribution.
- Dev tools (1.3.9–1.3.11) written from scratch; sunbird has no licence, so no code is copied.

New decisions for Phase 2 (made when the research answers them):

- Loading path: **launcher (APC injection)** on Phoenix, decided in 2.1.7 (ruling, 2026-11-26), live-observed via a third-party tool (`docs/ARCHITECTURE.md`, "Phoenix is different"). Sideload (`version.dll`) stays how today's Electron client is reached.
- Hook timing: synchronous under the loader lock or suspended-thread — decided in 2.1.7.
- Proxy reach on Phoenix: **option 1, no hook** (2.1.8, ruling 2026-11-26): `--proxy-server` or the system proxy plus the PhoeCord CA in the system store. `HTTPS_PROXY` is ignored.

**Not doing in Phase 2:** UI plugins through WASM (declarative elements → GPUI); the UI layer on macOS and Linux (Windows first, Linux in-process transport later); a third-party plugin catalogue with signature checks; languages other than English and Russian; Android or iOS.

## Open questions

1. **DLL search.** Does Phoenix `Discord.exe` restrict DLL search to the system directory? (2.1.2)
2. **Hook timing.** ~~What is the real deadline: first frame, first gateway connect, first request?~~ Answered by live observation (2.1.7, 2026-11-26): hooks installed in 858 ms, before the client reports ready; no crash, live account traffic flowed normally afterward.
3. **Outgoing gateway.** Is there a function that sends client → server gateway frames in-process? (2.1.6)
4. **READY on Desktop.** Is READY applied outside the event registry? (2.1.6)
5. **Allocator stability.** How stable is the allocator discovery across builds? (2.1.5, 2.2.2)
6. **Prologues.** What share of hook targets have a prologue an entry hook can take? (2.2)
7. **Build cadence.** How often does Phoenix ship new builds? Our "within 48 hours" rule depends on it. (2.2)
8. **GPUI revision.** Does Phoenix pin the same GPUI revision as our reference build? (2.1.5)
9. **Store schema.** Is Phoenix's `cache.sqlite` stable enough to read? (2.1)
10. **Terms and risk.** Injecting into Phoenix is a Terms question for the owner. (not engineering)

## Who does what

| | Nanda | ken | buslan | matvee |
|:---|:---|:---|:---|:---|
| Area | core RE (binary, hooks), decisions, Plugin API, security, release, review and merging | proxy, transport, wire format, degradation | discovery, tooling, installer, signatures | RE support, locators, UI hooks, theme engine |
| Tasks | 2.1.1, 2.1.2, 2.1.5, 2.1.6, 2.1.7; 2.2.1, 2.2.8; 2.4.2, 2.4.3, 2.4.4, 2.4.5, 2.4.6, 2.4.7 | 2.1.3, 2.1.4, 2.1.8; 2.2.2, 2.2.3, 2.2.4, 2.2.7, 2.2.11; 2.3.3, 2.3.7; 2.4.1, 2.4.8 | 2.2.6, 2.2.9, 2.2.10, 2.3.8, 2.3.9 | 2.1.9, 2.1.10; 2.2.5; 2.3.1, 2.3.4 |

Unassigned contributor tasks: 2.3.2 (themes), 2.3.5, 2.3.6.

Plugin API changes (`phoecord-core::plugin`) are made by Nanda only.

## Tasks

Details, files and "done when" in [`ai/plan.md`](ai/plan.md). **This table is the only place task status is tracked**; `ROADMAP.md` and `ai/plan.md` carry no checkboxes.

### Stage 2.1. Phoenix reconnaissance · `0.7`

| Task | What | Who | Status |
|:--|:--|:--|:--|
| **2.1.1** 🔴 | Phoenix discovery: install paths on Windows and Linux, update `discover` module, `phoecord launch --channel phoenix` | Nanda | ✅ Windows ([recon](../research/phoenix-recon.md)): `discover` finds `%LOCALAPPDATA%\DiscordPhoenix\app` and reads the build from `Discord.exe`; `launch --channel phoenix` starts it through the proxy and `paths` shows the folder (run on the owner's PC: Phoenix 0.1.0.2208 started, TLS refused until the CA is trusted). Plain launch only, injection is 2.1.7. Update scheme: in-place `app\`, inferred, no update observed. Linux and macOS: no Phoenix location known, nothing looked for |
| **2.1.2** 🔴 | Binary inspection: PE headers, import table, DLL search restrictions, dependent-load flags, update scheme | Nanda | ✅ |
| **2.1.3** 🔴 | Proxy behavior: does Phoenix honour `HTTPS_PROXY`, the system proxy, the system CA store; does it pin | ken | ⏳ Windows answers written up ([recon](../research/phoenix-recon.md)): `--proxy-server` and the system proxy honoured, `HTTPS_PROXY` not, trusted PhoeCord CA accepted, no pin seen on `discord.com` / the gateway; open: endpoints beyond the account screen (uploads, voice, CDN) and Linux, both needs a Windows PC with Phoenix and an account the client accepts |
| **2.1.4** 🔴 | Gateway wire format for the proxy fallback: encoding (JSON / ETF), compression (zlib / zstd), new events and fields | ken | ⏳ gateway is JSON + zstd-stream and decodes with the current codec ([recon](../research/phoenix-recon.md)); open: `v=` and URL parameters, the event comparison with Electron and the fixtures in `tests/fixtures/phoenix/`, all of which needs a Windows PC with Phoenix and an account the client accepts |
| **2.1.5** 🔴 | Locate the HTTP transport in the binary: `ReqwestTransport::send`, the vtable, type layouts, allocator shims | Nanda | ✅ static analysis only, not run in Phoenix ([recon](../research/phoenix-recon.md) summary; addresses kept privately): `send`, the vtable, the request layout and the allocator shims found. Open for 2.2.2: the response layout |
| **2.1.6** 🔴 | Locate the Gateway registry: per-domain tables, seed events, handler ABI, READY handling; search for outgoing send | Nanda | ✅ static analysis only, not run in Phoenix ([recon](../research/phoenix-recon.md) summary; addresses kept privately): handler ABI, the outgoing send path and `READY`'s special handling were found (as before); the registry itself turns out to have no fixed table address — it is a per-instance tree-shaped structure built at runtime, reached through the context, not a static array. That answers the task's "done when" without one |
| **2.1.7** 🔴 | Loading path decision and prototype: sideload vs launcher; hook timing test | Nanda | ✅ both decisions live-confirmed (ruling, 2026-11-26) **with PhoeCord's own loader**, not a third-party tool: `phoecord dev inject` (launcher injection: `CreateProcess` suspended + `QueueUserAPC` + `LoadLibraryW`) ran `phoecord-loader` inside a real Phoenix process; `DllMain` and the worker fired, detected the build (RVA-free: a new `.text` sha256 for `0.1.0.2208`) and degraded cleanly (no signature table yet, UI layer off), no crash. First validated on a throwaway `cmd.exe` target, then the real client |
| **2.1.8** 🔴 | Proxy reach decision and fallback: option 1 / 2 / 3 from `ARCHITECTURE.md`; implement the chosen path | ken | ⏳ decided: option 1, no hook (ruling, 2026-11-26); nothing to build. Open: the check on a fresh install with no manual setup (`phoecord launch --channel phoenix` exists since 2.1.1), which needs a Windows PC with Phoenix |
| **2.1.9** 🟡 | Recon report: `docs/research/phoenix-recon.md` with all findings, decisions for 2.2–2.4; update `ARCHITECTURE.md` | matvee | ✅ merged, PR #117: part 3 of the [recon](../research/phoenix-recon.md) (summary, decisions for 2.2–2.4 with what changed from the plan, open questions by task, risks), a section for the 2.1.7 loading path, stale lines fixed; `ARCHITECTURE.md` (two transports, hooks and themes). The open parts of 2.1.3, 2.1.4 and 2.1.8 are listed, not run |
| **2.1.10** 🟡 | Gateway fixtures without personal data (was 1.1.9): READY, GUILD_CREATE, MESSAGE_*, PRESENCE_UPDATE, TYPING_START from a throwaway account, both encodings, scrubbed | matvee | 🚧 `scripts/scrub_fixture.py` (rewrites every string and snowflake-shaped value, keeps the Gateway envelope and keys/nesting/types; a shared `--map` keeps a repeated id consistent across files) and `tests/fixtures/README.md` are in (branch `feat/gateway-fixtures`); open: the actual capture, which needs a human with a throwaway account and a running client — cannot be done from here |

### Stage 2.2. Reverse engineering and AOB signatures · `0.8`

| Task | What | Who | Status |
|:--|:--|:--|:--|
| **2.2.1** 🔴 | Ghidra / IDA project on Phoenix Canary: GPUI symbols imported from the reference build (1.6) | Nanda | ✅ 10 of the 2.1.5/2.1.6 hook targets are named symbols (`http_send`, `http_send_poll`, `rust_alloc`, `rust_dealloc`, `gateway_registry_build`, `gateway_registry_insert`, `gateway_send_encode`, `gateway_send_budget`, `gateway_connection_loop`, `rust_realloc_candidate`). The Version Tracking session against the GPUI reference build is now read (fixed the headless-scripting blocker from last round): 11,300 correlations, 261 accepted / 384 available / 10,655 blocked; narrowed to `gpui::` itself, 18 accepted / 5 available / 2,111 blocked — the revision delta is "heavily ambiguous from monomorphization", not a clean miss (see [recon](../research/phoenix-recon.md)). Open for the GUI: reviewing the 384 available candidates and the 10,655 blocked associations |
| **2.2.2** 🔴 | HTTP transport hook: inline entry on `send`, vtable swap for `poll`, allocator-safe body replacement | ken | 🚧 part 1: a read-only inline detour on `send` (`hooks::detour`, Windows x64): a naked entry hands the request to `on_send`, which reads method, URL and body through `ReadProcessMemory` (`hooks::abi`, every field checked; URL and body verified in the recon notes, the method encoding inferred) and asks the plugin chain; a changed body or local reply is logged, not applied. Open: the `poll` vtable swap and `on_response` (the response layout is not known), body replacement (the `Bytes` vtable), a CFG check and a live run on Windows |
| **2.2.3** 🔴 | Gateway registry detours: per-event handler swap, `Engine::on_dispatch` wiring | ken | ⏸ blocked on RE: the registry is a per-instance `BTreeMap` built at run time, and the handlers' payload argument is opaque in the recon notes (no JSON or typed layout yet), so a detour cannot hand `d` to `Engine::on_dispatch`. Needed from a Windows session: the handler payload type and the registry entry layout; the `Engine::on_dispatch` bridge and panic guard are ready |
| **2.2.4** 🟡 | Outgoing gateway hook (if found in 2.1.6): `Engine::on_send` wiring | ken | 🚧 part 1: `gateway_send` locator (`Found<GatewaySend>`) for the outgoing frame encoder, and `Plan::from_hooks` maps Send to it. Open: the encoder takes a typed command enum and its output type is unknown, so the hook cannot build or read JSON yet; heartbeat, identify and resume are sent elsewhere. Until then the Send entry point is missing and every Phoenix launch uses the proxy (accepted by the 2.2.8 ruling) |
| **2.2.5** 🔴 | Typed locators (`Found<T>`) for decoded structures: struct sizes, field offsets, handler ABI | matvee | 🚧 part 2 merged, PR #118: `locators::http_transport::find` (`ReqwestTransport::send`'s entry, masked AOB), `locators::allocator::find` (`__rust_alloc`/`__rust_dealloc`, both or neither), `locators::gateway::find` (the registry builder — no fixed table address, per 2.1.6); `Image::find_ptr` (`.rdata` pointer scan, needs the new `Image::image_base`) is ready but unused. All three checked against Phoenix `0.1.0.2208` only (task 2.2.9 is cross-build verification); `http_poll` (the response future's vtable slot) stays open — the recon notes have no `.text` site that refers to it yet, and this framework does not guess one. Part 1 (PR #103) merged |
| **2.2.6** 🔴 | `phoecord check <exe>`: run all locators against a binary, CI on `.text` dumps | buslan | 🚧 part 1: AOB signatures; part 2: the 2.2.5 typed locators in the same report (run on the owner's PC against Phoenix 0.1.0.2208: `http_send` and the allocator found, `gateway_registry` ambiguous with 2 candidates); `scripts/check_builds.py` runs the check over a folder of builds kept outside the repository (in review, no real build run through it); open: the CI job on private dumps |
| **2.2.7** 🔴 | In-process transport wired to `Engine`: REST and Gateway through the hook layer, `Effects` delivery | ken | 🚧 the transport is all or nothing per launch (`Plan::all_in_process`, ruling 2026-11-26): with any locator missing every entry point passes and the proxy carries the launch; `InProcessTransport::wants_request` and a `SendSink` feed the `send` detour; the loader logs the network plan of a Phoenix build and skips network hooks on Electron. Open: calls from the `poll`, registry and send detours (2.2.2–2.2.4), the in-process `Network`, and starting the transport from the 2.2.8 launch mode with an `Engine` built from `config.toml` |
| **2.2.8** 🔴 | Phoenix installer and transport choice: `phoecord launch` picks in-process (no proxy) or proxy per launch from a cached signature check, shortcut, reconcile after Phoenix updates, uninstall | Nanda | 🚧 merged, PR #120: items 1, 2, 4 (reading the mark), 6 and 7 built and tested with fake locator results; item 3 holds (in-process starts no proxy); item 5 unchanged. `launch --channel phoenix` checks `Discord.exe` on disk, caches by `.text` + signature folder hash, injects the loader in both modes; shortcut "Discord Phoenix (PhoeCord)"; `--transport` / `[phoenix] transport`; uninstall removes the cache. Seams for 2.2.7: the loader acting on its mode and writing the "failed" mark. Open: the live run on the owner's PC |
| **2.2.9** 🔴 | Every signature verified unique on two adjacent builds | buslan | ⏳ |
| **2.2.10** 🟡 | Signature updates without rebuilding the loader; hot-fix path for TOML AOB entries | buslan | 🚧 part 1 merged, PR #116: the loader reads the folder at every start; `phoecord signatures install` / `list` (all files checked and staged before any is renamed into place; empty build tables refused) and the workflow in `signatures/README.md` (temp-folder tests only); open: where real tables are published, a live run with a client |
| **2.2.11** 🔴 | Loader check inside Discord Canary (was 1.5.9): one worker per start, in-memory hash equals the file's, under 20 ms, `loader connected` in the proxy log, clean uninstall | ken | ✅ checked on Discord Canary 1.0.1364 (Windows, a release build from `main`, sandboxed `PHOECORD_CONFIG`/`PHOECORD_DATA`): one `client build` line per start (none from the `--type=` children), the `.text` hash matched across 3 restarts, `loader connected` in the proxy log, no crash over three starts plus a 4-minute run, `phoecord uninstall` removed the DLL, shortcut, Run key and CA. **Not met: detection took ~130 ms, not under 20 ms** (consistent across 3 runs; `DiscordCanary.exe` is 224 MiB, so a `.text`-section SHA-256 this slow is plausible — open question for whoever set the 20 ms figure) |

### Stage 2.3. UI hooks and theme engine · `0.9`

| Task | What | Who | Status |
|:--|:--|:--|:--|
| **2.3.1** 🔴 | Theme engine: `theme.toml` → GPUI palette, live switching | matvee | ⏳ part 1 merged, PR #102; `themes/default.toml` ships (PR #122) with Discord's published surface colours under placeholder field names — still not a true no-op default, and no live hook: `phoecord-recon` has no `GlobalTheme` anchor for Phoenix (only Zed/GPUI's general pattern in `docs/research/gpui.md`); blocked on that, not on access |
| **2.3.2** 🟡 | 3 built-in themes + import of Vencord themes where they translate to a palette | Nanda | 🚧 `themes/default.toml` (dark), `light.toml`, `high-contrast.toml`; `phoecord theme import`; `docs/plugins/themes.md`. Open: the field names follow Phoenix's palette once 2.3.1 finds it |
| **2.3.3** 🔴 | PhoeCord settings tab in Phoenix: plugins, settings, install / remove packages with rights shown, en + ru | ken | ⏳ moved from matvee (ruling, 2026-11-26) |
| **2.3.4** 🔴 | MessageLogger UI: deleted messages struck through, edit history on hover | matvee | ⏳ `phoecord_loader::ui::message_logger` (PR #122): pure `Accessory::for_message` (deleted / edited-with-history), tested; not wired to a real message row — `MessageRow::render`'s address is not in `phoecord-recon`, only the app-level render chain (`docs/re/gpui-lifecycle.md`) is mapped |
| **2.3.5** 🟡 | Message context menu: plugin action entries in right-click and `...` menus | — | ⏳ |
| **2.3.6** 🟡 | Optimistic delete hook: `Store::delete_message` visible to plugins (MessageLogger sees local deletes) | — | ⏳ |
| **2.3.7** 🔴 | Degradation verified: unknown build → network only, no crashes; missing signature → that feature off only | ken | 🚧 automated part done: scenarios 1–3 as tests with fake locator results (`crates/phoecord-loader/tests/degradation.rs`), `ClientKind::of` (Phoenix vs Electron), `Plan::all_in_process()`; open: the live run inside a Phoenix client (needs a Windows PC with Phoenix, the locators of 2.2.5 and the transport of 2.2.7) |
| **2.3.8** 🟡 | CDN picture hook for USRBG and Decor (was 1.2.14's blocker): native avatar/banner loading by hash, not a REST route | buslan | ⏳ |
| **2.3.9** 🟡 | FixImagesQuality's default mode on Phoenix (was 1.2.15's open part): images in chat at original resolution through the picture-loading path of 2.3.8 | buslan | ⏳ |
| **2.3.10** 🔴 | Generic view-render wrap and message-row accessories | — | ⏳ added by ruling, 2026-11-26; blocked on 2.2.2/2.2.3 |
| **2.3.11** 🔴 | Every context menu | — | ⏳ added by ruling, 2026-11-26; blocked on 2.2.2/2.2.3 |
| **2.3.12** 🔴 | Modals | — | ⏳ added by ruling, 2026-11-26; blocked on 2.2.2/2.2.3 |
| **2.3.13** 🔴 | User names | — | ⏳ added by ruling, 2026-11-26; blocked on 2.2.2/2.2.3 |
| **2.3.14** 🔴 | Profiles | — | ⏳ added by ruling, 2026-11-26; blocked on 2.2.2/2.2.3 |
| **2.3.15** 🔴 | Chat input | — | ⏳ added by ruling, 2026-11-26; blocked on 2.2.2/2.2.3 |
| **2.3.16** 🔴 | Voice and audio | — | ⏳ added by ruling, 2026-11-26; blocked on 2.2.2/2.2.3 |
| **2.3.17** 🔴 | Account panel | — | ⏳ added by ruling, 2026-11-26; blocked on 2.2.2/2.2.3 |
| **2.3.18** 🔴 | Media | — | ⏳ added by ruling, 2026-11-26; blocked on 2.2.2/2.2.3 |
| **2.3.19** 🔴 | Channel and server lists | — | ⏳ added by ruling, 2026-11-26; blocked on 2.2.2/2.2.3 |
| **2.3.20** 🔴 | Client behaviour patches | — | ⏳ added by ruling, 2026-11-26; blocked on 2.2.2/2.2.3 |
| **2.3.21** 🔴 | Overlay | — | ⏳ added by ruling, 2026-11-26; blocked on 2.2.2/2.2.3 |
| **2.3.22** 🔴 | Themes picker | — | ⏳ added by ruling, 2026-11-26; blocked on 2.2.2/2.2.3 |
| **2.3.23** 🔴 | Toolbox | — | ⏳ added by ruling, 2026-11-26; blocked on 2.2.2/2.2.3 |

### Stage 2.4. Release v1.0 · `1.0`

| Task | What | Who | Status |
|:--|:--|:--|:--|
| **2.4.1** 🔴 | Top 10 plugins on Phoenix (network + UI parts): MessageLogger, FakeNitro, NoTrack, ClearURLs, SilentTyping, ShowHiddenChannels, Translate, PlatformIndicators, PermissionsViewer, ViewIcons | ken | ⏸ blocked: the UI parts need the 2.3 UI hooks and the network parts need the in-process transport live on Phoenix (2.2.2–2.2.4); through the proxy the network parts already work from Phase 1 |
| **2.4.2** 🔴 | Remaining v1.0 scope: every 🌐 and 🌐 + 🎨 plugin complete, the 30 most-used 🎨 plugins, all panels | Nanda | 🚧 scope ruled (Q1–Q6, 2026-11-26): `v1.0` column in PLUGINS.md, UI surfaces as 2.3.10–2.3.23; unblocked items (a)–(e) in progress |
| **2.4.3** 🔴 | Plugin API stabilization: `API_VERSION = 1` frozen, changes minor-only; additive UI events | Nanda | ✅ merged, PR #114: v1 frozen with the owner's rulings (Q1–Q8, 2026-11-26): `#[non_exhaustive]` types with `new` constructors, `ChainHost`, `Event::Ui(UiEvent::MenuAction)` + `Manifest.menu` / `[[menu]]`, `LOCAL_DELETE_EVENT`, WIT `ui(ui-event { name, payload })`; `wit/frozen/` and `tests/frozen_wit.rs` guard the WIT. WASM packages built for 0.5 must be rebuilt |
| **2.4.4** 🔴 | Developer docs `docs/plugins/`: quick start (Rust + TS), events, rights, publishing, localization | Nanda | ✅ PR #119: six pages (index, quick start, events, rights, publishing, localization) checked against `main` (`24e0362`); the quick starts were run from empty folders outside the repository; stale `plugin.toml` examples in `ARCHITECTURE.md` and the template fixed |
| **2.4.5** 🔴 | Installer 1.0 for Windows; macOS and Linux with the network layer | Nanda | 🚧 merged, PR #121: NSIS setup with the loader in `loader\`, `.dmg`, `.deb` and `.rpm` (Linux packages test-installed in CI); automatic updates from GitHub Releases checked against `SHA256SUMS` (rulings Q1–Q8, 2026-11-26); not yet run on a real machine; open: TERMS.md §4 and the update check |
| **2.4.6** 🔴 | Security audit of the CA, the proxy and the loader | Nanda | ⏳ |
| **2.4.7** 🔴 | v1.0 release: binaries, `SHA256SUMS`, `docs/CHANGELOG.md` | Nanda | ⏳ |
| **2.4.8** 🔴 | macOS check of the 0.5.1 fixes on a real Mac (was 1.4.10): no "damaged", Discord starts and updates through PhoeCord, clean uninstall | ken | ⛔ closed without the check (owner's ruling, 2026-11-26) |

Order: 2.1 → 2.2 → 2.3 → 2.4. Stage 2.1 must finish before 2.2 starts (the loading path, proxy reach and binary findings feed every hook). Stages 2.2 and 2.3 can overlap once the first hooks land (2.2.2–2.2.3). Stage 2.4 starts after 2.3.7 (degradation verified).

## Risks

- **Phoenix restricts DLL search.** The sideload plan changes to a launcher; `version.dll` becomes a fallback for clients that allow it. 2.1.2 answers this first.
- **Hook timing: too late from a worker thread.** Hooks must be in before the first registry read. The fallback is synchronous hooking under the loader lock (acceptable cost: bounded scan, ~1 s on a ~500 MB executable). 2.1.7 tests this.
- **Allocator mismatch.** Replacing a body with the wrong allocator corrupts the heap. The safe fallback is a local reply (`Verdict::Reply`) instead of body replacement, at the cost of losing `Verdict::Changed` for responses. 2.2.2 handles this.
- **Build cadence faster than 48 hours.** If Phoenix ships faster than signatures can be updated, the in-process transport is off more often than on. Mitigated by the proxy fallback and by moving locators from AOB to code (more resilient to layout changes).
- **A panic in a hook kills Discord.** Panic guard at every entry; a failed locator disables only its feature; the "degrade to network only" rule (2.3.7) is tested before release.
- **Antivirus flags the injected DLL.** Same as Phase 1: unsigned, `SHA256SUMS`, Defender false-positive submissions.
- **Account bans.** Client modification breaks Discord's Terms. The risk is the same as Phase 1 and is stated in `TERMS.md`.

## Living docs and publications

- [`docs/CHANGELOG.md`](../CHANGELOG.md): an "Unreleased" line in every PR; versions `0.7`–`1.0` at the end of stages.
- [`ARCHITECTURE.md`](../ARCHITECTURE.md): on transport changes, loader changes, UI hook design.
- [`docs/CODE_MAP.md`](../CODE_MAP.md): every new file, config key and env variable.
- [`docs/PLUGINS.md`](../PLUGINS.md): the plugin's "Where" cell for UI parts.
- README: pre-alpha note updated per stage.
- GitHub Releases: one per stage starting at `0.7`.

## How to start

Read [`AGENTS.md`](AGENTS.md), then [`ai/spec.md`](ai/spec.md) and your task in [`ai/plan.md`](ai/plan.md). Branch from a fresh `main`.

## Where things are

| What | Where |
|:--|:--|
| Plugin API | `crates/phoecord-core/src/plugin.rs` |
| Transport boundary (`Engine`) | `crates/phoecord-transport/src/lib.rs` |
| Plugin chain + WASM runtime | `crates/phoecord-plugin-runtime/src/` |
| Built-in plugins | `crates/phoecord-plugins/src/` |
| Proxy transport | `crates/phoecord-proxy/src/` |
| Loader | `crates/phoecord-loader/src/` |
| AOB scanner | `crates/phoecord-loader/src/aob.rs` |
| IPC | `crates/phoecord-loader/src/ipc.rs`, `pipe.rs` |
| Installer | `crates/phoecord-installer/src/` |
| Installer GUI | `crates/phoecord-installer-gui/src/` |
| CLI | `crates/phoecord-cli/src/main.rs` |
| Plugin SDK | `crates/phoecord-plugin-sdk/src/` |
| Signatures | `signatures/` |
| GPUI research | `docs/research/gpui.md` |
| Phoenix research | `docs/research/phoenix-desktop.md` |
| GPUI reference build | `research/gpui-reference/` |

# PhoeCord shared strings (English, the source language).
# Every message here must exist in ../ru/main.ftl with the same { $variables }.

plugin-enabled = Plugin { $name } enabled.
plugin-disabled = Plugin { $name } disabled.
plugin-unknown = No plugin named { $name }.
plugin-failed = Plugin { $name } failed and was disabled: { $reason }

proxy-listening = Proxy listening on { $addr }.
proxy-stopped = Proxy stopped.
proxy-port-busy = Port { $port } is already in use.

ca-created = Certificate authority created at { $path }.
ca-exists = Certificate authority already exists at { $path }.
ca-trusted = Certificate authority added to the trust store.
ca-trust-failed = Could not add the certificate authority to the trust store: { $reason }

error-generic = Something went wrong: { $reason }
error-config-invalid = The configuration file { $path } is invalid: { $reason }
error-permission-denied = Permission denied. Try running as administrator.

lang-changed = Language set to { $language }.

# CLI: setup, uninstall, paths
cli-setup-done = Done. Quit Discord fully, then start it with `phoecord launch`.
cli-uninstall-done = PhoeCord is removed. Restart Discord without `phoecord launch`.
cli-uninstall-incomplete = some steps could not be undone; they stay in install.log, run `phoecord uninstall` again
cli-setup-steps = { $count } new install step(s) recorded in install.log
cli-uninstall-removed = removed  { $what }
cli-uninstall-kept = kept     { $what }
cli-uninstall-failed = failed   { $what }: { $why }
cli-paths-config = config
cli-paths-ca = CA
cli-paths-plugins = plugins
cli-paths-language = language
lang-source-flag = --lang flag
lang-source-config = config.toml
lang-source-os = OS language
lang-source-default = default

# CLI: --help and subcommands
cmd-about = Vencord-style plugins for Discord, through a local proxy
cmd-run = Run the proxy (the default). Point Discord at it with `--proxy-server=http://127.0.0.1:18463`
cmd-launch = Run the proxy and start Discord through it (quit Discord first). Optional: the client command
cmd-setup = First-time setup: write the config, make the local CA, trust it and start the proxy at login
cmd-uninstall = Undo everything `setup` did: autostart, CA trust, the files it created (your config stays)
cmd-trust = Trust the local CA in the certificate stores Discord uses
cmd-untrust = Remove the local CA from those stores
cmd-plugins = List plugins and whether they are on
cmd-enable = Switch a plugin on
cmd-disable = Switch a plugin off
cmd-toggle = Flip a plugin (e.g. `phoecord toggle game_activity_toggle`)
cmd-log = Show the newest deleted and edited messages kept by message_logger
cmd-import-vencord = Switch on the PhoeCord plugins that match your enabled Vencord / Vesktop / Equicord plugins
cmd-selftest = Check everything works against the real Discord (no login needed)
cmd-paths = Print where config, CA and plugin data live
cli-arg-import-path = Path to a Vencord settings.json (found automatically when omitted)
cli-import-none = no Vencord, Vesktop or Equicord settings found; pass one with --path FILE
cli-import-found = Reading { $name } settings: { $path }
cli-import-read = cannot read { $path }
cli-import-summary = Switched on { $enabled }, already on { $already }, no PhoeCord counterpart for { $unmatched } Vencord plugin(s).
cli-help-usage = Usage:
cli-help-commands = Commands
cli-help-options = Options
cli-help-arguments = Arguments
cli-help-help = Print help
cli-help-version = Print version
cli-arg-lang = UI language: en or ru (default: `language` in config.toml, then the OS language)
cli-arg-log-n = How many entries to show (default 20)

# CLI: run, launch
cli-listening = PhoeCord listening on { $listen } (plugins: { $plugins })
cli-starting = starting { $program }
cli-launcher-exited = launcher exited ({ $status }); still serving, Ctrl+C to stop
cli-client-exited = Discord exited ({ $status })
cli-err-listen = listening on { $listen }
cli-err-start-discord = starting Discord
cli-err-no-discord-linux = no Discord found on PATH; pass the command: phoecord launch -- /path/to/discord
cli-err-no-discord-windows = no Discord found in %LOCALAPPDATA%; pass the command: phoecord launch -- C:\path\to\Discord.exe
cli-err-no-discord-macos = no Discord found in /Applications; pass the command: phoecord launch -- /path/to/Discord
cli-error-prefix = error
cli-warning-prefix = warning

# CLI: plugins, enable, disable, toggle
cli-plugin-on = on
cli-plugin-off = off
cli-plugins-vencord = (Vencord: { $name })
cli-no-such-plugin = no plugin named { $name } (see `phoecord plugins`)
cli-plugin-state-on = { $name }: on
cli-plugin-state-off = { $name }: off
cli-gateway-note = (gateway plugin: takes effect when Discord reconnects; Ctrl+R in Discord does that)
cli-config-line = config: { $path }
cli-ca-line = local CA: { $path } (only valid for { $hosts })
plugin-desc-fake_nitro = Send any emoji without Nitro (as linked images)
plugin-desc-game_activity_toggle = While on, your status doesn't show the game you're playing
plugin-desc-message_logger = Keep deleted messages visible and show edit history
plugin-desc-no_track = Block Discord's analytics and crash reports
plugin-desc-relationship_notifier = Notify you when you're unfriended or removed from a server or group
plugin-desc-silent_typing = Your typing indicator is never sent
plugin-desc-clear_urls = Strip tracking parameters from links you send
plugin-desc-commands = The /phoecord slash command: plugins, toggle
plugin-desc-i_love_spam = Do not hide messages from 'likely spammers'
plugin-desc-no_blocked_messages = Hide all blocked/ignored messages from chat completely
plugin-desc-no_reply_mention = Disables reply pings by default
plugin-desc-text_replace = Replace text in the messages you send
plugin-desc-send_timestamps = Turn `12:34` in your messages into a Discord timestamp
plugin-desc-anonymise_file_names = Anonymise uploaded file names
plugin-desc-silent_message_toggle = Send messages without notifications

# CLI: log
cli-log-empty = nothing logged yet ({ $path })
cli-log-place-channel = in channel { $channel }
cli-log-place-guild = in guild { $guild }, channel { $channel }
cli-log-edited = { $time }  ✏ { $author } { $place }: { $was } → { $content }{ $tail }
cli-log-deleted = { $time }  🗑 { $author } { $place }: { $content }{ $tail }

# CLI: uninstall report (what was removed)
cli-step-dir = folder { $path }
cli-step-file = file { $path }
cli-step-ca-trusted = CA { $path } from the system trust stores
cli-step-run-key = Run key value { $name }
cli-step-launch-agent = LaunchAgent { $label }
cli-step-systemd-unit = systemd user unit { $unit }
cli-kept-already-gone = already gone
cli-kept-not-empty = not empty
cli-kept-holds-log = holds the log of failed steps
cli-kept-line = kept     { $what } ({ $why })

# CLI: selftest
cli-selftest-rest-ok = ✓ REST: discord.com intercepted, local CA accepted, API reachable
cli-selftest-rest-fail = ✗ REST: { $reason }
cli-selftest-rest-timeout = ✗ REST: timed out
cli-selftest-gateway-ok = ✓ gateway { $combo }: HELLO round-tripped (heartbeat { $ms } ms)
cli-selftest-gateway-fail = ✗ gateway { $combo }: { $reason }
cli-selftest-gateway-timeout = ✗ gateway { $combo }: timed out
cli-selftest-undecodable = ✗ { $count } gateway payload(s) PhoeCord could not decode
cli-selftest-decoded = proxy decoded { $count } payload(s)
cli-selftest-failed = { $count } check(s) failed
cli-selftest-passed = all checks passed
cli-selftest-err-connect-closed = proxy closed the CONNECT
cli-selftest-err-proxy-answered = proxy answered { $line }
cli-selftest-err-http = HTTP { $status }: { $body }
cli-selftest-err-handshake = handshake
cli-selftest-err-closed-before-hello = closed before HELLO
cli-selftest-err-expected-hello = expected HELLO, got op { $op }

# Terms of use (TERMS.md)
terms-prompt = Type `yes` to accept these terms and install; anything else cancels:
terms-declined = Terms not accepted; nothing was installed.
cli-arg-accept-terms = Skip the terms prompt: you have read TERMS.md and accept it

# Bug fixes 1.4.8
cli-uninstall-stopped = stopped  PhoeCord proxy (process { $pid })
cli-uninstall-stop-failed = could not stop a running PhoeCord proxy: { $why }
cli-arg-channel = Which Discord to start: stable, ptb or canary (needed when more than one is installed)
cli-err-channel-not-installed = Discord { $channel } is not installed
cli-err-several-clients = several Discord clients are installed, so none is picked for you. Pass --channel:
    { $list }
cli-proxy-reused = A PhoeCord proxy (version { $version }) already runs on { $listen }; starting Discord through it.
cli-err-port-taken = { $listen } is used by another program, not by PhoeCord. Close it or change `listen` in config.toml.
cli-arg-setup-channel = Add a Start-menu shortcut that starts this Discord through PhoeCord: stable, ptb or canary (default: the only one installed)
cli-shortcut-created = shortcut "{ $name }" added: { $path }
cli-shortcut-unsupported = No shortcut on this system; start Discord with `phoecord launch --channel { $channel }`.
cli-shortcut-skipped = No shortcut added: { $why }

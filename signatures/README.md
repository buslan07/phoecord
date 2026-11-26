# Signatures

AOB (array-of-bytes) signatures that let the PhoeCord loader find functions in a specific Discord client build. They live apart from the loader code so they can be updated without rebuilding it (ARCHITECTURE.md, "Finding functions").

**This folder ships no real Discord signatures yet.** Real tables are added with the features that need them (Phase 1 stage 1.8 for the current client, Phase 2 for Phoenix). Never commit client binaries, disassembly or dumps of them: a signature is a short byte mask plus a name.

## Where the loader looks

`%LOCALAPPDATA%\PhoeCord\signatures\*.toml`. Files are read in name order; a later file replaces a build table with the same key. A file that does not parse is skipped and logged; the rest still load.

## Hot-fix: a new client build

The loader reads the folder at every client start, so a new table needs no new DLL.

1. Find the signatures in the new build and write them under its build key.
2. Check them: `phoecord check <client exe> --signatures <folder>` must exit with code 0.
3. Give users the file. Where real tables are published is not decided yet.
4. Users run `phoecord signatures install <file or folder>` and restart Discord.

`signatures install` checks every file first and writes nothing if one does not load, holds no signature, has an empty build table or is not `*.toml`; all files are written under temporary names first and only then renamed into place. It replaces a file of the same name and warns when a file that is read later defines the same build. `phoecord signatures list` shows what is installed. `phoecord uninstall` removes the files it added; a file you had put there yourself stays, even after it was replaced.

## Format

```toml
# signatures/example.toml
[build."0.0.123"]
theme_palette  = { aob = "48 8D 0D ?? ?? ?? ?? E8 ?? ?? ?? ?? 48 8B D8", rip = 3 }
message_render = { aob = "40 53 48 83 EC 20 48 8B D9 E8" }

# Same version number, a specific hotfix build: key = "<version>+<.text SHA-256 prefix>"
[build."0.0.123+9687f167"]
theme_palette  = { aob = "..." }
```

- **Build key.** `"<version>"` is the file version of the client exe (`a.b.c.d` from its version resource). `"<version>+<hex>"` additionally requires the lowercase hex SHA-256 of the exe's `.text` section to start with `<hex>` (at least 8 digits). The hash form wins over the plain form when both match. The loader logs both values at start (`%LOCALAPPDATA%\PhoeCord\loader.log`).
- **Entry name.** Free-form, one per function; the feature that uses it asks for it by name.
- **`aob`.** Hex bytes separated by spaces; `??` (or `?`) matches any byte. A pattern of only wildcards is rejected.
- **`rip`** (optional). Byte offset inside the match of a 32-bit RIP-relative operand. The result is the address that operand points to (`match + rip + 4 + rel32`), instead of the match itself. For `48 8D 0D ?? ?? ?? ??` (`lea rcx, [rip+disp]`) the operand starts at byte 3, so `rip = 3`.
- Unknown keys in an entry are an error, so typos do not silently disable a signature.

## Uniqueness

A signature counts as found only if it matches **exactly once** in the client's `.text`. Zero matches and several matches both disable only the feature that needs it; the loader logs which one happened. Anchor signatures on strings and on calls into GPUI crates rather than on code the compiler inlines differently from build to build.

## Checking a build

`phoecord check <path to the client exe> [--signatures <folder>]` reads the exe from disk, without running it, picks the table for its build and prints every signature as found (with its RVA), missed or ambiguous. The folder defaults to `signatures` under the current directory; pass `%LOCALAPPDATA%\PhoeCord\signatures` to check what the loader would use. The exit code is 0 only when a table matches, it is not empty, every signature resolves and every file in the folder loads.

Client exes are never committed. To check several at once, keep them in a folder outside the repository and run `python -I scripts/check_builds.py <folder> --phoecord <phoecord binary>`. For Phoenix builds add `--phoenix`: it requires a `DiscordPhoenix` folder in every exe's path (`<folder>/<version>/DiscordPhoenix/Discord.exe`), which is what makes a missed typed locator fail. `--min-builds 2` also fails when there are fewer than two exes.

## Results

Resolved addresses are RVAs (relative to the exe's module base). The loader adds the base of the running process.

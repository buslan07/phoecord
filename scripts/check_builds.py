#!/usr/bin/env python3
"""Run `phoecord check` on every client exe of a folder kept outside the repository.

Client builds are never committed (docs/AGENTS.md), so this runs where they are: a developer's
machine, or a CI job that has them. Every `*.exe` under the folder is read, not run.

Usage: python -I scripts/check_builds.py <folder> [--phoecord <binary>]
       [--signatures <folder>] [--min-builds N] [--phoenix]

Exit code 0 only when the folder holds at least N builds (default 1) and every one passes.

`phoecord check` fails on a missed typed locator only for an exe with a `DiscordPhoenix` folder
in its path. With --phoenix every exe must have one (for example
`<folder>/<version>/DiscordPhoenix/Discord.exe`), so no build slips through unchecked.
Task 2.2.9 asks for two adjacent Phoenix builds: use --phoenix --min-builds 2.
"""

import argparse
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent


def main():
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("builds", type=Path, help="folder with client exes, outside the repository")
    parser.add_argument("--phoecord", default="phoecord", help="the phoecord binary to run")
    parser.add_argument("--signatures", type=Path, default=ROOT / "signatures")
    parser.add_argument("--min-builds", type=int, default=1, metavar="N")
    parser.add_argument(
        "--phoenix", action="store_true", help="require a DiscordPhoenix folder in every exe's path"
    )
    args = parser.parse_args()

    builds = args.builds.resolve()
    if not builds.is_dir():
        print(f"not a folder: {builds}", file=sys.stderr)
        return 1
    if builds == ROOT or ROOT in builds.parents:
        print(f"client builds must stay outside the repository: {builds}", file=sys.stderr)
        return 1
    exes = sorted(p for p in builds.rglob("*") if p.is_file() and p.suffix.lower() == ".exe")
    if len(exes) < max(args.min_builds, 1):
        print(f"{len(exes)} client exe(s) in {builds}, need {max(args.min_builds, 1)}", file=sys.stderr)
        return 1
    if args.phoenix:
        unmarked = [p for p in exes if "discordphoenix" not in (part.lower() for part in p.parts)]
        for exe in unmarked:
            print(f"no DiscordPhoenix folder in the path: {exe}", file=sys.stderr)
        if unmarked:
            return 1

    failed = []
    for exe in exes:
        name = exe.relative_to(builds)
        print(f"==> {name}", flush=True)
        command = [args.phoecord, "check", str(exe), "--signatures", str(args.signatures)]
        try:
            code = subprocess.run(command).returncode
        except OSError as e:
            print(f"cannot run {args.phoecord}: {e}", file=sys.stderr)
            return 1
        if code != 0:
            failed.append(name)

    print(f"{len(exes) - len(failed)} of {len(exes)} builds passed")
    for name in failed:
        print(f"failed: {name}")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())

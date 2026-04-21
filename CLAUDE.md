# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Status

**WIP — currently does not compile.** The codebase has known compilation errors being actively worked through.

## Commands

```bash
cargo build           # build (currently fails)
cargo test            # run tests
cargo test <name>     # run a single test
cargo doc --open      # generate and open docs
cargo clippy          # lint
```

## Architecture

Single-crate `no_std` embedded-hal driver for the Honeywell ABP series I2C pressure sensors.

**Entry point:** `src/lib.rs` — the entire driver is one file.

**Core struct:** `Abp<I2C, D>` — generic over an `embedded-hal` I2C implementation and a delay provider. Instantiated via `Abp::new(i2c, delay, part_nr)` where `part_nr` is the Honeywell part number string (e.g. `"ABPDNNN150PGAA3"`). The constructor parses the part number to extract pressure range, unit, sensor type, I2C address, and capabilities.

**Part number parsing** (`new()`): Positions in the part number string encode:
- bytes 8–10: numeric max pressure
- byte 11: unit (`M`=mbar, `B`=bar, `K`=kPa, `P`=psi)
- byte 12: type (`D`=differential, `G`=gauge)
- byte 13: I2C address selector (`0`–`7` → `0x08`–`0x78`)
- byte 14: transfer function (`A`/`T`=no sleep, `D`/`S`=sleep; `D`/`T`=has thermometer)

**Reading data:**
- `read()` → `nb::Result<f32>` — reads 2 bytes, returns pressure in Pa or `WouldBlock` if stale
- `pressure_and_temperature()` → reads 4 bytes, returns pressure (temperature decode is incomplete)

**Bit decoding:** `decode_pressure()` and `decode_pressure_and_temperature()` use the `bitmatch` crate to extract the 2-bit status field and 14-bit pressure value from the raw I2C response bytes.

**Known issues:**
- `convert_pressure()` formula appears inverted (maps output counts to pressure, not pressure to Pa)
- `pressure_and_temperature()` returns pressure but ignores the decoded temperature
- `read()` has unresolved type errors around `nb::Error` wrapping
- `quick-error` and `substring` crates are dependencies but `substring` usage via `.substring()` is the main string-slicing mechanism in `new()`

## Workflow

- add Doc Comments for every function, explain what it does and its in- and output 
- Use git worktrees for feature work to keep changes isolated from the current workspace. Before starting any non-trivial implementation, create a worktree on a new branch rather than working directly on the checked-out branch. Place worktrees in .worktrees in the project directory
- For every non-trivial implementation check crates.io if there is already a crate implementing the functionality. Use the `crates-mcp` MCP server (tools: `crates_search`, `crates_get`, `crates_get_versions`, `crates_get_dependencies`) — it has direct API access and is more reliable than context7 for Rust crates.
- use openspec to plan changes and new features
- when archiving the change, update [CHANGELOG.md](CHANGELOG.md):
- Put entries under a new version. Follow @[semantic versioning](https://semver.org/)
- Follow @[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) format — write for humans, not diffs
- Use [TODO.md](TODO.md) to track pending work
- Never push to the upstream repository unless specifically instructed
- update this Claude.md with important learnings
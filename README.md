# BURT — Beta Utility for Recording & Tracing
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

## Overview
**BURT** is a plugin inspired by [SAR](https://github.com/p2sr/SourceAutoRecord)
that aims to provide helpful functionality for leaked Portal 2 Beta builds.

The whole plugin is written in Rust with direct engine FFI.
Currently the only supported build is `July 2009 (852_0)`.

It's highly recommended to run it alongside
[nikolan](https://github.com/nikolan123)'s
[Beta Patcher](https://github.com/nikolan123/portal2-beta-patcher).

## Download
You can install BURT from the [releases page](https://github.com/Dap1n/burt/releases).
Put `burt.dll` in the same folder as your Portal 2 Beta `hl2.exe`.

Launch the game and execute `plugin_load burt` in console to load the plugin.

It's also highly recommended to create `portal2/cfg/autoexec.cfg`
file and write `plugin_load burt` there, so you don't have to load it manually each time.

## Commands

All commands are prefixed with `burt_`. Run any of them with `?` as the
first argument for a built-in description (e.g. `burt_sr_start ?`).

| Command | Description |
| --- | --- |
| `burt_sr_start [use_vault_save]` | Start a speedrun. By default loads `vault.sav` if available, otherwise starts a new game. Automatically finishes demo recording after appearance of ending screen on the last map `p2_lab_hub_6` |
| `burt_sr_stop` | End the current speedrun manually. |
| `burt_rec <0\|1>` | Toggle auto-recording on save/map load, outside of a speedrun. |
| `burt_sg_dbg <mode>` | Save Glitch debug camera. `-1` cleanup, `0` normal, `1` current→linked matrix transform, `2` inverted. |
| `burt_echo <color> <text>` | Colored console output. |

## Compiling from source
Make sure you have [Rust](https://rust-lang.org/tools/install)
and [Microsoft C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools) installed.

Clone the repo:
```
git clone https://github.com/Dap1n/burt
cd burt
```

Install the `i686-pc-windows-msvc` target:
```
rustup target add i686-pc-windows-msvc
```

Build the `.dll` (`i686-pc-windows-msvc` will be targeted automatically by `.cargo/config.toml`):
```
cargo build --release
```

Navigate to `target/i686-pc-windows-msvc/release` and find `burt.dll` in this folder.

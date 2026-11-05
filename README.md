# BURT — Beta Utility for Recording & Tracing
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

## Overview
**BURT** is a plugin inspired by [SAR](https://github.com/p2sr/SourceAutoRecord)
that aims to provide helpful functionality for leaked Portal 2 Beta builds.

The whole plugin is written in Rust with direct engine FFI.
Currently the only supported build is `July 2009 (852_0)`.

To get this build, you can obtain `.dat` and `.blob` with [Steam2 downloader](https://github.com/extremebleem/steam2_downloader) and use [nikolan](https://github.com/nikolan123)'s
[Beta Patcher](https://github.com/nikolan123/portal2-beta-patcher).

If you don't want to deal with it, you can install it from the [HL2 Beta website](https://hl2-beta.ru/index.php?action=downloads;sa=view;down=355;language=english).
These builds can be acquired in the [Portal Series Beta Research](https://discord.gg/36ePcNnTVv) server, where you can find GoFile links as well.

## Download
You can install BURT from the [releases page](https://github.com/Dap1n/burt/releases).
Put `burt.dll` in the same folder as your Portal 2 Beta `hl2.exe`.

Launch the game and execute `plugin_load burt` in the console to load the plugin.

It's also highly recommended to create a `portal2/cfg/autoexec.cfg`
file and write `plugin_load burt` there, so you don't have to load it manually each time.

You may also write `burt_clear_mdmp` as a second line, so your folder won't get bloated with `.mdmp` files if there have been too many crashes in the past.

## Commands
You can find all the BURT commands in [COMMANDS.md](COMMANDS.md). These include the `burt_*` and `bvar_*` ones.

BVar stands for BURT Variable, which really is just a fancy command that changes exactly one static atomic. That way I was able to make helpful descriptions for them as well as proper input sanitization (in P1/P2 writing something like `255+255+255` as a number in a ConVar would result in `-1`, bypassing the defined clamping).

## TASing
Since BURT v0.2.0, there's TAS support for the July 2009 build very similar to SAR!

If you want to experiment with it, check out [TASING.md](TASING.md).

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

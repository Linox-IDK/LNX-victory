# LNX-victory

A fork of [victory-gta6](https://github.com/fxceless/victory-gta6) (itself a fork
of [VIctory](https://github.com/Marvisak/VIctory)), the compatibility layer that
runs an Xbox Series title natively on Windows from files the user supplies.
This fork is tuned for **8 GB NVIDIA cards** (tested on an RTX 5060 Ti 8 GB,
Ryzen 5 9600X, 32 GB RAM, Windows 11).

**This repository contains no game files, code or assets.** You need your own
copy. Not affiliated with or endorsed by the title's publisher.

## Where it stands (2026-11-24)

**Not playable yet, but the world renders.** Series X profile, 1080p output.

| Stage | State |
| --- | --- |
| Boot, menus | works, 30 fps |
| Opening cutscene | plays, in 3D, correct colours |
| Prison scenes and first gameplay | reached; characters, cars, sky and lighting draw |
| Frame rate in the world | about 2.5 fps on average, 4-4.6 fps in lighter shots (was ~1.5) |
| Textures | an 8 GB card filled up and left people untextured; the new texture-quality switch is the fix being tested |
| Shadowed areas, skin | still dark: the title's ray-traced bounce light does not run yet |
| Stability | no driver hang or device loss in our runs |

![Sunset yard](docs/screenshots/sunset-yard.webp)
![Night street, opening credits](docs/screenshots/night-street.png)
![Visiting room before the texture fix: geometry in place, textures missing (VRAM full)](docs/screenshots/visiting-room-no-textures.webp)

## What this fork adds

- **Texture quality for 8 GB cards** (`VICTORY_TEXTURE_SKIP_MIPS=1`): big
  block-compressed textures become tiled resources with memory only from
  their second level down (about 4x less each); views clamp sampling there.
  Size, levels and subresource numbers stay as the title made them.
- **Shader cache that survives rebuilds**: keyed on the translator's source
  and the GPU's wave width instead of the whole executable.
- **Pre-compiling**: `victory translate` with `VICTORY_WARM_CACHE=1` fills the
  runtime shader cache offline (26,599 shaders, ~26,260 translate).
- **Pipeline builders that are not starved**: `VICTORY_PIPELINE_WORKERS`,
  `VICTORY_PIPELINE_PRIORITY=normal`.
- **Colour fix**: R9G9B9E5 is no longer renamed to R11G11B10 by default (the
  title packs its light in that format; renamed, every frame came out purple).
- **Shader translation**: `v_cvt_f32_f64`; the lane-mask restore from
  upstream that lets the irradiance-cache programs translate.
- **Series S** two-plane composition, a retry for texture formats the host
  refuses as unordered access, an optional present gamma (`VICTORY_GAMMA`).

## Settings we run with

Series X profile, `VICTORY_DISPLAY_MODE=1920x1080`, `VICTORY_RELAY=2`,
`VICTORY_WIDE_BACK=1`, `VICTORY_WIDE_REFRESH=1`, `VICTORY_WIDE_MERGE=1`,
`VICTORY_CARD_REHASH_RUNS=1`, `VICTORY_JOIN_COPIES=1`,
`VICTORY_PIPELINE_WAIT_MS=50`, `VICTORY_SHADER_LOOP_LIMIT=1024`,
`VICTORY_TEXTURE_SKIP_MIPS=1`. Measured: the relay took the cutscene from
~650 to ~450 ms a frame; the rest together to ~370 ms.

## Known problems

- Dark shadows and skin (ray-traced bounce light, upstream work in progress).
- Still images flicker with another copy of themselves.
- About 14 shaders in gameplay still do not translate (descriptors read as
  data, some wave64 forms), so some objects are missing.
- Most of a frame is spent copying title memory to and from the card, not on
  the GPU itself (~80-100 ms of GPU work per frame).

## Credits

- **[VIctory](https://github.com/Marvisak/VIctory)** by Marvisak and
  contributors: the base of everything (GPL-3.0-or-later, as is this fork).
- **[victory-gta6](https://github.com/fxceless/victory-gta6)** by fxceless:
  the world rendering, lighting and speed work this fork builds on.
- **[XCL](https://github.com/ivall/xcl)** by ivall and contributors, used as a reference.
- Changes developed with [Claude Code](https://claude.com/claude-code).

Everything below this line is upstream VIctory's own README, unchanged.

* * *

# VIctory

Runs Grand Theft Auto VI (Xbox Series X|S) natively on Windows, from files
the user supplies.

The title's x86-64 code runs unchanged inside our process; VIctory supplies
the platform underneath it and converts the title's data, ahead of time where
it can and on first use where it must (shaders). See
[docs/architecture.md](docs/architecture.md).

## Status (2026-11-23)

**Not playable yet.** The title boots, runs its menus at its own 30 fps and
loads the world, where rendering fails.

| Stage | State |
|-------|-------|
| Boot, first-boot screens (accessibility, language, settings, legal) | works, 30 fps |
| Intro video with sound | works |
| Start screen, profile and saves | works |
| Loading into the world (about 50 s) | works |
| The world (first mission's intro) | reached and runs with indirect draws; its title card renders, the scene after it is still dark; 0.5-1.8 s per frame |

Tested on one machine: Windows 11, AMD RX 6700 XT.

> **Warning.** Entering the world on a real GPU can currently hang it, and
> Windows then resets the display driver (every monitor goes black for a
> moment). To look further, use a diagnostics build with
> `VICTORY_D3D12_WARP=1`, which renders on the software adapter.

What each part does today and what it still needs:

| Subsystem | Today | Needs |
|-----------|-------|-------|
| Loader | Runs the title's executable and its DLLs in our process; crash reports name the faulting code and are kept in `crashes.log` | – |
| Runtime services | One signed-in offline user, licences, packages, saves that survive a kill at any step, text entry dialog | Speech is silent; the dialog and relaunch are untested live |
| Network | Offline by design: the title cannot reach the network | – |
| Files and streaming | Archives and streamed data read, zlib and BCPack decoded in parallel | Loading is limited by writes into title memory |
| Input | XInput pads, keyboard, keyboard stand-in pad | Trigger rumble |
| Audio | Menu, video and loading sound; XMA voices and Opus streams decode | Reverb is silent; Opus results unverified against the title |
| GPU: menus | Draws, compute and presentation on host D3D12; only changed title memory is copied | Frame pacing drifts on one screen in release builds |
| GPU: world | Mesh pipelines, shader-written buffers, descriptor lifetimes, indirect execution (arguments clamped), pipelines made by worker threads and kept on disk | Most of the scene is still missing (untranslated shaders drop draws); frames take seconds; real queries, depth bounds, HDR and scaling are missing |
| Shaders | Console GPU code recompiled to DXIL at first use, cached on disk. Menus: all. World: mesh 8,191 of 8,506, vertex/pixel 10,617 of 12,346, compute 1,643 of 2,539 | 64-bit image atomics, merged geometry stages, the ray-tracing builder's shaders; some compute programs need a GPU with 64-lane waves |
| Textures | Console tiled layouts and detiling for the modes the title uses | Render-target metadata planes |
| Series S mode | Runs through the menus to the loading screen | The world, as above |

## Getting started

Requirements: Windows 10/11 x64, a CPU with AVX2, a Direct3D 12 GPU with
shader model 6.6 (and mesh shaders for the world), Rust (pinned in
`rust-toolchain.toml`).

The title also needs several GiB of available Windows commit at startup.
Check **Committed** under **Task Manager → Performance → Memory**. If
`crashes.log` reports Windows error 1455, close other applications or increase
the Windows paging file size, then retry.

```powershell
cargo xtask install-hooks                # asset-policy check on every commit
copy victory.example.toml victory.toml   # then set your paths (outside the repo)
cargo test --workspace                   # needs no game data
cargo run -p victory -- config           # check your paths
```

Run the title:

```powershell
cargo build --release -p victory
target\release\victory.exe run <game dir>\<the title's exe> --execute
```

The window caption shows the framerate. Saves, settings and `crashes.log`
are kept under the data folder (default `%LOCALAPPDATA%\victory\data`), derived
data under the cache folder.

Input: XInput pads. With none connected the keyboard is a pad while the
title's window has focus:

| Keys | Pad |
|------|-----|
| W A S D / I J K L | left / right stick |
| arrows | D-pad |
| Enter, Backspace, X, Y | A, B, X, Y |
| Q / E, Z / C | shoulders, triggers |
| F / H | stick clicks |
| Esc, Tab | Menu, View |

## Commands

```
victory inspect <exe|dll> [--imports] [--exports]
victory census [paths...] [--missing] [--strict]
victory lib-exports <import.lib> [--list]
victory shader-info [paths...] [--verbose] [--disasm]
victory translate [paths...] [--validate] [--pipelines] [--list] [--jobs N]
victory run <exe> [--execute] [--unimplemented trap|zero] [--strict]
            [--console series-x|series-s] [-- <title arguments>]
victory config
```

Logging: `VICTORY_LOG=victory_loader=trace,info` (a release build logs
info and above).

## Debugging

A release build has no debugging aids. Build them in with

```powershell
cargo build --profile diag -p victory --features diagnostics   # target\diag\victory.exe
```

| Variable | Effect |
|----------|--------|
| `VICTORY_LOG=<target>=debug,...` | debug and trace output |
| `VICTORY_STACKS=20,60` | every thread's stack at those seconds |
| `VICTORY_WATCH=<rva>,...` | logs a title u32 when it changes |
| `VICTORY_KEYS=30:Enter,35:X` | presses keys at those seconds, focus or not |
| `VICTORY_CRASH_SAVE=<rva>:<len>,...` | saves title memory to the cache folder on a crash |
| `VICTORY_CAPTURE=<dir>` | saves presented frames (powers of two, or `VICTORY_CAPTURE_FRAMES=<list>`) |
| `VICTORY_DUMP_TARGETS=1` | with the two above, saves every render target of those frames |
| `VICTORY_D3D12_DEBUG=1` | host debug layer |
| `VICTORY_D3D12_GBV=1` | with the above, GPU-based validation (bad descriptor use reported, not faulted; slow) |
| `VICTORY_D3D12_DRED=1` | on a GPU failure, the allocation hit and the commands in flight |
| `VICTORY_PIPELINE_WAIT_MS=24` | reduce the per-frame wait for background pipeline creation from the default of 500 ms. A draw can be omitted until its pipeline is ready |
| `VICTORY_SKIP_PIPELINES=<hash>,...` | omit draws and dispatches, including indirect ones, whose pipeline contains a listed 16-digit shader hash |
| `VICTORY_GPU_TIMING=1` | with the above, each list of a submission run and timed on its own; slow lists logged with their slowest commands |
| `VICTORY_D3D12_WARP=1` | host device on the software adapter (slow; cannot reset the driver) |
| `VICTORY_SHADER_LOOP_LIMIT=<n>` | translated shader loops stop after `n` iterations per lane instead of 2^20 (own shader cache entries); if a lower `n` makes world frames much faster, loops hitting the limit are what makes them slow |
| `VICTORY_AUDIO_CAPTURE=<file.wav>` | records what the title plays (our process only) |
| `VICTORY_XMA_DUMP=<dir>` | saves the first 32 XMA voices' context and input |

Every build appends crashes and host GPU failures to `crashes.log` in the
title's data folder.

## License

GPL-3.0-or-later. See [LICENSE](LICENSE).

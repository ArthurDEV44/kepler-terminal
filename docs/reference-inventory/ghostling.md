# Ghostling Reference Inventory

Status: initial focused pass
Date: 2026-07-01
Reference source: `C:\dev\ghostling`
Scope: Ghostling lessons for Hera, especially libghostty embedding, host
effects, PTY to terminal ingestion, render-state iteration, input encoding and
renderer independence.

## Executive Takeaways

Ghostling is not a terminal engine reference. It is a host integration
reference: a tiny C/Raylib application that embeds `libghostty-vt`, wires a
Unix PTY into it, snapshots render state, iterates rows/cells and draws through
a simple 2D renderer.

The high-value lessons:

1. The strongest idea is the embedding shape: host reads PTY bytes, calls
   `ghostty_terminal_vt_write`, updates render state, then renders from row and
   cell iterators.
2. Ghostling proves that a terminal core can know nothing about PTY, windowing
   or drawing while still exposing enough state for a real host.
3. The input encoders are a useful model: keyboard, mouse, focus and scrollbar
   behavior are host concerns, but they must query terminal modes before
   emitting escape sequences.
4. Host effects matter. The terminal needs callbacks or effect outputs for PTY
   responses, size queries, device attributes, title changes and similar
   protocol side effects.
5. Do not copy Ghostling's language stack for Hera. C is used here to
   showcase the C API, not because C is the right engine language.
6. Do not copy Ghostling's PTY layer for cross-platform Hera. It is Unix-only
   in this repo, built around `forkpty`, `fcntl`, `ioctl` and `libutil`.
7. Do copy the narrowness: no tabs, splits, session management, config system
   or renderer dependency in the terminal core.

Confidence: high for host-loop, PTY, input, render-state and build findings
because this pass covers the single `main.c`, `README.md`, `CMakeLists.txt`,
`flake.nix` and repo instructions under `C:\dev\ghostling`.

## Why Ghostling Matters

The README states the purpose directly: Ghostling is a minimal libghostty
terminal built around the libghostty C API in a single C file
(`C:\dev\ghostling\README.md:1`, `C:\dev\ghostling\README.md:5`). It uses
Raylib for windowing and rendering, is single-threaded, and deliberately uses a
2D renderer rather than Ghostty's production GPU renderers
(`C:\dev\ghostling\README.md:7`). It is explicitly not meant to be a full
daily-use terminal (`C:\dev\ghostling\README.md:16`).

The important architecture claim is that `libghostty-vt` handles VT parsing,
terminal state, styles, text reflow, scrollback and renderer state, but contains
no renderer drawing or windowing code (`C:\dev\ghostling\README.md:30`,
`C:\dev\ghostling\README.md:33`). The README also says Raylib is not required
and that libghostty lets any renderer layer on top of its render-state API
(`C:\dev\ghostling\README.md:160`, `C:\dev\ghostling\README.md:165`,
`C:\dev\ghostling\README.md:168`).

Hera implication: this is the cleanest small example of the host boundary
Hera wants to expose, even though Hera should implement its core in Rust.

## Codebase Map

| Area | Local path | What it contains | Hera relevance |
|---|---|---|---|
| Host app | `C:\dev\ghostling\main.c` | Single-file C host: PTY, input, effects, render loop. | Best small embedding walkthrough. |
| Build | `C:\dev\ghostling\CMakeLists.txt` | CMake, Raylib, fetched Ghostty commit, font header generation. | Shows libghostty-vt consumer build shape, not Hera stack. |
| Dev shell | `C:\dev\ghostling\flake.nix` | Zig 0.15.2, CMake, Ninja, Linux X/Wayland deps, Darwin SDK cleanup. | Dependency evidence and platform limits. |
| Font asset | `C:\dev\ghostling\fonts` | Embedded JetBrains Mono font. | Simple renderer fixture, not core logic. |
| Repo notes | `C:\dev\ghostling\AGENTS.md` | Build commands and libghostty header location after build. | Confirms generated/fetched API surface. |

## Language And Platform Decision

Ghostling does not change Hera's language decision. It is a C demo because
libghostty wants to showcase a broad, minimal C API, not because terminal cores
should be written in C.

| Zone | Ghostling evidence | Hera decision |
|---|---|---|
| Terminal core | Ghostling delegates core state to `libghostty-vt`, not to its C app (`C:\dev\ghostling\README.md:30`). | Keep Hera core Rust-owned. |
| Public embedding API | README says the C API enables thin bindings in many languages (`C:\dev\ghostling\README.md:152`, `C:\dev\ghostling\README.md:155`). | Future C ABI is valid, but generated around Rust. |
| Host renderer | Raylib is explicitly optional (`C:\dev\ghostling\README.md:160`). | Keep `terminal-render-model` renderer-neutral. |
| PTY | Main code uses Unix headers, `forkpty`, nonblocking fd and `ioctl` (`C:\dev\ghostling\main.c:13`, `C:\dev\ghostling\main.c:15`, `C:\dev\ghostling\main.c:56`, `C:\dev\ghostling\main.c:87`, `C:\dev\ghostling\main.c:1458`). | Use Rust PTY traits and per-platform impls. |
| Windows | README says Ghostling itself has not implemented/tested Windows, although libghostty-vt supports it (`C:\dev\ghostling\README.md:69`). | Do not use Ghostling as Windows reference. |
| Build stack | CMake project is C-only and fetches Ghostty/Raylib (`C:\dev\ghostling\CMakeLists.txt:1`, `C:\dev\ghostling\CMakeLists.txt:2`, `C:\dev\ghostling\CMakeLists.txt:18`, `C:\dev\ghostling\CMakeLists.txt:38`). | Hera remains Cargo/Rust-first. |

The right interpretation: Ghostling validates a C ABI as an eventual
distribution surface. It rejects C as a reason to move Hera's internal engine
out of Rust.

## Build And Dependency Shape

Ghostling is a CMake project requiring CMake 3.19 and C
(`C:\dev\ghostling\CMakeLists.txt:1`, `C:\dev\ghostling\CMakeLists.txt:2`).
It resolves Zig from the active environment before adding Ghostty
(`C:\dev\ghostling\CMakeLists.txt:13`) because Ghostty's CMake path delegates
to Zig. It uses Raylib 5.5, either system-installed or fetched from GitHub
(`C:\dev\ghostling\CMakeLists.txt:18`, `C:\dev\ghostling\CMakeLists.txt:19`,
`C:\dev\ghostling\CMakeLists.txt:21`, `C:\dev\ghostling\CMakeLists.txt:24`).
It fetches Ghostty from GitHub at a pinned commit
(`C:\dev\ghostling\CMakeLists.txt:38`, `C:\dev\ghostling\CMakeLists.txt:39`,
`C:\dev\ghostling\CMakeLists.txt:40`) and links the app against Raylib and
`ghostty-vt` (`C:\dev\ghostling\CMakeLists.txt:61`,
`C:\dev\ghostling\CMakeLists.txt:64`).

Platform linking is partial: macOS adds IOKit, Cocoa and OpenGL frameworks
(`C:\dev\ghostling\CMakeLists.txt:67`, `C:\dev\ghostling\CMakeLists.txt:68`,
`C:\dev\ghostling\CMakeLists.txt:69`, `C:\dev\ghostling\CMakeLists.txt:70`).
Unix non-Apple links `util` for `forkpty`
(`C:\dev\ghostling\CMakeLists.txt:74`, `C:\dev\ghostling\CMakeLists.txt:75`).
The Nix shell pins Zig 0.15.2 and adds Linux X/Wayland libraries
(`C:\dev\ghostling\flake.nix:22`, `C:\dev\ghostling\flake.nix:24`,
`C:\dev\ghostling\flake.nix:25`, `C:\dev\ghostling\flake.nix:34`,
`C:\dev\ghostling\flake.nix:35`, `C:\dev\ghostling\flake.nix:42`).

Hera implication: this is a good external-consumer build example for a future
ABI package, not a template for Hera's Rust workspace.

## PTY And Byte Ingestion

Ghostling's PTY path is intentionally small. `pty_spawn` creates a pseudo
terminal with `forkpty`, sets the initial window size, chooses a shell from
`$SHELL`, passwd or `/bin/sh`, sets `TERM=xterm-256color`, and makes the master
fd nonblocking (`C:\dev\ghostling\main.c:43`,
`C:\dev\ghostling\main.c:47`, `C:\dev\ghostling\main.c:56`,
`C:\dev\ghostling\main.c:65`, `C:\dev\ghostling\main.c:80`,
`C:\dev\ghostling\main.c:87`). Writes retry partial writes and drop the
remainder on `EAGAIN` (`C:\dev\ghostling\main.c:98`,
`C:\dev\ghostling\main.c:103`, `C:\dev\ghostling\main.c:111`,
`C:\dev\ghostling\main.c:113`). Reads drain available bytes and feed them into
`ghostty_terminal_vt_write` (`C:\dev\ghostling\main.c:126`,
`C:\dev\ghostling\main.c:132`, `C:\dev\ghostling\main.c:138`).

Hera implication: the flow is right, the implementation is not portable
enough. For Hera:

- `terminal-core` should expose `write_bytes`.
- `terminal-pty` should own POSIX PTY and Windows ConPTY.
- Backpressure policy must be explicit. Ghostling's drop-on-`EAGAIN` is fine
  for a demo, not for a product core.
- The terminal core must not spawn processes.

## Input Encoding

Ghostling delegates input encoding to libghostty instead of hardcoding escape
tables. Mouse handling syncs encoder options from terminal state, provides
screen/cell geometry, emits press/release/motion events, and either forwards
wheel events to the application or scrolls the viewport when mouse tracking is
off (`C:\dev\ghostling\main.c:318`, `C:\dev\ghostling\main.c:323`,
`C:\dev\ghostling\main.c:330`, `C:\dev\ghostling\main.c:347`,
`C:\dev\ghostling\main.c:383`, `C:\dev\ghostling\main.c:397`,
`C:\dev\ghostling\main.c:418`, `C:\dev\ghostling\main.c:439`).

Keyboard handling syncs encoder options from terminal state, scans Raylib key
events, records unshifted codepoints for Kitty keyboard protocol, tracks
consumed modifiers, encodes with `ghostty_key_encoder_encode`, then writes the
result to the PTY (`C:\dev\ghostling\main.c:444`,
`C:\dev\ghostling\main.c:449`, `C:\dev\ghostling\main.c:454`,
`C:\dev\ghostling\main.c:513`, `C:\dev\ghostling\main.c:521`,
`C:\dev\ghostling\main.c:530`, `C:\dev\ghostling\main.c:547`,
`C:\dev\ghostling\main.c:550`). It falls back to direct UTF-8 writes if no key
event consumed text (`C:\dev\ghostling\main.c:557`,
`C:\dev\ghostling\main.c:562`).

Hera implication: input encoding belongs outside `terminal-core`, but it
needs a terminal-mode aware API. Hera should eventually provide a Rust input
encoder crate or host helper, rather than forcing each UI adapter to maintain
escape tables.

## Host Effects

Ghostling registers host effects so terminal protocol queries can be answered.
`effect_write_pty` writes terminal-generated responses back to the PTY
(`C:\dev\ghostling\main.c:1100`, `C:\dev\ghostling\main.c:1104`,
`C:\dev\ghostling\main.c:1109`). `effect_size` answers XTWINOPS size queries
with rows, columns and cell pixel size (`C:\dev\ghostling\main.c:1112`,
`C:\dev\ghostling\main.c:1114`, `C:\dev\ghostling\main.c:1119`,
`C:\dev\ghostling\main.c:1122`). `effect_device_attributes` reports device
capabilities (`C:\dev\ghostling\main.c:1126`,
`C:\dev\ghostling\main.c:1129`, `C:\dev\ghostling\main.c:1136`,
`C:\dev\ghostling\main.c:1140`). Other effects handle version, title changes
and color-scheme query behavior (`C:\dev\ghostling\main.c:1153`,
`C:\dev\ghostling\main.c:1161`, `C:\dev\ghostling\main.c:1167`,
`C:\dev\ghostling\main.c:1176`, `C:\dev\ghostling\main.c:1179`,
`C:\dev\ghostling\main.c:1182`).

Main then attaches those effects to the terminal through `ghostty_terminal_set`
(`C:\dev\ghostling\main.c:1301`, `C:\dev\ghostling\main.c:1313`,
`C:\dev\ghostling\main.c:1316`, `C:\dev\ghostling\main.c:1318`,
`C:\dev\ghostling\main.c:1320`, `C:\dev\ghostling\main.c:1322`,
`C:\dev\ghostling\main.c:1324`, `C:\dev\ghostling\main.c:1326`).

Hera implication: this validates a host-effect surface, but Hera should
prefer queued effects over direct reentrant callbacks. A Rust API like
`Terminal::drain_effects()` is easier to reason about than arbitrary host calls
during parsing.

## Render State Boundary

Ghostling's renderer proves the snapshot model in miniature. Main creates a
terminal with rows, columns and scrollback, resizes it with cell pixel
dimensions, then creates render-state handles and reusable row/cell iterators
(`C:\dev\ghostling\main.c:1275`, `C:\dev\ghostling\main.c:1278`,
`C:\dev\ghostling\main.c:1279`, `C:\dev\ghostling\main.c:1289`,
`C:\dev\ghostling\main.c:1381`, `C:\dev\ghostling\main.c:1383`,
`C:\dev\ghostling\main.c:1390`, `C:\dev\ghostling\main.c:1397`).

Each frame, after resize, PTY read and input handling, the host snapshots the
terminal into the render state with `ghostty_render_state_update`
(`C:\dev\ghostling\main.c:1430`, `C:\dev\ghostling\main.c:1491`,
`C:\dev\ghostling\main.c:1517`, `C:\dev\ghostling\main.c:1522`,
`C:\dev\ghostling\main.c:1531`). The render function pulls colors, obtains a
row iterator, loops rows, obtains cell iterators, reads grapheme length,
grapheme buffer, foreground/background colors and style, then draws through
Raylib (`C:\dev\ghostling\main.c:798`, `C:\dev\ghostling\main.c:812`,
`C:\dev\ghostling\main.c:823`, `C:\dev\ghostling\main.c:837`,
`C:\dev\ghostling\main.c:839`, `C:\dev\ghostling\main.c:845`,
`C:\dev\ghostling\main.c:848`, `C:\dev\ghostling\main.c:871`,
`C:\dev\ghostling\main.c:891`, `C:\dev\ghostling\main.c:895`,
`C:\dev\ghostling\main.c:901`, `C:\dev\ghostling\main.c:925`).

It also clears dirty row state after rendering and resets global dirty state at
the end of the frame (`C:\dev\ghostling\main.c:936`,
`C:\dev\ghostling\main.c:938`, `C:\dev\ghostling\main.c:1014`,
`C:\dev\ghostling\main.c:1016`).

Hera implication: this is exactly the shape for `terminal-render-model`.
Instead of exposing renderer callbacks, Hera should expose a stable pull
snapshot with colors, cursor, rows, cells, style, graphemes and dirty flags.

## Kitty Graphics And Advanced Protocols

Ghostling enables Kitty graphics by setting a storage limit and enabling file,
temp-file and shared-memory mediums (`C:\dev\ghostling\main.c:1329`,
`C:\dev\ghostling\main.c:1331`, `C:\dev\ghostling\main.c:1332`,
`C:\dev\ghostling\main.c:1338`, `C:\dev\ghostling\main.c:1340`,
`C:\dev\ghostling\main.c:1342`). It registers a PNG decoder before terminal
creation (`C:\dev\ghostling\main.c:1269`,
`C:\dev\ghostling\main.c:1273`).

The rendering side iterates Kitty placements by layer, gets images, resolves
viewport position, image size, format, data pointer, grid size, source
rectangle and offsets, then uploads a temporary Raylib texture
(`C:\dev\ghostling\main.c:662`, `C:\dev\ghostling\main.c:670`,
`C:\dev\ghostling\main.c:674`, `C:\dev\ghostling\main.c:679`,
`C:\dev\ghostling\main.c:682`, `C:\dev\ghostling\main.c:685`,
`C:\dev\ghostling\main.c:694`, `C:\dev\ghostling\main.c:702`,
`C:\dev\ghostling\main.c:717`, `C:\dev\ghostling\main.c:726`,
`C:\dev\ghostling\main.c:739`, `C:\dev\ghostling\main.c:746`,
`C:\dev\ghostling\main.c:759`). The code itself warns that this is simple but
inefficient because every visible image is re-uploaded every frame
(`C:\dev\ghostling\main.c:657`, `C:\dev\ghostling\main.c:662`).

Hera implication: Ghostling shows why image protocols should remain M1
metadata/placeholders. Rendering them correctly requires storage limits, decode
hooks, placement layers, media policies and renderer caches.

## Main Loop Shape

The main loop is compact:

```text
while window open:
  resize terminal and PTY
  send focus transitions if enabled
  drain PTY into terminal
  handle scrollbar, keyboard, mouse
  update render state from terminal
  draw render state
```

The exact local sequence is visible in `main.c`: resize calls both
`ghostty_terminal_resize` and `ioctl(TIOCSWINSZ)` (`C:\dev\ghostling\main.c:1436`,
`C:\dev\ghostling\main.c:1446`, `C:\dev\ghostling\main.c:1458`), PTY reading
calls `pty_read` (`C:\dev\ghostling\main.c:1491`), input is forwarded only while
the child is alive (`C:\dev\ghostling\main.c:1520`,
`C:\dev\ghostling\main.c:1522`, `C:\dev\ghostling\main.c:1524`), then render
state is updated and drawn (`C:\dev\ghostling\main.c:1531`,
`C:\dev\ghostling\main.c:1546`, `C:\dev\ghostling\main.c:1548`).

Hera implication: great host-loop reference for `terminal-cli` or a future
demo host. Not enough for a production cross-platform runtime.

## What Hera Should Copy

Copy:

- A tiny embeddable terminal API.
- Byte ingestion separate from PTY ownership.
- Renderer-neutral render state.
- Row/cell iterators or snapshot slices with dirty flags.
- Terminal-mode aware key and mouse encoders.
- Host effects for PTY responses, size, device attributes and titles.
- Scrollbar/viewport API as terminal state, not renderer state.
- Clear separation between terminal features and GUI features.

Do not copy:

- C as Hera's core implementation language.
- Raylib as a dependency or design constraint.
- Unix-only PTY assumptions.
- Drop-on-backpressure behavior for product code.
- Reentrant callback-heavy host effects without an effect queue.
- Per-frame image texture upload.
- Broad image rendering in M1.

## Recommended Hera Shape

Ghostling reinforces this target:

```text
terminal-core
  write_bytes(bytes)
  resize(cols, rows, cell_px)
  drain_effects()
  snapshot_render_state()

terminal-render-model
  colors
  cursor
  rows
  cells
  grapheme data
  style data
  dirty flags
  image placeholders

terminal-pty
  POSIX PTY
  Windows ConPTY
  lifecycle
  backpressure
  resize propagation

host adapter
  reads PTY
  feeds terminal-core
  encodes input from terminal modes
  draws render model
```

Bottom line: Ghostling is the best small proof that an embeddable terminal core
can be renderer-agnostic and host-driven. For Hera, it strengthens the API
boundary, but it does not weaken the Rust-first decision.

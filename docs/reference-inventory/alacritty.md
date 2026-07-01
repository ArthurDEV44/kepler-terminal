# Alacritty Reference Inventory

Status: initial focused pass
Date: 2026-07-01
Reference source: `C:\dev\alacritty`
Scope: Alacritty architecture lessons for Kepler, especially Rust M1 core,
grid, reflow, PTY and render damage.

## Executive Takeaways

Alacritty is the strongest Rust-native reference for Kepler M1. Ghostty is a
better long-term architecture reference for embeddable render state and
page-based scrollback, but Alacritty is the cleaner first implementation
reference because it already proves a Rust terminal crate, `vte` parser
integration, cross-platform PTY, grid storage, reflow and damage tracking.

The high-value lessons:

1. Use Rust as the product language. Alacritty's workspace already ships a
   Rust terminal library plus platform-specific Rust PTY code.
2. Hide `vte` behind Kepler actions. Alacritty exposes `vte`, but Kepler should
   keep the parser swappable.
3. Start M1 with an Alacritty-style grid: visible rows plus scrollback in a
   ring-like storage layer.
4. Implement resize and reflow early. Alacritty's `grid/resize.rs` is the most
   concrete Rust reference for wrap-aware resizing.
5. Treat damage as a core output. Alacritty has `TermDamage`, per-line damage
   bounds and reset semantics.
6. Keep renderer and app dependencies outside the terminal core. Alacritty's
   app uses winit, glutin and OpenGL, while `alacritty_terminal` remains a
   separate crate.
7. Copy the reference-test style: replay recorded bytes into a terminal and
   compare serialized grid state.

Confidence: high for Rust/core/grid/PTY/test findings because they are grounded
in source files under `C:\dev\alacritty`. Medium for renderer conclusions
because this pass sampled renderer/display boundaries rather than auditing every
OpenGL path.

## Why Alacritty Matters

Alacritty describes itself as a fast, cross-platform OpenGL terminal emulator
(`C:\dev\alacritty\README.md:5`) and supports BSD, Linux, macOS and Windows
(`C:\dev\alacritty\README.md:18`). It requires OpenGL ES 2.0 and, on Windows,
ConPTY support (`C:\dev\alacritty\README.md:53`,
`C:\dev\alacritty\README.md:54`).

The repo is especially useful for Kepler because it separates the app crate from
the terminal crate. The workspace contains `alacritty`, `alacritty_terminal`,
`alacritty_config` and `alacritty_config_derive`
(`C:\dev\alacritty\Cargo.toml:1`). The `alacritty_terminal` crate describes
itself as a library for writing terminal emulators
(`C:\dev\alacritty\alacritty_terminal\Cargo.toml:6`) and re-exports `Term`,
`Grid` and `vte` (`C:\dev\alacritty\alacritty_terminal\src\lib.rs:18`,
`C:\dev\alacritty\alacritty_terminal\src\lib.rs:19`,
`C:\dev\alacritty\alacritty_terminal\src\lib.rs:20`).

Kepler implication: Alacritty is the best local proof that Kepler can stay
Rust-first across Windows, Linux and macOS without introducing C#, C++, Zig,
Swift or Objective-C into the core.

## Codebase Map

| Area | Local path | What it contains | Kepler relevance |
|---|---|---|---|
| Workspace | `C:\dev\alacritty\Cargo.toml` | Rust workspace with app, terminal and config crates. | Confirms a Rust split between core and app is viable. |
| Terminal crate | `C:\dev\alacritty\alacritty_terminal` | `Term`, `Grid`, parser integration, PTY, events, tests. | Primary M1 reference. |
| App crate | `C:\dev\alacritty\alacritty` | winit/glutin/OpenGL display, input, config, windows. | Host/app boundary reference, not core. |
| Grid | `C:\dev\alacritty\alacritty_terminal\src\grid` | Storage, rows, scrollback, resize/reflow. | First Kepler storage model reference. |
| Terminal state | `C:\dev\alacritty\alacritty_terminal\src\term` | State, modes, selection, damage, renderable content. | Core state semantics and damage. |
| PTY | `C:\dev\alacritty\alacritty_terminal\src\tty` | Unix PTY, Windows ConPTY, evented read/write. | `terminal-pty` reference. |
| Event loop | `C:\dev\alacritty\alacritty_terminal\src\event_loop.rs` | PTY read/write loop, parser, wakeups. | Runtime boundary and replay recorder. |
| Display/render | `C:\dev\alacritty\alacritty\src\display`, `C:\dev\alacritty\alacritty\src\renderer` | Renderable content, damage tracker, OpenGL renderer. | Render-model extraction lessons. |
| Reference tests | `C:\dev\alacritty\alacritty_terminal\tests\ref.rs`, `tests/ref` | Recorded byte streams plus expected grid JSON. | Fixture model for Kepler. |

## Workspace And Language Strategy

The root workspace uses Rust edition 2024 and rust-version 1.85.0
(`C:\dev\alacritty\Cargo.toml:9`, `C:\dev\alacritty\Cargo.toml:10`). The
terminal crate depends on `vte`, `parking_lot`, `polling`, `regex-automata`,
`unicode-width`, `rustix`, `rustix-openpty` and `windows-sys`
(`C:\dev\alacritty\alacritty_terminal\Cargo.toml:19`,
`C:\dev\alacritty\alacritty_terminal\Cargo.toml:28`,
`C:\dev\alacritty\alacritty_terminal\Cargo.toml:29`,
`C:\dev\alacritty\alacritty_terminal\Cargo.toml:30`,
`C:\dev\alacritty\alacritty_terminal\Cargo.toml:35`,
`C:\dev\alacritty\alacritty_terminal\Cargo.toml:40`).

The app crate uses `winit`, `glutin`, `crossfont`, `copypasta`, `objc2` on
macOS and `windows-sys` on Windows
(`C:\dev\alacritty\alacritty\Cargo.toml:24`,
`C:\dev\alacritty\alacritty\Cargo.toml:27`,
`C:\dev\alacritty\alacritty\Cargo.toml:28`,
`C:\dev\alacritty\alacritty\Cargo.toml:74`,
`C:\dev\alacritty\alacritty\Cargo.toml:88`). Linux backends are feature-gated
with `x11` and `wayland` features (`C:\dev\alacritty\alacritty\Cargo.toml:103`).

Kepler implication:

- Keep core crates Rust-only.
- Platform-specific Rust modules are enough for PTY and host adapters.
- macOS framework integration can use Rust `objc2` where necessary.
- App/render dependencies should live outside `terminal-core`.
- Do not import Alacritty's OpenGL bias into Kepler's core API.

## Terminal Core Shape

`alacritty_terminal` exposes modules for events, event loop, grid, index,
selection, sync, term, thread, tty and vi mode
(`C:\dev\alacritty\alacritty_terminal\src\lib.rs:7`). The central type is
`Term<T>` (`C:\dev\alacritty\alacritty_terminal\src\term\mod.rs:268`). It stores
focus state, vi cursor, selection, active grid, inactive grid, tabs, scroll
region, colors, modes, title state, damage and config
(`C:\dev\alacritty\alacritty_terminal\src\term\mod.rs:269`,
`C:\dev\alacritty\alacritty_terminal\src\term\mod.rs:275`,
`C:\dev\alacritty\alacritty_terminal\src\term\mod.rs:281`,
`C:\dev\alacritty\alacritty_terminal\src\term\mod.rs:287`,
`C:\dev\alacritty\alacritty_terminal\src\term\mod.rs:301`,
`C:\dev\alacritty\alacritty_terminal\src\term\mod.rs:325`).

The active and inactive grids implement primary/alternate screen behavior.
The active grid is the alternate buffer while alternate screen is enabled,
otherwise it is the primary buffer
(`C:\dev\alacritty\alacritty_terminal\src\term\mod.rs:279`). The inactive grid
is the opposite buffer (`C:\dev\alacritty\alacritty_terminal\src\term\mod.rs:285`).
`swap_alt` swaps the grids, copies cursor state where needed, resets alternate
screen contents and clears selection
(`C:\dev\alacritty\alacritty_terminal\src\term\mod.rs:713`,
`C:\dev\alacritty\alacritty_terminal\src\term\mod.rs:716`,
`C:\dev\alacritty\alacritty_terminal\src\term\mod.rs:723`,
`C:\dev\alacritty\alacritty_terminal\src\term\mod.rs:731`).

Kepler implication:

- M1 can start with `Terminal { state, parser_boundary }` and an internal
  Alacritty-style `Grid<Cell>`.
- Primary and alternate screen should be separate grids/screens.
- Do not model alternate screen as a viewport flag.
- Keep selection and vi/search UI state either outside the core or clearly
  marked as user-layer state.

## Parser Integration

Alacritty uses the `vte` crate directly (`C:\dev\alacritty\alacritty_terminal\Cargo.toml:28`).
The terminal crate also publicly re-exports `vte`
(`C:\dev\alacritty\alacritty_terminal\src\lib.rs:20`).

The parser is not owned by `Term`. In the PTY event loop, `State` stores
`parser: ansi::Processor` (`C:\dev\alacritty\alacritty_terminal\src\event_loop.rs:404`)
and PTY reads call `state.parser.advance(&mut **terminal, &buf[..unprocessed])`
(`C:\dev\alacritty\alacritty_terminal\src\event_loop.rs:154`). Reference tests
follow the same shape: create `ansi::Processor`, create `Term`, advance parser
with recorded bytes, then compare the grid
(`C:\dev\alacritty\alacritty_terminal\tests\ref.rs:114`,
`C:\dev\alacritty\alacritty_terminal\tests\ref.rs:116`).

Kepler implication:

- Reuse `vte` first, but do not leak it through public Kepler APIs.
- Kepler should own `TerminalAction` or equivalent normalized parser actions.
- Keeping parser state outside `Term` is viable, but Kepler may prefer
  `Terminal { parser, state }` for simpler embedding and replay.
- Public API should ingest bytes, not require callers to drive `vte`.

## Grid And Scrollback Storage

Alacritty's grid is a compact M1-friendly model. `Grid<T>` is "grid based
terminal content storage" (`C:\dev\alacritty\alacritty_terminal\src\grid\mod.rs:81`).
It owns raw storage, line/column counts, display offset and max scroll limit
(`C:\dev\alacritty\alacritty_terminal\src\grid\mod.rs:121`,
`C:\dev\alacritty\alacritty_terminal\src\grid\mod.rs:126`,
`C:\dev\alacritty\alacritty_terminal\src\grid\mod.rs:134`,
`C:\dev\alacritty\alacritty_terminal\src\grid\mod.rs:137`).

History is line-count bounded. `update_history` shrinks raw storage when current
history exceeds configured history and clamps display offset
(`C:\dev\alacritty\alacritty_terminal\src\grid\mod.rs:154`). `scroll_display`
handles delta, page up, page down and top by changing display offset within
history bounds (`C:\dev\alacritty\alacritty_terminal\src\grid\mod.rs:164`).
`history_size` is total lines minus screen lines
(`C:\dev\alacritty\alacritty_terminal\src\grid\mod.rs:516`).

The raw storage is a vector of rows with a modular `zero` index, visible line
count and active length (`C:\dev\alacritty\alacritty_terminal\src\grid\storage.rs:33`,
`C:\dev\alacritty\alacritty_terminal\src\grid\storage.rs:41`,
`C:\dev\alacritty\alacritty_terminal\src\grid\storage.rs:44`,
`C:\dev\alacritty\alacritty_terminal\src\grid\storage.rs:50`). Storage
rotation is optimized by changing `zero` rather than rearranging elements
(`C:\dev\alacritty\alacritty_terminal\src\grid\storage.rs:17`). Runtime growth
uses `initialize`, and shrink can truncate cached capacity
(`C:\dev\alacritty\alacritty_terminal\src\grid\storage.rs:126`,
`C:\dev\alacritty\alacritty_terminal\src\grid\storage.rs:107`).

Rows track occupied cells with `occ` to avoid scanning/resetting full row
contents unnecessarily (`C:\dev\alacritty\alacritty_terminal\src\grid\row.rs:17`,
`C:\dev\alacritty\alacritty_terminal\src\grid\row.rs:20`). Shrinking a row can
return removed non-empty cells for reflow (`C:\dev\alacritty\alacritty_terminal\src\grid\row.rs:70`).

Kepler implication:

- Use Alacritty-style ring storage for M1 because it is simple, Rust-native and
  proven.
- Add Kepler's stable row handles and generation now, even if storage starts as
  ring-based.
- Keep Ghostty-style page/chunk storage as the long-session destination.
- Add byte budget policy before claiming "huge scrollback".

## Resize And Reflow

Alacritty's resize path is the most useful Rust reference for Kepler M1. Grid
resize accepts a `reflow` flag and new line/column counts
(`C:\dev\alacritty\alacritty_terminal\src\grid\resize.rs:14`). It grows or
shrinks visible lines, then grows or shrinks columns
(`C:\dev\alacritty\alacritty_terminal\src\grid\resize.rs:22`,
`C:\dev\alacritty\alacritty_terminal\src\grid\resize.rs:28`).

Line growth pulls from history while possible and adjusts cursor plus display
offset (`C:\dev\alacritty\alacritty_terminal\src\grid\resize.rs:40`,
`C:\dev\alacritty\alacritty_terminal\src\grid\resize.rs:54`,
`C:\dev\alacritty\alacritty_terminal\src\grid\resize.rs:63`,
`C:\dev\alacritty\alacritty_terminal\src\grid\resize.rs:67`). Line shrink
keeps the cursor near the bottom by pushing history upward, then clamps cursors
and storage (`C:\dev\alacritty\alacritty_terminal\src\grid\resize.rs:73`,
`C:\dev\alacritty\alacritty_terminal\src\grid\resize.rs:84`,
`C:\dev\alacritty\alacritty_terminal\src\grid\resize.rs:95`).

Column growth unwraps rows when wrapline markers show content can move back to
the previous line (`C:\dev\alacritty\alacritty_terminal\src\grid\resize.rs:100`,
`C:\dev\alacritty\alacritty_terminal\src\grid\resize.rs:102`,
`C:\dev\alacritty\alacritty_terminal\src\grid\resize.rs:146`). Column shrink
wraps overflowing content forward, handles wide-char spacer edge cases and
clamps display offset (`C:\dev\alacritty\alacritty_terminal\src\grid\resize.rs:244`,
`C:\dev\alacritty\alacritty_terminal\src\grid\resize.rs:272`,
`C:\dev\alacritty\alacritty_terminal\src\grid\resize.rs:289`,
`C:\dev\alacritty\alacritty_terminal\src\grid\resize.rs:371`).

`Term::resize` applies this to active and inactive grids differently depending
on whether alternate screen is active. Primary reflows when not on alt screen,
and alternate reflows only when it is the active grid
(`C:\dev\alacritty\alacritty_terminal\src\term\mod.rs:655`,
`C:\dev\alacritty\alacritty_terminal\src\term\mod.rs:676`,
`C:\dev\alacritty\alacritty_terminal\src\term\mod.rs:677`,
`C:\dev\alacritty\alacritty_terminal\src\term\mod.rs:678`).

Kepler implication:

- Resize is not optional infrastructure. It shapes storage, cursor, selection,
  scrollback, damage and semantic handles.
- M1 should include reflow fixtures before adding PTY complexity.
- Wide characters, wrapped lines and saved cursor must be in the first resize
  test corpus.
- Kepler should expose a clear policy: primary screen reflows, alternate screen
  has stricter/no-history semantics.

## Damage And Render Model

Alacritty's core has a useful damage model. `LineDamageBounds` stores damaged
left and right columns (`C:\dev\alacritty\alacritty_terminal\src\term\mod.rs:137`).
`TermDamage` is either full damage or partial damage with an iterator
(`C:\dev\alacritty\alacritty_terminal\src\term\mod.rs:176`,
`C:\dev\alacritty\alacritty_terminal\src\term\mod.rs:178`,
`C:\dev\alacritty\alacritty_terminal\src\term\mod.rs:183`). `Term::damage`
documents that user-controlled UI elements such as vi cursor and selection are
not part of the damage state and that `reset_damage` should be called after
reading damage (`C:\dev\alacritty\alacritty_terminal\src\term\mod.rs:450`,
`C:\dev\alacritty\alacritty_terminal\src\term\mod.rs:454`).

Alacritty also exposes renderable terminal content. `Term::renderable_content`
returns `RenderableContent` (`C:\dev\alacritty\alacritty_terminal\src\term\mod.rs:637`),
which includes selection, cursor, display offset, colors, mode and a display
iterator (`C:\dev\alacritty\alacritty_terminal\src\term\mod.rs:2393`). The app
crate wraps this into display-specific `RenderableContent`, converting terminal
cells into `RenderableCell`s and skipping empty cells/spacers
(`C:\dev\alacritty\alacritty\src\display\content.rs:24`,
`C:\dev\alacritty\alacritty\src\display\content.rs:153`,
`C:\dev\alacritty\alacritty\src\display\content.rs:179`).

The display layer adds a frame-level damage tracker. `DamageTracker` tracks
current and next frame damage (`C:\dev\alacritty\alacritty\src\display\damage.rs:12`,
`C:\dev\alacritty\alacritty\src\display\damage.rs:24`) and can shape damage
into compositor rectangles (`C:\dev\alacritty\alacritty\src\display\damage.rs:92`).
`FrameDamage` can damage individual lines, points or mark the frame fully
damaged (`C:\dev\alacritty\alacritty\src\display\damage.rs:140`,
`C:\dev\alacritty\alacritty\src\display\damage.rs:152`,
`C:\dev\alacritty\alacritty\src\display\damage.rs:163`).

Kepler implication:

- `terminal-render-model` should expose both content and damage.
- Core damage should be independent from UI overlays.
- A renderer-neutral snapshot can combine Alacritty's `RenderableContent` idea
  with Ghostty's richer render-state snapshot.
- M1 can start with full or line damage, then add cell spans as needed.

## Cell, Hyperlink, Selection And Search Primitives

Alacritty cells carry primary char, colors, flags and optional extra storage.
Flags include wrapline, wide char, wide-char spacer and leading wide-char spacer
(`C:\dev\alacritty\alacritty_terminal\src\term\cell.rs:15`,
`C:\dev\alacritty\alacritty_terminal\src\term\cell.rs:21`,
`C:\dev\alacritty\alacritty_terminal\src\term\cell.rs:22`,
`C:\dev\alacritty\alacritty_terminal\src\term\cell.rs:23`,
`C:\dev\alacritty\alacritty_terminal\src\term\cell.rs:28`). Extra storage
holds zero-width characters and hyperlinks
(`C:\dev\alacritty\alacritty_terminal\src\term\cell.rs:125`). Hyperlinks have
an identifier and URI (`C:\dev\alacritty\alacritty_terminal\src\term\cell.rs:44`,
`C:\dev\alacritty\alacritty_terminal\src\term\cell.rs:78`,
`C:\dev\alacritty\alacritty_terminal\src\term\cell.rs:81`).

Selection supports simple, block, semantic and line modes
(`C:\dev\alacritty\alacritty_terminal\src\selection.rs:93`,
`C:\dev\alacritty\alacritty_terminal\src\selection.rs:103`). Semantic selection
expands to semantic boundaries (`C:\dev\alacritty\alacritty_terminal\src\selection.rs:105`)
and `range_semantic` uses `semantic_search_left` and `semantic_search_right`
(`C:\dev\alacritty\alacritty_terminal\src\selection.rs:298`,
`C:\dev\alacritty\alacritty_terminal\src\selection.rs:313`,
`C:\dev\alacritty\alacritty_terminal\src\selection.rs:314`).

Kepler implication:

- Model wide-char spacers and zero-width chars from the start.
- Hyperlink storage belongs in core/protocol, but rendering is host-specific.
- Semantic selection is not the same as agent session semantics, but it proves
  row/cell semantic metadata belongs near terminal state.
- Selection should survive scroll/resize through stable handles where possible.

## PTY And Runtime Boundary

Alacritty's `tty` module defines configuration options, shell options,
`EventedReadWrite`, `ChildEvent` and `EventedPty`
(`C:\dev\alacritty\alacritty_terminal\src\tty\mod.rs:21`,
`C:\dev\alacritty\alacritty_terminal\src\tty\mod.rs:48`,
`C:\dev\alacritty\alacritty_terminal\src\tty\mod.rs:65`,
`C:\dev\alacritty\alacritty_terminal\src\tty\mod.rs:82`,
`C:\dev\alacritty\alacritty_terminal\src\tty\mod.rs:92`). This is not as
general as WezTerm's `portable-pty`, but it is concrete and Rust-native.

Unix PTY uses `rustix_openpty::openpty` (`C:\dev\alacritty\alacritty_terminal\src\tty\unix.rs:21`)
and creates a PTY from `openpty` with the requested window size
(`C:\dev\alacritty\alacritty_terminal\src\tty\unix.rs:195`,
`C:\dev\alacritty\alacritty_terminal\src\tty\unix.rs:196`). It implements
`EventedReadWrite`, returning reader and writer handles
(`C:\dev\alacritty\alacritty_terminal\src\tty\unix.rs:323`,
`C:\dev\alacritty\alacritty_terminal\src\tty\unix.rs:372`,
`C:\dev\alacritty\alacritty_terminal\src\tty\unix.rs:377`), and implements
resize through `OnResize` (`C:\dev\alacritty\alacritty_terminal\src\tty\unix.rs:406`,
`C:\dev\alacritty\alacritty_terminal\src\tty\unix.rs:411`).

Windows uses ConPTY. The ConPTY module imports `CreatePseudoConsole`,
`ResizePseudoConsole`, `ClosePseudoConsole` and `HPCON`
(`C:\dev\alacritty\alacritty_terminal\src\tty\windows\conpty.rs:11`). It also
dynamically loads the functions when needed
(`C:\dev\alacritty\alacritty_terminal\src\tty\windows\conpty.rs:78`) and wraps
blocking reads/writes through helper threads and pipes
(`C:\dev\alacritty\alacritty_terminal\src\tty\windows\blocking.rs:1`,
`C:\dev\alacritty\alacritty_terminal\src\tty\windows\blocking.rs:53`,
`C:\dev\alacritty\alacritty_terminal\src\tty\windows\blocking.rs:151`).

The PTY event loop owns the parser and batches PTY IO. It has constants for max
bytes before terminal synchronization and while the terminal is locked
(`C:\dev\alacritty\alacritty_terminal\src\event_loop.rs:23`,
`C:\dev\alacritty\alacritty_terminal\src\event_loop.rs:26`). It reads PTY bytes,
optionally records them into `alacritty.recording`, advances the parser and
wakes the UI (`C:\dev\alacritty\alacritty_terminal\src\event_loop.rs:104`,
`C:\dev\alacritty\alacritty_terminal\src\event_loop.rs:149`,
`C:\dev\alacritty\alacritty_terminal\src\event_loop.rs:154`,
`C:\dev\alacritty\alacritty_terminal\src\event_loop.rs:167`).

Kepler implication:

- `terminal-pty` can be Rust-only.
- Put PTY in M2, but design core ingestion around batched bytes from day one.
- Add explicit backpressure and synchronization budgets.
- ConPTY should be a transport adapter, never the source of terminal truth.

## App, Display And Renderer Boundary

The app crate depends on `alacritty_terminal` locally
(`C:\dev\alacritty\alacritty\Cargo.toml:11`) and initializes windows through
winit/glutin. `main.rs` imports winit's `EventLoop`
(`C:\dev\alacritty\alacritty\src\main.rs:24`) and creates a processor
(`C:\dev\alacritty\alacritty\src\main.rs:208`). `WindowContext` owns display,
terminal, PTY event loop, notifier and renderer state
(`C:\dev\alacritty\alacritty\src\window_context.rs:50`,
`C:\dev\alacritty\alacritty\src\window_context.rs:53`,
`C:\dev\alacritty\alacritty\src\window_context.rs:214`).

Terminal creation and PTY creation happen in app/window setup:
`Term::new` is called from window context
(`C:\dev\alacritty\alacritty\src\window_context.rs:193`), then `tty::new`
creates the PTY (`C:\dev\alacritty\alacritty\src\window_context.rs:201`), then
`PtyEventLoop::new` wires PTY, terminal and event proxy
(`C:\dev\alacritty\alacritty\src\window_context.rs:214`). Drawing calls
`display.draw` with terminal, config, search state and cursor state
(`C:\dev\alacritty\alacritty\src\window_context.rs:391`).

The renderer itself is OpenGL-oriented. `Renderer` owns text and rectangle
renderers (`C:\dev\alacritty\alacritty\src\renderer\mod.rs:89`) and draws
iterators of `RenderableCell` (`C:\dev\alacritty\alacritty\src\renderer\mod.rs:177`).

Kepler implication:

- Alacritty's host/app split is worth copying, not its OpenGL renderer.
- `terminal-core` must have zero GPUI/winit/glutin dependencies.
- `terminal-render-model` should be consumed by GPUI, CLI snapshots and tests.
- The app layer can handle overlays: cursor blink, IME, search bars, message
  bars, debug damage, hyperlink preview.

## Reference Tests And Fixtures

Alacritty's reference tests are a direct template for Kepler. The macro lists
many recorded cases: alternate reset, hyperlinks, history, delete/insert lines,
saved cursor, wrapline alt toggle, zero-width chars, VTTET cursor movement,
scrolling, origin mode and more
(`C:\dev\alacritty\alacritty_terminal\tests\ref.rs:30`). Each test folder has
`alacritty.recording`, `size.json`, `grid.json` and `config.json`
(`C:\dev\alacritty\alacritty_terminal\tests\ref.rs:101`,
`C:\dev\alacritty\alacritty_terminal\tests\ref.rs:102`,
`C:\dev\alacritty\alacritty_terminal\tests\ref.rs:103`,
`C:\dev\alacritty\alacritty_terminal\tests\ref.rs:104`).

The test creates a `Term`, creates a `vte` processor, replays the recording,
initializes/truncates grid storage, then compares expected and actual grids
(`C:\dev\alacritty\alacritty_terminal\tests\ref.rs:113`,
`C:\dev\alacritty\alacritty_terminal\tests\ref.rs:114`,
`C:\dev\alacritty\alacritty_terminal\tests\ref.rs:116`,
`C:\dev\alacritty\alacritty_terminal\tests\ref.rs:119`,
`C:\dev\alacritty\alacritty_terminal\tests\ref.rs:122`).

Kepler implication:

- Create fixture format now: bytes, size, config, expected snapshot.
- Include Alacritty reference cases in Kepler's fixture backlog.
- Test core with no PTY and no renderer.
- Add a debug CLI command to record/replay byte streams.

## What Kepler Should Copy

Copy these ideas directly:

- Rust crate separation: app outside terminal core.
- `vte` parser integration as implementation detail.
- `Grid<Cell>` with visible rows plus scrollback history.
- Ring-like storage with cheap rotation.
- Row occupancy tracking.
- Wrapline-driven resize/reflow.
- Primary/alternate screen swap via separate grids.
- `TermDamage` style full/partial damage.
- Renderable content iterator.
- Recorded byte fixtures plus serialized expected grid.
- Rust Unix PTY and Windows ConPTY adapters.

## What Kepler Should Adapt

Adapt these ideas rather than copying blindly:

- Alacritty exposes `vte`; Kepler should hide it.
- Alacritty history is line-count based; Kepler needs line plus byte budgets.
- Alacritty grid indices are not stable enough for semantic session indexes;
  Kepler needs stable row handles and generations.
- Alacritty renderable content is app-oriented; Kepler needs a serializable
  renderer-neutral snapshot.
- Alacritty damage is viewport-focused; Kepler should connect damage to
  snapshots, replay and remote rendering.
- Alacritty's PTY traits are useful, but WezTerm's `portable-pty` surface is a
  stronger public trait model.

## What Kepler Should Avoid

Avoid these traps:

- Do not inherit Alacritty's "terminal app first" product constraints.
- Do not tie core render output to OpenGL, glutin or winit.
- Do not expose raw `Grid` internals as public API.
- Do not rely only on line-count history for long agent sessions.
- Do not mix input/UI selection state with durable semantic session metadata.
- Do not defer reflow tests. Alacritty shows that reflow is core behavior.
- Do not add tabs/splits to Kepler core. Alacritty explicitly leaves tabs and
  splits to window managers or multiplexers (`C:\dev\alacritty\README.md:104`).

## Language Decision For Kepler

Alacritty strengthens the Rust-first language policy more than any other
reference so far.

| Zone | Alacritty reference | Kepler language decision |
|---|---|---|
| Terminal state core | Rust `alacritty_terminal::Term` | Rust only. |
| Parser | Rust `vte` crate | Rust wrapper behind Kepler actions. |
| Grid/storage | Rust ring-like `Storage<Row<Cell>>` | Rust, with stable handles added. |
| Resize/reflow | Rust `grid/resize.rs` | Rust, fixture-driven. |
| PTY Unix | Rust `rustix-openpty` path | Rust plus Unix syscalls/bindings. |
| PTY Windows | Rust ConPTY via `windows-sys` | Rust plus `windows-rs` or `windows-sys`. |
| macOS framework access | Rust `objc2` in app crate | Rust adapter only if needed. |
| Renderer | Rust OpenGL app renderer | Future GPUI adapter, no core dependency. |
| Fixtures | Rust tests and JSON grids | Rust fixture harness. |

Decision: Alacritty is the practical M1 blueprint. Ghostty remains the richer
long-term render-state and scrollback architecture reference.

## Follow-Up Fixtures For Kepler

Alacritty-inspired fixture backlog:

1. Basic printable text, SGR and truecolor.
2. Zero-width chars and wide-char spacers.
3. Wrapline on/off and cursor pending wrap.
4. Resize wider with unwrap.
5. Resize narrower with wrap.
6. Primary vs alternate screen resize.
7. Saved cursor in alternate screen.
8. Scroll region up/down preserving history.
9. Delete/insert lines and chars.
10. Hyperlink OSC 8.
11. Selection across wrapped lines.
12. Semantic selection boundaries.
13. Term damage full vs partial.
14. Display offset while reading new bytes.
15. ConPTY replay corpus feeding the same core.

## Open Questions For Later Passes

Open points not fully audited in this first Alacritty file:

- Exact performance tradeoff between Alacritty storage and Ghostty page storage
  for 100k to 1M lines.
- Whether Kepler should fork/patch `vte` or wrap only.
- How much of Alacritty's damage model survives a remote/snapshot renderer.
- Whether Alacritty's PTY loop backpressure is enough for Paneflow sessions.
- How to merge Alacritty-style grid storage with Ghostty-style tracked pins.

None of these block M1. The immediate extraction is clear: use Alacritty as the
Rust implementation template, then add Kepler-specific stable handles, snapshots,
byte budgets and semantic sidecars.

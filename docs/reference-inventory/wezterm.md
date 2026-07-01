# WezTerm Reference Inventory

Status: initial focused pass
Date: 2026-07-01
Reference source: `C:\dev\wezterm`
Scope: WezTerm architecture lessons for Kepler, especially Rust terminal core,
parser/action layering, PTY abstraction, mux/runtime, renderable state and
cross-platform host boundaries.

## Executive Takeaways

WezTerm is the strongest Rust reference for a full terminal product architecture:
terminal core, parser, PTY, mux, GUI, remote client, font/rendering and platform
windowing all live in one Rust workspace. For Kepler, its value is not that M1
should copy WezTerm's full surface. Its value is that it proves the main product
layers can remain Rust while platform-specific code is isolated behind crates
and traits.

The high-value lessons:

1. Keep the terminal engine as a crate. `wezterm-term` is a GUI-free,
   PTY-free terminal model fed by `advance_bytes`.
2. Keep PTY behind a trait crate. `portable-pty` defines `PtySystem`,
   `MasterPty`, `SlavePty`, `Child` and uses Unix PTY or Windows ConPTY behind
   the same API.
3. Keep parser output as owned actions before applying to state. WezTerm parses
   bytes into `Action` values, coalesces printable chars and then applies those
   actions to `TerminalState`.
4. Track row identity separately from physical storage. `StableRowIndex` is the
   key concept that makes scrollback, dirty rows and remote render caches sane.
5. Track changes with sequence numbers. WezTerm's lines carry `SequenceNo`,
   which lets mux/client layers ask what changed since a known version.
6. Treat mux as a later layer. The mux is powerful, but M1 Kepler should start
   with the same boundaries, not the same feature volume.
7. Keep Kepler Rust-first across Windows, Linux and macOS. WezTerm uses native
   platform dependencies, but they sit at window/font/PTY boundaries, not in the
   terminal state core.

Confidence: high for workspace, terminal-core, PTY, mux and renderable-state
findings because they are grounded in the local source tree under
`C:\dev\wezterm`. Medium for GUI/render conclusions because this pass sampled
the renderer/windowing crates rather than auditing every WebGPU/OpenGL path.

## Why WezTerm Matters

The root README positions WezTerm as a GPU-accelerated cross-platform terminal
emulator and multiplexer implemented in Rust (`C:\dev\wezterm\README.md:3`).
That matters for Kepler because WezTerm is not only a terminal model; it is a
shipping Rust product across desktop platforms.

The workspace is broad. The root `Cargo.toml` includes product crates such as
`wezterm-escape-parser`, `wezterm-gui`, `wezterm-surface`, `wezterm-term`,
`portable-pty`, `termwiz` and `vtparse` (`C:\dev\wezterm\Cargo.toml:2`,
`C:\dev\wezterm\Cargo.toml:11`, `C:\dev\wezterm\Cargo.toml:13`,
`C:\dev\wezterm\Cargo.toml:17`, `C:\dev\wezterm\Cargo.toml:171`,
`C:\dev\wezterm\Cargo.toml:217`, `C:\dev\wezterm\Cargo.toml:235`,
`C:\dev\wezterm\Cargo.toml:251`, `C:\dev\wezterm\Cargo.toml:258`,
`C:\dev\wezterm\Cargo.toml:259`).

It also has platform and renderer dependencies: macOS Cocoa/CoreGraphics,
Wayland, X11, WinAPI/Windows and WGPU (`C:\dev\wezterm\Cargo.toml:66`,
`C:\dev\wezterm\Cargo.toml:72`, `C:\dev\wezterm\Cargo.toml:237`,
`C:\dev\wezterm\Cargo.toml:238`, `C:\dev\wezterm\Cargo.toml:263`,
`C:\dev\wezterm\Cargo.toml:265`, `C:\dev\wezterm\Cargo.toml:268`,
`C:\dev\wezterm\Cargo.toml:270`).

Kepler implication: WezTerm validates Rust-first architecture, not "pure Rust
with zero native bindings." Native OS APIs are acceptable when hidden behind
the right crate boundary.

## Codebase Map

| Area | Local path | What it contains | Kepler relevance |
|---|---|---|---|
| Terminal core | `C:\dev\wezterm\term` | `Terminal`, `TerminalState`, `Screen`, performer, VT state. | Best reference for a Rust engine crate boundary. |
| Parser tables | `C:\dev\wezterm\vtparse` | DEC ANSI state machine, UTF-8, CSI/OSC/DCS callbacks. | Parser comparison with `alacritty-vte`, not first choice for Kepler M1. |
| Escape parser | `C:\dev\wezterm\wezterm-escape-parser` | Converts parser callbacks into typed `Action`, `CSI`, OSC, Sixel, Kitty image actions. | Reference for `terminal-protocol` enum design. |
| Surface | `C:\dev\wezterm\wezterm-surface` | Lines, cells, sequence numbers, hyperlinks, clusters. | Render model and dirty-line reference. |
| PTY | `C:\dev\wezterm\pty` | `portable-pty`, Unix PTY, Windows ConPTY. | Primary `terminal-pty` reference. |
| Mux | `C:\dev\wezterm\mux` | Panes, tabs, domains, PTY read threads, action coalescing. | Runtime and future session model reference. |
| Client render cache | `C:\dev\wezterm\wezterm-client` | Remote/client renderable cache and dirty fetch logic. | Reference for future remote/headless rendering. |
| GUI | `C:\dev\wezterm\wezterm-gui` | Term window, render code, input, panes, overlays. | Later host reference, not M1 core. |
| Window crate | `C:\dev\wezterm\window` | Platform windowing and OS integration. | Boundary evidence for platform-specific host code. |
| Test data | `C:\dev\wezterm\test-data` | Terminal visual/protocol sample data. | Fixture source for later conformance tests. |

## Language And Platform Decision

WezTerm strongly supports staying Rust-first for Kepler.

The terminal core is a Rust crate named `wezterm-term`
(`C:\dev\wezterm\term\Cargo.toml:3`) and describes itself as a virtual terminal
emulator core (`C:\dev\wezterm\term\Cargo.toml:7`). The PTY crate is also Rust
and describes itself as a cross-platform PTY interface
(`C:\dev\wezterm\pty\Cargo.toml:1`, `C:\dev\wezterm\pty\Cargo.toml:7`).

The right split for Kepler:

| Kepler zone | Language | WezTerm evidence |
|---|---|---|
| Parser adapter | Rust | Parser/action crates are Rust. |
| Terminal state | Rust | `wezterm-term` owns full state and escape handling. |
| Surface/render model | Rust | `wezterm-surface` owns lines, cells and sequence numbers. |
| PTY | Rust plus OS FFI | `portable-pty` hides Unix PTY and Windows ConPTY behind traits. |
| Mux/session runtime | Rust | `mux` is Rust, with threads and trait objects. |
| Native host/windowing | Rust plus OS APIs | WezTerm pulls Cocoa, Wayland, X11, WinAPI and Windows crates at boundaries. |

Do not introduce C#, Objective-C, Swift or Zig for Kepler's core. If Kepler
needs native UI or OS integration later, use thin Rust FFI or platform crates
inside host adapters. The engine should remain Rust-owned.

## Terminal Core Boundary

`wezterm-term` gives the cleanest statement of the core boundary. The README
says the crate provides the virtual terminal emulator core
(`C:\dev\wezterm\term\README.md:3`) and is full featured: escape parsing,
keyboard/mouse input encoding, screen cells, scrollback, Sixel, iTerm2 images,
OSC 8 hyperlinks and terminal cell attributes (`C:\dev\wezterm\term\README.md:8`).

It also explicitly excludes GUI and PTY management. The host provides a writer
and feeds output bytes through `advance_bytes` (`C:\dev\wezterm\term\README.md:13`,
`C:\dev\wezterm\term\README.md:16`). The source docs repeat the same shape
(`C:\dev\wezterm\term\src\lib.rs:1`, `C:\dev\wezterm\term\src\lib.rs:6`,
`C:\dev\wezterm\term\src\lib.rs:11`, `C:\dev\wezterm\term\src\lib.rs:14`).

Kepler implication: build a `terminal-core` crate with no GUI and no PTY
ownership. Feed it bytes or actions, let it write responses through a writer,
and expose render snapshots/dirty lines.

## Terminal Shape

WezTerm's `Terminal` is intentionally small: it owns `TerminalState` and a
parser (`C:\dev\wezterm\term\src\terminal.rs:85`,
`C:\dev\wezterm\term\src\terminal.rs:87`,
`C:\dev\wezterm\term\src\terminal.rs:89`). Construction wires size, config,
terminal identity and writer into `TerminalState`, then creates a parser
(`C:\dev\wezterm\term\src\terminal.rs:154`,
`C:\dev\wezterm\term\src\terminal.rs:155`).

The ingestion API is the important part:

| API | Source | Lesson |
|---|---|---|
| `advance_bytes` | `C:\dev\wezterm\term\src\terminal.rs:164` | Feed arbitrary PTY chunks. |
| parser to performer | `C:\dev\wezterm\term\src\terminal.rs:170` | Parse into actions and apply them to state. |
| `perform_actions` | `C:\dev\wezterm\term\src\terminal.rs:176` | Allows replaying typed actions without raw bytes. |
| seqno bump | `C:\dev\wezterm\term\src\terminal.rs:165`, `C:\dev\wezterm\term\src\terminal.rs:177` | Change tracking begins at terminal ingress. |

Kepler shape:

```text
Terminal
  state: TerminalState
  parser: ParserAdapter

advance_bytes(bytes)
  -> parser.parse(bytes)
  -> performer.apply(action)
  -> state.seqno changes
  -> dirty render rows
```

This is a better M1 target than a full app loop.

## Parser And Action Layer

WezTerm has a two-step parser design:

1. `vtparse` implements a DEC ANSI parser modified for UTF-8
   (`C:\dev\wezterm\vtparse\src\lib.rs:2`) and exposes `VTActor` callbacks
   (`C:\dev\wezterm\vtparse\src\lib.rs:90`).
2. `wezterm-escape-parser` converts low-level callbacks into typed `Action`
   values (`C:\dev\wezterm\wezterm-escape-parser\src\lib.rs:42`).

`Action` is a useful model for Kepler's `terminal-protocol`: print,
`PrintString`, control, device control, OSC, CSI, escape, Sixel, termcap and
Kitty image actions (`C:\dev\wezterm\wezterm-escape-parser\src\lib.rs:46`,
`C:\dev\wezterm\wezterm-escape-parser\src\lib.rs:50`,
`C:\dev\wezterm\wezterm-escape-parser\src\lib.rs:54`,
`C:\dev\wezterm\wezterm-escape-parser\src\lib.rs:55`,
`C:\dev\wezterm\wezterm-escape-parser\src\lib.rs:57`,
`C:\dev\wezterm\wezterm-escape-parser\src\lib.rs:60`,
`C:\dev\wezterm\wezterm-escape-parser\src\lib.rs:61`).

`Action::append_to` coalesces printable characters into `PrintString`
(`C:\dev\wezterm\wezterm-escape-parser\src\lib.rs:67`,
`C:\dev\wezterm\wezterm-escape-parser\src\lib.rs:69`,
`C:\dev\wezterm\wezterm-escape-parser\src\lib.rs:72`,
`C:\dev\wezterm\wezterm-escape-parser\src\lib.rs:80`). The parser wrapper owns
a `VTParser`, exposes `parse`, `parse_first`, `parse_as_vec` and implements
`VTActor` to emit higher-level actions
(`C:\dev\wezterm\wezterm-escape-parser\src\parser\mod.rs:64`,
`C:\dev\wezterm\wezterm-escape-parser\src\parser\mod.rs:76`,
`C:\dev\wezterm\wezterm-escape-parser\src\parser\mod.rs:92`,
`C:\dev\wezterm\wezterm-escape-parser\src\parser\mod.rs:130`,
`C:\dev\wezterm\wezterm-escape-parser\src\parser\mod.rs:166`,
`C:\dev\wezterm\wezterm-escape-parser\src\parser\mod.rs:210`).

Kepler implication: keep `alacritty-vte` as the simpler M1 parser seed, but
borrow WezTerm's typed action vocabulary. The public protocol should look more
like `Action` than like raw `vte::Perform`.

## Performer And Semantics

`Performer` is where typed parser actions mutate terminal state. It dispatches
actions to print, control, DCS, OSC, ESC, CSI, Sixel and Kitty image handlers
(`C:\dev\wezterm\term\src\terminalstate\performer.rs:252`,
`C:\dev\wezterm\term\src\terminalstate\performer.rs:267`,
`C:\dev\wezterm\term\src\terminalstate\performer.rs:268`,
`C:\dev\wezterm\term\src\terminalstate\performer.rs:274`,
`C:\dev\wezterm\term\src\terminalstate\performer.rs:275`,
`C:\dev\wezterm\term\src\terminalstate\performer.rs:276`,
`C:\dev\wezterm\term\src\terminalstate\performer.rs:277`,
`C:\dev\wezterm\term\src\terminalstate\performer.rs:278`,
`C:\dev\wezterm\term\src\terminalstate\performer.rs:279`,
`C:\dev\wezterm\term\src\terminalstate\performer.rs:281`).

The semantic surface is broad:

| Surface | Source | Kepler relevance |
|---|---|---|
| Print path | `C:\dev\wezterm\term\src\terminalstate\performer.rs:366` | Hot path for grid mutation. |
| CSI SGR | `C:\dev\wezterm\term\src\terminalstate\performer.rs:495`, `C:\dev\wezterm\term\src\terminalstate\mod.rs:2652` | Attribute parsing and pen updates. |
| OSC titles | `C:\dev\wezterm\term\src\terminalstate\performer.rs:741`, `C:\dev\wezterm\term\src\terminalstate\performer.rs:753`, `C:\dev\wezterm\term\src\terminalstate\performer.rs:762` | Host title events. |
| OSC 8 hyperlinks | `C:\dev\wezterm\term\src\terminalstate\performer.rs:769` | Cell hyperlink attribute. |
| OSC 52 clipboard | `C:\dev\wezterm\term\src\terminalstate\performer.rs:784`, `C:\dev\wezterm\term\src\terminalstate\performer.rs:789` | Must be host-gated in Kepler. |
| User vars | `C:\dev\wezterm\term\src\terminalstate\performer.rs:829` | Useful later for shell integration. |
| Semantic prompt markers | `C:\dev\wezterm\term\src\terminalstate\performer.rs:863`, `C:\dev\wezterm\term\src\terminalstate\performer.rs:886`, `C:\dev\wezterm\term\src\terminalstate\performer.rs:897` | Good future Paneflow-style intelligence surface. |
| Current working directory | `C:\dev\wezterm\term\src\terminalstate\performer.rs:936` | Shell integration state. |
| Dynamic colors | `C:\dev\wezterm\term\src\terminalstate\performer.rs:942`, `C:\dev\wezterm\term\src\terminalstate\performer.rs:987` | Palette mutation model. |

Kepler implication: M1 should implement the essential subset explicitly and
record unsupported actions. Do not hide unsupported behavior behind no-ops.

## TerminalState

`TerminalState` is a dense state owner. It stores primary/alternate screen,
pen attributes, cursor, wrap/insert/origin modes, margins, application cursor
keys, modifyOtherKeys, Sixel state, color registers, keypad mode, bracketed
paste, mouse modes, keyboard encoding, charsets, titles, palette, clipboard,
handlers, current working directory, writer, image cache, user vars, Kitty
image state, sequence number, unicode version and ConPTY quirks
(`C:\dev\wezterm\term\src\terminalstate\mod.rs:247`,
`C:\dev\wezterm\term\src\terminalstate\mod.rs:250`,
`C:\dev\wezterm\term\src\terminalstate\mod.rs:253`,
`C:\dev\wezterm\term\src\terminalstate\mod.rs:322`,
`C:\dev\wezterm\term\src\terminalstate\mod.rs:334`,
`C:\dev\wezterm\term\src\terminalstate\mod.rs:347`,
`C:\dev\wezterm\term\src\terminalstate\mod.rs:370`).

It exposes sequence-number and screen accessors
(`C:\dev\wezterm\term\src\terminalstate\mod.rs:592`,
`C:\dev\wezterm\term\src\terminalstate\mod.rs:596`,
`C:\dev\wezterm\term\src\terminalstate\mod.rs:697`,
`C:\dev\wezterm\term\src\terminalstate\mod.rs:703`). Resize is state-owned:
it increments seqno, resizes both primary and alt screens, updates tabs and
marks lines dirty (`C:\dev\wezterm\term\src\terminalstate\mod.rs:852`,
`C:\dev\wezterm\term\src\terminalstate\mod.rs:876`,
`C:\dev\wezterm\term\src\terminalstate\mod.rs:888`,
`C:\dev\wezterm\term\src\terminalstate\mod.rs:928`).

Kepler implication: resist scattering state across parser, renderer and PTY.
One terminal state owner should coordinate modes, cursor, screens, metadata and
dirty tracking.

## Screen And Scrollback

`Screen` stores lines in a `VecDeque<Line>`, with visible rows at the bottom and
scrollback before them (`C:\dev\wezterm\term\src\screen.rs:15`,
`C:\dev\wezterm\term\src\screen.rs:24`). It also stores
`stable_row_index_offset`, physical rows and physical cols
(`C:\dev\wezterm\term\src\screen.rs:29`,
`C:\dev\wezterm\term\src\screen.rs:41`,
`C:\dev\wezterm\term\src\screen.rs:43`).

Resize is wrap-aware. `Screen::resize` calls rewrap paths when width changes,
resizes lines, adjusts capacity and handles ConPTY-specific behavior
(`C:\dev\wezterm\term\src\screen.rs:193`,
`C:\dev\wezterm\term\src\screen.rs:225`,
`C:\dev\wezterm\term\src\screen.rs:234`,
`C:\dev\wezterm\term\src\screen.rs:251`,
`C:\dev\wezterm\term\src\screen.rs:287`).

Stable row conversion is explicit:

| API | Source | Lesson |
|---|---|---|
| `stable_range` | `C:\dev\wezterm\term\src\screen.rs:493` | Convert stable row ranges to physical rows. |
| `phys_to_stable_row_index` | `C:\dev\wezterm\term\src\screen.rs:522` | Preserve identity as rows are purged. |
| `stable_row_to_phys` | `C:\dev\wezterm\term\src\screen.rs:527` | Map old stable rows back when still retained. |
| `visible_row_to_stable_row` | `C:\dev\wezterm\term\src\screen.rs:538` | Convert cursor/viewport rows to render ids. |
| `get_changed_stable_rows` | `C:\dev\wezterm\term\src\screen.rs:909` | Dirty-row query by stable ids and seqno. |

Kepler implication: adopt stable row ids early. This is the practical answer
to scrollback identity, remote render caches, selection stability and dirty
render ranges.

## Lines, Cells And Dirty Tracking

`wezterm-surface` gives a compact render model. `Line` stores a sequence number
(`C:\dev\wezterm\wezterm-surface\src\line\line.rs:46`,
`C:\dev\wezterm\wezterm-surface\src\line\line.rs:49`), supports resize, wrap,
dirty checks, visible-cell iteration, cell mutation, hyperlink scanning,
semantic zone ranges and clustering
(`C:\dev\wezterm\wezterm-surface\src\line\line.rs:206`,
`C:\dev\wezterm\wezterm-surface\src\line\line.rs:214`,
`C:\dev\wezterm\wezterm-surface\src\line\line.rs:283`,
`C:\dev\wezterm\wezterm-surface\src\line\line.rs:478`,
`C:\dev\wezterm\wezterm-surface\src\line\line.rs:531`,
`C:\dev\wezterm\wezterm-surface\src\line\line.rs:751`,
`C:\dev\wezterm\wezterm-surface\src\line\line.rs:1027`,
`C:\dev\wezterm\wezterm-surface\src\line\line.rs:1043`).

Kepler implication: store per-line change identity, not just a coarse "screen
dirty" boolean. Renderer and host APIs need to ask for changed rows without
rebuilding the full viewport every frame.

## Renderable Boundary

The mux exposes terminal state to renderers through a renderable adapter.
`RenderableDimensions` includes columns, viewport rows, scrollback rows,
physical top, scrollback top, dpi, pixel size and reverse-video flag
(`C:\dev\wezterm\mux\src\renderable.rs:25`,
`C:\dev\wezterm\mux\src\renderable.rs:36`,
`C:\dev\wezterm\mux\src\renderable.rs:41`). Cursor position is converted from
visible coordinates to stable coordinates (`C:\dev\wezterm\mux\src\renderable.rs:50`,
`C:\dev\wezterm\mux\src\renderable.rs:51`).

Key helpers:

| Helper | Source | Purpose |
|---|---|---|
| `terminal_get_dirty_lines` | `C:\dev\wezterm\mux\src\renderable.rs:62` | Query dirty stable rows. |
| `terminal_with_lines` | `C:\dev\wezterm\mux\src\renderable.rs:89` | Borrow lines by stable row range. |
| `terminal_get_lines` | `C:\dev\wezterm\mux\src\renderable.rs:113` | Copy lines for rendering or remote transfer. |
| `terminal_get_dimensions` | `C:\dev\wezterm\mux\src\renderable.rs:128` | Snapshot viewport and scrollback dimensions. |

Kepler implication: define `RenderSnapshot` or `RenderableTerminal` as a core
output boundary. Renderer code should consume stable rows, visible cells,
cursor and dimensions, not mutate terminal state.

## PTY Boundary

`portable-pty` is a strong reference for `terminal-pty`. The crate docs show
the intended flow: get the native PTY system, open a PTY with `PtySize`, spawn
a command, clone a reader and take a writer (`C:\dev\wezterm\pty\src\lib.rs:9`,
`C:\dev\wezterm\pty\src\lib.rs:16`, `C:\dev\wezterm\pty\src\lib.rs:29`,
`C:\dev\wezterm\pty\src\lib.rs:30`, `C:\dev\wezterm\pty\src\lib.rs:33`,
`C:\dev\wezterm\pty\src\lib.rs:36`).

The traits are exactly the right abstraction shape:

| Trait/API | Source | Kepler relevance |
|---|---|---|
| `PtySize` | `C:\dev\wezterm\pty\src\lib.rs:63` | Rows, cols, pixel width/height. |
| `MasterPty` | `C:\dev\wezterm\pty\src\lib.rs:88` | Resize, get size, reader, writer. |
| `Child` / `ChildKiller` | `C:\dev\wezterm\pty\src\lib.rs:130`, `C:\dev\wezterm\pty\src\lib.rs:150` | Child lifecycle and kill. |
| `SlavePty` | `C:\dev\wezterm\pty\src\lib.rs:163` | Spawn command. |
| `PtySystem` | `C:\dev\wezterm\pty\src\lib.rs:263` | Platform factory. |
| `native_pty_system` | `C:\dev\wezterm\pty\src\lib.rs:400` | Runtime platform selection. |

Unix uses `openpty` (`C:\dev\wezterm\pty\src\unix.rs:20`,
`C:\dev\wezterm\pty\src\unix.rs:36`,
`C:\dev\wezterm\pty\src\unix.rs:70`), while Windows uses ConPTY
(`C:\dev\wezterm\pty\src\win\conpty.rs:10`,
`C:\dev\wezterm\pty\src\win\conpty.rs:12`,
`C:\dev\wezterm\pty\src\win\conpty.rs:13`). The native type aliases select
Unix or ConPTY (`C:\dev\wezterm\pty\src\lib.rs:405`,
`C:\dev\wezterm\pty\src\lib.rs:407`).

Kepler implication: use `portable-pty` directly or model `terminal-pty` after
it. Do not build Windows PTY code in C# and do not let ConPTY details leak into
terminal-core.

## Mux And Runtime

The mux is where WezTerm becomes a product. `Mux` stores panes, tabs, windows,
domains and clients (`C:\dev\wezterm\mux\src\lib.rs:102`,
`C:\dev\wezterm\mux\src\lib.rs:103`,
`C:\dev\wezterm\mux\src\lib.rs:105`,
`C:\dev\wezterm\mux\src\lib.rs:106`,
`C:\dev\wezterm\mux\src\lib.rs:107`). `Pane` is the main boundary: it exposes
cursor position, lines, dimensions, title, reader, writer, resize and kill
(`C:\dev\wezterm\mux\src\pane.rs:167`,
`C:\dev\wezterm\mux\src\pane.rs:172`,
`C:\dev\wezterm\mux\src\pane.rs:200`,
`C:\dev\wezterm\mux\src\pane.rs:232`,
`C:\dev\wezterm\mux\src\pane.rs:234`,
`C:\dev\wezterm\mux\src\pane.rs:239`,
`C:\dev\wezterm\mux\src\pane.rs:240`,
`C:\dev\wezterm\mux\src\pane.rs:241`,
`C:\dev\wezterm\mux\src\pane.rs:253`).

`LocalDomain::spawn_pane` opens a PTY, spawns a command, creates a
`wezterm_term::Terminal`, enables ConPTY quirks when relevant and wraps it in a
`LocalPane` (`C:\dev\wezterm\mux\src\domain.rs:589`,
`C:\dev\wezterm\mux\src\domain.rs:590`,
`C:\dev\wezterm\mux\src\domain.rs:612`,
`C:\dev\wezterm\mux\src\domain.rs:618`,
`C:\dev\wezterm\mux\src\domain.rs:619`,
`C:\dev\wezterm\mux\src\domain.rs:622`,
`C:\dev\wezterm\mux\src\domain.rs:633`).

`LocalPane` implements `Pane` and bridges PTY/terminal/renderable behavior
(`C:\dev\wezterm\mux\src\localpane.rs:124`,
`C:\dev\wezterm\mux\src\localpane.rs:139`). Resize updates both PTY and
terminal (`C:\dev\wezterm\mux\src\localpane.rs:417`,
`C:\dev\wezterm\mux\src\localpane.rs:424`), while writer and reader come from
the PTY bridge (`C:\dev\wezterm\mux\src\localpane.rs:428`,
`C:\dev\wezterm\mux\src\localpane.rs:436`).

Kepler implication: M1 needs a smaller "session runtime" with one pane first.
Still, name the seams now: `Pane`, `Domain`, `Session`, `TerminalCore`,
`PtyHandle`, `RendererSnapshot`.

## PTY Read And Coalescing

WezTerm's PTY read path is valuable. A blocking read thread exists because
non-blocking reads are not portable across all PTY/TTY types
(`C:\dev\wezterm\mux\src\lib.rs:275`,
`C:\dev\wezterm\mux\src\lib.rs:276`,
`C:\dev\wezterm\mux\src\lib.rs:279`). It sends bytes through a socketpair to a
parser thread (`C:\dev\wezterm\mux\src\lib.rs:264`,
`C:\dev\wezterm\mux\src\lib.rs:295`,
`C:\dev\wezterm\mux\src\lib.rs:310`,
`C:\dev\wezterm\mux\src\lib.rs:320`).

`parse_buffered_data` parses bytes into actions, detects synchronized output,
holds actions during sync, coalesces data briefly, and then sends action batches
to the mux (`C:\dev\wezterm\mux\src\lib.rs:130`,
`C:\dev\wezterm\mux\src\lib.rs:137`,
`C:\dev\wezterm\mux\src\lib.rs:142`,
`C:\dev\wezterm\mux\src\lib.rs:154`,
`C:\dev\wezterm\mux\src\lib.rs:170`,
`C:\dev\wezterm\mux\src\lib.rs:180`,
`C:\dev\wezterm\mux\src\lib.rs:230`). `send_actions_to_mux` applies parsed
actions and sends `PaneOutput` notifications
(`C:\dev\wezterm\mux\src\lib.rs:121`,
`C:\dev\wezterm\mux\src\lib.rs:128`).

Kepler implication: do not parse and repaint on every tiny read if the app is
trying to draw a frame. A small coalescing window plus synchronized-output
handling should live in the runtime or render scheduler.

## Remote Render Cache

`wezterm-client` has a client-side renderable cache for remote panes.
`RenderableInner` stores a local line cache keyed by stable rows, dimensions,
seqno, polling state and RTT fields (`C:\dev\wezterm\wezterm-client\src\pane\renderable.rs:56`,
`C:\dev\wezterm\wezterm-client\src\pane\renderable.rs:65`,
`C:\dev\wezterm\wezterm-client\src\pane\renderable.rs:66`,
`C:\dev\wezterm\wezterm-client\src\pane\renderable.rs:68`,
`C:\dev\wezterm\wezterm-client\src\pane\renderable.rs:71`,
`C:\dev\wezterm\wezterm-client\src\pane\renderable.rs:77`). It uses an LRU
sized from scrollback lines (`C:\dev\wezterm\wezterm-client\src\pane\renderable.rs:100`)
and can compute dirty rows from deltas and cursor movement
(`C:\dev\wezterm\wezterm-client\src\pane\renderable.rs:319`,
`C:\dev\wezterm\wezterm-client\src\pane\renderable.rs:323`,
`C:\dev\wezterm\wezterm-client\src\pane\renderable.rs:377`).

Kepler implication: stable rows plus seqno unlock remote/headless rendering
later without redesigning the core. This is worth baking into M1 even if remote
mode is out of scope.

## Tests And Fixtures

WezTerm's tests are a compatibility map. Useful areas:

| Test area | Source | Kepler lesson |
|---|---|---|
| C0 controls | `C:\dev\wezterm\term\src\test\c0.rs:6`, `C:\dev\wezterm\term\src\test\c0.rs:17`, `C:\dev\wezterm\term\src\test\c0.rs:24`, `C:\dev\wezterm\term\src\test\c0.rs:40` | BS, LF, CR, tabs. |
| C1 controls | `C:\dev\wezterm\term\src\test\c1.rs:6`, `C:\dev\wezterm\term\src\test\c1.rs:19`, `C:\dev\wezterm\term\src\test\c1.rs:63` | IND, NEL, RI. |
| CSI behavior | `C:\dev\wezterm\term\src\test\csi.rs:216`, `C:\dev\wezterm\term\src\test\csi.rs:235`, `C:\dev\wezterm\term\src\test\csi.rs:253`, `C:\dev\wezterm\term\src\test\csi.rs:368`, `C:\dev\wezterm\term\src\test\csi.rs:397` | Cursor, repeat, insert/erase, scrollback erase. |
| Resize/reflow | `C:\dev\wezterm\term\src\test\mod.rs:767`, `C:\dev\wezterm\term\src\test\mod.rs:922`, `C:\dev\wezterm\term\src\test\mod.rs:1031` | Wrap-aware resizing is non-trivial. |
| Region scroll and alt screen | `C:\dev\wezterm\term\src\test\mod.rs:1190`, `C:\dev\wezterm\term\src\test\mod.rs:1236`, `C:\dev\wezterm\term\src\test\mod.rs:1283` | Dirty lines and scrollback correctness. |
| Hyperlinks | `C:\dev\wezterm\term\src\test\mod.rs:1298` | OSC 8 semantics. |
| Selection | `C:\dev\wezterm\term\src\test\selection.rs:69` | Wrapped selection behavior. |
| Images | `C:\dev\wezterm\term\src\test\image.rs:14`, `C:\dev\wezterm\term\src\test\image.rs:31` | Future Kitty/iTerm image support, not M1. |
| Parser edge cases | `C:\dev\wezterm\vtparse\src\lib.rs:770`, `C:\dev\wezterm\vtparse\src\lib.rs:808`, `C:\dev\wezterm\vtparse\src\lib.rs:819`, `C:\dev\wezterm\vtparse\src\lib.rs:842`, `C:\dev\wezterm\vtparse\src\lib.rs:915`, `C:\dev\wezterm\vtparse\src\lib.rs:950` | OSC, CSI, colons, parameter overflow. |

Kepler implication: use these as fixture inspiration after Alacritty reference
tests. The first Kepler test suite should cover C0/C1, CSI basics, resize,
alternate screen, dirty rows, hyperlinks and parser overflow.

## What Kepler Should Copy

Copy these ideas:

1. `Terminal { state, parser }` as a small core object.
2. `advance_bytes` as the byte-ingress API.
3. `perform_actions` or test-only action replay for deterministic tests.
4. Typed protocol actions between parser and terminal state.
5. Stable row identifiers separate from physical storage.
6. Per-line sequence numbers and dirty queries.
7. Renderable dimensions and line access as a read boundary.
8. `portable-pty`-style trait abstraction.
9. Runtime coalescing around synchronized output.
10. Domain/pane vocabulary for future sessions, but start with one local pane.

## What Kepler Should Avoid

Avoid these traps:

1. Copying WezTerm's entire mux before Kepler has one excellent local terminal.
2. Copying WezTerm's parser stack when `alacritty-vte` is simpler for M1.
3. Letting GUI/windowing dependencies enter `terminal-core`.
4. Treating Sixel, Kitty image, remote clients and Lua config as M1 scope.
5. Hiding unsupported OSC/CSI behind silent no-ops.
6. Assuming every WezTerm product concept belongs in Kepler's public API.
7. Building platform PTY code in C#, Objective-C or Zig when Rust already works.

## Recommended Kepler Shape

M1:

```text
terminal-protocol
  TerminalAction
  CsiAction
  OscAction
  Mode
  PrivateMode
  SgrAttribute

terminal-core
  Terminal
    state: TerminalState
    parser: VteAdapter
  Screen
  StableRowId
  SequenceNo
  RenderSnapshot

terminal-pty
  PtySystem
  MasterPty
  Child
  native platform adapters

terminal-runtime
  LocalPane
  read thread
  small coalescing window
  synchronized output handling
```

Later:

```text
terminal-mux
  Pane
  Tab
  Domain
  WindowSession
  remote transport
```

The sequencing matters. WezTerm proves the full destination, but Kepler should
not import the full destination before the core is correct.

## Bottom Line

Use WezTerm as the reference for Rust product architecture: terminal core crate,
PTY trait crate, stable row identity, seqno-based dirty lines, renderable
snapshot boundary and mux/runtime separation. Keep Kepler full Rust for these
zones. Use `alacritty-vte` for the first parser seed, Alacritty for grid/reflow
fixtures, Ghostty for embeddable render-state instincts, and WezTerm for the
long-term architecture map.

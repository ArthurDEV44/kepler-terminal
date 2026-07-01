# Ghostty Reference Inventory

Status: initial focused pass
Date: 2026-07-01
Reference source: `C:\dev\ghostty`
Scope: Ghostty architecture lessons for Kepler, not a full Ghostty clone audit.

## Executive Takeaways

Ghostty is the strongest reference for Kepler's engine boundary: a terminal core
owns VT state, exposes a renderer-neutral render state, and can be embedded
through a narrow API. The pieces Kepler should copy are the architecture shape,
not the language stack.

The high-value lessons:

1. Model the terminal engine as an embeddable object, not as a UI widget.
2. Separate byte ingestion, terminal state, render snapshots, PTY IO and native
   app runtime.
3. Treat render output as a pull-based snapshot with dirty flags, row iterators
   and cell iterators.
4. Plan page/chunk scrollback early: stable references, byte budgets and
   viewport pins matter once sessions become long.
5. Keep semantic prompt/session metadata as state attached to rows and cells,
   but never make it authoritative for terminal correctness.
6. Keep Kepler Rust-first. Ghostty's Zig/C/Swift/GTK architecture is a reference
   point, not a reason to make Kepler multi-language.

Confidence: high for core/render/scrollback findings, because they are grounded
in Ghostty source and public headers. Medium for app-layer conclusions, because
this pass sampled the macOS and GTK sides rather than auditing every app file.

## Why Ghostty Matters

Ghostty describes itself as a fast, native, feature-rich terminal emulator in
the README (`C:\dev\ghostty\README.md:8`). The important part for Kepler is not
only the app: Ghostty exposes `libghostty` for native GUI frontends or embedding
(`C:\dev\ghostty\README.md:10`) and documents `libghostty-vt` as a virtual
terminal library for parsing sequences, maintaining terminal state, scrollback,
line wrapping and reflow (`C:\dev\ghostty\include\ghostty\vt.h:17`).

Ghostty's README also makes a useful architectural split explicit: shared Zig
core plus platform-native app frontends. The macOS app uses SwiftUI, Metal and
CoreText, while the Linux app uses GTK and system integrations
(`C:\dev\ghostty\README.md:130`). For Kepler, that validates a split between
engine and host adapter. It does not validate writing Kepler's engine in Zig or
building separate app stacks inside the core.

## Codebase Map

| Area | Local path | What it contains | Kepler relevance |
|---|---|---|---|
| Public VT C API | `C:\dev\ghostty\include\ghostty\vt*.h` | C entrypoints, terminal config, render-state API, tracked refs. | Public API shape for future C ABI and host embedding. |
| Terminal core | `C:\dev\ghostty\src\terminal` | Terminal, screens, page list, parser stream, render state. | Main reference for `terminal-core` and `terminal-render-model`. |
| C API implementation | `C:\dev\ghostty\src\terminal\c` | Zig implementation behind the C headers. | Boundary pattern, not code to port. |
| Termio runtime | `C:\dev\ghostty\src\termio*` | PTY IO, mailbox, read/write threading, renderer wakeups. | Reference for future `terminal-pty` and runtime queues. |
| PTY layer | `C:\dev\ghostty\src\pty.zig`, `C:\dev\ghostty\src\pty.c` | POSIX PTY, Windows PseudoConsole and pipe handling. | Cross-platform PTY evidence, but Kepler should use Rust. |
| Renderer | `C:\dev\ghostty\src\renderer*` | Generic renderer, OpenGL/Metal implementations, render thread. | Host/render adapter lessons, not core dependency. |
| Native app runtime | `C:\dev\ghostty\src\apprt`, `C:\dev\ghostty\macos` | GTK, SwiftUI/AppKit, app callbacks, surface lifecycle. | Confirms native host layer should stay outside Kepler core. |
| Examples | `C:\dev\ghostty\example` | C, Zig, Swift and WASM embedding examples. | Model for Kepler executable docs and fixture harnesses. |

## Product And Public API Positioning

Ghostty has two layers that Kepler should keep mentally separate:

- The terminal app: native, platform-integrated, optimized for real terminal
  usage.
- The embeddable engine: `libghostty-vt`, documented as a C library for a modern
  terminal emulator (`C:\dev\ghostty\include\ghostty\vt.h:17`).

`libghostty-vt` is explicitly about escape sequence processing, terminal state,
input encoding, scrollback, line wrapping and reflow
(`C:\dev\ghostty\include\ghostty\vt.h:17`). It also exposes render-state and
formatter API groups (`C:\dev\ghostty\include\ghostty\vt.h:32`). The API is
marked incomplete and work-in-progress (`C:\dev\ghostty\include\ghostty\vt.h:10`),
which is a useful warning for Kepler: do not publish a broad stable API before
the state model, row identity and snapshot model are proven.

Kepler implication: the first public API should be narrow:

- `Terminal::new`
- `Terminal::resize`
- `Terminal::write_bytes`
- `Terminal::snapshot`
- `Terminal::render_state`
- `Terminal::scroll_viewport`
- stable row/cell handles

Do not expose internals until fixtures prove the semantics.

## Core Terminal Model

Ghostty's core `Terminal` is the primary terminal emulation structure. The file
comment says it stores a grid of characters, scrollback buffer, cursor and
terminal operations (`C:\dev\ghostty\src\terminal\Terminal.zig:1`). It imports
`ScreenSet` (`C:\dev\ghostty\src\terminal\Terminal.zig:34`) and stores screens
behind the terminal for primary/alternate behavior
(`C:\dev\ghostty\src\terminal\Terminal.zig:44`).

`ScreenSet` is explicitly the multiple-screen container for primary and
alternate screens (`C:\dev\ghostty\src\terminal\ScreenSet.zig:1`). It starts
with the primary screen and creates alternate screens when needed
(`C:\dev\ghostty\src\terminal\ScreenSet.zig:1`). `Terminal` has switch logic for
screens (`C:\dev\ghostty\src\terminal\Terminal.zig:2985`) and tests for primary
and alternate switching around the 1047/1048/1049 family
(`C:\dev\ghostty\src\terminal\Terminal.zig:12977`).

The `Screen` layer owns per-screen details: page list, cursor, saved cursor,
selection, Kitty keyboard/images and semantic prompt state
(`C:\dev\ghostty\src\terminal\Screen.zig:38`,
`C:\dev\ghostty\src\terminal\Screen.zig:48`,
`C:\dev\ghostty\src\terminal\Screen.zig:53`,
`C:\dev\ghostty\src\terminal\Screen.zig:69`).

Kepler implication:

- `terminal-core` should own `Terminal`, `ScreenSet`, `Screen`, cursor, modes,
  tabs, charsets and scrollback.
- Primary and alternate screens must be separate states, not a viewport flag.
- Alternate screen resize and scrollback behavior need dedicated fixtures.
- Selection should use tracked positions, not naked row indexes.

## Parser And Stream Boundary

Ghostty has its own VT parser. `Parser.zig` says it is a VT-series parser for
escape and control sequences (`C:\dev\ghostty\src\terminal\Parser.zig:1`) and
references the vt100.net DEC ANSI parser (`C:\dev\ghostty\src\terminal\Parser.zig:4`).
Parser output is expressed as actions expected to be performed by the caller
(`C:\dev\ghostty\src\terminal\Parser.zig:49`).

The stream layer then connects parser output to terminal mutation.
`stream.zig` documents a type that processes a stream of TTY controls
(`C:\dev\ghostty\src\terminal\stream.zig:405`), handles partial UTF-8 and escape
sequences (`C:\dev\ghostty\src\terminal\stream.zig:503`), and has a heap or
allocation-free setup path (`C:\dev\ghostty\src\terminal\stream.zig:437`).
`stream_terminal.zig` applies those actions to `Terminal`
(`C:\dev\ghostty\src\terminal\stream_terminal.zig:19`).

The C API has a similar external boundary: `ghostty_terminal_vt_write` updates
terminal state and can call effect callbacks such as writing back to the PTY
(`C:\dev\ghostty\include\ghostty\vt\terminal.h:1096`). The callback section
warns that callbacks run during `ghostty_terminal_vt_write`, must not reenter
`vt_write`, and must not block (`C:\dev\ghostty\include\ghostty\vt\terminal.h:45`).

Kepler implication:

- `alacritty/vte` can be used as parser seed, but Kepler must own the action
  layer and terminal semantics.
- The parser boundary should be internal. Public APIs should talk in Kepler
  events, snapshots and render state, not `vte::Perform`.
- Callback-based host effects are dangerous for reentrancy and latency. Prefer
  explicit effect queues for Kepler.

## Scrollback And Page Storage

Ghostty's `PageList` is the most important scrollback reference in this pass.
It is a linked list of pages for the screen (`C:\dev\ghostty\src\terminal\PageList.zig:1`).
The first page is the top of scrollback and the last page is the bottom active
page (`C:\dev\ghostty\src\terminal\PageList.zig:37`).

The data structure tracks pins and references. It says tracked pins are
automatically updated (`C:\dev\ghostty\src\terminal\PageList.zig:70`), uses page
serials/minimum serials to detect stale refs
(`C:\dev\ghostty\src\terminal\PageList.zig:135`), and maintains viewport pin
state for performance (`C:\dev\ghostty\src\terminal\PageList.zig:170`). It also
has a byte-budget concept: `explicit_max_size` only applies to scrollback pages,
while active pages must remain available (`C:\dev\ghostty\src\terminal\PageList.zig:147`).

Resize and reflow are integrated into the page list. `PageList.resize` has a
reflow option (`C:\dev\ghostty\src\terminal\PageList.zig:932`), reflow behavior
tracks columns, rows and active viewport fixes
(`C:\dev\ghostty\src\terminal\PageList.zig:980`), and tracked pins are adjusted
during reflow (`C:\dev\ghostty\src\terminal\PageList.zig:1279`). Tests cover
stale refs, tracked pins after reset, unwrap viewport pins and resize/remap
paths (`C:\dev\ghostty\src\terminal\PageList.zig:13638`,
`C:\dev\ghostty\src\terminal\PageList.zig:13686`,
`C:\dev\ghostty\src\terminal\PageList.zig:13812`,
`C:\dev\ghostty\src\terminal\PageList.zig:14009`).

Kepler implication:

- M1 may start simpler, but the public model should already assume stable row
  handles and generations.
- Memory policy should be hybrid: max lines plus max bytes.
- Scrollback internals should move toward chunks/pages, not an unbounded `Vec`.
- Viewport and semantic indexes should point to stable handles, not row numbers.
- Reflow tests must include tracked handles, selection and semantic markers.

## Render State Boundary

Ghostty's render state is almost exactly the boundary Kepler needs. The public
header says a render state is required to render the visible viewport, receives
updates from one terminal and exposes only dirty regions
(`C:\dev\ghostty\include\ghostty\vt\render.h:24`). Basic usage is create render
state, update it from terminal, then read from it
(`C:\dev\ghostty\include\ghostty\vt\render.h:36`).

Dirty tracking exists at two levels: global render-state dirty and per-row dirty
(`C:\dev\ghostty\include\ghostty\vt\render.h:44`). The API exposes viewport size,
cursor visibility and viewport position, row iteration, row cells and selection
(`C:\dev\ghostty\include\ghostty\vt\render.h:130`,
`C:\dev\ghostty\include\ghostty\vt\render.h:175`,
`C:\dev\ghostty\include\ghostty\vt\render.h:442`,
`C:\dev\ghostty\include\ghostty\vt\render.h:557`,
`C:\dev\ghostty\include\ghostty\vt\render.h:213`). Grapheme clusters can be
encoded as UTF-8 (`C:\dev\ghostty\include\ghostty\vt\render.h:619`).

The implementation explains why render state lives under `src/terminal`, not
`src/renderer`: it is generic to multiple renderers and `libghostty-vt`
(`C:\dev\ghostty\src\terminal\render.zig:20`). It is state required to render a
screen and is reused across render calls while retaining dirty-region knowledge
(`C:\dev\ghostty\src\terminal\render.zig:25`). Updating render state consumes
terminal/screen dirty state (`C:\dev\ghostty\src\terminal\render.zig:261`) and
handles full redraw triggers when terminal dirty state, screen dirty state or
viewport pins change (`C:\dev\ghostty\src\terminal\render.zig:275`).

The example `c-vt-render` turns this into executable documentation: it creates a
terminal and render state, writes VT content, updates the render state, checks
dirty flags, reads colors/cursor, iterates rows and cells, and resets dirty
state (`C:\dev\ghostty\example\c-vt-render\src\main.c:24`,
`C:\dev\ghostty\example\c-vt-render\src\main.c:75`,
`C:\dev\ghostty\example\c-vt-render\src\main.c:157`,
`C:\dev\ghostty\example\c-vt-render\src\main.c:257`).

Kepler implication:

- `terminal-render-model` should be a first-class crate.
- Render output should be pull-based: update state, inspect dirty region, then
  iterate rows and cells.
- Dirty state must be explicit and resettable by the renderer/host.
- The render model should carry cursor, selection, styles, hyperlinks, graphemes
  and image placeholders.
- GPUI must consume this model. `terminal-core` must not import GPUI.

## PTY And Runtime Threading

Ghostty's termio layer is a separate runtime layer for terminal IO. The module
comment says termio reads and writes bytes for a PTY or PTY-like device
(`C:\dev\ghostty\src\termio.zig:1`) and is built from Termio, Backend and
Mailbox components (`C:\dev\ghostty\src\termio.zig:4`). It explicitly supports
multi-threaded operation, with reads and writes happening on separate threads to
improve throughput and latency under heavy IO load
(`C:\dev\ghostty\src\termio.zig:14`).

`Termio.zig` is renderer-aware but still separate from renderer implementation:
it holds `renderer_state`, `renderer_wakeup`, `renderer_mailbox` and
`surface_mailbox` (`C:\dev\ghostty\src\termio\Termio.zig:44`,
`C:\dev\ghostty\src\termio\Termio.zig:48`,
`C:\dev\ghostty\src\termio\Termio.zig:51`,
`C:\dev\ghostty\src\termio\Termio.zig:54`). `processOutput` can be called
manually with PTY data or by the read thread
(`C:\dev\ghostty\src\termio\Termio.zig:640`), locks renderer state while
processing (`C:\dev\ghostty\src\termio\Termio.zig:646`), and wakes renderer or
mailbox paths afterward (`C:\dev\ghostty\src\termio\Termio.zig:668`,
`C:\dev\ghostty\src\termio\Termio.zig:695`).

The writer-thread layer says the reader side is handled by Termio/backends, and
the writer thread handles PTY writes plus events such as synchronized output and
linefeed mode, offloading the hot read/parser path
(`C:\dev\ghostty\src\termio\Thread.zig:1`). Resize messages are coalesced
(`C:\dev\ghostty\src\termio\Thread.zig:28`) and mailbox wakeups drive work
(`C:\dev\ghostty\src\termio\Thread.zig:451`).

The exec backend starts subprocesses with PTYs and spins a read thread
(`C:\dev\ghostty\src\termio\Exec.zig:1`). It creates a read thread with POSIX or
Windows entrypoints (`C:\dev\ghostty\src\termio\Exec.zig:139`) and forwards
read data into `Termio.processOutput`
(`C:\dev\ghostty\src\termio\Exec.zig:1338`,
`C:\dev\ghostty\src\termio\Exec.zig:1390`).

Kepler implication:

- `terminal-core` should stay headless and synchronous enough for tests.
- `terminal-pty` should be separate, trait-based and runtime-owned.
- PTY reads should feed byte batches into core; writes/effects should be queued.
- Resize coalescing and backpressure belong in `terminal-pty` or host runtime,
  not in terminal state logic.

## Cross-Platform PTY Handling

Ghostty's PTY layer switches by target OS. `Pty` maps Windows to `WindowsPty`,
iOS to `NullPty`, and other targets to `PosixPty`
(`C:\dev\ghostty\src\pty.zig:19`). POSIX uses `openpty` through C includes
(`C:\dev\ghostty\src\pty.c:4`, `C:\dev\ghostty\src\pty.c:11`,
`C:\dev\ghostty\src\pty.c:22`) and opens master/slave file descriptors in
`PosixPty.open` (`C:\dev\ghostty\src\pty.zig:134`). Windows builds pipes and a
PseudoConsole, then calls `CreatePseudoConsole`
(`C:\dev\ghostty\src\pty.zig:325`, `C:\dev\ghostty\src\pty.zig:370`,
`C:\dev\ghostty\src\pty.zig:430`).

Kepler implication:

- Cross-platform PTY does not require C#, Objective-C, Swift, Zig or C++.
- Rust can model the same split with `cfg(target_os = "windows")` and Unix
  modules.
- Windows ConPTY quirks belong behind a Rust trait implementation.
- POSIX PTY code should be isolated behind Unix-specific Rust modules using
  `rustix`, `nix`, `libc` or direct bindings.

## Renderer And Native App Boundary

Ghostty's renderer module says the renderer turns internal screen state into an
output format and is closely tied to the windowing system, which prepares
system-specific API state such as an OpenGL context or Vulkan surface
(`C:\dev\ghostty\src\renderer.zig:1`, `C:\dev\ghostty\src\renderer.zig:5`).
The renderer module selects Metal and OpenGL implementations
(`C:\dev\ghostty\src\renderer.zig:18`, `C:\dev\ghostty\src\renderer.zig:19`).

Renderer threading is explicit. `renderer/Thread.zig` represents renderer
thread logic and can be woken to render (`C:\dev\ghostty\src\renderer\Thread.zig:1`).
It has a bounded mailbox (`C:\dev\ghostty\src\renderer\Thread.zig:35`), a wakeup
handle (`C:\dev\ghostty\src\renderer\Thread.zig:47`), and calls renderer
thread-enter/exit hooks because some APIs, such as OpenGL, need per-thread setup
(`C:\dev\ghostty\src\renderer\Thread.zig:233`). The generic renderer owns a
terminal render state (`C:\dev\ghostty\src\renderer\generic.zig:222`), updates
frame data (`C:\dev\ghostty\src\renderer\generic.zig:1122`) and updates the
terminal render state per frame (`C:\dev\ghostty\src\renderer\generic.zig:1205`).

The macOS side is SwiftUI/AppKit-facing. `Ghostty.App.swift` imports SwiftUI
(`C:\dev\ghostty\macos\Sources\Ghostty\Ghostty.App.swift:1`), creates runtime
callbacks into the core (`C:\dev\ghostty\macos\Sources\Ghostty\Ghostty.App.swift:58`),
and constructs the core app with `ghostty_app_new`
(`C:\dev\ghostty\macos\Sources\Ghostty\Ghostty.App.swift:73`). The GTK side
embeds surfaces and GTK widgets, including a scrolled-window wrapper around a
surface (`C:\dev\ghostty\src\apprt\gtk\class\surface_scrolled_window.zig:14`).

Kepler implication:

- Native app/runtime code should be host adapters, not terminal engine code.
- A future GPUI adapter should consume `terminal-render-model` and own all GPUI
  details.
- macOS-specific framework integration can be Rust plus `objc2` later if
  needed, but not part of M1 core.
- Linux native app details should not leak into core by way of GTK types.

## Semantic Prompt And Agent-Aware State

Ghostty has semantic prompt state inside screen/terminal structures. `Screen`
stores OSC 133 prompt state (`C:\dev\ghostty\src\terminal\Screen.zig:69`), tracks
semantic prompt optimized state (`C:\dev\ghostty\src\terminal\Screen.zig:96`),
and applies semantic content to newly written cells
(`C:\dev\ghostty\src\terminal\Screen.zig:168`). Tests cover semantic prompt and
prompt cursor behavior (`C:\dev\ghostty\src\terminal\Terminal.zig:12097`,
`C:\dev\ghostty\src\terminal\Terminal.zig:12557`) and selection line behavior
at semantic prompt boundaries (`C:\dev\ghostty\src\terminal\Screen.zig:8002`).

Kepler implication:

- Semantic metadata is worth modeling early because Paneflow/agent workflows
  need command blocks, prompts and outputs.
- It should be stored as optional metadata on stable row/cell/session handles.
- Terminal correctness must not depend on semantic metadata being present.
- OSC 133 should start as a protocol event plus sidecar index, not as rendering
  behavior.

## Examples As Executable Documentation

Ghostty's examples directory is structured around language/API surfaces. The
README says `c-*` directories use the C API and `zig-*` directories use the Zig
API (`C:\dev\ghostty\example\README.md:4`). Even C examples use Zig's build
system, but not Zig as their language (`C:\dev\ghostty\example\README.md:9`).

The examples are especially useful for Kepler because they describe how an
external host should drive the engine:

- `c-vt-stream` creates a terminal, writes VT sequences with
  `ghostty_terminal_vt_write`, then formats output
  (`C:\dev\ghostty\example\c-vt-stream\src\main.c:15`,
  `C:\dev\ghostty\example\c-vt-stream\src\main.c:22`,
  `C:\dev\ghostty\example\c-vt-stream\src\main.c:57`).
- `c-vt-render` demonstrates render-state update and row/cell iteration
  (`C:\dev\ghostty\example\c-vt-render\README.md:3`).
- `swift-vt-xcframework` demonstrates consuming `libghostty-vt` from a Swift
  package (`C:\dev\ghostty\example\swift-vt-xcframework\README.md:3`).

Kepler implication:

- Build examples should be kept close to each public API.
- A debug CLI should act like the C examples: create terminal, write bytes,
  update render state, print rows/cells.
- Future FFI examples can wait until Rust core APIs stabilize.

## Tests, Fuzzing And Compatibility Clues

Ghostty's tests are embedded across terminal files. The most useful categories
for Kepler:

- Terminal full reset and alternate screen behavior
  (`C:\dev\ghostty\src\terminal\Terminal.zig:12649`,
  `C:\dev\ghostty\src\terminal\Terminal.zig:12977`).
- Resize with reflow and saved cursor
  (`C:\dev\ghostty\src\terminal\Terminal.zig:12805`).
- PageList reflow, tracked pins and stale reference behavior
  (`C:\dev\ghostty\src\terminal\PageList.zig:11236`,
  `C:\dev\ghostty\src\terminal\PageList.zig:13638`,
  `C:\dev\ghostty\src\terminal\PageList.zig:13686`).
- Render dirty state tests (`C:\dev\ghostty\src\terminal\render.zig:1105`).
- Fuzzer-discovered terminal crash regression
  (`C:\dev\ghostty\src\terminal\Terminal.zig:13200`).

Kepler implication:

- Do not wait for UI to test terminal correctness.
- Build fixture families for page storage, resize, alternate screen and render
  dirty state before adding complex PTY/runtime features.
- Fuzz byte ingestion early once parser/action handling exists.

## Language Decision For Kepler

Ghostty proves that a serious terminal can be cross-platform with a shared core
and native host layers. It does not prove that Kepler should use Ghostty's
languages.

For Kepler:

| Zone | Ghostty reference | Kepler language decision |
|---|---|---|
| Terminal state core | Zig `src/terminal` | Rust only. |
| Public embedding API | C headers over Zig core | Rust public API first, optional C ABI later. |
| VT parser | Zig parser/stream | Rust wrapper around parser boundary, likely `alacritty/vte` first. |
| Scrollback/page storage | Zig PageList | Rust chunks/pages with stable handles. |
| Render state | Zig core plus C API | Rust `terminal-render-model`. |
| PTY | Zig plus small C includes for POSIX, Windows APIs for ConPTY | Rust trait impls with `windows-rs` and Unix bindings. |
| macOS app | SwiftUI/AppKit plus Metal/CoreText | Future host adapter only, likely GPUI first for Paneflow. |
| Linux app | GTK plus OpenGL | Future host adapter only, no GTK in core. |
| Examples | C, Zig, Swift, WASM examples | Rust examples first, C ABI examples only after API maturity. |

Decision: Ghostty strengthens the Rust-first plan. The right extraction is
conceptual: engine boundary, page storage, render-state API and runtime
separation. The wrong extraction would be importing Zig/Swift/GTK language debt
into Kepler.

## What Kepler Should Copy

Copy the architectural ideas:

- Embeddable terminal core with narrow public API.
- Primary and alternate screens as separate state.
- Page/chunk storage for long scrollback.
- Tracked row/grid refs with generation/stale detection.
- Render state as pull snapshot with dirty flags.
- Row/cell iterators and explicit dirty reset.
- PTY/runtime layer outside the core.
- Examples that act as executable docs.
- Semantic prompt metadata as optional state, not correctness logic.

## What Kepler Should Avoid

Avoid these traps:

- Do not make the core depend on GPUI, GTK, SwiftUI, Metal or OpenGL.
- Do not expose a broad stable API before row identity and snapshots are proven.
- Do not implement host callbacks that can block or reenter byte ingestion.
- Do not make ConPTY or POSIX PTY behavior shape `terminal-core`.
- Do not start with image rendering. Preserve metadata first.
- Do not store scrollback as an unbounded line vector.
- Do not treat semantic prompt detection as authoritative.

## Follow-Up Fixtures For Kepler

Ghostty-inspired fixture backlog:

1. Primary/alternate screen switches: 47, 1047, 1048, 1049.
2. Resize narrower/wider with wrap markers and saved cursor.
3. Page/chunk scrollback trim with stable row handles.
4. Viewport pin survives scrollback growth and prune.
5. Selection survives resize and scrollback movement.
6. Render-state full dirty vs partial dirty.
7. Row dirty reset after render.
8. Grapheme cluster iteration and UTF-8 export.
9. OSC 133 prompt/input/output metadata.
10. Full reset invalidates or remaps tracked refs as designed.
11. Windows ConPTY replay feeding raw bytes into the same core.
12. Long-session memory cap: 100k lines and 1M logical lines.

## Open Questions For Later Passes

Open points not fully audited in this first Ghostty file:

- Exact memory profile of PageList under very large scrollback.
- Detailed behavior of Kitty image state under resize/reflow.
- Full GTK surface lifecycle and renderer shutdown paths.
- Full macOS CoreText/Metal text shaping pipeline.
- Benchmark methodology for parser, render state and GPU frame update.

These are not blockers for Kepler M1. The M1 extraction is already clear:
headless Rust core, stable render model, page-aware storage design, PTY outside
core, semantic sidecar.

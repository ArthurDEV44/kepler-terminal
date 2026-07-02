# Rio Reference Inventory

Status: initial focused pass
Date: 2026-07-01
Reference source: `C:\dev\rio`
Scope: Rio architecture lessons for Hera, especially Rust workspace shape,
terminal backend, parser/runtime loop, render snapshots, Sugarloaf renderer and
cross-platform host boundaries.

## Executive Takeaways

Rio is valuable for Hera, but not as the primary terminal-core seed.
Alacritty remains the cleaner M1 implementation reference for state, grid and
reflow. Rio's strongest signal is the bridge from terminal state to modern
rendering: dirty rows, visible snapshots, style and extras tables, batched grid
emission, and a renderer crate with Metal, Vulkan, WGPU and CPU backends.

The high-value lessons:

1. Stay Rust-first. Rio is a Rust workspace with backend, PTY, window,
   renderer and app crates.
2. Treat Rio's terminal model as an Alacritty-derived variant, not a fresh
   semantic authority. Its grid module explicitly says it was originally taken
   from Alacritty.
3. Copy the boundary idea, not the product surface: terminal backend mutates
   `Crosswords`, renderer pulls `snapshot_visible`, and app code emits rows
   into Sugarloaf grid renderers.
4. Keep damage as core output. Rio has `TerminalDamage`, `Row::dirty`,
   `damage_event_in_flight` and event-driven `TerminalDamaged` wakeups.
5. Borrow synchronized update policy. Rio buffers DECSET 2026 synchronized
   updates with a timeout and byte cap, then emits damage.
6. Use Sugarloaf as a renderer reference. Its context layer selects WGPU,
   Metal, Vulkan or CPU, while font fallback stays platform-aware.
7. Do not import graphics-protocol complexity into Hera M1. Rio's Kitty,
   iTerm2 and Sixel work is useful backlog evidence, not a first milestone.

Confidence: high for workspace, terminal backend, parser/runtime, PTY and
renderer-boundary findings because they are grounded in local source under
`C:\dev\rio`. Medium for full Sugarloaf conclusions because this pass sampled
the renderer and context paths rather than auditing every shader and backend
resource lifetime.

## Why Rio Matters

Rio positions itself as a modern terminal built to run everywhere
(`C:\dev\rio\README.md:8`) and currently states an MSRV of 1.96.0
(`C:\dev\rio\README.md:48`). The root workspace includes `sugarloaf`,
`teletypewriter`, `rio-backend`, `rio-window` and `frontends/rioterm`
(`C:\dev\rio\Cargo.toml:1`, `C:\dev\rio\Cargo.toml:2`,
`C:\dev\rio\Cargo.toml:3`, `C:\dev\rio\Cargo.toml:4`,
`C:\dev\rio\Cargo.toml:6`, `C:\dev\rio\Cargo.toml:8`,
`C:\dev\rio\Cargo.toml:10`). The workspace is Rust 2021, versioned at 0.4.8,
and centralizes workspace dependencies including WGPU and Objective-C runtime
bindings (`C:\dev\rio\Cargo.toml:15`, `C:\dev\rio\Cargo.toml:17`,
`C:\dev\rio\Cargo.toml:31`, `C:\dev\rio\Cargo.toml:32`,
`C:\dev\rio\Cargo.toml:33`, `C:\dev\rio\Cargo.toml:36`,
`C:\dev\rio\Cargo.toml:64`, `C:\dev\rio\Cargo.toml:78`).

Hera implication: Rio validates a Rust product split with native platform
APIs at crate boundaries. It does not justify C#, Objective-C source, Zig or
C++ in the terminal engine.

## Codebase Map

| Area | Local path | What it contains | Hera relevance |
|---|---|---|---|
| Backend crate | `C:\dev\rio\rio-backend` | Terminal state, grid, parser handler, events, graphics protocols. | Useful for snapshot and damage design, secondary for core semantics. |
| Grid and state | `C:\dev\rio\rio-backend\src\crosswords` | `Crosswords`, active/inactive grids, reflow, style/extras tables. | Alacritty-derived M1 storage and damage reference. |
| Parser/runtime | `C:\dev\rio\rio-backend\src\performer` | Raw VTE parser, handler trait, synchronized updates, PTY loop. | Runtime throttling and parser boundary reference. |
| PTY crate | `C:\dev\rio\teletypewriter` | Unix PTY and Windows ConPTY wrapper. | Secondary PTY reference after WezTerm `portable-pty`. |
| Renderer crate | `C:\dev\rio\sugarloaf` | Sugarloaf text/grid renderer, context backends, font fallback. | Best Rio lesson for Hera renderer model. |
| Window crate | `C:\dev\rio\rio-window` | Winit fork, X11, Wayland, macOS, Windows and WASM platform code. | Host boundary reference, not terminal core. |
| App frontend | `C:\dev\rio\frontends\rioterm` | Window/app loop, event routing, context manager, screen renderer. | Shows how snapshots feed renderer state. |
| Graphics protocols | `C:\dev\rio\rio-backend\src\ansi` | Kitty graphics, Sixel, iTerm2 images, glyph protocol. | Backlog and fixture source, not M1 rendering. |
| Fixtures/tests | `C:\dev\rio\rio-backend\tests\sixel`, grid tests | Reflow tests, Sixel image fixtures, protocol unit tests. | Useful fixture patterns for later compatibility. |

## Language And Platform Decision

Rio supports the existing Hera decision: Rust stays the implementation
language across Windows, Linux and macOS. Native APIs appear through Rust crates
and target-specific modules.

| Hera zone | Language | Rio evidence | Decision |
|---|---|---|---|
| Terminal state core | Rust | `rio-backend` is a Rust crate with `cdylib` and `rlib` outputs (`C:\dev\rio\rio-backend\Cargo.toml:2`, `C:\dev\rio\rio-backend\Cargo.toml:14`). | Keep Hera core Rust-only. |
| Parser adapter | Rust | Rio owns a Rust parser wrapper and handler trait (`C:\dev\rio\rio-backend\src\performer\parser\mod.rs:32`, `C:\dev\rio\rio-backend\src\performer\handler.rs:91`). | Use Rust parser boundary. |
| PTY adapter | Rust plus OS FFI | `teletypewriter` describes itself as a Rust PTY crate and uses Unix or Windows modules (`C:\dev\rio\teletypewriter\Cargo.toml:2`, `C:\dev\rio\teletypewriter\Cargo.toml:3`, `C:\dev\rio\teletypewriter\src\lib.rs:3`, `C:\dev\rio\teletypewriter\src\lib.rs:8`). | Use Rust traits plus platform impls. |
| Renderer model | Rust | Sugarloaf is a Rust rendering crate with desktop and WASM intent (`C:\dev\rio\sugarloaf\Cargo.toml:2`, `C:\dev\rio\sugarloaf\Cargo.toml:22`). | Keep snapshots Rust-owned. |
| Native renderer | Rust plus GPU APIs | Sugarloaf selects WGPU, Metal, Vulkan or CPU in Rust (`C:\dev\rio\sugarloaf\src\context\mod.rs:17`, `C:\dev\rio\sugarloaf\src\context\mod.rs:39`). | Adapters may use native APIs, not core. |
| Window host | Rust plus platform crates | `rio-window` is a Winit fork with X11 and Wayland features (`C:\dev\rio\rio-window\Cargo.toml:14`, `C:\dev\rio\rio-window\Cargo.toml:17`). | Host crate only. |
| Apple integration | Rust plus Objective-C runtime crates | Rio depends on `objc`/`objc2`, but in Rust manifests (`C:\dev\rio\Cargo.toml:78`, `C:\dev\rio\rio-window\Cargo.toml:60`). | No Objective-C source unless Rust bindings fail. |

The cross-platform answer stays strict: Hera should not add C#, Objective-C,
Swift, Zig, C++ or C to the terminal engine. Thin FFI can exist only outside
`terminal-core`, behind platform-specific adapters or generated C ABI surfaces.

## Backend Boundary

`rio-backend` describes itself as backend infrastructure for Rio terminal and
exports both `cdylib` and `rlib` crate types
(`C:\dev\rio\rio-backend\Cargo.toml:10`,
`C:\dev\rio\rio-backend\Cargo.toml:14`). It depends on `sugarloaf`,
`rio-grapheme-width`, `teletypewriter`, `rio-window` and `simdutf`
(`C:\dev\rio\rio-backend\Cargo.toml:36`,
`C:\dev\rio\rio-backend\Cargo.toml:37`,
`C:\dev\rio\rio-backend\Cargo.toml:38`,
`C:\dev\rio\rio-backend\Cargo.toml:46`,
`C:\dev\rio\rio-backend\Cargo.toml:47`). Default features enable Wayland and
X11, while WGPU is optional and forwards into Sugarloaf
(`C:\dev\rio\rio-backend\Cargo.toml:55`,
`C:\dev\rio\rio-backend\Cargo.toml:65`).

Hera implication: Rio's backend is more app-coupled than Hera's
`terminal-core` should be. Keep Hera's core free of renderer, window and
clipboard dependencies. Use Rio to study the snapshot boundary, not to copy the
crate dependency graph.

## Terminal State And Grid

The central state type is `Crosswords<U>` (`C:\dev\rio\rio-backend\src\crosswords\mod.rs:423`).
It owns an inactive grid for alternate/primary screen swapping
(`C:\dev\rio\rio-backend\src\crosswords\mod.rs:432`) and tracks whether a
damage event is already in flight (`C:\dev\rio\rio-backend\src\crosswords\mod.rs:460`).
It exposes full damage marking, damage extraction, display offset, resize,
visible snapshots and alternate-screen swapping
(`C:\dev\rio\rio-backend\src\crosswords\mod.rs:524`,
`C:\dev\rio\rio-backend\src\crosswords\mod.rs:562`,
`C:\dev\rio\rio-backend\src\crosswords\mod.rs:612`,
`C:\dev\rio\rio-backend\src\crosswords\mod.rs:687`,
`C:\dev\rio\rio-backend\src\crosswords\mod.rs:1331`,
`C:\dev\rio\rio-backend\src\crosswords\mod.rs:1505`).

The grid module explicitly says it was originally taken from Alacritty
(`C:\dev\rio\rio-backend\src\crosswords\grid\mod.rs:1`). `Grid<T>` stores
cursor, saved cursor, raw storage, columns, visible lines, display offset and
max scroll limit (`C:\dev\rio\rio-backend\src\crosswords\grid\mod.rs:35`,
`C:\dev\rio\rio-backend\src\crosswords\grid\mod.rs:36`,
`C:\dev\rio\rio-backend\src\crosswords\grid\mod.rs:40`,
`C:\dev\rio\rio-backend\src\crosswords\grid\mod.rs:44`,
`C:\dev\rio\rio-backend\src\crosswords\grid\mod.rs:47`,
`C:\dev\rio\rio-backend\src\crosswords\grid\mod.rs:50`,
`C:\dev\rio\rio-backend\src\crosswords\grid\mod.rs:57`,
`C:\dev\rio\rio-backend\src\crosswords\grid\mod.rs:60`). Rio adds a `style_set`
and `extras_table` directly on the grid (`C:\dev\rio\rio-backend\src\crosswords\grid\mod.rs:66`,
`C:\dev\rio\rio-backend\src\crosswords\grid\mod.rs:70`).

Hera implication:

- Use Alacritty as canonical M1 grid reference.
- Use Rio to study style interning and extras indirection.
- Preserve alternate screen as a real inactive grid/screen, not a viewport flag.
- Keep `display_offset`, history size and dirty state observable through
  snapshots rather than renderer callbacks.

## Resize And Reflow

Rio's resize code follows the Alacritty shape: `Grid::resize` accepts `reflow`,
new line count and new column count, then grows or shrinks lines and columns
(`C:\dev\rio\rio-backend\src\crosswords\grid\resize.rs:14`,
`C:\dev\rio\rio-backend\src\crosswords\grid\resize.rs:19`,
`C:\dev\rio\rio-backend\src\crosswords\grid\resize.rs:20`,
`C:\dev\rio\rio-backend\src\crosswords\grid\resize.rs:25`,
`C:\dev\rio\rio-backend\src\crosswords\grid\resize.rs:26`). Column growth and
shrink paths both explicitly handle reflow, wrap flags, wide-char spacers,
cursor movement and display offset clamping
(`C:\dev\rio\rio-backend\src\crosswords\grid\resize.rs:90`,
`C:\dev\rio\rio-backend\src\crosswords\grid\resize.rs:103`,
`C:\dev\rio\rio-backend\src\crosswords\grid\resize.rs:126`,
`C:\dev\rio\rio-backend\src\crosswords\grid\resize.rs:159`,
`C:\dev\rio\rio-backend\src\crosswords\grid\resize.rs:231`,
`C:\dev\rio\rio-backend\src\crosswords\grid\resize.rs:234`,
`C:\dev\rio\rio-backend\src\crosswords\grid\resize.rs:238`,
`C:\dev\rio\rio-backend\src\crosswords\grid\resize.rs:283`,
`C:\dev\rio\rio-backend\src\crosswords\grid\resize.rs:367`,
`C:\dev\rio\rio-backend\src\crosswords\grid\resize.rs:369`).

Hera implication: reflow remains a fixture-first feature. Rio gives useful
edge-case coverage around wide chars and cursor movement, but the authoritative
first path should still be Alacritty plus local golden fixtures.

## Parser And Synchronized Updates

Rio's parser module states the boundary clearly: it is a parser for raw VTE
protocol that delegates actions to `Perform`
(`C:\dev\rio\rio-backend\src\performer\parser\mod.rs:32`). It keeps caps for
intermediates and OSC params (`C:\dev\rio\rio-backend\src\performer\parser\mod.rs:23`,
`C:\dev\rio\rio-backend\src\performer\parser\mod.rs:24`) and optimizes ground
state by using `memchr` to find ESC, then decoding printable prefixes
(`C:\dev\rio\rio-backend\src\performer\parser\mod.rs:657`,
`C:\dev\rio\rio-backend\src\performer\parser\mod.rs:663`,
`C:\dev\rio\rio-backend\src\performer\parser\mod.rs:666`,
`C:\dev\rio\rio-backend\src\performer\parser\mod.rs:685`).

The semantic handler boundary is `Handler` (`C:\dev\rio\rio-backend\src\performer\handler.rs:91`).
`Processor` owns parser state and exposes `advance`
(`C:\dev\rio\rio-backend\src\performer\handler.rs:536`,
`C:\dev\rio\rio-backend\src\performer\handler.rs:549`). The implementation of
`Perform` for Rio's performer maps parser callbacks into handler operations
(`C:\dev\rio\rio-backend\src\performer\handler.rs:889`,
`C:\dev\rio\rio-backend\src\performer\handler.rs:988`,
`C:\dev\rio\rio-backend\src\performer\handler.rs:1150`).

The strongest Rio-specific lesson is synchronized update handling. Rio caps a
DECSET 2026 synchronized update at 150 ms and 2 MiB
(`C:\dev\rio\rio-backend\src\performer\handler.rs:29`,
`C:\dev\rio\rio-backend\src\performer\handler.rs:33`) and recognizes BSU/ESU
CSI sequences (`C:\dev\rio\rio-backend\src\performer\handler.rs:38`,
`C:\dev\rio\rio-backend\src\performer\handler.rs:42`). It buffers synchronized
bytes, exposes `sync_bytes_count`, terminates on byte pressure, and updates
SyncUpdate mode (`C:\dev\rio\rio-backend\src\performer\handler.rs:607`,
`C:\dev\rio\rio-backend\src\performer\handler.rs:613`,
`C:\dev\rio\rio-backend\src\performer\handler.rs:618`,
`C:\dev\rio\rio-backend\src\performer\handler.rs:632`,
`C:\dev\rio\rio-backend\src\performer\handler.rs:1233`).

Hera implication: normalize parser output behind Hera actions, but copy the
idea of explicit synchronized-update budget limits. This is exactly the kind of
policy that prevents terminal rendering from being thrashed by large TUIs.

## Runtime And PTY

Rio's runtime loop is driven by `Machine<T, U>` over `teletypewriter::EventedPty`
and `EventListener` (`C:\dev\rio\rio-backend\src\performer\mod.rs:65`). Runtime
state owns `handler::Processor` (`C:\dev\rio\rio-backend\src\performer\mod.rs:77`,
`C:\dev\rio\rio-backend\src\performer\mod.rs:80`). PTY reads use a 1 MiB buffer,
but cap locked processing at `u16::MAX` bytes per locked pass
(`C:\dev\rio\rio-backend\src\performer\mod.rs:34`,
`C:\dev\rio\rio-backend\src\performer\mod.rs:36`,
`C:\dev\rio\rio-backend\src\performer\mod.rs:170`,
`C:\dev\rio\rio-backend\src\performer\mod.rs:209`,
`C:\dev\rio\rio-backend\src\performer\mod.rs:215`). When parsed bytes produce
damage, the loop sends `RioEvent::TerminalDamaged`
(`C:\dev\rio\rio-backend\src\performer\mod.rs:228`,
`C:\dev\rio\rio-backend\src\performer\mod.rs:366`).

`teletypewriter` exposes `ProcessReadWrite`, `EventedPty`, `WinsizeBuilder` and
platform implementations (`C:\dev\rio\teletypewriter\src\lib.rs:26`,
`C:\dev\rio\teletypewriter\src\lib.rs:28`,
`C:\dev\rio\teletypewriter\src\lib.rs:30`,
`C:\dev\rio\teletypewriter\src\lib.rs:54`,
`C:\dev\rio\teletypewriter\src\lib.rs:64`). Unix creates PTYs via spawn or fork
(`C:\dev\rio\teletypewriter\src\unix\mod.rs:397`,
`C:\dev\rio\teletypewriter\src\unix\mod.rs:626`). Windows uses ConPTY
(`C:\dev\rio\teletypewriter\src\windows\mod.rs:18`,
`C:\dev\rio\teletypewriter\src\windows\mod.rs:35`,
`C:\dev\rio\teletypewriter\src\windows\conpty.rs:27`,
`C:\dev\rio\teletypewriter\src\windows\conpty.rs:66`,
`C:\dev\rio\teletypewriter\src\windows\conpty.rs:248`).

Hera implication: WezTerm `portable-pty` remains the cleaner PTY API
reference, but Rio adds useful runtime budget ideas: bounded locked parsing,
evented wakeups and explicit damage events.

## Damage And Snapshot Boundary

Rio's event layer models terminal damage explicitly. `TerminalDamage` separates
full, partial, cursor-only and no-op states, and comments state that per-row
dirty decisions live on `Row::dirty` in the snapshot
(`C:\dev\rio\rio-backend\src\event\mod.rs:49`,
`C:\dev\rio\rio-backend\src\event\mod.rs:59`,
`C:\dev\rio\rio-backend\src\event\mod.rs:65`). `RioEvent` includes
`TerminalDamaged`, `UpdateGraphics` and glyph protocol events
(`C:\dev\rio\rio-backend\src\event\mod.rs:72`,
`C:\dev\rio\rio-backend\src\event\mod.rs:82`,
`C:\dev\rio\rio-backend\src\event\mod.rs:84`,
`C:\dev\rio\rio-backend\src\event\mod.rs:95`,
`C:\dev\rio\rio-backend\src\event\mod.rs:106`).

The frontend materializes this boundary in `RenderableContent`: pending update,
frame damage, visible rows, style table, extras map, display offset and history
size (`C:\dev\rio\frontends\rioterm\src\context\renderable.rs:41`,
`C:\dev\rio\frontends\rioterm\src\context\renderable.rs:53`,
`C:\dev\rio\frontends\rioterm\src\context\renderable.rs:66`,
`C:\dev\rio\frontends\rioterm\src\context\renderable.rs:74`,
`C:\dev\rio\frontends\rioterm\src\context\renderable.rs:75`,
`C:\dev\rio\frontends\rioterm\src\context\renderable.rs:82`,
`C:\dev\rio\frontends\rioterm\src\context\renderable.rs:88`,
`C:\dev\rio\frontends\rioterm\src\context\renderable.rs:93`). The renderer
calls `terminal.snapshot_visible`, fills visible rows, style table and extras,
then stores `frame_damage`
(`C:\dev\rio\frontends\rioterm\src\renderer\mod.rs:371`,
`C:\dev\rio\frontends\rioterm\src\renderer\mod.rs:374`,
`C:\dev\rio\frontends\rioterm\src\renderer\mod.rs:375`,
`C:\dev\rio\frontends\rioterm\src\renderer\mod.rs:376`,
`C:\dev\rio\frontends\rioterm\src\renderer\mod.rs:405`).

Hera implication: this is the Rio pattern to keep. `terminal-core` should
produce a renderer-neutral snapshot with dirty rows, interned styles and
extras/hyperlink references. GPUI, Sugarloaf or a debug CLI should consume that
snapshot without owning terminal mutation.

## Sugarloaf Renderer

Sugarloaf is Rio's strongest contribution to this research pass. It describes
itself as a multiplatform rendering engine based on WebGPU with desktop and
WebAssembly targets (`C:\dev\rio\sugarloaf\Cargo.toml:22`). The public API
exports `Sugarloaf`, `SugarloafBackend`, `SugarloafRenderer`,
`SugarloafWindow` and `SugarloafWindowSize`
(`C:\dev\rio\sugarloaf\src\lib.rs:47`,
`C:\dev\rio\sugarloaf\src\lib.rs:48`).

Backends are selected in `SugarloafRenderer::default`: WGPU for web, Vulkan on
non-Apple Unix, WGPU on Windows, CPU on Android and Metal on macOS
(`C:\dev\rio\sugarloaf\src\sugarloaf.rs:199`,
`C:\dev\rio\sugarloaf\src\sugarloaf.rs:203`,
`C:\dev\rio\sugarloaf\src\sugarloaf.rs:211`,
`C:\dev\rio\sugarloaf\src\sugarloaf.rs:219`,
`C:\dev\rio\sugarloaf\src\sugarloaf.rs:227`,
`C:\dev\rio\sugarloaf\src\sugarloaf.rs:230`). The context layer wraps WGPU,
Metal, Vulkan and CPU variants and routes resize/support checks by enum match
(`C:\dev\rio\sugarloaf\src\context\mod.rs:17`,
`C:\dev\rio\sugarloaf\src\context\mod.rs:19`,
`C:\dev\rio\sugarloaf\src\context\mod.rs:21`,
`C:\dev\rio\sugarloaf\src\context\mod.rs:23`,
`C:\dev\rio\sugarloaf\src\context\mod.rs:24`,
`C:\dev\rio\sugarloaf\src\context\mod.rs:39`,
`C:\dev\rio\sugarloaf\src\context\mod.rs:113`).

`Sugarloaf` exposes render and grid-render entry points, then dispatches to
WGPU, Metal or Vulkan rendering paths
(`C:\dev\rio\sugarloaf\src\sugarloaf.rs:922`,
`C:\dev\rio\sugarloaf\src\sugarloaf.rs:934`,
`C:\dev\rio\sugarloaf\src\sugarloaf.rs:950`,
`C:\dev\rio\sugarloaf\src\sugarloaf.rs:954`,
`C:\dev\rio\sugarloaf\src\sugarloaf.rs:958`,
`C:\dev\rio\sugarloaf\src\sugarloaf.rs:997`,
`C:\dev\rio\sugarloaf\src\sugarloaf.rs:1031`,
`C:\dev\rio\sugarloaf\src\sugarloaf.rs:1142`). The screen renderer keeps a
map of Sugarloaf `GridRenderer` instances and emits dirty rows into them
(`C:\dev\rio\frontends\rioterm\src\screen\mod.rs:80`,
`C:\dev\rio\frontends\rioterm\src\screen\mod.rs:3813`,
`C:\dev\rio\frontends\rioterm\src\screen\mod.rs:3854`,
`C:\dev\rio\frontends\rioterm\src\screen\mod.rs:3867`,
`C:\dev\rio\frontends\rioterm\src\screen\mod.rs:3973`,
`C:\dev\rio\frontends\rioterm\src\screen\mod.rs:3977`,
`C:\dev\rio\frontends\rioterm\src\screen\mod.rs:4100`).

Font fallback is also platform-specific but Rust-contained. Sugarloaf's font
cache documents CoreText on macOS, fontconfig on Linux and font-kit on Windows
for fallback discovery (`C:\dev\rio\sugarloaf\src\font_cache.rs:99`,
`C:\dev\rio\sugarloaf\src\font_cache.rs:101`,
`C:\dev\rio\sugarloaf\src\font_cache.rs:102`,
`C:\dev\rio\sugarloaf\src\font_cache.rs:103`). The larger font module repeats
the same split and registers discovered fallback fonts into the library
(`C:\dev\rio\sugarloaf\src\font\mod.rs:244`,
`C:\dev\rio\sugarloaf\src\font\mod.rs:265`,
`C:\dev\rio\sugarloaf\src\font\mod.rs:267`,
`C:\dev\rio\sugarloaf\src\font\mod.rs:270`,
`C:\dev\rio\sugarloaf\src\font\mod.rs:321`,
`C:\dev\rio\sugarloaf\src\font\mod.rs:330`).

Hera implication: if Hera later owns a renderer, Sugarloaf is a better
modern Rust renderer reference than Alacritty's OpenGL path. But M1 should
still define only a renderer-neutral `terminal-render-model`.

## Window And Frontend Boundary

`rio-window` describes itself as a Winit fork maintained for Rio
(`C:\dev\rio\rio-window\Cargo.toml:14`) and defaults to X11 and Wayland
features (`C:\dev\rio\rio-window\Cargo.toml:17`,
`C:\dev\rio\rio-window\Cargo.toml:18`,
`C:\dev\rio\rio-window\Cargo.toml:19`). The frontend wires `Crosswords`,
`Sugarloaf`, `Machine` and `teletypewriter` in app context code
(`C:\dev\rio\frontends\rioterm\src\context\mod.rs:13`,
`C:\dev\rio\frontends\rioterm\src\context\mod.rs:20`,
`C:\dev\rio\frontends\rioterm\src\context\mod.rs:25`,
`C:\dev\rio\frontends\rioterm\src\context\mod.rs:45`,
`C:\dev\rio\frontends\rioterm\src\context\mod.rs:51`,
`C:\dev\rio\frontends\rioterm\src\context\mod.rs:62`,
`C:\dev\rio\frontends\rioterm\src\context\mod.rs:160`,
`C:\dev\rio\frontends\rioterm\src\context\mod.rs:310`).

Hera implication: keep this assembly role outside `terminal-core`. Hera can
later have a `terminal-host` or GPUI adapter crate, but the core must stay
headless and testable.

## Graphics Protocols

Rio has broad graphics-protocol work. ANSI modules include graphics, iTerm2
images, Kitty graphics, Kitty virtual placements and Sixel
(`C:\dev\rio\rio-backend\src\ansi\mod.rs:7`,
`C:\dev\rio\rio-backend\src\ansi\mod.rs:8`,
`C:\dev\rio\rio-backend\src\ansi\mod.rs:9`,
`C:\dev\rio\rio-backend\src\ansi\mod.rs:10`,
`C:\dev\rio\rio-backend\src\ansi\mod.rs:12`). Graphics state tracks Sixel and
iTerm2 atlas graphics, Kitty images, placements, virtual placements, chunking
state, total bytes and a default 320 MB limit
(`C:\dev\rio\rio-backend\src\ansi\graphics.rs:18`,
`C:\dev\rio\rio-backend\src\ansi\graphics.rs:21`,
`C:\dev\rio\rio-backend\src\ansi\graphics.rs:153`,
`C:\dev\rio\rio-backend\src\ansi\graphics.rs:155`,
`C:\dev\rio\rio-backend\src\ansi\graphics.rs:156`,
`C:\dev\rio\rio-backend\src\ansi\graphics.rs:200`,
`C:\dev\rio\rio-backend\src\ansi\graphics.rs:206`,
`C:\dev\rio\rio-backend\src\ansi\graphics.rs:262`).

Rio also keeps inactive-screen Kitty graphics state and swaps it on screen
switch (`C:\dev\rio\rio-backend\src\ansi\graphics.rs:224`,
`C:\dev\rio\rio-backend\src\ansi\graphics.rs:346`,
`C:\dev\rio\rio-backend\src\ansi\graphics.rs:352`). Sixel tests load fixture
files under `tests/sixel` and compare decoded RGBA data
(`C:\dev\rio\rio-backend\src\ansi\sixel.rs:867`,
`C:\dev\rio\rio-backend\src\ansi\sixel.rs:868`,
`C:\dev\rio\rio-backend\src\ansi\sixel.rs:900`,
`C:\dev\rio\rio-backend\src\ansi\sixel.rs:913`).

Hera implication: M1 should parse and cap unknown graphics payloads, but
rendering images is not a first milestone. Rio is a strong later reference for
state shape, memory pressure and fixtures.

## Tests And Fixtures

Rio has useful targeted tests:

- Grid reflow tests cover shrink, grow, disabled reflow and multiline cases
  (`C:\dev\rio\rio-backend\src\crosswords\grid\tests.rs:162`,
  `C:\dev\rio\rio-backend\src\crosswords\grid\tests.rs:188`,
  `C:\dev\rio\rio-backend\src\crosswords\grid\tests.rs:215`,
  `C:\dev\rio\rio-backend\src\crosswords\grid\tests.rs:253`,
  `C:\dev\rio\rio-backend\src\crosswords\grid\tests.rs:277`,
  `C:\dev\rio\rio-backend\src\crosswords\grid\tests.rs:308`,
  `C:\dev\rio\rio-backend\src\crosswords\grid\tests.rs:331`).
- Sixel tests cover parsing, resizing, positions and fixture file loading
  (`C:\dev\rio\rio-backend\src\ansi\sixel.rs:678`,
  `C:\dev\rio\rio-backend\src\ansi\sixel.rs:770`,
  `C:\dev\rio\rio-backend\src\ansi\sixel.rs:839`,
  `C:\dev\rio\rio-backend\src\ansi\sixel.rs:848`,
  `C:\dev\rio\rio-backend\src\ansi\sixel.rs:866`).
- Kitty graphics has a large parser test block around transmission,
  placement, deletion, query and size caps
  (`C:\dev\rio\rio-backend\src\ansi\kitty_graphics_protocol.rs:1500`,
  `C:\dev\rio\rio-backend\src\ansi\kitty_graphics_protocol.rs:1547`,
  `C:\dev\rio\rio-backend\src\ansi\kitty_graphics_protocol.rs:1592`,
  `C:\dev\rio\rio-backend\src\ansi\kitty_graphics_protocol.rs:1620`,
  `C:\dev\rio\rio-backend\src\ansi\kitty_graphics_protocol.rs:1959`,
  `C:\dev\rio\rio-backend\src\ansi\kitty_graphics_protocol.rs:1969`).

Hera implication: borrow Rio's reflow and graphics fixture ideas, but start
with Alacritty-style terminal golden snapshots for M1 correctness.

## What Hera Should Copy

Copy:

- Rust crate boundaries with platform-specific code behind crates.
- `snapshot_visible`-style renderer pull boundary.
- `TerminalDamage` plus per-row dirty bits.
- Style interning and extras table indirection.
- Event-driven damage wakeups with an in-flight damage guard.
- Synchronized update timeout and byte budget.
- Sugarloaf's backend selection ideas for a later renderer.
- Graphics payload caps and fixture style for future protocol work.

Do not copy:

- Renderer/window dependencies inside terminal core.
- Full graphics-protocol surface in M1.
- Rio's app-level coupling between backend, window, clipboard and renderer.
- Sugarloaf as a mandatory Hera runtime dependency.
- Rio's terminal semantics as the primary reference when Alacritty and WezTerm
  provide cleaner core boundaries.

## Recommended Hera Shape

Rio reinforces this target:

```text
terminal-core
  Terminal { state, parser }
  Screen { primary, alternate }
  Grid/scrollback storage
  Damage + snapshot generation

terminal-render-model
  RenderFrame
  RenderRow
  RenderCell
  StyleTable
  ExtrasTable
  Damage

host or renderer adapter
  pulls snapshot
  rebuilds dirty rows
  owns GPU/window/font platform decisions
```

Bottom line: Rio is a renderer-boundary and modern Rust desktop reference. For
Hera M1, keep Alacritty/VTE as the seed, WezTerm as the product architecture
guide, Ghostty as the long-term scrollback/render-state target, and Rio as the
proof that a Rust terminal can feed a serious cross-platform renderer without
moving core correctness into the renderer.

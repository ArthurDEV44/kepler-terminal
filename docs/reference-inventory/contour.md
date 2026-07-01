# Contour Reference Inventory

Status: initial focused pass
Date: 2026-07-01
Reference source: `C:\dev\contour`
Scope: Contour lessons for Kepler, especially C++23 module separation,
cross-platform PTY handling, parser/backend boundaries, grid reflow, Sixel and
image protocols, OSC 133 shell integration, semantic block query, buffer
capture, keyboard encoding and renderer isolation.

## Executive Takeaways

Contour is a high-value protocol and fixture reference, not a stack template
for Kepler.

The high-value lessons:

1. Contour validates a layered terminal split: `vtparser`, `vtbackend`,
   `vtpty`, `vtrasterizer`, `text_shaper` and the Qt app are separate CMake
   modules.
2. The parser is a clean event source, but Kepler should still wrap
   `alacritty/vte` first. Contour is useful for parser fixture ideas,
   especially PM/APC/OSC/DCS and bulk UTF-8 handling.
3. The backend is the important part: `Terminal` owns parser, PTY, screens,
   render double buffer, semantic tracker, input generator and image pool.
   Kepler should copy the boundary ideas, not the broad object size.
4. Contour gives excellent evidence for text reflow as a first-class mode.
   It documents DEC private mode 2028 and implements reflow inside grid resize
   with LineSoA-backed lines.
5. Contour is stronger than Kitty as a Windows reference because it has a real
   ConPTY path plus Unix PTY and SSH session implementations.
6. Contour is one of the best references for the semantic session layer:
   OSC 133, a semantic block tracker, a discoverable DEC mode 2034 and DCS JSON
   replies.
7. Sixel, Good Image Protocol and buffer capture should stay out of Kepler M1
   rendering, but their metadata and payload boundaries should shape the future
   protocol model.
8. The renderer and font stack confirm Kepler's adapter rule. DirectWrite,
   CoreText, fontconfig, Freetype, Harfbuzz, OpenGL and Qt belong outside the
   headless core.
9. Contour's C++23/Qt/CMake/vcpkg stack is not a reason to add C++ to Kepler.
   It reinforces a Rust-first core with narrow platform adapters.

Confidence: high for build, module, parser, backend, PTY, reflow, semantic
protocol, Sixel/image and renderer boundary findings because this pass covered
local docs, CMake manifests, parser files, backend files, PTY files, protocol
docs and representative tests under `C:\dev\contour`. Medium for full GUI
conclusions because the Qt/QML app was sampled only enough to understand the
host boundary.

## Why Contour Matters

Contour describes itself as a modern modal virtual terminal emulator
(`C:\dev\contour\README.md:11`) and advertises Linux, macOS, FreeBSD, OpenBSD
and Windows support (`C:\dev\contour\README.md:16`). The local feature page
narrows that to Linux, macOS, FreeBSD and Windows
(`C:\dev\contour\docs\features.md:3`). It is therefore useful for
cross-platform design, but not as a promise that every implementation detail is
portable in the same way.

The most important Contour value for Kepler is not its GUI. The reusable asset
is the combination of:

- C++23 terminal modules with explicit parser, backend, PTY and rasterizer
  boundaries (`C:\dev\contour\CMakeLists.txt:8`,
  `C:\dev\contour\CMakeLists.txt:12`,
  `C:\dev\contour\src\CMakeLists.txt:8`).
- Advanced protocol docs for synchronized output, text reflow, OSC 133,
  semantic block query, buffer capture and CSI u keyboard input
  (`C:\dev\contour\docs\features.md:21`,
  `C:\dev\contour\docs\features.md:22`,
  `C:\dev\contour\docs\features.md:26`).
- Windows ConPTY implementation details plus Unix PTY and SSH session variants
  (`C:\dev\contour\src\vtpty\ConPty.h:17`,
  `C:\dev\contour\src\vtpty\UnixPty.h:26`,
  `C:\dev\contour\src\vtpty\SshSession.h:105`).

Kepler implication: Contour is a protocol and fixture mine. It should influence
Kepler's boundaries, tests and backlog, but not Kepler's language stack.

## Codebase Map

| Area | Local path | What it contains | Kepler relevance |
|---|---|---|---|
| VT parser | `C:\dev\contour\src\vtparser` | Parser state machine, event interface, parser extension hooks, parser tests. | Secondary parser behavior reference after `alacritty/vte`. |
| Backend core | `C:\dev\contour\src\vtbackend` | Terminal, Screen, Grid, LineSoA, render buffers, input generator, semantic tracker, image protocols. | Strong terminal semantics and fixture reference. |
| PTY/process | `C:\dev\contour\src\vtpty` | Abstract PTY, ConPTY, Unix PTY, process lifecycle, optional SSH session. | Useful cross-platform PTY evidence. |
| Rasterizer | `C:\dev\contour\src\vtrasterizer` | Renderer-neutral-ish render buffer consumption, text/image/cursor/box drawing renderers. | Renderer adapter reference, not core dependency. |
| Text shaping | `C:\dev\contour\src\text_shaper` | DirectWrite, CoreText, fontconfig, Freetype and Harfbuzz shaping paths. | Strong reason to keep fonts outside core. |
| GUI app | `C:\dev\contour\src\contour` | Qt/QML app, config, OpenGL renderer integration, terminal session orchestration. | Host app reference only. |
| VT extension docs | `C:\dev\contour\docs\vt-extensions` | Synchronized output, reflow, OSC 133, semantic block query, buffer capture, CSI u. | High-value protocol backlog and docs source. |
| Tests | `C:\dev\contour\src\*_test.cpp` | Parser, grid, backend, Sixel, Good Image Protocol, renderer and input tests. | Strong fixture inspiration. |
| Build | `C:\dev\contour\CMakeLists.txt`, `vcpkg.json` | C++23, CMake modules, Qt, Catch2, libssh2, font libs and platform switches. | Reinforces Rust/Cargo simplicity for Kepler. |

## Language And Platform Decision

Contour reinforces the existing Kepler decision: keep the owned implementation
Rust-first. Contour is written as a C++23 project with CMake and C sources
enabled (`C:\dev\contour\CMakeLists.txt:8`,
`C:\dev\contour\CMakeLists.txt:12`,
`C:\dev\contour\CMakeLists.txt:14`,
`C:\dev\contour\CMakeLists.txt:15`). That is coherent for Contour, but it is
not the right default for a new headless Rust terminal engine.

| Kepler zone | Language | Contour evidence | Decision |
|---|---|---|---|
| Terminal state core | Rust | Contour's backend is a static C++ library linked to parser and PTY modules (`C:\dev\contour\src\vtbackend\CMakeLists.txt:93`, `C:\dev\contour\src\vtbackend\CMakeLists.txt:108`). | Keep Kepler core Rust-only. |
| VT parser | Rust wrapper | Contour has a custom parser event interface and parser table (`C:\dev\contour\src\vtparser\ParserEvents.h:15`, `C:\dev\contour\src\vtparser\Parser-impl.h:25`). | Wrap `alacritty/vte`; use Contour for tests. |
| Grid and reflow | Rust | Contour models reflow with `Grid`, `Line` and `LineSoA` (`C:\dev\contour\src\vtbackend\Grid.h:546`, `C:\dev\contour\src\vtbackend\Line.h:57`). | Implement in Rust with fixture parity. |
| Semantic layer | Rust | OSC 133 and DEC mode 2034 are backend concepts (`C:\dev\contour\src\vtbackend\SemanticBlockTracker.h:30`, `C:\dev\contour\docs\vt-extensions\semantic-block-query.md:18`). | Rust observer sidecar. |
| PTY layer | Rust traits plus OS FFI | Contour abstracts PTY and dispatches ConPTY or Unix PTY by platform (`C:\dev\contour\src\vtpty\Pty.h:52`, `C:\dev\contour\src\vtpty\Pty.cpp:17`). | Rust trait boundary with ConPTY and Unix impls. |
| Windows ConPTY | Rust plus Win32 FFI | Contour loads or calls ConPTY APIs (`C:\dev\contour\src\vtpty\ConPty.cpp:17`, `C:\dev\contour\src\vtpty\ConPty.cpp:98`). | Use `windows-rs`, not C# or C++. |
| Unix PTY | Rust plus Unix syscalls | Contour uses `openpty`, `winsize`, read/write and resize (`C:\dev\contour\src\vtpty\UnixPty.cpp:67`, `C:\dev\contour\src\vtpty\UnixPty.cpp:87`, `C:\dev\contour\src\vtpty\UnixPty.cpp:367`). | Use `rustix`, `libc` or `nix`. |
| Renderer and fonts | Rust adapter later | Contour uses DirectWrite, CoreText, fontconfig, Freetype and Harfbuzz (`C:\dev\contour\src\text_shaper\CMakeLists.txt:11`, `C:\dev\contour\src\text_shaper\CMakeLists.txt:16`, `C:\dev\contour\src\text_shaper\CMakeLists.txt:19`, `C:\dev\contour\src\text_shaper\CMakeLists.txt:37`). | Keep outside `terminal-core`. |
| GUI host | Rust app adapter | Contour uses Qt6 for the GUI path (`C:\dev\contour\CMakeLists.txt:179`). | Kepler exports a Rust render model for GPUI or future hosts. |

Bottom line: Contour's value is its terminal maturity, not its language mix.
Kepler should mine its protocols and tests while keeping a simpler Rust public
surface.

## Build And Platform Shape

Contour's build makes the module split explicit. The root project is C++ and C,
requires C++23, disables compiler extensions and adds `src`
(`C:\dev\contour\CMakeLists.txt:8`,
`C:\dev\contour\CMakeLists.txt:12`,
`C:\dev\contour\CMakeLists.txt:14`,
`C:\dev\contour\CMakeLists.txt:15`,
`C:\dev\contour\CMakeLists.txt:160`). The source tree adds subdirectories for
`crispy`, `text_shaper`, `vtpty`, `vtparser`, `vtbackend`, `vtrasterizer` and
`contour` (`C:\dev\contour\src\CMakeLists.txt:7`,
`C:\dev\contour\src\CMakeLists.txt:8`).

Windows support is not abstract marketing. The docs require Windows 10 Fall
Creators Update because Contour uses ConPTY
(`C:\dev\contour\README.md:122`, `C:\dev\contour\README.md:123`) and document
an external `conpty.dll` workaround for Windows 10 mouse limitations
(`C:\dev\contour\README.md:125`, `C:\dev\contour\README.md:127`,
`C:\dev\contour\README.md:129`). The `vtpty` CMake file switches between
`UnixPty` and `ConPty` sources and can embed OpenConsole artifacts
(`C:\dev\contour\src\vtpty\CMakeLists.txt:64`,
`C:\dev\contour\src\vtpty\CMakeLists.txt:67`,
`C:\dev\contour\src\vtpty\CMakeLists.txt:97`,
`C:\dev\contour\src\vtpty\CMakeLists.txt:121`).

Kepler implication: define platform policy by target triple and adapter crate,
not by adding C++/C# just because Windows is hard. The hard part is isolating
the behavior, not changing the core language.

## Parser Boundary

Contour's parser exposes the exact kind of event interface Kepler needs behind
its parser adapter. `ParserEvents` separates printable text, C0 execution, ESC,
CSI, OSC, DCS hook/unhook and APC events
(`C:\dev\contour\src\vtparser\ParserEvents.h:31`,
`C:\dev\contour\src\vtparser\ParserEvents.h:38`,
`C:\dev\contour\src\vtparser\ParserEvents.h:56`,
`C:\dev\contour\src\vtparser\ParserEvents.h:100`,
`C:\dev\contour\src\vtparser\ParserEvents.h:108`,
`C:\dev\contour\src\vtparser\ParserEvents.h:118`,
`C:\dev\contour\src\vtparser\ParserEvents.h:130`,
`C:\dev\contour\src\vtparser\ParserEvents.h:140`,
`C:\dev\contour\src\vtparser\ParserEvents.h:154`,
`C:\dev\contour\src\vtparser\ParserEvents.h:156`,
`C:\dev\contour\src\vtparser\ParserEvents.h:158`).

The parser state set covers Ground, Escape, CSI, DCS, OSC, APC, PM and SOS
ignore states (`C:\dev\contour\src\vtparser\Parser.h:23`,
`C:\dev\contour\src\vtparser\Parser.h:376`,
`C:\dev\contour\src\vtparser\Parser.h:379`,
`C:\dev\contour\src\vtparser\Parser.h:383`,
`C:\dev\contour\src\vtparser\Parser.h:388`,
`C:\dev\contour\src\vtparser\Parser.h:389`,
`C:\dev\contour\src\vtparser\Parser.h:390`,
`C:\dev\contour\src\vtparser\Parser.h:391`). `Parser` is generic over a
`ParserEventsConcept`, tracks Unicode grapheme scan state and optimizes bulk
text while in Ground state (`C:\dev\contour\src\vtparser\Parser.h:500`,
`C:\dev\contour\src\vtparser\Parser.h:669`,
`C:\dev\contour\src\vtparser\Parser.h:690`,
`C:\dev\contour\src\vtparser\Parser.h:699`,
`C:\dev\contour\src\vtparser\Parser.h:715`,
`C:\dev\contour\src\vtparser\Parser-impl.h:431`,
`C:\dev\contour\src\vtparser\Parser-impl.h:451`,
`C:\dev\contour\src\vtparser\Parser-impl.h:477`).

Kepler implication: keep `alacritty/vte` as the first parser dependency, but
design Kepler's internal `TerminalAction` layer to be parser-agnostic. Contour
is useful for testing PM/APC coverage and split UTF-8 bulk text cases
(`C:\dev\contour\src\vtparser\Parser_test.cpp:46`,
`C:\dev\contour\src\vtparser\Parser_test.cpp:58`,
`C:\dev\contour\src\vtparser\Parser_test.cpp:69`,
`C:\dev\contour\src\vtparser\Parser_test.cpp:116`).

## Terminal State, Screens And Render Buffer

`Terminal` is the central backend object. It owns the PTY, parser, screen pages
and render buffer state (`C:\dev\contour\src\vtbackend\Terminal.h:580`,
`C:\dev\contour\src\vtbackend\Terminal.h:778`,
`C:\dev\contour\src\vtbackend\Terminal.h:1330`,
`C:\dev\contour\src\vtbackend\Terminal.h:1517`,
`C:\dev\contour\src\vtbackend\Terminal.h:1634`,
`C:\dev\contour\src\vtbackend\Terminal.h:1664`,
`C:\dev\contour\src\vtbackend\Terminal.h:1785`). The constructor allocates
primary plus additional pages, then assigns the current screen
(`C:\dev\contour\src\vtbackend\Terminal.cpp:190`,
`C:\dev\contour\src\vtbackend\Terminal.cpp:191`,
`C:\dev\contour\src\vtbackend\Terminal.cpp:198`,
`C:\dev\contour\src\vtbackend\Terminal.cpp:200`).

Input from the PTY is read into pooled buffers and parsed through
`parseFragment` (`C:\dev\contour\src\vtbackend\Terminal.cpp:255`,
`C:\dev\contour\src\vtbackend\Terminal.cpp:273`,
`C:\dev\contour\src\vtbackend\Terminal.cpp:281`,
`C:\dev\contour\src\vtbackend\Terminal.cpp:296`,
`C:\dev\contour\src\vtbackend\Terminal.cpp:359`). Tests and host callers can
write directly into the screen path (`C:\dev\contour\src\vtbackend\Terminal.cpp:1392`,
`C:\dev\contour\src\vtbackend\Terminal.cpp:1406`,
`C:\dev\contour\src\vtbackend\Terminal.cpp:1484`,
`C:\dev\contour\src\vtbackend\Terminal.cpp:1491`).

The render side is double-buffered. `RenderBuffer` stores cells, lines and
image fragments, while `RenderDoubleBuffer` exposes front/back buffer and swap
state (`C:\dev\contour\src\vtbackend\RenderBuffer.h:66`,
`C:\dev\contour\src\vtbackend\RenderBuffer.h:100`,
`C:\dev\contour\src\vtbackend\RenderBuffer.h:107`,
`C:\dev\contour\src\vtbackend\RenderBuffer.h:118`,
`C:\dev\contour\src\vtbackend\RenderBuffer.h:135`,
`C:\dev\contour\src\vtbackend\RenderBuffer.h:153`,
`C:\dev\contour\src\vtbackend\RenderBuffer.h:174`). The terminal refresh path
fills the back buffer and swaps it on demand
(`C:\dev\contour\src\vtbackend\Terminal.cpp:389`,
`C:\dev\contour\src\vtbackend\Terminal.cpp:396`,
`C:\dev\contour\src\vtbackend\Terminal.cpp:415`,
`C:\dev\contour\src\vtbackend\Terminal.cpp:418`,
`C:\dev\contour\src\vtbackend\Terminal.cpp:430`,
`C:\dev\contour\src\vtbackend\Terminal.cpp:504`,
`C:\dev\contour\src\vtbackend\Terminal.cpp:510`).

Kepler implication: copy the pull/snapshot idea, not the exact class. Kepler's
core should expose render frames and damage without owning the renderer or PTY.

## Grid, Scrollback, Resize And Reflow

Contour's `Screen` is the mutable terminal screen and wraps a `Grid`
(`C:\dev\contour\src\vtbackend\Screen.h:131`,
`C:\dev\contour\src\vtbackend\Screen.h:135`,
`C:\dev\contour\src\vtbackend\Screen.h:571`,
`C:\dev\contour\src\vtbackend\Screen.h:640`). `Grid` owns the visible page and
history, exposes logical-line iteration and tracks whether reflow is enabled
(`C:\dev\contour\src\vtbackend\Grid.h:539`,
`C:\dev\contour\src\vtbackend\Grid.h:546`,
`C:\dev\contour\src\vtbackend\Grid.h:568`,
`C:\dev\contour\src\vtbackend\Grid.h:570`,
`C:\dev\contour\src\vtbackend\Grid.h:609`,
`C:\dev\contour\src\vtbackend\Grid.h:625`).

Rows are `Line` objects backed by `LineSoA`, with explicit wrapped and
wrappable flags (`C:\dev\contour\src\vtbackend\Line.h:57`,
`C:\dev\contour\src\vtbackend\Line.h:62`,
`C:\dev\contour\src\vtbackend\Line.h:89`,
`C:\dev\contour\src\vtbackend\Line.h:239`,
`C:\dev\contour\src\vtbackend\Line.h:242`,
`C:\dev\contour\src\vtbackend\Line.h:274`,
`C:\dev\contour\src\vtbackend\Line.h:418`). Grid resize handles page growth,
history pull-down, column changes and reflow across logical lines
(`C:\dev\contour\src\vtbackend\Grid.cpp:593`,
`C:\dev\contour\src\vtbackend\Grid.cpp:647`,
`C:\dev\contour\src\vtbackend\Grid.cpp:662`,
`C:\dev\contour\src\vtbackend\Grid.cpp:694`,
`C:\dev\contour\src\vtbackend\Grid.cpp:751`,
`C:\dev\contour\src\vtbackend\Grid.cpp:813`,
`C:\dev\contour\src\vtbackend\Grid.cpp:875`).

Contour also documents reflow as a protocol mode. DEC private mode 2028 enables
or disables reflow for the current and following lines
(`C:\dev\contour\docs\vt-extensions\line-reflow-mode.md:3`,
`C:\dev\contour\docs\vt-extensions\line-reflow-mode.md:5`,
`C:\dev\contour\docs\vt-extensions\line-reflow-mode.md:9`,
`C:\dev\contour\docs\vt-extensions\line-reflow-mode.md:13`,
`C:\dev\contour\docs\vt-extensions\line-reflow-mode.md:15`). The terminal mode
handler propagates TextReflow into screen/grid behavior
(`C:\dev\contour\src\vtbackend\Terminal.cpp:2573`,
`C:\dev\contour\src\vtbackend\Terminal.cpp:2734`,
`C:\dev\contour\src\vtbackend\Terminal.cpp:2805`).

Kepler implication: reflow must be fixture-first. Contour's grid tests provide
direct cases for resize, history movement, shrink/grow and reflow
(`C:\dev\contour\src\vtbackend\Grid_test.cpp:325`,
`C:\dev\contour\src\vtbackend\Grid_test.cpp:400`,
`C:\dev\contour\src\vtbackend\Grid_test.cpp:426`,
`C:\dev\contour\src\vtbackend\Grid_test.cpp:471`,
`C:\dev\contour\src\vtbackend\Grid_test.cpp:511`,
`C:\dev\contour\src\vtbackend\Grid_test.cpp:512`).

## PTY And Process Lifecycle

Contour's PTY abstraction is close to the shape Kepler needs. `Pty` exposes
read, wakeup, write, page size and resize operations
(`C:\dev\contour\src\vtpty\Pty.h:52`,
`C:\dev\contour\src\vtpty\Pty.h:55`,
`C:\dev\contour\src\vtpty\Pty.h:89`,
`C:\dev\contour\src\vtpty\Pty.h:93`,
`C:\dev\contour\src\vtpty\Pty.h:106`,
`C:\dev\contour\src\vtpty\Pty.h:109`,
`C:\dev\contour\src\vtpty\Pty.h:112`). `createPty` selects ConPTY on Windows
and Unix PTY elsewhere (`C:\dev\contour\src\vtpty\Pty.cpp:17`,
`C:\dev\contour\src\vtpty\Pty.cpp:20`,
`C:\dev\contour\src\vtpty\Pty.cpp:22`).

The Windows path dynamically handles ConPTY APIs and pipe IO
(`C:\dev\contour\src\vtpty\ConPty.cpp:17`,
`C:\dev\contour\src\vtpty\ConPty.cpp:22`,
`C:\dev\contour\src\vtpty\ConPty.cpp:76`,
`C:\dev\contour\src\vtpty\ConPty.cpp:98`,
`C:\dev\contour\src\vtpty\ConPty.cpp:178`,
`C:\dev\contour\src\vtpty\ConPty.cpp:232`,
`C:\dev\contour\src\vtpty\ConPty.cpp:257`,
`C:\dev\contour\src\vtpty\ConPty.cpp:280`). `Process_win32` attaches startup
info to the ConPTY and watches process exit in a thread
(`C:\dev\contour\src\vtpty\Process_win32.cpp:101`,
`C:\dev\contour\src\vtpty\Process_win32.cpp:187`,
`C:\dev\contour\src\vtpty\Process_win32.cpp:193`,
`C:\dev\contour\src\vtpty\Process_win32.cpp:254`).

The Unix path uses `openpty`, read selectors, read/write and terminal resize
with `winsize` (`C:\dev\contour\src\vtpty\UnixPty.cpp:67`,
`C:\dev\contour\src\vtpty\UnixPty.cpp:73`,
`C:\dev\contour\src\vtpty\UnixPty.cpp:87`,
`C:\dev\contour\src\vtpty\UnixPty.cpp:231`,
`C:\dev\contour\src\vtpty\UnixPty.cpp:307`,
`C:\dev\contour\src\vtpty\UnixPty.cpp:324`,
`C:\dev\contour\src\vtpty\UnixPty.cpp:367`,
`C:\dev\contour\src\vtpty\UnixPty.cpp:374`). The Unix process path forks and
attaches the process to the PTY (`C:\dev\contour\src\vtpty\Process_unix.cpp:150`,
`C:\dev\contour\src\vtpty\Process_unix.cpp:154`,
`C:\dev\contour\src\vtpty\Process_unix.cpp:157`).

Kepler implication: keep PTY outside `terminal-core`, but build it as a real
portable crate. Contour supports the same direction as WezTerm's
`portable-pty`: trait first, OS quirks isolated.

## Protocol Extensions And Semantic Layer

Contour is a strong reference for agent-friendly terminal semantics.

OSC 133 shell integration is documented as a command sequence with A, B, C and
D events for prompt start, prompt end, command output start and command finish
(`C:\dev\contour\docs\vt-extensions\osc-133-shell-integration.md:1`,
`C:\dev\contour\docs\vt-extensions\osc-133-shell-integration.md:7`,
`C:\dev\contour\docs\vt-extensions\osc-133-shell-integration.md:19`,
`C:\dev\contour\docs\vt-extensions\osc-133-shell-integration.md:34`,
`C:\dev\contour\docs\vt-extensions\osc-133-shell-integration.md:46`,
`C:\dev\contour\docs\vt-extensions\osc-133-shell-integration.md:59`). The
backend interface mirrors those concepts (`C:\dev\contour\src\vtbackend\ShellIntegration.h:16`,
`C:\dev\contour\src\vtbackend\ShellIntegration.h:25`,
`C:\dev\contour\src\vtbackend\ShellIntegration.h:32`,
`C:\dev\contour\src\vtbackend\ShellIntegration.h:41`).

The semantic block query protocol is even more relevant to Kepler. It provides
a machine-readable query mechanism returning JSON blocks of command data and
requires OSC 133 shell integration
(`C:\dev\contour\docs\vt-extensions\semantic-block-query.md:3`,
`C:\dev\contour\docs\vt-extensions\semantic-block-query.md:9`,
`C:\dev\contour\docs\vt-extensions\semantic-block-query.md:18`,
`C:\dev\contour\docs\vt-extensions\semantic-block-query.md:31`,
`C:\dev\contour\docs\vt-extensions\semantic-block-query.md:40`,
`C:\dev\contour\docs\vt-extensions\semantic-block-query.md:73`,
`C:\dev\contour\docs\vt-extensions\semantic-block-query.md:83`,
`C:\dev\contour\docs\vt-extensions\semantic-block-query.md:149`). The backend
tracks a session token, completed blocks and current block metadata
(`C:\dev\contour\src\vtbackend\SemanticBlockTracker.h:14`,
`C:\dev\contour\src\vtbackend\SemanticBlockTracker.h:22`,
`C:\dev\contour\src\vtbackend\SemanticBlockTracker.h:30`,
`C:\dev\contour\src\vtbackend\SemanticBlockTracker.h:37`,
`C:\dev\contour\src\vtbackend\SemanticBlockTracker.h:42`,
`C:\dev\contour\src\vtbackend\SemanticBlockTracker.h:66`,
`C:\dev\contour\src\vtbackend\SemanticBlockTracker.h:71`,
`C:\dev\contour\src\vtbackend\SemanticBlockTracker.h:76`).

Terminal mode 2034 toggles the tracker and replies with a token
(`C:\dev\contour\src\vtbackend\Terminal.cpp:2680`,
`C:\dev\contour\src\vtbackend\Terminal.cpp:2683`,
`C:\dev\contour\src\vtbackend\Terminal.cpp:2684`). Screen dispatch then marks
prompt/output/command boundaries and can reply with JSON blocks
(`C:\dev\contour\src\vtbackend\Screen.cpp:3810`,
`C:\dev\contour\src\vtbackend\Screen.cpp:3900`,
`C:\dev\contour\src\vtbackend\Screen.cpp:4059`,
`C:\dev\contour\src\vtbackend\Screen.cpp:4094`,
`C:\dev\contour\src\vtbackend\Screen.cpp:4111`,
`C:\dev\contour\src\vtbackend\Screen.cpp:4121`).

Kepler implication: the semantic layer should be a Rust observer sidecar with
stable row handles and session tokens. It must never corrupt terminal state if
OSC 133 is absent, malformed or disabled.

## Sixel, Images And Buffer Capture

Contour treats Sixel as a real feature (`C:\dev\contour\docs\features.md:25`)
and has a dedicated Sixel parser plus image builder. `SixelParser` implements a
parser extension, consumes stream fragments and delegates image construction to
events (`C:\dev\contour\src\vtbackend\SixelParser.h:19`,
`C:\dev\contour\src\vtbackend\SixelParser.h:25`,
`C:\dev\contour\src\vtbackend\SixelParser.h:43`,
`C:\dev\contour\src\vtbackend\SixelParser.h:73`,
`C:\dev\contour\src\vtbackend\SixelParser.h:77`,
`C:\dev\contour\src\vtbackend\SixelParser.h:83`,
`C:\dev\contour\src\vtbackend\SixelParser.h:137`,
`C:\dev\contour\src\vtbackend\SixelParser.h:165`,
`C:\dev\contour\src\vtbackend\SixelParser.h:166`). Its tests cover ground
patterns, raster attributes, repeat, colors, rewind, newline and aspect ratio
edge cases (`C:\dev\contour\src\vtbackend\SixelParser_test.cpp:25`,
`C:\dev\contour\src\vtbackend\SixelParser_test.cpp:179`,
`C:\dev\contour\src\vtbackend\SixelParser_test.cpp:216`,
`C:\dev\contour\src\vtbackend\SixelParser_test.cpp:246`,
`C:\dev\contour\src\vtbackend\SixelParser_test.cpp:286`,
`C:\dev\contour\src\vtbackend\SixelParser_test.cpp:321`,
`C:\dev\contour\src\vtbackend\SixelParser_test.cpp:377`).

Contour also has Good Image Protocol tests for upload, render, layers, invalid
format, cursor update and resource limits
(`C:\dev\contour\src\vtbackend\GoodImageProtocol_test.cpp:71`,
`C:\dev\contour\src\vtbackend\GoodImageProtocol_test.cpp:117`,
`C:\dev\contour\src\vtbackend\GoodImageProtocol_test.cpp:223`,
`C:\dev\contour\src\vtbackend\GoodImageProtocol_test.cpp:237`,
`C:\dev\contour\src\vtbackend\GoodImageProtocol_test.cpp:252`,
`C:\dev\contour\src\vtbackend\GoodImageProtocol_test.cpp:488`,
`C:\dev\contour\src\vtbackend\GoodImageProtocol_test.cpp:525`,
`C:\dev\contour\src\vtbackend\GoodImageProtocol_test.cpp:558`). Screen image
rendering handles uploaded images, named images, one-shot raw data, layers,
auto-scroll and Sixel cursor placement (`C:\dev\contour\src\vtbackend\Screen.h:324`,
`C:\dev\contour\src\vtbackend\Screen.h:342`,
`C:\dev\contour\src\vtbackend\Screen.h:353`,
`C:\dev\contour\src\vtbackend\Screen.h:365`,
`C:\dev\contour\src\vtbackend\Screen.cpp:2332`,
`C:\dev\contour\src\vtbackend\Screen.cpp:2337`,
`C:\dev\contour\src\vtbackend\Screen.cpp:2367`,
`C:\dev\contour\src\vtbackend\Screen.cpp:2368`).

Buffer capture is also worth tracking for agent workflows. Contour documents a
VT sequence to capture screen text for shell integration and replies in
UTF-8 chunks (`C:\dev\contour\docs\vt-extensions\buffer-capture.md:3`,
`C:\dev\contour\docs\vt-extensions\buffer-capture.md:16`,
`C:\dev\contour\docs\vt-extensions\buffer-capture.md:25`,
`C:\dev\contour\docs\vt-extensions\buffer-capture.md:27`). The backend exposes
`XTCAPTURE` and a capture buffer code, then implements capture in `Screen`
(`C:\dev\contour\src\vtbackend\Functions.h:127`,
`C:\dev\contour\src\vtbackend\Functions.h:662`,
`C:\dev\contour\src\vtbackend\Functions.h:732`,
`C:\dev\contour\src\vtbackend\Screen.cpp:1976`,
`C:\dev\contour\src\vtbackend\Screen.cpp:1997`,
`C:\dev\contour\src\vtbackend\Screen.cpp:2013`,
`C:\dev\contour\src\vtbackend\Screen.cpp:2041`).

Kepler implication: M1 should parse, cap and preserve unknown image/protocol
payload metadata, but not implement image rendering. Buffer capture is more
relevant earlier because it aligns with agent-visible terminal state.

## Input, Keyboard And Mouse

Contour documents CSI u as an extended keyboard protocol with mode entry, mode
exit and a stack of keyboard protocol flags
(`C:\dev\contour\docs\vt-extensions\csi-u-extended-keyboard-protocol.md:5`,
`C:\dev\contour\docs\vt-extensions\csi-u-extended-keyboard-protocol.md:8`,
`C:\dev\contour\docs\vt-extensions\csi-u-extended-keyboard-protocol.md:14`,
`C:\dev\contour\docs\vt-extensions\csi-u-extended-keyboard-protocol.md:17`,
`C:\dev\contour\docs\vt-extensions\csi-u-extended-keyboard-protocol.md:24`,
`C:\dev\contour\docs\vt-extensions\csi-u-extended-keyboard-protocol.md:35`,
`C:\dev\contour\docs\vt-extensions\csi-u-extended-keyboard-protocol.md:106`).

The input generator handles event flags, alternate keys, associated text,
legacy vs non-legacy modes, mouse protocols and Win32 input mode
(`C:\dev\contour\src\vtbackend\InputGenerator.cpp:336`,
`C:\dev\contour\src\vtbackend\InputGenerator.cpp:345`,
`C:\dev\contour\src\vtbackend\InputGenerator.cpp:403`,
`C:\dev\contour\src\vtbackend\InputGenerator.cpp:607`,
`C:\dev\contour\src\vtbackend\InputGenerator.cpp:628`,
`C:\dev\contour\src\vtbackend\InputGenerator.cpp:1105`,
`C:\dev\contour\src\vtbackend\InputGenerator.h:404`,
`C:\dev\contour\src\vtbackend\InputGenerator.h:411`,
`C:\dev\contour\src\vtbackend\InputGenerator.h:509`,
`C:\dev\contour\src\vtbackend\InputGenerator.h:517`).

Kepler implication: keyboard encoding belongs in a separate Rust helper crate
or module outside the headless state core. The terminal core should model mode
state and expose enough configuration for the input encoder.

## Renderer And Text Shaping Boundary

Contour's renderer stack confirms the adapter split. `vtrasterizer` links
against `vtbackend`, `crispy::core` and `text_shaper`
(`C:\dev\contour\src\vtrasterizer\CMakeLists.txt:41`,
`C:\dev\contour\src\vtrasterizer\CMakeLists.txt:47`). It has explicit
renderers for background, cursor, decorations, text and box drawing, with tests
for renderer reconfiguration and box drawing
(`C:\dev\contour\src\vtrasterizer\BackgroundRenderer.h:24`,
`C:\dev\contour\src\vtrasterizer\CursorRenderer.h:19`,
`C:\dev\contour\src\vtrasterizer\BoxDrawingRenderer.h:17`,
`C:\dev\contour\src\vtrasterizer\TextRenderer_test.cpp:816`,
`C:\dev\contour\src\vtrasterizer\TextRenderer_test.cpp:892`,
`C:\dev\contour\src\vtrasterizer\BoxDrawingRenderer_test.cpp:79`).

Text shaping is platform-heavy. CMake includes DirectWrite on Windows,
CoreText on macOS and fontconfig/Freetype paths on Unix-like platforms
(`C:\dev\contour\src\text_shaper\CMakeLists.txt:11`,
`C:\dev\contour\src\text_shaper\CMakeLists.txt:16`,
`C:\dev\contour\src\text_shaper\CMakeLists.txt:19`,
`C:\dev\contour\src\text_shaper\CMakeLists.txt:34`,
`C:\dev\contour\src\text_shaper\CMakeLists.txt:37`,
`C:\dev\contour\src\text_shaper\CMakeLists.txt:43`). The code exposes
DirectWrite, CoreText and OpenShaper options
(`C:\dev\contour\src\vtrasterizer\FontDescriptions.h:27`,
`C:\dev\contour\src\vtrasterizer\FontDescriptions.h:29`,
`C:\dev\contour\src\vtrasterizer\FontDescriptions.h:177`,
`C:\dev\contour\src\vtrasterizer\FontDescriptions.h:178`).

Kepler implication: keep `terminal-render-model` platform-neutral. Font
resolution, glyph shaping, texture atlases and OpenGL/GPUI resources belong in
host adapters.

## Tests And Fixture Mining

Contour has a large Catch2 surface. Useful fixture zones:

- Parser: PM/APC, split UTF-8, preceding graphic character
  (`C:\dev\contour\src\vtparser\Parser_test.cpp:46`,
  `C:\dev\contour\src\vtparser\Parser_test.cpp:58`,
  `C:\dev\contour\src\vtparser\Parser_test.cpp:69`,
  `C:\dev\contour\src\vtparser\Parser_test.cpp:148`).
- Grid and reflow: logical lines, resize with history, column shrink/grow,
  reflow (`C:\dev\contour\src\vtbackend\Grid_test.cpp:193`,
  `C:\dev\contour\src\vtbackend\Grid_test.cpp:325`,
  `C:\dev\contour\src\vtbackend\Grid_test.cpp:400`,
  `C:\dev\contour\src\vtbackend\Grid_test.cpp:426`,
  `C:\dev\contour\src\vtbackend\Grid_test.cpp:512`).
- Sixel and image protocols: Sixel parser, Good Image Protocol upload/render,
  layers and resource limits (`C:\dev\contour\src\vtbackend\SixelParser_test.cpp:25`,
  `C:\dev\contour\src\vtbackend\SixelParser_test.cpp:216`,
  `C:\dev\contour\src\vtbackend\GoodImageProtocol_test.cpp:71`,
  `C:\dev\contour\src\vtbackend\GoodImageProtocol_test.cpp:223`,
  `C:\dev\contour\src\vtbackend\GoodImageProtocol_test.cpp:525`).
- Renderer: box drawing and render geometry reconfiguration
  (`C:\dev\contour\src\vtrasterizer\BoxDrawingRenderer_test.cpp:79`,
  `C:\dev\contour\src\vtrasterizer\TextRenderer_test.cpp:816`,
  `C:\dev\contour\src\vtrasterizer\TextRenderer_test.cpp:1137`,
  `C:\dev\contour\src\vtrasterizer\TextRenderer_test.cpp:1171`).
- Capability/function tables: terminfo gaps, OSC/DCS capability definitions
  and XTCAPTURE (`C:\dev\contour\src\vtbackend\Capabilities_test.cpp:39`,
  `C:\dev\contour\src\vtbackend\Functions_test.cpp:84`,
  `C:\dev\contour\src\vtbackend\Functions.h:662`).

Kepler implication: extract fixture ideas before implementing clever behavior.
Contour is especially useful after M1, when Kepler has a stable snapshot format
and can compare reflow, Sixel metadata, semantic blocks and capture output.

## What Kepler Should Copy

Copy these ideas:

1. Layer names and boundaries: parser, backend state, PTY, render model,
   renderer adapter and text shaping adapter.
2. PTY trait surface: read, wakeup, write, size and resize, with platform
   runtime selection outside core.
3. Explicit reflow mode and wrap metadata. Mode 2028 is a useful compatibility
   clue even if Kepler does not expose that exact extension in M1.
4. Semantic session observer: OSC 133 events, completed blocks, current block,
   tokened query and JSON-ish external shape.
5. Buffer capture as a product-relevant terminal operation.
6. Render double-buffer idea as a host-facing snapshot pattern.
7. Fixture categories: PM/APC parser, split UTF-8, reflow, Sixel, image layers,
   keyboard protocol modes and renderer geometry changes.

## What Kepler Should Avoid

Avoid these choices:

1. Do not import C++ into Kepler core. Contour's C++23 architecture is mature,
   but Kepler's leverage is a Rust-first public API.
2. Do not let terminal state own GUI, OpenGL, Qt, DirectWrite, CoreText or
   fontconfig types.
3. Do not merge parser callbacks, screen mutation, image pool, semantic tracker,
   PTY and render buffers into a single public object.
4. Do not implement Sixel or Good Image Protocol rendering in M1. Preserve
   metadata and payload boundaries first.
5. Do not make semantic blocks authoritative. OSC 133 is optional shell
   cooperation, not terminal correctness.
6. Do not treat ConPTY as a source of terminal truth. It is a stream transport
   with platform-specific lifecycle and resize quirks.

## Recommended Kepler Shape

Contour supports this shape:

- `terminal-core`: Rust-only terminal state, screens, grid, modes, reflow,
  snapshots and byte ingestion.
- `terminal-protocol`: parser-normalized actions, OSC/DCS/APC/PM payload
  models, keyboard protocol state, semantic events and image placeholders.
- `terminal-pty`: Rust PTY trait with Windows ConPTY, Unix PTY and future SSH
  adapters.
- `terminal-render-model`: render buffer, damage, cursor, selection, hyperlink
  spans, image placeholders and capture-friendly text slices.
- `terminal-fixtures`: Contour-derived reflow, parser, semantic, Sixel,
  keyboard and capture cases.

Final decision: Contour is a major reference for compatibility depth and
agent-friendly terminal semantics. It does not change the language plan.
Kepler remains Rust-first, with Contour used as a protocol, fixture and adapter
boundary reference.

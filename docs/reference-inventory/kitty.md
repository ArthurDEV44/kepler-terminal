# Kitty Reference Inventory

Status: initial focused pass
Date: 2026-07-01
Reference source: `C:\dev\kitty`
Scope: Kitty lessons for Hera, especially advanced terminal protocols,
parser/screen compatibility, graphics protocol boundaries, keyboard encoding,
dirty line tracking, resize/reflow behavior and cross-platform caveats.

## Executive Takeaways

Kitty is a high-value compatibility and protocol reference, not a direct
architecture seed for Hera's core.

The high-value lessons:

1. Kitty's protocol documents are the main asset: graphics, keyboard, text
   sizing, file transfer, desktop notifications and shell integration.
2. The graphics protocol is broad enough to justify deferral in Hera M1. It
   covers transmission media, image IDs, placements, deletion, z-order,
   Unicode placeholders, animation, quotas and replies.
3. The keyboard protocol is more relevant earlier: it gives a concrete model
   for progressive `CSI u` enhancement, alternate keys, event types and text
   codepoints.
4. Kitty's C screen engine is useful as a compatibility oracle, but its
   monolithic `Screen` plus parser plus graphics manager shape is not the
   clean headless Rust API Hera wants.
5. The dirty-line and resize/reflow paths are worth studying. `LineBuf`,
   `HistoryBuf`, `screen_resize`, pager history and graphics resize handling
   expose real terminal edge cases.
6. Do not copy Kitty's stack for Hera. Kitty mixes Python, C, Go, Objective-C
   and platform GLFW backends. Hera should stay Rust-first with narrow OS FFI
   adapters.
7. Do not use Kitty as a Windows PTY reference. The local code path is POSIX
   PTY and `fork`, while install/build docs focus on macOS and Linux binaries.
8. Use Kitty tests as fixture inspiration for parser, screen, Unicode, keyboard
   and graphics compatibility, especially once Hera has a stable core
   snapshot harness.

Confidence: high for protocol, parser/screen, graphics and build/platform
findings because this pass covers local docs, C/Python/Go manifests, screen,
parser, graphics, child process code and test files under `C:\dev\kitty`.
Medium for full renderer conclusions because this pass sampled the renderer,
font and GLFW paths rather than auditing every shader and OpenGL resource
lifetime.

## Why Kitty Matters

Kitty presents itself as a fast, feature-rich, cross-platform, GPU-based
terminal (`C:\dev\kitty\README.asciidoc:1`). The important nuance for Hera is
that "cross-platform" here does not mean the codebase is a clean
Windows/Linux/macOS architecture template. The binary install docs mention
pre-built binaries for macOS or Linux (`C:\dev\kitty\docs\binary.rst:9`), while
the build docs focus on Linux and macOS dependencies
(`C:\dev\kitty\docs\build.rst:16`, `C:\dev\kitty\docs\build.rst:28`,
`C:\dev\kitty\docs\build.rst:44`, `C:\dev\kitty\docs\build.rst:152`).

The strongest Hera value is therefore not "copy Kitty". It is "treat Kitty
as a protocol authority and compatibility stress test". Kitty documents and
implements protocols that serious terminal apps eventually hit, but several of
those protocols are too large for Hera M1.

## Codebase Map

| Area | Local path | What it contains | Hera relevance |
|---|---|---|---|
| Screen core | `C:\dev\kitty\kitty\screen.c`, `screen.h` | C terminal screen, parser ownership, main/alt line buffers, history, graphics manager, modes and callbacks. | Compatibility oracle, not target API shape. |
| VT parser | `C:\dev\kitty\kitty\vt-parser.c`, `vt-parser.h` | Custom parser state machine, CSI/OSC/DCS/APC dispatch, write buffers. | Secondary parser behavior reference after `alacritty/vte`. |
| Line/history buffers | `C:\dev\kitty\kitty\line-buf.c`, `history.c` | Visible line buffer, dirty flags, ring history, pager history and rewrap helpers. | Useful for dirty/reflow edge cases. |
| Graphics protocol | `C:\dev\kitty\kitty\graphics.c`, `graphics.h`, `parse-graphics-command.h` | Kitty image state, storage quotas, transmission, placement, z-order, animation and render data. | Future image placeholder and fixture reference, not M1 rendering. |
| Protocol docs | `C:\dev\kitty\docs\*-protocol.rst` | Graphics, keyboard, text sizing, file transfer, notifications and misc extensions. | Highest-value Kitty artifact for Hera. |
| Keyboard input | `C:\dev\kitty\kitty\key_encoding.c`, `keys.c` | GLFW key event encoding, progressive Kitty keyboard protocol flags, text fallback. | Good input-encoder model outside core. |
| PTY/process | `C:\dev\kitty\kitty\child.py`, `child.c`, `child-monitor.c` | POSIX PTY creation, fork/exec, poll loop, resize and child lifecycle. | Unix PTY reference only, not Windows/ConPTY. |
| Renderer/window | `C:\dev\kitty\kitty\gl.c`, `glfw.c`, `state.c`, `core_text.m`, `fontconfig.c` | OpenGL/GLFW, Cocoa/CoreText and Linux font integration. | Renderer complexity reference, not M1 dependency. |
| Tests | `C:\dev\kitty\kitty_tests` | Parser, screen, graphics, keys, mouse, multicell and font tests. | Strong future fixture source. |
| Build | `C:\dev\kitty\setup.py`, `pyproject.toml`, `go.mod` | Python build script, C extensions, Go tools/kittens, platform packaging. | Shows why Hera should avoid multi-language core debt. |

## Language And Platform Decision

Kitty reinforces the existing Hera decision: Rust stays the implementation
language for core, protocol model, render model and PTY API. Kitty's language
mix is a product-specific tradeoff, not a reason to add C, C#, Objective-C,
Swift, Zig or Go to Hera's engine.

| Hera zone | Language | Kitty evidence | Decision |
|---|---|---|---|
| Terminal state core | Rust | Kitty's core screen is C and owns parser, buffers, history and graphics manager (`C:\dev\kitty\kitty\screen.h:113`, `C:\dev\kitty\kitty\screen.h:133`, `C:\dev\kitty\kitty\screen.h:134`, `C:\dev\kitty\kitty\screen.h:135`, `C:\dev\kitty\kitty\screen.h:194`). | Do not copy the C core. Keep Hera core Rust-only. |
| VT parser | Rust wrapper | Kitty has a custom C parser with explicit states and dispatch (`C:\dev\kitty\kitty\vt-parser.c:178`, `C:\dev\kitty\kitty\vt-parser.c:287`, `C:\dev\kitty\kitty\vt-parser.c:478`, `C:\dev\kitty\kitty\vt-parser.c:1114`, `C:\dev\kitty\kitty\vt-parser.c:1417`). | Use `alacritty/vte` behind Hera actions; use Kitty as behavior oracle. |
| Protocol docs/schema | Rust | Kitty documents extensions in RST (`C:\dev\kitty\docs\protocol-extensions.rst:1`, `C:\dev\kitty\docs\protocol-extensions.rst:28`). | Model protocol metadata in Rust types. |
| Graphics/image state | Rust later | Kitty's graphics manager has storage limits, render data and animation state (`C:\dev\kitty\kitty\graphics.h:140`, `C:\dev\kitty\kitty\graphics.h:146`, `C:\dev\kitty\kitty\graphics.h:154`, `C:\dev\kitty\kitty\graphics.c:26`, `C:\dev\kitty\kitty\graphics.c:27`). | M1 parses/caps metadata only. Full image state later. |
| Keyboard encoder | Rust helper crate | Kitty encodes GLFW events with protocol flags and text fallback (`C:\dev\kitty\kitty\key_encoding.c:423`, `C:\dev\kitty\kitty\key_encoding.c:430`, `C:\dev\kitty\kitty\key_encoding.c:431`, `C:\dev\kitty\kitty\key_encoding.c:446`, `C:\dev\kitty\kitty\keys.c:184`). | Build a Rust input encoder outside `terminal-core`. |
| PTY | Rust traits plus OS FFI | Kitty uses POSIX `openpty`, `fork`, `execvp` and `TIOCSWINSZ` (`C:\dev\kitty\kitty\child.py:212`, `C:\dev\kitty\kitty\child.py:337`, `C:\dev\kitty\kitty\child.c:84`, `C:\dev\kitty\kitty\child.c:170`, `C:\dev\kitty\kitty\child-monitor.c:622`). | Use portable Rust PTY traits with Windows ConPTY and Unix impls. |
| Renderer/window | Rust adapter later | Kitty uses C/OpenGL/GLFW, Objective-C Cocoa/CoreText and fontconfig (`C:\dev\kitty\setup.py:1021`, `C:\dev\kitty\setup.py:1042`, `C:\dev\kitty\kitty\cocoa_window.m:14`, `C:\dev\kitty\kitty\core_text.m:18`, `C:\dev\kitty\kitty\fontconfig.c:11`). | Keep renderer outside core. No Objective-C source unless Rust bindings fail. |
| Build/tooling | Rust/Cargo | Kitty requires Python >= 3.11 and Go 1.26 for its toolchain (`C:\dev\kitty\pyproject.toml:2`, `C:\dev\kitty\go.mod:1`, `C:\dev\kitty\go.mod:3`). | Hera should stay Cargo-first. |

Bottom line: Kitty is an argument against multi-language core drift. Its
technical value is protocol maturity, not stack shape.

## Build And Platform Shape

Kitty's build script compiles C extensions, builds GLFW platform modules and
builds static Go-based kittens/tools. The main build compiles
`kitty/fast_data_types`, then GLFW, then kittens
(`C:\dev\kitty\setup.py:1215`, `C:\dev\kitty\setup.py:1219`). The GLFW backend
selection is platform-specific: `cocoa` on macOS, `x11 wayland` elsewhere
(`C:\dev\kitty\setup.py:1042`). Mac-specific C/Objective-C files are filtered
from the generic C list unless building macOS
(`C:\dev\kitty\setup.py:1015`, `C:\dev\kitty\setup.py:1021`).

Packaging paths are Linux/macOS-oriented. The build creates Linux package
content when not macOS and not Windows, and macOS bundle content for macOS
package types (`C:\dev\kitty\setup.py:1929`, `C:\dev\kitty\setup.py:1933`).
Static tools are built across several Unix-like targets and macOS/Linux
architectures (`C:\dev\kitty\setup.py:1337`, `C:\dev\kitty\setup.py:1340`,
`C:\dev\kitty\setup.py:1345`).

Hera implication: cross-platform support should be expressed as Rust target
triples and platform modules, not as a Python/C/Go build orchestrator.

## Parser And Screen Architecture

Kitty's parser is a custom C parser. `Parser` exposes write-buffer creation,
commit, input-space checks and parse worker entry points
(`C:\dev\kitty\kitty\vt-parser.h:13`, `C:\dev\kitty\kitty\vt-parser.h:34`,
`C:\dev\kitty\kitty\vt-parser.h:35`, `C:\dev\kitty\kitty\vt-parser.h:36`,
`C:\dev\kitty\kitty\vt-parser.h:37`). CSI parsing has a 256-parameter cap
(`C:\dev\kitty\kitty\vt-parser.c:22`), and parser states cover normal, ESC,
CSI, OSC, DCS, APC, PM and SOS (`C:\dev\kitty\kitty\vt-parser.c:178`,
`C:\dev\kitty\kitty\vt-parser.c:179`). Printable text is dispatched directly
to `screen_draw_text` (`C:\dev\kitty\kitty\vt-parser.c:244`,
`C:\dev\kitty\kitty\vt-parser.c:254`).

`Screen` owns the parser and state directly. It allocates the VT parser, main
and alternate `LineBuf`, a `HistoryBuf`, and graphics managers
(`C:\dev\kitty\kitty\screen.c:120`, `C:\dev\kitty\kitty\screen.c:141`,
`C:\dev\kitty\kitty\screen.c:143`, `C:\dev\kitty\kitty\screen.h:133`,
`C:\dev\kitty\kitty\screen.h:134`). It handles draw text, graphics commands,
buffer switching, key encoding flags and paused rendering
(`C:\dev\kitty\kitty\screen.c:1235`, `C:\dev\kitty\kitty\screen.c:1603`,
`C:\dev\kitty\kitty\screen.c:1624`, `C:\dev\kitty\kitty\screen.c:1803`,
`C:\dev\kitty\kitty\screen.c:3365`).

Hera implication: do not mirror this as one large object. Keep a smaller
`Terminal { state, parser }`, hide parser callbacks behind Hera actions, and
keep graphics, PTY and renderer concerns outside the core.

## Grid, Scrollback, Resize And Dirty State

Kitty's visible buffer is `LineBuf`: CPU/GPU cell storage plus line attributes
(`C:\dev\kitty\kitty\line-buf.h:13`, `C:\dev\kitty\kitty\line-buf.h:18`).
It marks dirty text at the line level (`C:\dev\kitty\kitty\line-buf.c:42`,
`C:\dev\kitty\kitty\line-buf.c:48`, `C:\dev\kitty\kitty\line-buf.c:53`) and
implements index/reverse-index scroll operations
(`C:\dev\kitty\kitty\line-buf.c:335`, `C:\dev\kitty\kitty\line-buf.c:356`).

Scrollback is `HistoryBuf`, with segmented storage, a ring-like index, dirty
line flags and optional pager history (`C:\dev\kitty\kitty\history.h:26`,
`C:\dev\kitty\kitty\history.h:29`, `C:\dev\kitty\kitty\history.h:30`,
`C:\dev\kitty\kitty\history.h:31`, `C:\dev\kitty\kitty\history.c:165`,
`C:\dev\kitty\kitty\history.c:167`). It pushes lines into history and can
rewrap pager history (`C:\dev\kitty\kitty\history.c:299`,
`C:\dev\kitty\kitty\history.c:313`, `C:\dev\kitty\kitty\history.c:435`,
`C:\dev\kitty\kitty\history.c:510`). Resize and rewrap involve both line
buffers and history, then resize graphics managers
(`C:\dev\kitty\kitty\screen.c:243`, `C:\dev\kitty\kitty\screen.c:249`,
`C:\dev\kitty\kitty\screen.c:534`, `C:\dev\kitty\kitty\screen.c:567`,
`C:\dev\kitty\kitty\screen.c:572`, `C:\dev\kitty\kitty\screen.c:575`).

Hera implication:

- Copy the idea of explicit dirty-line tracking.
- Keep resize/reflow fixture-driven from the first implementation.
- Preserve graphics/image metadata through resize later, but not in M1.
- Prefer Ghostty-style page/chunk design for long-term huge scrollback rather
  than adopting Kitty's C buffer layout directly.

## Renderer And Font Boundary

Kitty's renderer boundary is not the Hera target, but it exposes practical
risks. Global window render state carries a `Screen *`, OS windows track
`needs_render` and render calls, and render functions are tied to OpenGL/GLFW
state (`C:\dev\kitty\kitty\state.h:183`, `C:\dev\kitty\kitty\state.h:186`,
`C:\dev\kitty\kitty\state.h:413`, `C:\dev\kitty\kitty\state.h:435`,
`C:\dev\kitty\kitty\state.h:463`, `C:\dev\kitty\kitty\state.h:588`).
Font fallback is platform-specific: CoreText on macOS and fontconfig on Linux
paths appear in source (`C:\dev\kitty\kitty\core_text.m:382`,
`C:\dev\kitty\kitty\core_text.m:389`, `C:\dev\kitty\kitty\fontconfig.c:84`,
`C:\dev\kitty\kitty\fontconfig.c:99`).

Hera implication: renderer and font integration belong in adapters. The core
should emit a renderer-neutral snapshot and dirty regions, not OpenGL resources
or platform font objects.

## Protocol Extensions

Kitty's protocol extension docs are its most reusable artifact. The extension
overview lists graphics, keyboard, text sizing, drag and drop, multiple
cursors, file transfer and misc protocols
(`C:\dev\kitty\docs\protocol-extensions.rst:28`,
`C:\dev\kitty\docs\protocol-extensions.rst:29`,
`C:\dev\kitty\docs\protocol-extensions.rst:30`,
`C:\dev\kitty\docs\protocol-extensions.rst:31`,
`C:\dev\kitty\docs\protocol-extensions.rst:32`,
`C:\dev\kitty\docs\protocol-extensions.rst:33`,
`C:\dev\kitty\docs\protocol-extensions.rst:40`).

Examples worth keeping near Hera:

- Graphics protocol: image transfer, placement and display over APC
  (`C:\dev\kitty\docs\graphics-protocol.rst:1`,
  `C:\dev\kitty\docs\graphics-protocol.rst:249`).
- Keyboard protocol: opt-in progressive `CSI u` behavior
  (`C:\dev\kitty\docs\keyboard-protocol.rst:83`,
  `C:\dev\kitty\docs\keyboard-protocol.rst:87`,
  `C:\dev\kitty\docs\keyboard-protocol.rst:138`,
  `C:\dev\kitty\docs\keyboard-protocol.rst:292`).
- File transfer protocol: OSC 5113 sessions
  (`C:\dev\kitty\docs\file-transfer-protocol.rst:19`,
  `C:\dev\kitty\docs\file-transfer-protocol.rst:543`,
  `C:\dev\kitty\docs\file-transfer-protocol.rst:545`).
- Text sizing protocol: OSC-driven multicell text
  (`C:\dev\kitty\docs\text-sizing-protocol.rst:12`,
  `C:\dev\kitty\docs\text-sizing-protocol.rst:52`,
  `C:\dev\kitty\docs\text-sizing-protocol.rst:277`).
- Desktop notifications: OSC 99 support and query behavior
  (`C:\dev\kitty\docs\desktop-notifications.rst:7`,
  `C:\dev\kitty\docs\desktop-notifications.rst:373`).
- Shell integration prompt markers: OSC 133-compatible marks
  (`C:\dev\kitty\docs\shell-integration.rst:422`,
  `C:\dev\kitty\docs\shell-integration.rst:430`,
  `C:\dev\kitty\docs\shell-integration.rst:448`).

Hera implication: M1 should safely capture and bound OSC/DCS/APC payloads,
but should not promise all extensions. Prioritize keyboard metadata and shell
marks before graphics rendering.

## Kitty Graphics

The graphics protocol is the clearest "not M1" signal. The spec goals include
flexible performant image display without requiring terminals to understand
arbitrary image formats (`C:\dev\kitty\docs\graphics-protocol.rst:4`,
`C:\dev\kitty\docs\graphics-protocol.rst:9`). It supports direct data, files,
temporary files and shared memory (`C:\dev\kitty\docs\graphics-protocol.rst:327`,
`C:\dev\kitty\docs\graphics-protocol.rst:332`,
`C:\dev\kitty\docs\graphics-protocol.rst:338`). It defines query support
(`C:\dev\kitty\docs\graphics-protocol.rst:408`,
`C:\dev\kitty\docs\graphics-protocol.rst:442`), image and placement IDs
(`C:\dev\kitty\docs\graphics-protocol.rst:480`,
`C:\dev\kitty\docs\graphics-protocol.rst:481`), Unicode placeholders
(`C:\dev\kitty\docs\graphics-protocol.rst:557`,
`C:\dev\kitty\docs\graphics-protocol.rst:561`), deletion
(`C:\dev\kitty\docs\graphics-protocol.rst:748`) and animation
(`C:\dev\kitty\docs\graphics-protocol.rst:860`).

The implementation mirrors that complexity. `GraphicsCommand` includes action,
transmission, deletion, ids, placement ids, dimensions, offsets, z-index and
payload size (`C:\dev\kitty\kitty\graphics.h:14`,
`C:\dev\kitty\kitty\graphics.h:15`, `C:\dev\kitty\kitty\graphics.h:16`,
`C:\dev\kitty\kitty\graphics.h:25`, `C:\dev\kitty\kitty\graphics.h:26`).
The command parser maps control keys including transmission type, placement id
and z-index, then dispatches to `screen_handle_graphics_command`
(`C:\dev\kitty\kitty\parse-graphics-command.h:23`,
`C:\dev\kitty\kitty\parse-graphics-command.h:24`,
`C:\dev\kitty\kitty\parse-graphics-command.h:30`,
`C:\dev\kitty\kitty\parse-graphics-command.h:44`,
`C:\dev\kitty\kitty\parse-graphics-command.h:395`). The manager enforces a
320 MB default storage limit, a 10000 pixel max image dimension, quota trimming
and sorted render data (`C:\dev\kitty\kitty\graphics.c:26`,
`C:\dev\kitty\kitty\graphics.c:27`, `C:\dev\kitty\kitty\graphics.c:281`,
`C:\dev\kitty\kitty\graphics.c:1210`, `C:\dev\kitty\kitty\graphics.c:1301`).

Hera implication: M1 should parse enough to not corrupt the stream and expose
image placeholders or ignored-payload events. Real transfer media, decode,
placement, animation and rendering belong after the headless core is credible.

## Keyboard Protocol And Input Encoding

Kitty keyboard is more relevant to Hera than Kitty graphics. The protocol
starts from a backward-compatible default and allows applications to opt into
progressive enhancement (`C:\dev\kitty\docs\keyboard-protocol.rst:23`,
`C:\dev\kitty\docs\keyboard-protocol.rst:83`). The short application sequence
is push flags on startup and pop on exit
(`C:\dev\kitty\docs\keyboard-protocol.rst:87`,
`C:\dev\kitty\docs\keyboard-protocol.rst:92`). Advanced key events use
`CSI unicode-key-code:alternate-key-codes ; modifiers:event-type ;
text-as-codepoints u` (`C:\dev\kitty\docs\keyboard-protocol.rst:138`).
Flags include disambiguation, event reporting, alternate keys, all keys and
text reporting (`C:\dev\kitty\docs\keyboard-protocol.rst:301`,
`C:\dev\kitty\docs\keyboard-protocol.rst:304`,
`C:\dev\kitty\docs\keyboard-protocol.rst:305`,
`C:\dev\kitty\docs\keyboard-protocol.rst:306`,
`C:\dev\kitty\docs\keyboard-protocol.rst:307`,
`C:\dev\kitty\docs\keyboard-protocol.rst:308`).

The implementation is a host-side concern. `encode_glfw_key_event` receives a
GLFW key event, cursor-key mode and current key encoding flags
(`C:\dev\kitty\kitty\key_encoding.c:423`). It maps flags to alternate key,
text reporting and embedded text behavior
(`C:\dev\kitty\kitty\key_encoding.c:430`,
`C:\dev\kitty\kitty\key_encoding.c:431`,
`C:\dev\kitty\kitty\key_encoding.c:432`). Then `keys.c` writes text or encoded
key bytes to the child process (`C:\dev\kitty\kitty\keys.c:184`,
`C:\dev\kitty\kitty\keys.c:186`, `C:\dev\kitty\kitty\keys.c:192`).

Hera implication: the core needs to expose modes and key encoding flags, but
input encoding should live outside `terminal-core`, probably in a future Rust
host helper crate.

## PTY And Process Lifecycle

Kitty's PTY path is Unix-shaped. Python opens the PTY with `os.openpty`, then
forks through C (`C:\dev\kitty\kitty\child.py:212`,
`C:\dev\kitty\kitty\child.py:337`, `C:\dev\kitty\kitty\child.py:342`).
The C function uses `fork`, sets up the child session and ends with `execvp`
(`C:\dev\kitty\kitty\child.c:84`, `C:\dev\kitty\kitty\child.c:114`,
`C:\dev\kitty\kitty\child.c:146`, `C:\dev\kitty\kitty\child.c:170`).
The child monitor uses `poll`, gates reads on parser input capacity, writes
pending output, handles `SIGCHLD`, and resizes PTYs through `TIOCSWINSZ`
(`C:\dev\kitty\kitty\child-monitor.c:622`,
`C:\dev\kitty\kitty\child-monitor.c:637`,
`C:\dev\kitty\kitty\child-monitor.c:1540`,
`C:\dev\kitty\kitty\child-monitor.c:1671`,
`C:\dev\kitty\kitty\child-monitor.c:1679`).

Hera implication: Kitty is useful for Unix event-loop pressure and resize
edge cases, but not for Windows. WezTerm `portable-pty` remains the better PTY
API model.

## Shell Integration And Semantics

Kitty has deep shell integration: prompts, command status, CWD reporting,
completion and remote helpers. The docs describe OSC 133 prompt marks
(`C:\dev\kitty\docs\shell-integration.rst:422`,
`C:\dev\kitty\docs\shell-integration.rst:430`,
`C:\dev\kitty\docs\shell-integration.rst:442`,
`C:\dev\kitty\docs\shell-integration.rst:448`). The bash integration emits
CWD OSC 7, prompt marks and command status markers
(`C:\dev\kitty\shell-integration\bash\kitty.bash:201`,
`C:\dev\kitty\shell-integration\bash\kitty.bash:207`,
`C:\dev\kitty\shell-integration\bash\kitty.bash:255`,
`C:\dev\kitty\shell-integration\bash\kitty.bash:258`). The screen side turns
OSC prompt/context/desktop/clipboard events into callbacks
(`C:\dev\kitty\kitty\screen.c:3124`,
`C:\dev\kitty\kitty\screen.c:3129`,
`C:\dev\kitty\kitty\screen.c:3150`,
`C:\dev\kitty\kitty\screen.c:3180`).

Hera implication: shell marks are valuable but should remain observer
metadata. They must not become required for terminal correctness.

## Tests And Fixture Value

Kitty has a broad compatibility suite:

- Parser tests cover charsets, parser threading, UTF-8, ESC/CSI/OSC/DCS/APC and
  graphics commands (`C:\dev\kitty\kitty_tests\parser.py:77`,
  `C:\dev\kitty\kitty_tests\parser.py:91`,
  `C:\dev\kitty\kitty_tests\parser.py:188`,
  `C:\dev\kitty\kitty_tests\parser.py:677`,
  `C:\dev\kitty\kitty_tests\parser.py:688`,
  `C:\dev\kitty\kitty_tests\parser.py:822`,
  `C:\dev\kitty\kitty_tests\parser.py:861`,
  `C:\dev\kitty\kitty_tests\parser.py:902`,
  `C:\dev\kitty\kitty_tests\parser.py:909`).
- Screen tests cover draw, Unicode clusters, resize, dirty lines, scrollback,
  hyperlinks, OSC 52, key encoding flags and prompt marking
  (`C:\dev\kitty\kitty_tests\screen.py:13`,
  `C:\dev\kitty\kitty_tests\screen.py:124`,
  `C:\dev\kitty\kitty_tests\screen.py:299`,
  `C:\dev\kitty\kitty_tests\screen.py:694`,
  `C:\dev\kitty\kitty_tests\screen.py:985`,
  `C:\dev\kitty\kitty_tests\screen.py:1109`,
  `C:\dev\kitty\kitty_tests\screen.py:1240`,
  `C:\dev\kitty\kitty_tests\screen.py:1264`,
  `C:\dev\kitty\kitty_tests\screen.py:1340`).
- Graphics tests cover load/query, temp files, put, layering, parents, Unicode
  placeholders, scroll, deletion, animation and quota behavior
  (`C:\dev\kitty\kitty_tests\graphics.py:403`,
  `C:\dev\kitty\kitty_tests\graphics.py:472`,
  `C:\dev\kitty\kitty_tests\graphics.py:618`,
  `C:\dev\kitty\kitty_tests\graphics.py:650`,
  `C:\dev\kitty\kitty_tests\graphics.py:666`,
  `C:\dev\kitty\kitty_tests\graphics.py:765`,
  `C:\dev\kitty\kitty_tests\graphics.py:995`,
  `C:\dev\kitty\kitty_tests\graphics.py:1062`,
  `C:\dev\kitty\kitty_tests\graphics.py:1154`,
  `C:\dev\kitty\kitty_tests\graphics.py:1298`).
- Key tests exercise key encoding, mouse encoding, shortcut mapping and
  alternate-key fallback (`C:\dev\kitty\kitty_tests\keys.py:17`,
  `C:\dev\kitty\kitty_tests\keys.py:492`,
  `C:\dev\kitty\kitty_tests\keys.py:518`,
  `C:\dev\kitty\kitty_tests\keys.py:696`,
  `C:\dev\kitty\kitty_tests\keys.py:816`).
- Multicell tests cover wide/multicell text and resize/reflow edge cases
  (`C:\dev\kitty\kitty_tests\multicell.py:14`,
  `C:\dev\kitty\kitty_tests\multicell.py:511`,
  `C:\dev\kitty\kitty_tests\multicell.py:658`).

Hera implication: once Hera has golden snapshots, translate a narrow set of
Kitty tests into fixtures. Do not import the whole suite blindly.

## What Hera Should Copy

Copy:

- Kitty protocol docs as backlog references.
- Keyboard protocol flags and progressive enhancement model.
- Safe payload limits for OSC/DCS/APC-like data.
- Dirty-line tracking and explicit dirty state.
- Resize/reflow edge-case awareness.
- Graphics placeholders as metadata, not images in M1.
- Query/reply semantics for protocols as host effects.
- Parser/screen tests as future fixture inspiration.

Do not copy:

- C as core implementation language.
- Python build/controller architecture for Hera internals.
- Go tools/kittens as a required runtime model.
- Objective-C source for macOS unless Rust bindings fail.
- OpenGL/GLFW as a core dependency.
- POSIX PTY assumptions as cross-platform design.
- Full Kitty graphics in M1.
- Monolithic `Screen` owning parser, renderer-facing data, graphics state and
  host callbacks.

## Recommended Hera Shape

Kitty reinforces this target:

```text
terminal-core
  Rust terminal state
  primary/alternate screens
  parser action application
  dirty/damage tracking
  safe unknown payload capture

terminal-protocol
  typed OSC/CSI/DCS/APC metadata
  keyboard protocol flags
  shell marker events
  graphics placeholders

terminal-render-model
  renderer-neutral cells
  styles
  cursor
  hyperlinks
  dirty rows
  image placeholder refs

terminal-pty
  portable Rust trait
  Unix PTY implementation
  Windows ConPTY implementation
  resize/backpressure/lifecycle policy

future input helper
  keyboard encoder
  mouse encoder
  mode-aware escape generation
```

Bottom line: Kitty is one of the best references for "what a modern terminal
eventually has to understand". It is not the shape Hera should implement
first. Use Kitty to define compatibility pressure, then keep Hera M1 smaller:
Rust core, parser boundary, snapshots, dirty rows, safe protocol metadata and
no image rendering promise.

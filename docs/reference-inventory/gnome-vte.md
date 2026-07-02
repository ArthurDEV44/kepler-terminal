# GNOME VTE Reference Inventory

Status: initial focused pass
Date: 2026-07-02
Reference source: `C:\dev\terminal-research\gnome-vte`
Requested path note: `C:\dev\vte` is `alacritty/vte` in this workspace, not
GNOME VTE. The GNOME VTE checkout audited here is
`C:\dev\terminal-research\gnome-vte`.
Scope: GNOME VTE lessons for Hera, especially parser structure, GTK widget
integration, terminal state, ring and scrollback storage, rewrap invariants,
Unix PTY handling, renderer boundary, OSC 8, OSC 133 and Sixel.

## Executive Takeaways

GNOME VTE is a high-value behavior oracle, not a stack template for Hera.

The high-value lessons:

1. VTE is explicitly a GTK terminal widget, not a headless engine API. Its
   README says it provides a virtual terminal widget for GTK applications
   (`C:\dev\terminal-research\gnome-vte\README.md:4`).
2. The implementation is C plus C++, built by Meson, with GTK, GLib, Pango,
   Cairo, FriBidi, ICU, PCRE2, LZ4 and simdutf dependencies
   (`C:\dev\terminal-research\gnome-vte\meson.build:157`,
   `C:\dev\terminal-research\gnome-vte\meson.build:165`,
   `C:\dev\terminal-research\gnome-vte\meson.build:664`,
   `C:\dev\terminal-research\gnome-vte\meson.build:667`,
   `C:\dev\terminal-research\gnome-vte\meson.build:669`,
   `C:\dev\terminal-research\gnome-vte\meson.build:672`,
   `C:\dev\terminal-research\gnome-vte\meson.build:673`).
3. The internal `Terminal` shape is useful: it owns a parser, PTY pointer,
   normal screen, alternate screen and `RingView`
   (`C:\dev\terminal-research\gnome-vte\src\vteinternal.hh:359`,
   `C:\dev\terminal-research\gnome-vte\src\vteinternal.hh:366`,
   `C:\dev\terminal-research\gnome-vte\src\vteinternal.hh:468`,
   `C:\dev\terminal-research\gnome-vte\src\vteinternal.hh:469`,
   `C:\dev\terminal-research\gnome-vte\src\vteinternal.hh:843`).
4. The public surface is a `GtkWidget` API with feed, PTY, scrollback and
   contents-writing methods, so it is not the shape Hera should expose
   (`C:\dev\terminal-research\gnome-vte\src\vte\vteterminal.h:59`,
   `C:\dev\terminal-research\gnome-vte\src\vte\vteterminal.h:159`,
   `C:\dev\terminal-research\gnome-vte\src\vte\vteterminal.h:213`,
   `C:\dev\terminal-research\gnome-vte\src\vte\vteterminal.h:362`,
   `C:\dev\terminal-research\gnome-vte\src\vte\vteterminal.h:604`).
5. The ring and rewrap documents are the most important Hera material. VTE
   treats rewrap as a row-based transformation with paragraph reconstruction,
   marker relocation and scroll position preservation
   (`C:\dev\terminal-research\gnome-vte\doc\rewrap.txt:279`,
   `C:\dev\terminal-research\gnome-vte\doc\rewrap.txt:286`,
   `C:\dev\terminal-research\gnome-vte\doc\rewrap.txt:344`,
   `C:\dev\terminal-research\gnome-vte\doc\rewrap.txt:378`).
6. VTE gives a crisp rule: normal screen is resized and rewrapped even while
   alternate screen is visible; alternate screen is not rewrapped
   (`C:\dev\terminal-research\gnome-vte\doc\rewrap.txt:109`,
   `C:\dev\terminal-research\gnome-vte\doc\rewrap.txt:117`,
   `C:\dev\terminal-research\gnome-vte\src\vte.cc:8342`,
   `C:\dev\terminal-research\gnome-vte\src\vte.cc:8345`).
7. The PTY path is POSIX-oriented. It is useful for Linux and macOS behavior,
   but not for Windows ConPTY architecture
   (`C:\dev\terminal-research\gnome-vte\src\pty.cc:425`,
   `C:\dev\terminal-research\gnome-vte\src\pty.cc:428`,
   `C:\dev\terminal-research\gnome-vte\src\pty.cc:299`,
   `C:\dev\terminal-research\gnome-vte\src\pty.cc:330`).
8. Renderer code confirms Hera's adapter rule. VTE uses `RingView`, Cairo,
   GTK4 GSK and PangoCairo, which belong outside Hera's headless core
   (`C:\dev\terminal-research\gnome-vte\src\ringview.hh:54`,
   `C:\dev\terminal-research\gnome-vte\src\drawing-cairo.hh:27`,
   `C:\dev\terminal-research\gnome-vte\src\drawing-gsk.hh:47`,
   `C:\dev\terminal-research\gnome-vte\src\fonts-pangocairo.cc:185`).
9. OSC 8 hyperlinks and OSC 133 shell integration are stored as cell/ring
   metadata, reinforcing Hera's semantic sidecar direction
   (`C:\dev\terminal-research\gnome-vte\src\vteseq.cc:1746`,
   `C:\dev\terminal-research\gnome-vte\src\vteseq.cc:1826`,
   `C:\dev\terminal-research\gnome-vte\src\attr.hh:24`,
   `C:\dev\terminal-research\gnome-vte\src\cell.hh:227`).
10. Sixel is optional and deeply stateful. It is useful for a future protocol
    backlog, not Hera M1 rendering
    (`C:\dev\terminal-research\gnome-vte\meson_options.txt:111`,
    `C:\dev\terminal-research\gnome-vte\src\vteseq.cc:5495`,
    `C:\dev\terminal-research\gnome-vte\src\sixel-parser.hh:165`,
    `C:\dev\terminal-research\gnome-vte\src\sixel-context.cc:463`).

Confidence: high for build shape, public API, internal terminal ownership,
parser, ring, rewrap, PTY, renderer, shell integration and Sixel findings
because this pass covered local README/build files, public headers, internal
terminal files, parser files, ring files, PTY files, renderer files, protocol
handlers and tests under `C:\dev\terminal-research\gnome-vte`. Medium for full
GTK host behavior because the app/widget layer was sampled only enough to
understand the boundary.

## Why GNOME VTE Matters

GNOME VTE matters because it is old enough to have accumulated terminal scars
that newer engines rediscover: scroll position after resize, alternate screen
resize policy, tab behavior during rewrap, ambiguous-width policy, DEC scrolling
region caveats, BiDi, shell integration, Sixel image payloads and API pressure
from real GTK applications.

The most reusable Hera asset is not the C API. It is the documented behavior
and internal shape behind it:

- A public GTK widget API (`C:\dev\terminal-research\gnome-vte\src\vte\vteterminal.h:59`).
- An internal `Terminal` class that owns parser, PTY, screens, ring view and
  drawing hooks (`C:\dev\terminal-research\gnome-vte\src\vteinternal.hh:243`,
  `C:\dev\terminal-research\gnome-vte\src\vteinternal.hh:359`,
  `C:\dev\terminal-research\gnome-vte\src\vteinternal.hh:366`,
  `C:\dev\terminal-research\gnome-vte\src\vteinternal.hh:470`,
  `C:\dev\terminal-research\gnome-vte\src\vteinternal.hh:843`).
- A row ring designed around freezing, streaming and rewrapping
  (`C:\dev\terminal-research\gnome-vte\src\ring.hh:52`,
  `C:\dev\terminal-research\gnome-vte\src\ring.hh:132`,
  `C:\dev\terminal-research\gnome-vte\src\ring.cc:1338`).
- A rewrap design doc that is more valuable than many implementations
  (`C:\dev\terminal-research\gnome-vte\doc\rewrap.txt:14`,
  `C:\dev\terminal-research\gnome-vte\doc\rewrap.txt:205`,
  `C:\dev\terminal-research\gnome-vte\doc\rewrap.txt:435`).

Hera implication: GNOME VTE should shape compatibility tests and invariants,
especially resize and scrollback behavior. It should not shape Hera's public
API, language stack or renderer stack.

## Codebase Map

| Area | Local path | What it contains | Hera relevance |
|---|---|---|---|
| Public API | `C:\dev\terminal-research\gnome-vte\src\vte` | `VteTerminal`, `VtePty`, regex and version headers. | API cautionary tale: mature but GTK-bound. |
| Widget bridge | `C:\dev\terminal-research\gnome-vte\src\widget.*`, `src\vtegtk.cc` | GObject/GTK integration, PTY binding, properties and signal surface. | Host adapter reference only. |
| Internal terminal | `C:\dev\terminal-research\gnome-vte\src\vteinternal.hh`, `src\vte.cc`, `src\vteseq.cc` | Parser, screens, PTY, processing loop, modes, drawing and sequence handlers. | Strong behavior reference, too coupled for Hera. |
| Parser | `C:\dev\terminal-research\gnome-vte\src\parser.*`, `parser-seq.py` | CSI, DCS, OSC, escape parsing and generated command tables. | Secondary parser oracle after `alacritty/vte`. |
| Ring and rows | `C:\dev\terminal-research\gnome-vte\src\ring.*`, `vterowdata.*`, `cell.hh` | Row storage, frozen streams, attrs, hyperlinks, images and rewrap. | Strong scrollback and rewrap design source. |
| PTY and spawn | `C:\dev\terminal-research\gnome-vte\src\pty.*`, `src\vtepty.cc`, `src\spawn.cc` | POSIX PTY open, resize, child setup and async spawn API. | Unix PTY behavior reference only. |
| Renderer | `C:\dev\terminal-research\gnome-vte\src\drawing-*`, `ringview.*`, `fonts-pangocairo.*` | Cairo, GTK4 GSK, RingView, BiDi and font metrics. | Confirms render-model adapter split. |
| Protocol extras | `C:\dev\terminal-research\gnome-vte\src\sixel-*`, `image.*`, `attr.hh` | Sixel parser/context, images, shell integration and attrs. | Backlog and metadata lessons. |
| Docs | `C:\dev\terminal-research\gnome-vte\doc` | Rewrap, ambiguous width and scrolling-region behavior notes. | High-value fixture specs. |
| Tests | `C:\dev\terminal-research\gnome-vte\src\*-test.cc`, `sixel-fuzzer.cc` | Parser, stream, tabstop, unicode width, UTF-8 and Sixel tests. | Fixture mining after M1. |

## Language And Platform Decision

GNOME VTE reinforces the existing Hera decision: stay Rust-first. VTE's
language mix is coherent for a GTK component, but it is not an argument for
bringing C, C++, C#, Objective-C or Zig into Hera's core.

| Hera zone | Language | GNOME VTE evidence | Decision |
|---|---|---|---|
| Terminal state core | Rust | VTE's internal terminal is C++ and tightly attached to GTK/GObject host logic (`C:\dev\terminal-research\gnome-vte\src\vteinternal.hh:243`, `C:\dev\terminal-research\gnome-vte\src\vtegtk.cc:3346`). | Keep Hera core Rust-only and headless. |
| VT parser | Rust wrapper | VTE has a mature parser and generated sequence tables (`C:\dev\terminal-research\gnome-vte\src\parser.hh:394`, `C:\dev\terminal-research\gnome-vte\src\parser-seq.py:981`). | Keep `alacritty/vte` first; use GNOME VTE as a test oracle. |
| Grid, rows and scrollback | Rust | VTE's ring stores rows, streams, attrs, hyperlinks and images (`C:\dev\terminal-research\gnome-vte\src\ring.hh:52`, `C:\dev\terminal-research\gnome-vte\src\ring.hh:132`, `C:\dev\terminal-research\gnome-vte\src\ring.hh:241`, `C:\dev\terminal-research\gnome-vte\src\ring.hh:259`). | Implement Rust storage with explicit memory budgets. |
| Resize and reflow | Rust | VTE documents normal vs alternate rewrap and implements ring marker relocation (`C:\dev\terminal-research\gnome-vte\doc\rewrap.txt:109`, `C:\dev\terminal-research\gnome-vte\doc\rewrap.txt:117`, `C:\dev\terminal-research\gnome-vte\src\ring.cc:1372`, `C:\dev\terminal-research\gnome-vte\src\ring.cc:1602`). | Copy invariants into fixtures, not code. |
| PTY public API | Rust traits | VTE exposes `VtePty` and POSIX PTY behavior (`C:\dev\terminal-research\gnome-vte\src\vte\vtepty.h:62`, `C:\dev\terminal-research\gnome-vte\src\pty.cc:425`). | Use Rust trait boundary; Unix and ConPTY are separate impls. |
| Windows ConPTY | Rust plus Win32 FFI | VTE has no ConPTY model in the audited PTY path; it opens POSIX PTYs with `posix_openpt` (`C:\dev\terminal-research\gnome-vte\src\pty.cc:428`). | Use Windows Terminal and WezTerm, not VTE, for ConPTY. |
| Linux and macOS PTY | Rust plus Unix syscalls | VTE uses `TIOCSWINSZ`, `TIOCGWINSZ` and POSIX child setup (`C:\dev\terminal-research\gnome-vte\src\pty.cc:184`, `C:\dev\terminal-research\gnome-vte\src\pty.cc:299`, `C:\dev\terminal-research\gnome-vte\src\pty.cc:330`). | Useful behavior source for Unix adapters. |
| Renderer model | Rust data model | VTE draws via Cairo/GSK/Pango and GTK widget callbacks (`C:\dev\terminal-research\gnome-vte\src\vte.cc:10075`, `C:\dev\terminal-research\gnome-vte\src\vte.cc:10110`). | Keep renderer-neutral snapshots in Rust. |
| Fonts and BiDi | Host adapter | VTE uses RingView and PangoCairo measurement (`C:\dev\terminal-research\gnome-vte\src\ringview.hh:54`, `C:\dev\terminal-research\gnome-vte\src\fonts-pangocairo.cc:185`). | Keep font shaping outside `terminal-core`. |
| Semantic session layer | Rust sidecar | OSC 133 maps into cell attributes (`C:\dev\terminal-research\gnome-vte\src\attr.hh:24`, `C:\dev\terminal-research\gnome-vte\src\cell.hh:227`). | Optional observer metadata, never terminal truth. |
| Image protocols | Rust metadata first | Sixel is optional and enters a separate data syntax (`C:\dev\terminal-research\gnome-vte\src\vteseq.cc:5627`, `C:\dev\terminal-research\gnome-vte\src\vteseq.cc:5642`). | Parse/cap/preserve before rendering. |

Bottom line: GNOME VTE is a compatibility and behavior reference. It does not
change the Hera language plan.

## Build And Platform Shape

VTE builds as a Meson project with C and C++ compiler checks
(`C:\dev\terminal-research\gnome-vte\meson.build:17`,
`C:\dev\terminal-research\gnome-vte\meson.build:157`,
`C:\dev\terminal-research\gnome-vte\meson.build:165`). It has GTK3 and GTK4 API
names (`C:\dev\terminal-research\gnome-vte\meson.build:79`,
`C:\dev\terminal-research\gnome-vte\meson.build:80`) and optional features for
FriBidi, GnuTLS, ICU, GTK3, GTK4, Sixel and systemd
(`C:\dev\terminal-research\gnome-vte\meson_options.txt:68`,
`C:\dev\terminal-research\gnome-vte\meson_options.txt:82`,
`C:\dev\terminal-research\gnome-vte\meson_options.txt:90`,
`C:\dev\terminal-research\gnome-vte\meson_options.txt:97`,
`C:\dev\terminal-research\gnome-vte\meson_options.txt:104`,
`C:\dev\terminal-research\gnome-vte\meson_options.txt:111`,
`C:\dev\terminal-research\gnome-vte\meson_options.txt:118`).

Core dependencies include Cairo, GLib, GObject, Pango, PCRE2, LZ4 and simdutf
(`C:\dev\terminal-research\gnome-vte\meson.build:664`,
`C:\dev\terminal-research\gnome-vte\meson.build:667`,
`C:\dev\terminal-research\gnome-vte\meson.build:669`,
`C:\dev\terminal-research\gnome-vte\meson.build:672`,
`C:\dev\terminal-research\gnome-vte\meson.build:673`). GTK dependencies are
conditional (`C:\dev\terminal-research\gnome-vte\meson.build:718`,
`C:\dev\terminal-research\gnome-vte\meson.build:724`).

Hera implication: VTE's build is a warning. If Hera imports GTK, Cairo,
Pango or GObject into its core, it stops being a portable engine and becomes a
platform toolkit component.

## Public API And Widget Boundary

`VteTerminal` is a GTK widget (`C:\dev\terminal-research\gnome-vte\src\vte\vteterminal.h:59`).
The API exposes widget construction, feeding bytes, PTY attachment, scrollback
configuration and contents writing
(`C:\dev\terminal-research\gnome-vte\src\vte\vteterminal.h:159`,
`C:\dev\terminal-research\gnome-vte\src\vte\vteterminal.h:213`,
`C:\dev\terminal-research\gnome-vte\src\vte\vteterminal.h:217`,
`C:\dev\terminal-research\gnome-vte\src\vte\vteterminal.h:362`,
`C:\dev\terminal-research\gnome-vte\src\vte\vteterminal.h:568`,
`C:\dev\terminal-research\gnome-vte\src\vte\vteterminal.h:604`).

The `Widget` bridge owns GTK and PTY references and forwards PTY attachment to
the internal terminal (`C:\dev\terminal-research\gnome-vte\src\widget.hh:347`,
`C:\dev\terminal-research\gnome-vte\src\widget.hh:799`,
`C:\dev\terminal-research\gnome-vte\src\widget.hh:820`,
`C:\dev\terminal-research\gnome-vte\src\widget.cc:2226`,
`C:\dev\terminal-research\gnome-vte\src\widget.cc:2232`).

Hera implication: expose a headless `Terminal` API first. GUI, PTY and widget
ownership should be adapter concerns.

## Internal Terminal State

VTE's internal `Terminal` class is the useful shape hidden behind the GTK API.
It owns:

- `vte::parser::Parser` for control sequence parsing
  (`C:\dev\terminal-research\gnome-vte\src\vteinternal.hh:359`).
- `vte::base::Pty` and PTY IO source state
  (`C:\dev\terminal-research\gnome-vte\src\vteinternal.hh:366`,
  `C:\dev\terminal-research\gnome-vte\src\vteinternal.hh:372`,
  `C:\dev\terminal-research\gnome-vte\src\vteinternal.hh:375`).
- Normal and alternate screens with an active screen pointer
  (`C:\dev\terminal-research\gnome-vte\src\vteinternal.hh:468`,
  `C:\dev\terminal-research\gnome-vte\src\vteinternal.hh:469`,
  `C:\dev\terminal-research\gnome-vte\src\vteinternal.hh:470`).
- Rewrap policy and `RingView`
  (`C:\dev\terminal-research\gnome-vte\src\vteinternal.hh:509`,
  `C:\dev\terminal-research\gnome-vte\src\vteinternal.hh:843`).
- Processing, feed and widget draw/snapshot methods
  (`C:\dev\terminal-research\gnome-vte\src\vteinternal.hh:1041`,
  `C:\dev\terminal-research\gnome-vte\src\vteinternal.hh:1142`,
  `C:\dev\terminal-research\gnome-vte\src\vteinternal.hh:1144`,
  `C:\dev\terminal-research\gnome-vte\src\vteinternal.hh:1261`).

VTE initializes the normal screen with scrollback streams and the alternate
screen without those streams (`C:\dev\terminal-research\gnome-vte\src\vte.cc:8412`,
`C:\dev\terminal-research\gnome-vte\src\vte.cc:8413`,
`C:\dev\terminal-research\gnome-vte\src\vte.cc:8414`).

Hera implication: copy the conceptual ownership. Reject the GTK object shape.
Hera's core object can own parser plus terminal state, but must not own PTY,
renderer or widget state.

## Parser Boundary

VTE has its own parser. The parser comments describe the main control sequence
parser, while `Parser::feed` advances raw codepoints through states such as CSI,
DCS and OSC (`C:\dev\terminal-research\gnome-vte\src\parser.hh:216`,
`C:\dev\terminal-research\gnome-vte\src\parser.hh:394`,
`C:\dev\terminal-research\gnome-vte\src\parser.hh:421`,
`C:\dev\terminal-research\gnome-vte\src\parser.hh:425`,
`C:\dev\terminal-research\gnome-vte\src\parser.hh:427`).

Dispatch methods parse host CSI, DCS and OSC commands
(`C:\dev\terminal-research\gnome-vte\src\parser.hh:932`,
`C:\dev\terminal-research\gnome-vte\src\parser.hh:953`,
`C:\dev\terminal-research\gnome-vte\src\parser.hh:1095`,
`C:\dev\terminal-research\gnome-vte\src\parser.cc:323`). Sequence definitions
are generated from `parser-seq.py`, including unripe DECSIXEL DCS handling
(`C:\dev\terminal-research\gnome-vte\src\parser-seq.py:981`).

Hera implication: `alacritty/vte` remains the parser seed because it is Rust
and already shaped as a parser crate. GNOME VTE should become a secondary oracle
for ambiguous behavior, command coverage, DCS/OSC handling and Sixel edge cases.

## Ring, Rows And Scrollback

Each `VteScreen` owns a `Ring`, and `row_data` points at that ring
(`C:\dev\terminal-research\gnome-vte\src\vteinternal.hh:146`,
`C:\dev\terminal-research\gnome-vte\src\vteinternal.hh:150`,
`C:\dev\terminal-research\gnome-vte\src\vteinternal.hh:151`). `Ring` exposes
containment, row indexing, insertion, append, removal, scrollback drop, visible
rows and rewrap (`C:\dev\terminal-research\gnome-vte\src\ring.hh:52`,
`C:\dev\terminal-research\gnome-vte\src\ring.hh:73`,
`C:\dev\terminal-research\gnome-vte\src\ring.hh:101`,
`C:\dev\terminal-research\gnome-vte\src\ring.hh:102`,
`C:\dev\terminal-research\gnome-vte\src\ring.hh:104`).

The ring stores row records and streams. Row records point into text and attr
streams, and the ring also tracks hyperlinks and image maps
(`C:\dev\terminal-research\gnome-vte\src\ring.hh:132`,
`C:\dev\terminal-research\gnome-vte\src\ring.hh:152`,
`C:\dev\terminal-research\gnome-vte\src\ring.hh:241`,
`C:\dev\terminal-research\gnome-vte\src\ring.hh:259`,
`C:\dev\terminal-research\gnome-vte\src\ring.hh:279`). Row data and cell attrs
are packed tightly, with explicit layout assertions
(`C:\dev\terminal-research\gnome-vte\src\vterowdata.hh:44`,
`C:\dev\terminal-research\gnome-vte\src\vterowdata.cc:84`,
`C:\dev\terminal-research\gnome-vte\src\cell.hh:127`,
`C:\dev\terminal-research\gnome-vte\src\cell.hh:238`,
`C:\dev\terminal-research\gnome-vte\src\cell.hh:261`).

Hera implication: M1 can start simpler than VTE, but the public model should
not assume raw vector row ownership. Stable row handles, chunking and memory
budgets should be designed before huge scrollback becomes a rewrite.

## Resize And Rewrap

VTE's rewrap document is one of the strongest references in the research set.
It states that terminals remember soft wrapping for copy-paste, that tabs behave
like cursor positioning for rewrap purposes, and that scroll position after a
resize is tricky (`C:\dev\terminal-research\gnome-vte\doc\rewrap.txt:20`,
`C:\dev\terminal-research\gnome-vte\doc\rewrap.txt:63`,
`C:\dev\terminal-research\gnome-vte\doc\rewrap.txt:78`,
`C:\dev\terminal-research\gnome-vte\doc\rewrap.txt:205`).

The key policy is explicit:

- Normal screen should always be resized and rewrapped, even if alternate
  screen is visible (`C:\dev\terminal-research\gnome-vte\doc\rewrap.txt:109`).
- Alternate screen should not be rewrapped
  (`C:\dev\terminal-research\gnome-vte\doc\rewrap.txt:117`).
- Active and saved cursors should be updated through rewrap
  (`C:\dev\terminal-research\gnome-vte\doc\rewrap.txt:129`).
- Infinite scrollback can make rewrap slow, so VTE exposes
  `vte_terminal_set_rewrap_on_resize`
  (`C:\dev\terminal-research\gnome-vte\doc\rewrap.txt:435`,
  `C:\dev\terminal-research\gnome-vte\doc\rewrap.txt:439`).

The implementation follows that policy. `Terminal::set_size` resizes the PTY,
updates visible rows, rewraps the normal screen with `m_rewrap_on_resize`, and
resizes the alternate screen without rewrap
(`C:\dev\terminal-research\gnome-vte\src\vte.cc:8303`,
`C:\dev\terminal-research\gnome-vte\src\vte.cc:8322`,
`C:\dev\terminal-research\gnome-vte\src\vte.cc:8338`,
`C:\dev\terminal-research\gnome-vte\src\vte.cc:8342`,
`C:\dev\terminal-research\gnome-vte\src\vte.cc:8345`).

`Ring::rewrap` freezes rows, maps row/column markers into text offsets, rebuilds
row records, maps markers back, and rebuilds image top maps
(`C:\dev\terminal-research\gnome-vte\src\ring.cc:1338`,
`C:\dev\terminal-research\gnome-vte\src\ring.cc:1370`,
`C:\dev\terminal-research\gnome-vte\src\ring.cc:1372`,
`C:\dev\terminal-research\gnome-vte\src\ring.cc:1501`,
`C:\dev\terminal-research\gnome-vte\src\ring.cc:1602`,
`C:\dev\terminal-research\gnome-vte\src\ring.cc:1628`).

Hera implication: reflow must be fixture-first. The engine should track wrap
metadata, cursor markers, saved cursor markers, viewport anchors and semantic
row handles before reflow is treated as reliable.

## PTY And Process Lifecycle

VTE's PTY layer is useful for Unix behavior. `Pty` owns an fd and exposes child
setup, size get/set and creation helpers
(`C:\dev\terminal-research\gnome-vte\src\pty.hh:30`,
`C:\dev\terminal-research\gnome-vte\src\pty.hh:57`,
`C:\dev\terminal-research\gnome-vte\src\pty.hh:59`,
`C:\dev\terminal-research\gnome-vte\src\pty.hh:67`). The implementation opens
POSIX PTYs with `posix_openpt`, includes fallback behavior, configures child
session setup, and resizes with `TIOCSWINSZ`
(`C:\dev\terminal-research\gnome-vte\src\pty.cc:184`,
`C:\dev\terminal-research\gnome-vte\src\pty.cc:281`,
`C:\dev\terminal-research\gnome-vte\src\pty.cc:299`,
`C:\dev\terminal-research\gnome-vte\src\pty.cc:425`,
`C:\dev\terminal-research\gnome-vte\src\pty.cc:428`).

The public `VtePty` API exposes creation, child setup, size handling and spawn
async methods (`C:\dev\terminal-research\gnome-vte\src\vte\vtepty.h:62`,
`C:\dev\terminal-research\gnome-vte\src\vte\vtepty.h:75`,
`C:\dev\terminal-research\gnome-vte\src\vte\vtepty.h:95`,
`C:\dev\terminal-research\gnome-vte\src\vte\vtepty.h:109`,
`C:\dev\terminal-research\gnome-vte\src\vtepty.cc:477`).

Hera implication: use VTE for Unix PTY lifecycle details, not cross-platform
PTY architecture. Windows ConPTY must be learned from Windows Terminal, WezTerm
and Contour.

## Renderer, Fonts And RingView

VTE's renderer is not portable as a core dependency, but its boundary lessons
are useful. `RingView` provides a view over a continuous ring segment and is
updated with ring, rows and width
(`C:\dev\terminal-research\gnome-vte\src\ringview.hh:54`,
`C:\dev\terminal-research\gnome-vte\src\ringview.hh:69`,
`C:\dev\terminal-research\gnome-vte\src\ringview.hh:70`,
`C:\dev\terminal-research\gnome-vte\src\ringview.hh:71`,
`C:\dev\terminal-research\gnome-vte\src\ringview.hh:78`). Drawing has Cairo
and GTK4 GSK paths
(`C:\dev\terminal-research\gnome-vte\src\drawing-cairo.hh:27`,
`C:\dev\terminal-research\gnome-vte\src\drawing-cairo.hh:73`,
`C:\dev\terminal-research\gnome-vte\src\drawing-gsk.hh:47`,
`C:\dev\terminal-research\gnome-vte\src\drawing-gsk.hh:57`).

Terminal drawing updates the ring view, draws rows, supports GTK3 cairo draw and
GTK4 snapshot paths
(`C:\dev\terminal-research\gnome-vte\src\vte.cc:9426`,
`C:\dev\terminal-research\gnome-vte\src\vte.cc:9445`,
`C:\dev\terminal-research\gnome-vte\src\vte.cc:10075`,
`C:\dev\terminal-research\gnome-vte\src\vte.cc:10110`,
`C:\dev\terminal-research\gnome-vte\src\vte.cc:10188`).

Hera implication: keep the render model serializable and renderer-neutral.
BiDi, shaping, font metrics and toolkit drawing belong to adapters.

## Semantic Metadata And Protocol Extras

VTE stores OSC 8 hyperlinks through ring/cell metadata. `set_current_hyperlink`
handles OSC 8 and updates the default cell hyperlink index
(`C:\dev\terminal-research\gnome-vte\src\vteseq.cc:1746`,
`C:\dev\terminal-research\gnome-vte\src\vteseq.cc:1753`,
`C:\dev\terminal-research\gnome-vte\src\vteseq.cc:1811`,
`C:\dev\terminal-research\gnome-vte\src\vteseq.cc:1822`). The ring owns a
hyperlink pool (`C:\dev\terminal-research\gnome-vte\src\ring.hh:120`,
`C:\dev\terminal-research\gnome-vte\src\ring.hh:241`).

Shell integration is represented as attributes. `ShellIntegrationMode` maps
OSC 133 A/B/C to prompt, command and normal/output regions
(`C:\dev\terminal-research\gnome-vte\src\attr.hh:24`,
`C:\dev\terminal-research\gnome-vte\src\attr.hh:25`,
`C:\dev\terminal-research\gnome-vte\src\attr.hh:30`,
`C:\dev\terminal-research\gnome-vte\src\attr.hh:32`). The OSC dispatcher routes
iTerm2 shell integration to `set_current_shell_integration_mode`
(`C:\dev\terminal-research\gnome-vte\src\vteseq.cc:1826`,
`C:\dev\terminal-research\gnome-vte\src\vteseq.cc:7920`,
`C:\dev\terminal-research\gnome-vte\src\vteseq.cc:7921`).

Sixel is optional and uses a separate parser/context/image path
(`C:\dev\terminal-research\gnome-vte\src\meson.build:334`,
`C:\dev\terminal-research\gnome-vte\src\sixel-parser.hh:165`,
`C:\dev\terminal-research\gnome-vte\src\sixel-context.cc:351`,
`C:\dev\terminal-research\gnome-vte\src\sixel-context.cc:463`,
`C:\dev\terminal-research\gnome-vte\src\image.hh:28`,
`C:\dev\terminal-research\gnome-vte\src\ring.cc:1741`).

Hera implication: store protocol metadata in Rust-side typed events and cell
spans. Do not make shell integration authoritative. Do not implement image
rendering before parser, storage, resource caps and snapshot semantics are
stable.

## Tests And Fixture Mining

VTE has useful unit and fuzz targets wired through Meson:

- Parser tests
  (`C:\dev\terminal-research\gnome-vte\src\meson.build:854`,
  `C:\dev\terminal-research\gnome-vte\src\meson.build:864`).
- Sixel fuzzer and tests
  (`C:\dev\terminal-research\gnome-vte\src\meson.build:931`,
  `C:\dev\terminal-research\gnome-vte\src\meson.build:938`,
  `C:\dev\terminal-research\gnome-vte\src\meson.build:949`).
- Stream, tabstop, unicode-width and UTF-8 tests
  (`C:\dev\terminal-research\gnome-vte\src\meson.build:960`,
  `C:\dev\terminal-research\gnome-vte\src\meson.build:987`,
  `C:\dev\terminal-research\gnome-vte\src\meson.build:1029`,
  `C:\dev\terminal-research\gnome-vte\src\meson.build:1043`).

The docs are also fixture sources:

- Rewrap normal/alternate/cursor/scroll behavior
  (`C:\dev\terminal-research\gnome-vte\doc\rewrap.txt:106`,
  `C:\dev\terminal-research\gnome-vte\doc\rewrap.txt:126`,
  `C:\dev\terminal-research\gnome-vte\doc\rewrap.txt:205`).
- Ambiguous width policy
  (`C:\dev\terminal-research\gnome-vte\doc\ambiguous.txt:29`,
  `C:\dev\terminal-research\gnome-vte\doc\ambiguous.txt:41`).
- Scrolling region edge behavior
  (`C:\dev\terminal-research\gnome-vte\doc\scrolling-region.txt:13`,
  `C:\dev\terminal-research\gnome-vte\doc\scrolling-region.txt:51`,
  `C:\dev\terminal-research\gnome-vte\doc\scrolling-region.txt:118`).

Hera implication: convert VTE docs into explicit golden cases instead of
lifting C++ code. Rewrap, ambiguous width and scroll region behavior should be
tested before the renderer exists.

## What Hera Should Copy

Copy these ideas:

1. Normal screen and alternate screen are separate states with different
   scrollback and rewrap rules.
2. Rewrap needs explicit markers for cursor, saved cursor, viewport and
   paragraph boundaries.
3. Scrollback storage needs row identity and memory policy, not just a vector of
   visible lines.
4. Renderer output should come from a view/snapshot over terminal state, not
   from toolkit callbacks leaking into core.
5. OSC 8 and OSC 133 should be metadata on spans/rows/events.
6. Unknown or heavy protocols such as Sixel need parser/resource boundaries
   before drawing.
7. Behavior docs can become fixtures, especially rewrap and scrolling-region
   docs.

## What Hera Should Avoid

Avoid these choices:

1. Do not expose a GTK-style widget API as the core API.
2. Do not import GLib, GObject, GTK, Cairo or Pango into `terminal-core`.
3. Do not use GNOME VTE as Windows PTY evidence. It does not solve ConPTY.
4. Do not treat rewrap as a local resize detail. It touches scrollback, cursor,
   saved cursor, viewport, marks, shell integration spans and images.
5. Do not make semantic shell integration authoritative.
6. Do not implement Sixel rendering in M1.

## Recommended Hera Shape

GNOME VTE supports this shape:

- `terminal-core`: Rust terminal state, normal/alternate screens, row storage,
  modes, cursor, wrap metadata, reflow and snapshots.
- `terminal-protocol`: parser-normalized actions, OSC 8, OSC 133, DCS payloads,
  image placeholders and semantic events.
- `terminal-pty`: Rust PTY trait with Unix and Windows implementations learned
  from different references.
- `terminal-render-model`: ring/viewport snapshot, damage, cells, spans,
  cursor, selection and image placeholders.
- `terminal-fixtures`: VTE-derived rewrap, ambiguous width, scrolling-region,
  parser, UTF-8, tabstop and Sixel cases.

Final decision: GNOME VTE is a major compatibility reference. It strengthens
Hera's need for a Rust headless core with strong fixtures and narrow adapters.
It does not justify adding C, C++, GTK or GObject to Hera's implementation.

# vt100-rust Reference Inventory

Status: initial focused pass
Date: 2026-07-03
Reference source: `C:\dev\vt100-rust`
Scope: vt100-rust lessons for Hera, especially Rust headless parsing, `vte`
integration, in-memory screen state, formatted output, diff output, fixture
generation, property/replay testing and small-core limits.

## Executive Takeaways

vt100-rust is a strong minimal Rust reference for parser-to-screen wiring,
state reproduction and fixture design. It is not a complete terminal engine
blueprint for Hera.

The high-value lessons:

1. The crate's thesis is close to Hera M1: parse a terminal byte stream and
   expose an in-memory rendered representation
   (`C:\dev\vt100-rust\README.md:3`, `C:\dev\vt100-rust\src\lib.rs:1`).
2. It wraps `vte::Parser` directly and owns a screen performer. `Parser`
   stores `vte::Parser` plus `WrappedScreen`, processes bytes with
   `advance`, exposes immutable and mutable `Screen`, and implements
   `std::io::Write` (`C:\dev\vt100-rust\src\parser.rs:3`,
   `C:\dev\vt100-rust\src\parser.rs:48`,
   `C:\dev\vt100-rust\src\parser.rs:55`,
   `C:\dev\vt100-rust\src\parser.rs:87`).
3. `WrappedScreen` implements `vte::Perform`, mapping parser callbacks into
   terminal state mutations and callback side effects
   (`C:\dev\vt100-rust\src\perform.rs:5`,
   `C:\dev\vt100-rust\src\perform.rs:33`,
   `C:\dev\vt100-rust\src\perform.rs:34`,
   `C:\dev\vt100-rust\src\perform.rs:87`,
   `C:\dev\vt100-rust\src\perform.rs:198`).
4. The screen model explicitly stores primary and alternate grids, with
   alternate scrollback set to zero (`C:\dev\vt100-rust\src\screen.rs:55`,
   `C:\dev\vt100-rust\src\screen.rs:76`,
   `C:\dev\vt100-rust\src\screen.rs:651`,
   `C:\dev\vt100-rust\src\screen.rs:1139`,
   `C:\dev\vt100-rust\src\screen.rs:1176`).
5. The best API idea is reproduction by bytes: `state_formatted`,
   `state_diff`, `contents_formatted`, `contents_diff`, `input_mode_formatted`
   and `input_mode_diff` return byte streams sufficient to recreate state or
   move from a previous state (`C:\dev\vt100-rust\src\screen.rs:224`,
   `C:\dev\vt100-rust\src\screen.rs:236`,
   `C:\dev\vt100-rust\src\screen.rs:249`,
   `C:\dev\vt100-rust\src\screen.rs:311`,
   `C:\dev\vt100-rust\src\screen.rs:379`,
   `C:\dev\vt100-rust\src\screen.rs:412`).
6. The fixture model is excellent for Hera. Fixtures are progressive
   `.typescript` byte chunks plus JSON snapshots, and assertions check text,
   cursor, modes, cells, wide flags and attributes
   (`C:\dev\vt100-rust\tests\helpers\fixtures.rs:56`,
   `C:\dev\vt100-rust\tests\helpers\fixtures.rs:90`,
   `C:\dev\vt100-rust\tests\helpers\fixtures.rs:231`,
   `C:\dev\vt100-rust\tests\helpers\fixtures.rs:248`,
   `C:\dev\vt100-rust\tests\helpers\fixtures.rs:300`).
7. The verification harness is unusually relevant: it checks that formatted
   output and diff output reproduce the same screen state
   (`C:\dev\vt100-rust\tests\helpers\mod.rs:130`,
   `C:\dev\vt100-rust\tests\helpers\mod.rs:188`,
   `C:\dev\vt100-rust\tests\helpers\mod.rs:230`).
8. The test matrix includes fixture suites, split escape/UTF-8 cases, unknown
   OSC, scrollback, a 7,625-frame crawl replay and QuickCheck properties
   (`C:\dev\vt100-rust\tests\processing.rs:5`,
   `C:\dev\vt100-rust\tests\processing.rs:10`,
   `C:\dev\vt100-rust\tests\osc.rs:83`,
   `C:\dev\vt100-rust\tests\scroll.rs:16`,
   `C:\dev\vt100-rust\tests\window_contents.rs:550`,
   `C:\dev\vt100-rust\tests\quickcheck.rs:127`).
9. The limits are clear: simple `VecDeque<Row>` scrollback, no PTY layer, no
   renderer abstraction, no huge-history paging, no semantic layer and no
   advanced image protocol model (`C:\dev\vt100-rust\src\grid.rs:13`,
   `C:\dev\vt100-rust\src\grid.rs:66`,
   `C:\dev\vt100-rust\src\grid.rs:561`).
10. The AFL harness in this checkout is not current: it still references
    removed `bells_diff` and `title_formatted` APIs, while the changelog says
    those screen methods moved to callbacks
    (`C:\dev\vt100-rust\fuzz\src\main.rs:7`,
    `C:\dev\vt100-rust\fuzz\src\main.rs:70`,
    `C:\dev\vt100-rust\CHANGELOG.md:30`,
    `C:\dev\vt100-rust\CHANGELOG.md:36`).

Confidence: high for parser, screen, grid, formatted/diff output, callbacks,
fixtures and replay testing because this pass covered README, Cargo manifest,
public crate docs, core source files, test helpers, representative tests,
examples, property tests and stale fuzz harness under `C:\dev\vt100-rust`.
Medium for full
compatibility because the pass did not execute the complete test suite and the
AFL harness appears stale.

## Why vt100-rust Matters

vt100-rust matters because it is the smallest Rust reference that already
does what Hera M1 needs to prove: consume terminal bytes, keep headless screen
state, inspect cells, and emit a byte stream that can reconstruct visible
state. Its README calls it the parser component of a graphical terminal
emulator extracted into a crate, useful for programs like `screen` or `tmux`
(`C:\dev\vt100-rust\README.md:8`).

The implementation uses `vte = "0.15.0"` and `unicode-width = "0.2.1"`
(`C:\dev\vt100-rust\Cargo.toml:19`,
`C:\dev\vt100-rust\Cargo.toml:20`). That makes it especially useful as a
local proof that wrapping `vte` is enough to start, but not enough to finish.

Hera implication: vt100-rust should influence the first headless API and test
harness more than the final storage or rendering architecture.

## Codebase Map

| Area | Local path | What it contains | Hera relevance |
|---|---|---|---|
| Public API | `C:\dev\vt100-rust\src\lib.rs`, `src\parser.rs`, `src\screen.rs` | `Parser`, `Screen`, `Cell`, callbacks and public inspection methods. | Good M1 headless API reference. |
| Parser bridge | `C:\dev\vt100-rust\src\perform.rs` | `vte::Perform` implementation mapping print, execute, ESC, CSI and OSC into screen mutation. | Direct model for Hera's first action layer. |
| Screen state | `C:\dev\vt100-rust\src\screen.rs` | Primary/alternate grids, modes, cursor, attributes, mouse protocol state. | Small but useful state boundary. |
| Storage | `C:\dev\vt100-rust\src\grid.rs`, `src\row.rs`, `src\cell.rs` | Vec rows, VecDeque scrollback, 32-byte cells, row diff rendering. | M1 reference only, not huge scrollback design. |
| Callbacks | `C:\dev\vt100-rust\src\callbacks.rs` | Host side effects for bells, resize, title, clipboard and unhandled sequences. | Useful split between core state and host effects. |
| Fixtures | `C:\dev\vt100-rust\tests\helpers`, `tests\data\fixtures` | Progressive byte chunks plus JSON state snapshots. | Strong fixture format for Hera. |
| Crawl and properties | `C:\dev\vt100-rust\tests\data\crawl`, `tests\quickcheck.rs`, `fuzz` | Long replay corpus, QuickCheck properties and a stale AFL harness. | Good regression pressure, but repair AFL before reuse. |

## Parser And Performer Boundary

`Parser` is intentionally thin. It owns a `vte::Parser` and a `WrappedScreen`,
and `process` forwards bytes to `vte::Parser::advance`
(`C:\dev\vt100-rust\src\parser.rs:3`,
`C:\dev\vt100-rust\src\parser.rs:48`). The screen can be read or mutated
directly through `screen` and `screen_mut`
(`C:\dev\vt100-rust\src\parser.rs:55`,
`C:\dev\vt100-rust\src\parser.rs:62`).

`WrappedScreen` is the terminal semantic layer. It implements `vte::Perform`
and dispatches parser callbacks into screen operations: printable text goes to
`screen.text`, control bytes map to control actions, ESC/CSI dispatch maps to
named screen methods, and OSC 52 is intercepted for clipboard callbacks
(`C:\dev\vt100-rust\src\perform.rs:33`,
`C:\dev\vt100-rust\src\perform.rs:34`,
`C:\dev\vt100-rust\src\perform.rs:42`,
`C:\dev\vt100-rust\src\perform.rs:58`,
`C:\dev\vt100-rust\src\perform.rs:87`,
`C:\dev\vt100-rust\src\perform.rs:198`,
`C:\dev\vt100-rust\src\perform.rs:210`).

Hera implication: wrap `vte` behind a Hera-owned performer or action layer.
The parser must not be the public architecture.

## Screen State

`Screen` owns a primary `grid`, an `alternate_grid`, attributes, saved
attributes, mode bits and mouse protocol state
(`C:\dev\vt100-rust\src\screen.rs:55`,
`C:\dev\vt100-rust\src\screen.rs:56`,
`C:\dev\vt100-rust\src\screen.rs:57`). The alternate grid is created with zero
scrollback (`C:\dev\vt100-rust\src\screen.rs:76`), and the active grid is
selected by `MODE_ALTERNATE_SCREEN`
(`C:\dev\vt100-rust\src\screen.rs:635`,
`C:\dev\vt100-rust\src\screen.rs:643`).

DECSET/DECRST implement mode toggles including application cursor, origin mode,
mouse modes, alternate screen 47, alternate screen 1049 and bracketed paste
(`C:\dev\vt100-rust\src\screen.rs:1139`,
`C:\dev\vt100-rust\src\screen.rs:1150`,
`C:\dev\vt100-rust\src\screen.rs:1164`,
`C:\dev\vt100-rust\src\screen.rs:1176`,
`C:\dev\vt100-rust\src\screen.rs:1188`,
`C:\dev\vt100-rust\src\screen.rs:1205`). The public API exposes alternate
screen, cursor, cell and input mode state
(`C:\dev\vt100-rust\src\screen.rs:489`,
`C:\dev\vt100-rust\src\screen.rs:534`,
`C:\dev\vt100-rust\src\screen.rs:548`).

Hera implication: this is a compact reference for the M1 state surface. Add
more explicit typed state in Hera, but keep the active/alternate split equally
plain.

## Snapshot, Diff And Replay

The strongest vt100-rust idea is that the screen can emit terminal bytes that
reproduce itself. `state_formatted` combines visible content plus input modes,
while `state_diff` combines content diff plus input-mode diff
(`C:\dev\vt100-rust\src\screen.rs:224`,
`C:\dev\vt100-rust\src\screen.rs:236`). `contents_formatted` emits full
visible content, and `contents_diff` emits bytes to transform a prior screen
into the current one (`C:\dev\vt100-rust\src\screen.rs:249`,
`C:\dev\vt100-rust\src\screen.rs:311`).

The helper tests enforce this contract. `contents_formatted_reproduces_screen`
asserts that formatted bytes equal `state_formatted` after input modes, then
parses those bytes into a new parser and compares screens
(`C:\dev\vt100-rust\tests\helpers\mod.rs:130`,
`C:\dev\vt100-rust\tests\helpers\mod.rs:133`). The diff path builds
`contents_diff + input_mode_diff`, asserts equality with `state_diff`, replays
from the previous formatted screen, and compares the result
(`C:\dev\vt100-rust\tests\helpers\mod.rs:188`,
`C:\dev\vt100-rust\tests\helpers\mod.rs:194`,
`C:\dev\vt100-rust\tests\helpers\mod.rs:200`).

Hera implication: expose a typed snapshot, but also keep a byte-level replay
oracle. A good Hera fixture should prove `snapshot -> render model`,
`bytes -> snapshot` and `diff(prev, next) -> next`.

## Grid, Rows And Cells

`Grid` stores current rows, a `VecDeque<Row>` scrollback, scrollback length and
scrollback offset (`C:\dev\vt100-rust\src\grid.rs:4`,
`C:\dev\vt100-rust\src\grid.rs:13`,
`C:\dev\vt100-rust\src\grid.rs:14`). Visible rows are assembled by combining
scrollback rows and drawing rows (`C:\dev\vt100-rust\src\grid.rs:126`).
Scrolling pushes removed rows into scrollback and trims to `scrollback_len`
when no scroll region is active (`C:\dev\vt100-rust\src\grid.rs:561`,
`C:\dev\vt100-rust\src\grid.rs:566`,
`C:\dev\vt100-rust\src\grid.rs:568`).

Rows know wrap state and can render full or diff output
(`C:\dev\vt100-rust\src\row.rs:4`,
`C:\dev\vt100-rust\src\row.rs:78`,
`C:\dev\vt100-rust\src\row.rs:137`,
`C:\dev\vt100-rust\src\row.rs:284`). Cells are compact 32-byte values with a
fixed inline content buffer, width flags and attributes
(`C:\dev\vt100-rust\src\cell.rs:4`,
`C:\dev\vt100-rust\src\cell.rs:12`,
`C:\dev\vt100-rust\src\cell.rs:17`).

Hera implication: the storage is good enough for an M1 mental model, but not
for Paneflow-grade huge scrollback. Hera needs chunking/page IDs and memory
budgets earlier than vt100-rust does.

## Resize And Reflow

`Screen::set_size` resizes both grids, and `Grid::set_size` resizes row width
and row count (`C:\dev\vt100-rust\src\screen.rs:88`,
`C:\dev\vt100-rust\src\grid.rs:66`). When column count changes, the grid clears
row wrap flags instead of performing full scrollback reflow
(`C:\dev\vt100-rust\src\grid.rs:67`). This is intentionally smaller than the
Alacritty, WezTerm, Contour, Windows Terminal and GNOME VTE reflow models.

Hera implication: do not copy resize behavior as final architecture. Use
vt100-rust resize tests as sanity checks, not as the compatibility target for
reflow.

## Host Effects And Callbacks

The callback trait cleanly separates terminal state from host side effects:
bell, resize request, window title/icon, OSC 52 clipboard, and unhandled
character/control/ESC/CSI/OSC reporting
(`C:\dev\vt100-rust\src\callbacks.rs:3`,
`C:\dev\vt100-rust\src\callbacks.rs:6`,
`C:\dev\vt100-rust\src\callbacks.rs:12`,
`C:\dev\vt100-rust\src\callbacks.rs:15`,
`C:\dev\vt100-rust\src\callbacks.rs:27`,
`C:\dev\vt100-rust\src\callbacks.rs:36`,
`C:\dev\vt100-rust\src\callbacks.rs:39`,
`C:\dev\vt100-rust\src\callbacks.rs:55`,
`C:\dev\vt100-rust\src\callbacks.rs:66`).

Hera implication: callbacks should become typed side-effect events, not direct
host calls inside `terminal-core`.

## Fixtures, Properties And Long Replay

Fixtures are generated from `.in` files into progressive `.typescript` chunks
and JSON screen snapshots
(`C:\dev\vt100-rust\examples\generate_fixture.rs:13`,
`C:\dev\vt100-rust\examples\generate_fixture.rs:24`,
`C:\dev\vt100-rust\examples\generate_fixture.rs:35`). The fixture loader
replays each cumulative input and checks text, cursor, modes, mouse mode,
mouse encoding, cell content, wide flags and attributes
(`C:\dev\vt100-rust\tests\helpers\fixtures.rs:248`,
`C:\dev\vt100-rust\tests\helpers\fixtures.rs:252`,
`C:\dev\vt100-rust\tests\helpers\fixtures.rs:273`,
`C:\dev\vt100-rust\tests\helpers\fixtures.rs:281`).

The test suite covers common fixture families: text, wide/combining characters,
wrap, modes, alternate screen, scroll regions, scrollback, split escape
sequences, split UTF-8 and unknown OSC
(`C:\dev\vt100-rust\tests\text.rs:5`,
`C:\dev\vt100-rust\tests\text.rs:20`,
`C:\dev\vt100-rust\tests\text.rs:25`,
`C:\dev\vt100-rust\tests\text.rs:30`,
`C:\dev\vt100-rust\tests\mode.rs:10`,
`C:\dev\vt100-rust\tests\scroll.rs:7`,
`C:\dev\vt100-rust\tests\scroll.rs:16`,
`C:\dev\vt100-rust\tests\processing.rs:5`,
`C:\dev\vt100-rust\tests\processing.rs:10`,
`C:\dev\vt100-rust\tests\osc.rs:83`).

The long replay and property-testing story is also useful. `window_contents`
checks diff reproduction across 7,625 crawl frames
(`C:\dev\vt100-rust\tests\window_contents.rs:550`,
`C:\dev\vt100-rust\tests\window_contents.rs:554`,
`C:\dev\vt100-rust\tests\window_contents.rs:559`). QuickCheck runs structured
and random reproduction properties
(`C:\dev\vt100-rust\tests\quickcheck.rs:116`,
`C:\dev\vt100-rust\tests\quickcheck.rs:127`,
`C:\dev\vt100-rust\tests\quickcheck.rs:137`). The AFL harness is still useful
as a design sketch for byte-by-byte reproduction checks, but it references
removed screen APIs and should not be treated as active without repair
(`C:\dev\vt100-rust\fuzz\src\main.rs:7`,
`C:\dev\vt100-rust\fuzz\src\main.rs:70`,
`C:\dev\vt100-rust\CHANGELOG.md:30`,
`C:\dev\vt100-rust\CHANGELOG.md:36`).

Hera implication: this is the best local reference for the shape of M1
regression infrastructure.

## What Hera Should Copy

- A thin public `Parser` facade over an internal parser plus screen state.
- A Hera-owned performer/action layer between `vte` tokens and state mutation.
- A headless `Screen` API with cell, cursor, mode and text inspection.
- Byte-level `state_formatted` and `state_diff` equivalents as replay oracles.
- Progressive fixture chunks plus sparse JSON snapshots.
- Property tests that prove formatted and diff outputs reproduce state.
- Side-effect callbacks modeled outside pure screen mutation.

## What Hera Should Not Copy

- Terminal output generation tied to hardcoded escape helpers instead of a
  deliberate renderer/replay model (`C:\dev\vt100-rust\src\term.rs:1`).
- `VecDeque<Row>` as the final huge scrollback storage.
- Resize behavior that clears wrap state instead of doing reflow.
- Escape-byte reproduction as the only snapshot format.
- Direct callback methods as the final host boundary.
- Lack of typed protocol events for semantic metadata, links and images.
- Lack of PTY, renderer and semantic-session boundaries.

## Open Questions For Hera

1. Should M1 expose both typed snapshots and VT-byte reproduction from day one?
2. What is the first fixture schema: vt100-rust-style sparse JSON, a custom
   `TerminalSnapshot`, or both?
3. Should side effects be part of replay logs, or a separate host-event log?
4. How much of vt100-rust's fixture corpus should be imported before adding
   ConPTY replay fixtures?
5. Can `state_diff` become a render-model diff instead of a VT-byte diff while
   preserving the same reproducibility invariant?

## Bottom Line

vt100-rust should shape Hera's first headless API and test discipline. Use it
to prove byte ingestion, screen inspection, state snapshots, diffs, fixtures and
fuzzing. Do not use it as the final answer for huge scrollback, reflow,
renderer abstraction, PTY integration or semantic session intelligence.

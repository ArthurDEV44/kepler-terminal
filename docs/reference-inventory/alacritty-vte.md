# Alacritty VTE Reference Inventory

Status: initial focused pass
Date: 2026-07-01
Reference source: `C:\dev\vte`
Scope: parser boundary lessons for Kepler, especially VT byte parsing,
`Perform` dispatch, ANSI semantic mapping and fixture design.

## Executive Takeaways

Alacritty VTE is the cleanest Rust-native parser reference for Kepler. It is not
a terminal emulator, and that is exactly why it matters: it draws a hard line
between raw byte parsing and terminal semantics.

The high-value lessons:

1. Use Rust for the parser path. The crate is pure Rust, cross-platform by
   construction and does not need C, C#, Objective-C, Zig or Swift for this
   layer.
2. Depend on `vte` or mirror its boundary, but do not expose `vte::Perform` in
   Kepler's public API.
3. Treat `Perform` as an internal adapter that turns parser callbacks into
   Kepler-owned actions or direct terminal-core mutations.
4. Keep `terminal-protocol` responsible for normalized actions, modes, colors,
   keyboard reports and OSC policy.
5. Keep `terminal-core` responsible for screen state, grid, scrollback, cursor,
   selection, damage and rendering snapshots.
6. Use `ansi::Processor` as a reference for semantic dispatch, not as a
   complete terminal state model.
7. Reuse the test strategy: recorded byte streams, parser callback assertions,
   UTF-8 fragmentation, OSC/CSI/DCS edge cases and synchronized update cases.

Confidence: high for parser boundary, API shape and Rust portability because
the findings are grounded in `C:\dev\vte`. Medium for full VT compatibility
coverage because this pass audited the code structure and core tests, not every
escape sequence handled inside `src\ansi.rs`.

## Why Alacritty VTE Matters

The README defines the project narrowly: a parser for implementing virtual
terminal emulators in Rust (`C:\dev\vte\README.md:7`). It explicitly says the
parser follows Paul Williams' ANSI parser state machine
(`C:\dev\vte\README.md:9`) and that the parser does not assign meaning to the
parsed data. Meaning belongs to the host implementation of `Perform`
(`C:\dev\vte\README.md:12`, `C:\dev\vte\README.md:14`).

The crate metadata reinforces the same role: "Parser for implementing terminal
emulators" (`C:\dev\vte\Cargo.toml:3`), with parsing and no-std categories
(`C:\dev\vte\Cargo.toml:7`). The local snapshot is version 0.15.0, Rust edition
2021, MSRV 1.62.1 (`C:\dev\vte\Cargo.toml:11`,
`C:\dev\vte\Cargo.toml:13`, `C:\dev\vte\Cargo.toml:14`).

Kepler implication: `alacritty-vte` is not a reference for grid, scrollback,
PTY, renderer, tabs, sessions or UI. It is the reference for where parsing ends
and Kepler state begins.

## Codebase Map

| Area | Local path | What it contains | Kepler relevance |
|---|---|---|---|
| Crate metadata | `C:\dev\vte\Cargo.toml` | Features, dependencies, no-std/std split. | Confirms pure Rust parser dependency shape. |
| README | `C:\dev\vte\README.md` | Project scope and Paul Williams parser statement. | Sets the correct mental model: parser only. |
| Parser core | `C:\dev\vte\src\lib.rs` | State machine, byte advancement, UTF-8 handling, `Perform` trait. | Primary reference for `terminal-protocol` ingress. |
| Parameters | `C:\dev\vte\src\params.rs` | Fixed-size CSI/DCS params and subparameter iteration. | Model for preserving SGR subparameters and limits. |
| ANSI processor | `C:\dev\vte\src\ansi.rs` | Optional semantic layer mapping parser callbacks to `Handler`. | Reference for mode/color/keyboard action mapping. |
| Parse log example | `C:\dev\vte\examples\parselog.rs` | Minimal `Perform` implementation logging parser actions. | Good executable smoke harness shape. |
| Fixtures | `C:\dev\vte\tests\demo.vte` | Sample terminal byte stream fixture. | Seed format for parser regression tests. |
| Keyboard docs | `C:\dev\vte\doc\modifyOtherKeys-example.txt` | XTerm modifyOtherKeys examples. | Useful later for keyboard protocol conformance. |
| Changelog | `C:\dev\vte\CHANGELOG.md` | Parser API and protocol capability history. | Shows important API transitions and compatibility scope. |

## Language And Platform Decision

This layer should stay full Rust.

`vte` has no platform-specific OS dependency in the parser core. Its feature
shape is also conservative: default `std`, optional `ansi`, optional `serde`,
and `std` only forwards to `memchr/std` (`C:\dev\vte\Cargo.toml:18`,
`C:\dev\vte\Cargo.toml:19`, `C:\dev\vte\Cargo.toml:20`,
`C:\dev\vte\Cargo.toml:21`). Core dependencies are small Rust crates:
`arrayvec`, `memchr`, plus optional `bitflags`, `cursor-icon`, `log` and
`serde` (`C:\dev\vte\Cargo.toml:24`, `C:\dev\vte\Cargo.toml:28`).

Cross-platform implication: there is no good reason to introduce C, C#, Zig,
Objective-C or Swift for parser work. Native languages only become relevant at
host boundaries: PTY, windows, font/raster backends, IME, accessibility or
platform UI. The parser boundary should remain Rust on Windows, Linux and
macOS.

## Parser Core

The crate docs repeat the key architectural rule in source: `Parser` follows
Paul Williams' state machine, but the state machine does not assign meaning and
is not sufficient for a terminal emulator (`C:\dev\vte\src\lib.rs:3`,
`C:\dev\vte\src\lib.rs:4`, `C:\dev\vte\src\lib.rs:5`). `Parser` handles the
bookkeeping and `Perform` handles actions (`C:\dev\vte\src\lib.rs:6`,
`C:\dev\vte\src\lib.rs:8`).

The parser exposes `Params` and `ParamsIter` (`C:\dev\vte\src\lib.rs:44`) and
stores:

| Field family | Source | Why it matters |
|---|---|---|
| State machine state | `C:\dev\vte\src\lib.rs:57` | Parsing is resumable across byte chunks. |
| Intermediates | `C:\dev\vte\src\lib.rs:59` | Escape/CSI/DCS intermediates are preserved. |
| Params | `C:\dev\vte\src\lib.rs:61` | Parameter state is separated from semantics. |
| OSC raw buffer | `C:\dev\vte\src\lib.rs:63`, `C:\dev\vte\src\lib.rs:65` | `std` uses `Vec`, no-std uses fixed `ArrayVec`. |
| OSC param spans | `C:\dev\vte\src\lib.rs:67` | OSC params are borrowed slices into raw bytes. |
| Partial UTF-8 | `C:\dev\vte\src\lib.rs:70` | Split multi-byte chars are handled across calls. |

Hard parser limits are explicit: two intermediates, sixteen OSC params and a
default no-std OSC raw limit of 1024 bytes (`C:\dev\vte\src\lib.rs:46`,
`C:\dev\vte\src\lib.rs:47`, `C:\dev\vte\src\lib.rs:48`).

Kepler implication: parser state can live inside a `TerminalParser` struct, but
semantic state must not. Keep cursor, modes, tabs, colors, scrollback and
damage outside this parser layer.

## Advancement API

The main parser API is byte-slice based:

| API | Source | Kepler usage |
|---|---|---|
| `Parser::new` | `C:\dev\vte\src\lib.rs:74` | Create one parser per terminal instance. |
| `Parser::new_with_size` | `C:\dev\vte\src\lib.rs:91` | Only relevant for no-std OSC buffer tuning. |
| `Parser::advance` | `C:\dev\vte\src\lib.rs:111` | Feed PTY chunks directly. |
| `Parser::advance_until_terminated` | `C:\dev\vte\src\lib.rs:142` | Let semantic code stop mid-chunk for synchronized updates. |

The changelog shows why this matters: `Parser::advance` moved to byte slices in
0.14.0 (`C:\dev\vte\CHANGELOG.md:16`) and
`Parser::advance_until_terminated` was added to allow premature termination
(`C:\dev\vte\CHANGELOG.md:17`).

Kepler implication: feed PTY reads as slices, not byte-by-byte loops in Kepler.
The parser already owns chunk traversal, partial UTF-8 and resumability.

## Ground Fast Path And UTF-8

Ground state is optimized separately because it can only be left with ESC. The
source comments call this out and use `memchr` to find escape bytes quickly
(`C:\dev\vte\src\lib.rs:595`, `C:\dev\vte\src\lib.rs:597`,
`C:\dev\vte\src\lib.rs:599`, `C:\dev\vte\src\lib.rs:602`).

UTF-8 handling is part of the parser boundary. The crate documents differences
from the original state machine, including UTF-8 support and OSC termination by
BEL (`C:\dev\vte\src\lib.rs:20`, `C:\dev\vte\src\lib.rs:23`). The ground path
validates UTF-8, emits replacement behavior for invalid input and preserves
partial bytes across calls (`C:\dev\vte\src\lib.rs:611`,
`C:\dev\vte\src\lib.rs:672`).

Kepler implication: do not pre-decode PTY bytes into strings before parsing.
Kepler should feed raw bytes and receive already-classified printable chars or
control actions.

## State Machine Surface

The parser keeps the classic VT state set in one enum (`C:\dev\vte\src\lib.rs:735`).
Important transition handlers:

| Handler | Source | Meaning |
|---|---|---|
| CSI entry/ignore/intermediate/param | `C:\dev\vte\src\lib.rs:190`, `C:\dev\vte\src\lib.rs:219`, `C:\dev\vte\src\lib.rs:230`, `C:\dev\vte\src\lib.rs:241` | Structured control sequence parsing. |
| DCS entry/intermediate/param/passthrough | `C:\dev\vte\src\lib.rs:259`, `C:\dev\vte\src\lib.rs:289`, `C:\dev\vte\src\lib.rs:301`, `C:\dev\vte\src\lib.rs:319` | Device control string dispatch surface. |
| Escape and escape intermediate | `C:\dev\vte\src\lib.rs:342`, `C:\dev\vte\src\lib.rs:395` | Escape sequence dispatch. |
| OSC string | `C:\dev\vte\src\lib.rs:409` | Operating system command parsing. |
| Ground dispatch | `C:\dev\vte\src\lib.rs:724` | Printable text and C0/C1 execution. |

Kepler implication: Kepler should not reimplement these transitions in M1.
The immediate value is a battle-tested parser boundary while Kepler invests in
state, renderer model and host integration.

## Perform Boundary

`Perform` is the contract where raw parsing becomes terminal behavior. The trait
starts at `C:\dev\vte\src\lib.rs:763`.

| `Perform` method | Source | Kepler-owned target |
|---|---|---|
| `print(char)` | `C:\dev\vte\src\lib.rs:765` | Insert printable char with active attributes. |
| `execute(u8)` | `C:\dev\vte\src\lib.rs:768` | C0/C1 controls: LF, CR, BS, BEL, tab, charset shifts. |
| `hook(...)` | `C:\dev\vte\src\lib.rs:781` | Begin DCS dispatch. |
| `put(u8)` | `C:\dev\vte\src\lib.rs:785` | DCS passthrough bytes. |
| `unhook()` | `C:\dev\vte\src\lib.rs:791` | End DCS dispatch. |
| `osc_dispatch(...)` | `C:\dev\vte\src\lib.rs:794` | Title, colors, clipboard, hyperlinks, shell integration policy. |
| `csi_dispatch(...)` | `C:\dev\vte\src\lib.rs:801` | Cursor, erase, SGR, modes, reports, scroll regions. |
| `esc_dispatch(...)` | `C:\dev\vte\src\lib.rs:814` | ESC final actions and charsets. |
| `terminated()` | `C:\dev\vte\src\lib.rs:825` | Stop parsing when the semantic layer needs to flush or buffer. |

Kepler design:

```rust
struct KeplerPerformer<'a> {
    terminal: &'a mut TerminalCore,
    actions: &'a mut Vec<TerminalAction>,
}

impl vte::Perform for KeplerPerformer<'_> {
    // Internal adapter only. Do not expose this trait publicly.
}
```

For M1, direct mutation inside `terminal-core` is probably simpler than
allocating a `Vec<TerminalAction>` for every chunk. Still define a Kepler-owned
action vocabulary for tests and embedding, even if the fast path applies it
directly.

## Params And Subparameters

`Params` uses a fixed maximum of 32 entries (`C:\dev\vte\src\params.rs:5`) with
parallel arrays for params and subparameter counts (`C:\dev\vte\src\params.rs:8`,
`C:\dev\vte\src\params.rs:16`, `C:\dev\vte\src\params.rs:19`). The API tracks
full, clear, push and extend operations (`C:\dev\vte\src\params.rs:50`,
`C:\dev\vte\src\params.rs:55`, `C:\dev\vte\src\params.rs:62`,
`C:\dev\vte\src\params.rs:71`). `ParamsIter` yields slices so subparameters
survive dispatch (`C:\dev\vte\src\params.rs:89`,
`C:\dev\vte\src\params.rs:100`).

This is critical for modern SGR colors, where colon-separated forms must be
preserved. The changelog explicitly added CSI subparameter support in 0.9.0
(`C:\dev\vte\CHANGELOG.md:61`).

Kepler implication: the normalized action layer must not flatten params too
early. Preserve `[u16]` slices until the semantic handler knows the final
character.

## OSC Handling

OSC parsing is intentionally low-level. The parser records raw OSC bytes and
parameter spans, then dispatches borrowed slices (`C:\dev\vte\src\lib.rs:523`,
`C:\dev\vte\src\lib.rs:546`, `C:\dev\vte\src\lib.rs:556`,
`C:\dev\vte\src\lib.rs:577`). It supports BEL-terminated OSC by reporting the
terminator through `bell_terminated` (`C:\dev\vte\src\lib.rs:589`).

Limits and policy:

| Concern | Source | Kepler decision |
|---|---|---|
| Max OSC params | `C:\dev\vte\src\lib.rs:47`, `C:\dev\vte\src\lib.rs:531` | Accept parser limit, log ignored overflow in debug builds. |
| no-std raw size | `C:\dev\vte\src\lib.rs:48`, `C:\dev\vte\src\lib.rs:63` | Not relevant for desktop M1 unless Kepler targets no-std later. |
| std raw storage | `C:\dev\vte\src\lib.rs:65` | Add Kepler-level abuse limits for clipboard/title payloads. |
| BEL vs ST | `C:\dev\vte\src\lib.rs:412`, `C:\dev\vte\src\lib.rs:419` | Preserve terminator info for compatibility tests. |

Security implication: parser acceptance is not semantic acceptance. Kepler must
decide OSC 52 clipboard policy, hyperlinks, title changes and unsupported OSCs
at the semantic layer.

## ANSI Processor Layer

The optional `ansi` feature adds a semantic dispatcher: `Processor` wraps
`Parser`, `Performer` implements `vte::Perform`, and `Handler` receives
terminal-level callbacks (`C:\dev\vte\src\ansi.rs:271`,
`C:\dev\vte\src\ansi.rs:280`, `C:\dev\vte\src\ansi.rs:426`,
`C:\dev\vte\src\ansi.rs:495`).

`Processor::advance` handles synchronized updates by switching between direct
parser advancement and a buffering path (`C:\dev\vte\src\ansi.rs:298`,
`C:\dev\vte\src\ansi.rs:305`, `C:\dev\vte\src\ansi.rs:309`). The sync update
timeout is 150 ms (`C:\dev\vte\src\ansi.rs:36`), and sync state is tracked in
`ProcessorState` (`C:\dev\vte\src\ansi.rs:244`). The code stops or buffers
synchronized update sequences around DEC private mode 2026
(`C:\dev\vte\src\ansi.rs:353`, `C:\dev\vte\src\ansi.rs:370`,
`C:\dev\vte\src\ansi.rs:390`, `C:\dev\vte\src\ansi.rs:411`).

Kepler implication: synchronized update is worth supporting early because it
affects perceived rendering quality. The parser can help stop mid-chunk, but
the render scheduler must own the actual flush policy.

## Handler Semantics

`Handler` is a broad terminal action surface with default no-op methods
(`C:\dev\vte\src\ansi.rs:495`). It covers input, tabs, erases, modes, colors,
cursor icon, keyboard protocol and modifyOtherKeys (`C:\dev\vte\src\ansi.rs:506`,
`C:\dev\vte\src\ansi.rs:545`, `C:\dev\vte\src\ansi.rs:607`,
`C:\dev\vte\src\ansi.rs:632`, `C:\dev\vte\src\ansi.rs:671`,
`C:\dev\vte\src\ansi.rs:704`, `C:\dev\vte\src\ansi.rs:710`,
`C:\dev\vte\src\ansi.rs:723`).

The built-in `Performer` maps parser callbacks into that handler surface:
print goes to `input`, execute maps common C0 controls, DCS is mostly logged as
unhandled, and OSC/CSI receive detailed semantic handling
(`C:\dev\vte\src\ansi.rs:1284`, `C:\dev\vte\src\ansi.rs:1290`,
`C:\dev\vte\src\ansi.rs:1296`, `C:\dev\vte\src\ansi.rs:1311`,
`C:\dev\vte\src\ansi.rs:1329`, `C:\dev\vte\src\ansi.rs:1529`).

Kepler should treat this as a reference implementation for action mapping, not
as an API to expose. Reasons:

1. Handler defaults can hide unsupported behavior.
2. Kepler needs explicit unsupported-sequence telemetry.
3. Some actions should become render damage, mode changes or host callbacks.
4. Public embedding should depend on Kepler concepts, not `vte::ansi` concepts.

## Modes, Keyboard And Colors

The `ansi` module has useful modern protocol coverage:

| Surface | Source | Kepler relevance |
|---|---|---|
| Kitty keyboard protocol docs | `C:\dev\vte\src\ansi.rs:739` | Model for progressive keyboard enhancement. |
| XTerm modifyOtherKeys | `C:\dev\vte\src\ansi.rs:777` | Required for advanced shortcut fidelity. |
| Private modes | `C:\dev\vte\src\ansi.rs:900`, `C:\dev\vte\src\ansi.rs:916` | Cursor keys, mouse modes, alternate screen, bracketed paste, sync update. |
| Cursor shape/style | `C:\dev\vte\src\ansi.rs:821` | Cursor rendering state. |
| Attributes and SGR | `C:\dev\vte\src\ansi.rs:1136`, `C:\dev\vte\src\ansi.rs:1831` | Active cell style model. |
| SGR color parsing | `C:\dev\vte\src\ansi.rs:1930` | Truecolor and indexed color parsing reference. |
| Named colors | `C:\dev\vte\src\ansi.rs:1007` | Palette interface reference. |

Kepler implication: these mappings should inform `terminal-protocol`, but the
final enums should be Kepler-owned. This keeps space for Ghostty-style render
state, Alacritty-style grid state and future host APIs.

## Tests And Fixtures

The parser tests are directly useful because they target edge cases Kepler will
otherwise rediscover late:

| Test area | Source | Why Kepler should care |
|---|---|---|
| OSC parsing and BEL/ST termination | `C:\dev\vte\src\lib.rs:905`, `C:\dev\vte\src\lib.rs:956`, `C:\dev\vte\src\lib.rs:971` | Titles, color changes, clipboard and shell integration depend on OSC correctness. |
| OSC UTF-8 and string terminators | `C:\dev\vte\src\lib.rs:986`, `C:\dev\vte\src\lib.rs:1004` | Prevent split/terminator bugs in non-ASCII titles. |
| CSI max params and ignored long params | `C:\dev\vte\src\lib.rs:1055`, `C:\dev\vte\src\lib.rs:1078` | Defines overflow behavior. |
| CSI leading/trailing semicolons | `C:\dev\vte\src\lib.rs:1101`, `C:\dev\vte\src\lib.rs:1115` | Common compatibility footgun. |
| DCS parsing | `C:\dev\vte\src\lib.rs:1184`, `C:\dev\vte\src\lib.rs:1225` | Needed before Sixel or richer DCS support. |
| Fixed OSC buffer no-std cases | `C:\dev\vte\src\lib.rs:1335` | Useful if Kepler ever targets constrained environments. |
| Invalid and partial UTF-8 | `C:\dev\vte\src\lib.rs:1410`, `C:\dev\vte\src\lib.rs:1425`, `C:\dev\vte\src\lib.rs:1493` | PTY chunks can split anywhere. |
| Execute anywhere | `C:\dev\vte\src\lib.rs:1532` | Parser must handle control bytes outside happy paths. |

The `ansi` tests add semantic fixtures for terminal identity, truecolor, shell
startup, OSC 4/104 color updates and synchronized updates
(`C:\dev\vte\src\ansi.rs:2096`, `C:\dev\vte\src\ansi.rs:2108`,
`C:\dev\vte\src\ansi.rs:2157`, `C:\dev\vte\src\ansi.rs:2175`,
`C:\dev\vte\src\ansi.rs:2271`, `C:\dev\vte\src\ansi.rs:2283`,
`C:\dev\vte\src\ansi.rs:2321`, `C:\dev\vte\src\ansi.rs:2366`,
`C:\dev\vte\src\ansi.rs:2398`, `C:\dev\vte\src\ansi.rs:2417`).

Kepler implication: import the ideas, not necessarily the literal fixture
layout. The first Kepler parser test set should include split UTF-8, SGR
subparameters, OSC BEL/ST, DEC private modes, alternate screen and sync update.

## Example Harness

`examples\parselog.rs` is the minimal executable mental model. It defines a
`Log` performer (`C:\dev\vte\examples\parselog.rs:7`), implements `Perform`
(`C:\dev\vte\examples\parselog.rs:9`), creates `Parser::new`
(`C:\dev\vte\examples\parselog.rs:56`), reads stdin into 2048-byte chunks
(`C:\dev\vte\examples\parselog.rs:59`, `C:\dev\vte\examples\parselog.rs:62`)
and calls `advance` on each chunk (`C:\dev\vte\examples\parselog.rs:64`).

Kepler implication: build a similar `kepler-parse-log` dev binary early. It
should print Kepler-owned actions, not raw `vte::Perform` callbacks, so fixture
diffs validate the adapter layer.

## What Kepler Should Copy

Copy these ideas:

1. A byte-slice parser API.
2. One parser instance per terminal instance.
3. The `Perform` adapter boundary.
4. Parser resumability across arbitrary PTY chunks.
5. Ground-state fast path using raw byte scanning.
6. Parameter/subparameter preservation.
7. Explicit OSC terminator distinction.
8. Synchronized update termination path.
9. Edge-case parser tests before UI work.

## What Kepler Should Avoid

Avoid these traps:

1. Exposing `vte::Perform`, `vte::Params` or `vte::ansi::Handler` as Kepler's
   public API.
2. Treating `ansi::Processor` as a complete terminal emulator.
3. Letting Handler default no-ops hide unsupported sequences.
4. Flattening CSI params before knowing the final byte.
5. Decoding PTY bytes to strings before parsing.
6. Accepting every OSC semantically just because the parser can parse it.
7. Deferring sync update support until after renderer architecture hardens.

## Recommended Kepler Shape

M1 structure:

```text
terminal-protocol
  TerminalAction
  CsiAction
  OscAction
  Mode
  PrivateMode
  SgrAttribute
  KeyboardMode

terminal-core
  TerminalCore
  Grid
  Cursor
  CharsetState
  ModeState
  Damage
  VteAdapter (private)

terminal-pty
  raw byte chunks
  platform PTY backends
```

Flow:

```text
PTY bytes
  -> vte::Parser
  -> private KeplerPerformer
  -> TerminalAction or direct TerminalCore mutation
  -> damage/render snapshot
```

Decision: for performance and simplicity, M1 can mutate `TerminalCore` directly
inside `KeplerPerformer`, while tests expose a parallel action recorder. This
keeps the public API clean without forcing allocations into the hot path.

## Open Questions For Kepler

1. Should `terminal-protocol` expose an action recorder in public API, or keep
   it test-only until embedding use cases demand it?
2. Should OSC 52 clipboard be disabled by default, prompt-gated or host-gated?
3. Should synchronized update buffering live in parser adapter or render
   scheduler? Prefer scheduler ownership with parser termination support.
4. Should Kepler enable `vte`'s `ansi` feature, or implement its own semantic
   mapping from raw `Perform` callbacks? Prefer raw `Perform` for control, with
   `ansi.rs` as reference material.

## Bottom Line

Use `alacritty-vte` as the Rust parser backbone or exact boundary model. Do not
turn it into Kepler's public protocol. The parser gives Kepler the hard part of
VT byte classification; Kepler's product value starts immediately after that:
state correctness, damage, render snapshots, embeddability, PTY ergonomics and
cross-platform host integration.

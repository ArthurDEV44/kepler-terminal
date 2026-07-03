[PRD]
# PRD: M1 Headless Core Foundation

## Changelog

| Version | Date | Author | Summary |
|---------|------|--------|---------|
| 1.0 | 2026-07-03 | Arthur Jean | Initial draft for the first Hera implementation chantier |

## Problem Statement

1. Hera currently has a documented thesis and research corpus, but no Rust workspace that proves the terminal engine can ingest bytes, preserve terminal state and expose deterministic snapshots.
2. The existing README still lists `terminal-pty` as an initial crate while the decision register now defers PTY to M2. Future agents can start from the wrong surface unless the first chantier encodes the current M1 boundary.
3. Terminal correctness is not inspectable yet. Without fixtures, replay and a renderer-neutral output model, implementation can drift toward a product terminal surface before the core state model is proven.

**Why now:** `docs/research-map.md` exists and already resolves the first architecture decisions: wrap `alacritty/vte`, keep PTY outside M1, model `Terminal { state, parser }`, use pull snapshots plus damage and build fixtures before renderer work. The next useful step is no longer research. It is converting those decisions into a minimal, testable Rust foundation.

## Overview

This PRD defines the first Hera implementation chantier: create the M1 headless Rust workspace and prove the parser-to-state spine. The output is not a terminal app. It is a five-crate Rust workspace with `terminal-core`, `terminal-protocol`, `terminal-render-model`, `terminal-fixtures` and `terminal-cli`, plus enough behavior to ingest bytes, mutate visible state, switch primary and alternate screens, serialize snapshots and run focused golden fixtures.

The core design follows the research map: `terminal-core` owns terminal correctness, `terminal-protocol` owns normalized action and replay types, `terminal-render-model` owns renderer-neutral output, `terminal-fixtures` owns compatibility data and `terminal-cli` owns debug commands. `terminal-pty` is explicitly out of scope until M2.

The first vertical slice must show that bytes can flow through `vte` into Hera-owned state without exposing `vte::Perform` or parser-specific types in public APIs. Fixtures are part of the product, not a cleanup step.

## Goals

| Goal | Month-1 Target | Month-6 Target |
|------|---------------|----------------|
| Headless workspace health | 5 M1 crates compile with `cargo check --workspace` | Workspace includes M2 PTY crate without `terminal-core` depending on it |
| Fixture-backed behavior | 12 focused golden fixtures pass | 75 compatibility fixtures plus 3 long-session benchmark fixtures pass |
| Public API boundary | 0 public `vte` types exposed by Hera crates | 0 platform, PTY, renderer or Paneflow types exposed by `terminal-core` |
| Replay determinism | 100% identical snapshots across 2 consecutive replay runs for all M1 fixtures | 100% identical snapshots across recorded PTY replay corpus |

## Target Users

### Hera Implementer

- **Role:** Arthur or an implementation agent building the first Rust core.
- **Behaviors:** Reads README, research map and reference inventories, then implements story-scoped engine code.
- **Pain points:** Research decisions are spread across docs; without a PRD, agents can start from PTY, renderer or Paneflow integration too early.
- **Current workaround:** Manual reread of `docs/research-map.md` and `AGENTS.md` before each coding pass.
- **Success looks like:** A story can be implemented in one agent session with exact crate boundaries, tests and exclusions.

### Future Embedder

- **Role:** A future Paneflow or host-adapter engineer consuming Hera as a headless terminal engine.
- **Behaviors:** Needs byte ingestion, state snapshots, viewport output and replay before a real renderer or PTY adapter exists.
- **Pain points:** Existing terminal engines mix runtime, renderer, PTY, platform and product concerns in ways that are hard to embed cleanly.
- **Current workaround:** Reuse Alacritty-like terminal internals directly or adapt a product terminal surface.
- **Success looks like:** The core exposes renderer-neutral snapshots and stable row handles without host-specific assumptions.

## Research Findings

Key findings that informed this PRD:

### Competitive Context

- `alacritty/vte`: Provides a Rust parser boundary. The official docs state that the parser follows Paul Williams' state machine and does not assign terminal meaning by itself: https://docs.rs/vte/.
- Alacritty and Rio: Good references for Rust grid, scrollback and damage handoff, but app and renderer assumptions should not leak into Hera M1.
- WezTerm: Strong reference for `Terminal { state, parser }` and trait-based PTY surfaces, but its mux and app surface are too broad for the first core milestone.
- Ghostty: Strong reference for embeddability, snapshots and page-based scrollback, but its Zig/C API is reference material, not M1 implementation material.
- vt100-rust: Most relevant minimal Rust reference for byte ingestion, in-memory screen state, fixture generation and replay checks, but its `VecDeque<Row>` storage and resize behavior are not enough for Hera's long-session thesis.
- **Market gap:** A Rust-public, renderer-agnostic, fixture-first terminal core with bounded scrollback and deterministic replay before product UI.

### Best Practices Applied

- Use `vte` as a parser, not as Hera's public protocol. The `Perform` boundary stays private to Hera's adapter.
- Keep PTY outside the core. `portable-pty` documents a cross-platform PTY trait API, which supports the M2 boundary but is not needed for M1 byte ingestion: https://docs.rs/portable-pty.
- Validate terminal behavior through raw byte fixtures and golden snapshots, then extend toward VTTEST and xterm control sequence coverage: https://invisible-island.net/vttest/ and https://www.xfree86.org/current/ctlseqs.html.

*Full local research sources are in `docs/research-map.md` and `docs/reference-inventory/`. Context7 lookup found `portable-pty` as `/websites/rs_portable-pty`; exact Rust `vte` crate docs were not available through Context7, so docs.rs is the authority for `vte`.*

## Assumptions & Constraints

### Assumptions (to validate)

- `vte` 0.15.x remains the correct parser seed for M1, based on current docs.rs and local `alacritty-vte` inventory.
- A minimal direct-mutation `vte::Perform` adapter can coexist with Hera-owned action recording for tests without forcing per-byte allocations into the hot path.
- Ring or chunk-backed storage can satisfy M1 if public APIs already use stable row handles and generation fields.
- Twelve focused golden fixtures are enough to validate the first vertical slice before broader VTTEST or ConPTY replay work.

### Hard Constraints

- `terminal-core` must not depend on PTY code, GPUI, renderer crates, windowing, font stacks, platform APIs or Paneflow types.
- `vte::Perform`, `vte::Params` and parser-specific types must not appear in public Hera APIs.
- `terminal-pty` is not created in this PRD.
- Core logic stays Rust-only and platform-neutral.
- New files use ASCII text unless a touched file already intentionally uses non-ASCII.

## Quality Gates

These commands must pass for every user story:

- `cargo fmt --all -- --check` - Rust formatting is stable across the workspace.
- `cargo check --workspace` - All workspace crates typecheck.
- `cargo clippy --workspace --all-targets -- -D warnings` - Lints are treated as blocking defects.
- `cargo test --workspace` - Unit tests, golden fixtures and replay tests pass.

## Epics & User Stories

### EP-001: Workspace Boundary And Drift Cleanup

Create the Rust workspace and encode the M1 boundary so future agents start from the correct surface.

**Definition of Done:** The repo has the five M1 crates, no PTY crate, workspace-wide quality gates and docs that no longer contradict the M1 sequencing.

#### US-001: Scaffold M1 Cargo Workspace

**Description:** As a Hera implementer, I want a minimal Cargo workspace with the five M1 crates so that implementation can start inside enforceable crate boundaries.

**Priority:** P0
**Size:** S (2 pts)
**Dependencies:** None

**Acceptance Criteria:**

- [ ] Given a clean checkout, when `cargo metadata --no-deps` runs, then it lists `terminal-core`, `terminal-protocol`, `terminal-render-model`, `terminal-fixtures` and `terminal-cli`.
- [ ] Given the workspace is created, when `cargo check --workspace` runs, then all five crates compile.
- [ ] Given the M1 boundary, when the workspace members are inspected, then no `terminal-pty` crate exists.
- [ ] Given an invalid crate dependency from `terminal-core` to a platform or renderer crate, when quality gates run, then the dependency check fails or the code review flags it before merge.

#### US-002: Sync README With M1 Decision Register

**Description:** As a future implementation agent, I want README milestone text to match `docs/research-map.md` so that I do not start from stale PTY or research instructions.

**Priority:** P0
**Size:** XS (1 pt)
**Dependencies:** None

**Acceptance Criteria:**

- [ ] Given README lists initial crates, when it is read, then `terminal-pty` is marked M2 or absent from the M1 crate list.
- [ ] Given README describes the next step, when it is read, then it points to creating the M1 headless workspace instead of writing `docs/research-map.md`.
- [ ] Given `docs/research-map.md` and README both mention M1, when they are compared, then they agree that PTY is outside `terminal-core`.
- [ ] Given a future agent follows only README, when it scopes the first implementation pass, then it does not create PTY, renderer or Paneflow integration work.

### EP-002: Parser And Protocol Boundary

Define the private parser adapter and the Hera-owned protocol vocabulary needed to keep `vte` replaceable.

**Definition of Done:** Bytes can be parsed through `vte` internally, mapped to Hera-owned actions or state mutation and tested without leaking parser types publicly.

#### US-003: Define Core Protocol Types

**Description:** As a core implementer, I want explicit protocol and side-effect types so that parser callbacks become Hera concepts before they affect terminal state.

**Priority:** P0
**Size:** M (3 pts)
**Dependencies:** Blocked by US-001

**Acceptance Criteria:**

- [ ] Given `terminal-protocol`, when its public API is inspected, then it defines normalized actions for printable chars, C0 controls, ESC, CSI, OSC, DCS, APC and PM payloads needed by M1.
- [ ] Given OSC or DCS payload input exceeds the configured M1 payload limit, when it is converted to protocol data, then the result is a bounded ignored or truncated event with diagnostic metadata.
- [ ] Given CSI parameters include subparameters, when they are represented, then subparameter boundaries are preserved until semantic handling.
- [ ] Given a public API search for `vte::`, when it runs across Hera crates, then no public protocol type exposes parser-specific types.

#### US-004: Add Private VTE Adapter And Action Recorder

**Description:** As a core implementer, I want a private `vte` adapter so that byte chunks can be converted into Hera-owned actions or direct state mutations.

**Priority:** P0
**Size:** M (3 pts)
**Dependencies:** Blocked by US-003

**Acceptance Criteria:**

- [ ] Given a byte slice containing printable ASCII, when `Terminal::advance_bytes` runs, then the private adapter records or applies printable actions in input order.
- [ ] Given a byte slice splits a UTF-8 character across calls, when both chunks are advanced, then exactly one printable character reaches the state layer.
- [ ] Given malformed UTF-8 or unsupported sequences, when bytes are advanced, then the parser path does not panic and emits replacement or unsupported-sequence behavior according to the protocol policy.
- [ ] Given rustdoc is generated for public crates, when public items are inspected, then `vte::Perform` is not exposed.

### EP-003: Terminal State And Render Output

Build the minimum terminal state model that proves visible output, alternate screen behavior, scrollback identity and renderer-neutral snapshots.

**Definition of Done:** `terminal-core` can ingest common text/control sequences and `terminal-render-model` can expose a stable viewport snapshot with damage.

#### US-005: Define Terminal State, Dimensions And Cells

**Description:** As a core implementer, I want typed state primitives so that screen mutation, rendering and snapshots use one shared model.

**Priority:** P0
**Size:** M (3 pts)
**Dependencies:** Blocked by US-001, US-003

**Acceptance Criteria:**

- [ ] Given a new `Terminal` is created with columns and rows, when dimensions are valid, then it initializes primary and alternate screen state with cursor at row 0, column 0.
- [ ] Given zero columns, zero rows or dimensions above the M1 configured maximum, when `Terminal` is created, then it returns a typed error instead of panicking.
- [ ] Given a cell is empty, when it is serialized into a snapshot, then it has deterministic defaults for char, width and style.
- [ ] Given `terminal-core` dependencies are inspected, then no platform, PTY, renderer, GPUI or Paneflow dependency is present.

#### US-006: Implement Primary Screen Printing And Basic Controls

**Description:** As a core implementer, I want basic printable text and control handling so that the first fixtures can prove byte ingestion into visible state.

**Priority:** P0
**Size:** M (3 pts)
**Dependencies:** Blocked by US-004, US-005

**Acceptance Criteria:**

- [ ] Given printable ASCII bytes, when they are advanced, then cells in the primary screen contain the expected characters in row-major cursor order.
- [ ] Given CR, LF, BS and TAB controls, when they are advanced, then cursor position changes match documented M1 behavior.
- [ ] Given output reaches the last column, when auto-wrap is enabled, then the next printable character moves to the next line and wrap metadata is recorded.
- [ ] Given output exceeds the bottom row, when scrollback is enabled, then the top row moves into bounded scrollback instead of being lost silently.

#### US-007: Implement Alternate Screen Switching

**Description:** As a core implementer, I want primary and alternate screens to be distinct so that TUI-like flows do not pollute normal scrollback.

**Priority:** P0
**Size:** M (3 pts)
**Dependencies:** Blocked by US-006

**Acceptance Criteria:**

- [ ] Given DEC private mode 1049 is enabled, when text is printed, then it appears on the alternate screen and normal scrollback does not grow.
- [ ] Given alternate screen is disabled, when the terminal returns to primary screen, then prior primary content is visible again.
- [ ] Given an unsupported alternate-screen variant is received, when the sequence is advanced, then the terminal records an unsupported action without corrupting either screen.
- [ ] Given snapshots are serialized before and after a screen switch, when compared, then active screen identity is explicit.

#### US-008: Implement Bounded Scrollback With Stable Row Handles

**Description:** As a core implementer, I want bounded scrollback with stable row identity so that future selection, markers, search and replay do not depend on raw vector indexes.

**Priority:** P0
**Size:** L (5 pts)
**Dependencies:** Blocked by US-006

**Acceptance Criteria:**

- [ ] Given more rows are printed than visible height, when scrollback is queried, then rows have stable handles with generation fields.
- [ ] Given max line count is exceeded, when trimming happens, then trimmed handles become unavailable through an explicit not-found result.
- [ ] Given max byte budget is exceeded, when trimming happens, then scrollback memory stays within the configured budget plus 10% measurement tolerance.
- [ ] Given a caller asks for a raw row index outside the visible range, when using public APIs, then no such API exists; callers must use viewport coordinates or row handles.

#### US-009: Expose Render Snapshot And Damage Model

**Description:** As a future embedder, I want renderer-neutral snapshots so that GPUI, CLI debug tools and tests can consume the same terminal output.

**Priority:** P0
**Size:** M (3 pts)
**Dependencies:** Blocked by US-005, US-006

**Acceptance Criteria:**

- [ ] Given visible terminal state, when a render snapshot is requested, then it returns viewport rows, render cells, cursor state and active screen identity.
- [ ] Given text mutates one row, when damage is queried, then the damage region includes that row and excludes untouched rows.
- [ ] Given no state changes happened since the previous snapshot, when damage is queried, then it returns an empty damage set.
- [ ] Given a cell contains unsupported image metadata, when rendered through M1 snapshot, then it appears as a placeholder record and not as decoded image bytes.

### EP-004: Fixtures, Snapshots And Replay

Make correctness measurable through raw byte fixtures, golden snapshots and deterministic replay.

**Definition of Done:** The first fixture pack can replay byte streams into snapshots, compare expected output and catch unsupported or unsafe parser behavior.

#### US-010: Define Fixture Schema And Runner

**Description:** As a test author, I want a fixture schema and runner so that terminal behavior can be validated without manual shell testing.

**Priority:** P0
**Size:** M (3 pts)
**Dependencies:** Blocked by US-001, US-005

**Acceptance Criteria:**

- [ ] Given a fixture file with raw byte input and expected snapshot data, when the runner executes it, then it reports pass or fail with fixture name and failing field.
- [ ] Given malformed fixture JSON or TOML, when the runner loads it, then it returns a validation error with fixture path and failing field, and does not run partial assertions.
- [ ] Given the same fixture runs twice, when snapshots are compared, then serialized output is byte-identical across runs.
- [ ] Given a fixture references an unsupported feature, when the runner executes it, then the result can assert an unsupported event without marking the fixture as a crash.

#### US-011: Add Initial Golden Fixture Pack

**Description:** As a core implementer, I want focused golden fixtures so that the M1 vertical slice proves behavior beyond one local shell example.

**Priority:** P0
**Size:** M (3 pts)
**Dependencies:** Blocked by US-010, US-006, US-007

**Acceptance Criteria:**

- [ ] Given the fixture pack runs, then it covers plain text, CR/LF/BS/TAB, SGR reset, truecolor SGR, line wrap, scrollback trim, split UTF-8, unknown OSC, OSC 8 placeholder, bracketed paste mode and alternate screen 1049.
- [ ] Given a fixture contains bytes split at every possible chunk boundary for a selected sample, when replayed, then the final snapshot matches the unsplit input snapshot.
- [ ] Given an unknown OSC payload exceeds the limit, when fixture replay runs, then the expected bounded unsupported event is asserted.
- [ ] Given a fixture expected snapshot is wrong, when tests run, then the failure output identifies the first differing row, cell, cursor field or mode field.

#### US-012: Implement Snapshot Serialization And Replay Determinism

**Description:** As a Hera maintainer, I want typed snapshots and replay checks so that bugs can be reproduced without reparsing undocumented runtime state.

**Priority:** P0
**Size:** M (3 pts)
**Dependencies:** Blocked by US-009, US-010

**Acceptance Criteria:**

- [ ] Given terminal state, when a `TerminalSnapshot` is serialized and deserialized, then visible rows, cursor, modes, scrollback handles and active screen identity are preserved.
- [ ] Given a raw byte fixture and periodic snapshots, when replay runs from the beginning, then the final snapshot equals the golden snapshot.
- [ ] Given corrupted snapshot input, when deserialization runs, then it returns a typed error and never panics.
- [ ] Given optional semantic fields are absent, when deserialization runs, then terminal visual state still loads correctly.

### EP-005: Resize, CLI And M1 Proof

Add the minimum resize behavior and debug CLI surface required to inspect the headless core.

**Definition of Done:** Developers can inject bytes, dump snapshots, replay fixtures and compare outputs from a CLI without spawning a PTY process.

#### US-013: Implement Fixture-Driven Resize MVP

**Description:** As a core implementer, I want predictable resize behavior for primary screen first so that reflow starts behind tests rather than product assumptions.

**Priority:** P1
**Size:** L (5 pts)
**Dependencies:** Blocked by US-008, US-010

**Acceptance Criteria:**

- [ ] Given primary screen rows with wrap metadata, when width narrows, then reflow behavior matches the documented M1 fixture expectation.
- [ ] Given width widens after wrapped rows, when reflow runs, then cursor and row handles update according to snapshot generation rules.
- [ ] Given alternate screen is active, when resize runs, then normal scrollback is not polluted.
- [ ] Given invalid resize dimensions, when resize is requested, then the terminal returns a typed error and leaves prior state unchanged.

#### US-014: Add Headless Debug CLI Commands

**Description:** As a Hera implementer, I want a CLI that injects bytes and inspects snapshots so that core behavior is inspectable before PTY exists.

**Priority:** P1
**Size:** M (3 pts)
**Dependencies:** Blocked by US-009, US-010, US-012

**Acceptance Criteria:**

- [ ] Given a byte file path, when `terminal-cli inject <file>` runs, then it prints a deterministic snapshot summary.
- [ ] Given a fixture path, when `terminal-cli replay <fixture>` runs, then it exits 0 on pass and non-zero on assertion failure.
- [ ] Given two snapshot files, when `terminal-cli compare <a> <b>` runs, then it reports the first semantic difference.
- [ ] Given a user tries `terminal-cli run <command>`, when M1 CLI handles it, then it returns an explicit out-of-scope error because PTY is M2.

## Functional Requirements

- FR-01: The workspace must contain exactly the M1 crates defined in this PRD before M2 work starts.
- FR-02: `terminal-core` must expose byte ingestion through a Hera API and keep `vte` usage private.
- FR-03: `terminal-core` must model primary and alternate screens as distinct screen states.
- FR-04: `terminal-core` must expose typed errors at fallible boundaries instead of panicking.
- FR-05: `terminal-render-model` must expose viewport rows, render cells, cursor state, active screen identity and damage regions.
- FR-06: Scrollback APIs must use stable row handles with generation data and must not expose mutable internal row vectors.
- FR-07: Fixtures must use raw byte input plus golden snapshot assertions.
- FR-08: Snapshot replay must be deterministic across two consecutive runs for the same fixture.
- FR-09: M1 CLI must support inject, replay and compare commands.
- FR-10: M1 CLI must not spawn shells or PTY processes.

## Non-Functional Requirements

- **Performance:** In release mode on Arthur's current dev machine, replaying a 10,000-line ASCII fixture at 80x24 must complete in <=750 ms after the fixture runner exists.
- **Memory:** Replaying a 10,000-line fixture with a 5,000-line scrollback cap must keep terminal-owned heap growth <=64 MB, measured by a benchmark or test helper.
- **Security:** OSC, DCS, APC and PM payloads must have configured byte limits; payloads above the limit must not allocate more than limit plus 10% overhead.
- **Reliability:** All golden fixture snapshots must be byte-identical across 2 consecutive runs on the same machine.
- **Portability:** `terminal-core`, `terminal-protocol` and `terminal-render-model` must compile on host target without platform-specific `cfg(target_os)` branches in public APIs.
- **API Hygiene:** Public API search for `vte::`, `portable_pty::`, `gpui::`, `windows::` and `Paneflow` in M1 public items must return 0 matches.

## Edge Cases & Error States

Systematic coverage of unhappy paths. Evidence shows earlier defect discovery significantly reduces cost.

| # | Scenario | Trigger | Expected Behavior | User Message |
|---|----------|---------|-------------------|--------------|
| 1 | Empty byte input | `advance_bytes([])` | No state mutation and empty damage | N/A |
| 2 | Invalid dimensions | Terminal creation or resize with 0 rows or 0 columns | Typed error, prior state unchanged | "invalid terminal dimensions" |
| 3 | Split UTF-8 | Multi-byte char split across chunks | One printable char after complete sequence | N/A |
| 4 | Malformed UTF-8 | Invalid byte sequence in ground state | Replacement or unsupported behavior, no panic | N/A |
| 5 | Oversized OSC/DCS payload | Payload exceeds M1 byte limit | Bounded ignored/truncated event | "payload limit exceeded" in diagnostics |
| 6 | Scrollback budget exceeded | Line count or byte budget over limit | Old rows trimmed, handles invalidated explicitly | N/A |
| 7 | Alternate screen exit | Return from alternate to primary | Primary content restored, alternate scrollback not merged | N/A |
| 8 | Corrupt fixture file | Missing fields or invalid schema | Fixture runner fails before replay | "invalid fixture schema" |
| 9 | Snapshot mismatch | Golden differs from replay output | First differing field reported | "snapshot mismatch" |
| 10 | Out-of-scope CLI run | `terminal-cli run <command>` in M1 | Non-zero exit with PTY deferred message | "PTY execution is M2" |

## Risks & Mitigations

| # | Risk | Probability | Impact | Mitigation |
|---|------|------------|--------|------------|
| 1 | M1 scope expands into PTY or renderer work | Med | High | Keep `terminal-pty`, renderer adapters and Paneflow integration in Non-Goals; add CLI out-of-scope behavior for `run` |
| 2 | `vte` parser concepts leak into public Hera API | Med | High | Add public API search gate and keep adapter private |
| 3 | Scrollback implementation locks into raw indexes | High | High | Require stable row handles and generation fields from first scrollback story |
| 4 | Reflow bugs invalidate snapshots and row handles | High | High | Defer full reflow, start with fixture-driven primary-screen MVP |
| 5 | Fixture schema becomes too narrow | Med | Med | Include unsupported events, payload limits and snapshot serialization in schema from the beginning |
| 6 | Performance targets distract from correctness | Low | Med | Limit performance gates to 10,000-line M1 benchmarks and keep 100,000+ line work for later milestones |

## Non-Goals

Explicit boundaries: what this version does NOT include:

- No `terminal-pty` crate, shell spawning, POSIX PTY or Windows ConPTY runtime.
- No GPUI adapter, GPU renderer, font stack, windowing integration or desktop app.
- No Paneflow-specific API types or product integration.
- No terminal multiplexer.
- No shell integration magic or authoritative command detection.
- No full Kitty, iTerm2 or Sixel rendering. M1 may preserve safe metadata or placeholders only.
- No privileged host command execution from OSC payloads.

## Files NOT to Modify

- `docs/reference-inventory/*` - Evidence files should not be rewritten unless a new focused research pass changes a decision.
- `.git/*` - Git internals are not part of the chantier.
- `.codex/*` and `.agents/*` - Local agent metadata should not be edited for product implementation.
- Reference repos under `C:\dev\terminal-research`, `C:\dev\vte`, `C:\dev\wezterm`, `C:\dev\alacritty`, `C:\dev\ghostty`, `C:\dev\xterm.js`, `C:\dev\vt100-rust` - External reference sources are read-only inputs, not vendored Hera code.

## Technical Considerations

Frame as questions for engineering input, not mandates:

- **Architecture:** Should `Terminal::advance_bytes` mutate state directly through a private performer in M1, while a test-only recorder validates Hera actions? Recommended: yes for performance and scope control. Engineering to confirm module shape.
- **Protocol Model:** Which action types belong in `terminal-protocol` public API now versus crate-private test helpers? Recommended: expose durable protocol and replay types, keep parser-adapter details private.
- **Storage:** Should M1 use a ring with stable handles or implement page storage immediately? Recommended: ring/chunk storage with page-ready handle APIs. Engineering to confirm complexity.
- **Snapshot Format:** Should snapshots use `serde` from day one? Recommended: yes for fixtures and CLI compare. Engineering to confirm crate feature flags.
- **Fixture Format:** JSON, TOML or RON? Recommended: JSON for machine diffability plus raw byte fixtures. Engineering to confirm ergonomics.
- **Dependency Versions:** Use current `vte` from docs.rs and pin workspace dependencies in `Cargo.toml`. Engineering to verify latest compatible versions before implementation.
- **README Drift:** Should README be updated before code or in the workspace scaffold story? Recommended: in US-002, before deeper implementation.

## Success Metrics

| Metric | Baseline (current) | Target | Timeframe | How Measured |
|--------|-------------------|--------|-----------|-------------|
| Workspace crates compile | 0 crates | 5 crates compile | End of US-001 | `cargo check --workspace` |
| Public parser leakage | N/A, no API | 0 public `vte::` references | End of US-004 | `rg "pub .*vte::|vte::" crates docs as scoped by story` plus rustdoc inspection |
| Golden fixture count | 0 fixtures | 12 focused fixtures passing | End of US-011 | `cargo test --workspace` |
| Replay determinism | 0 replay tests | 100% byte-identical snapshots across 2 runs | End of US-012 | Fixture runner deterministic replay check |
| Scrollback memory cap | No scrollback implementation | <=64 MB heap growth for 10,000-line fixture with 5,000-line cap | End of US-008 or US-011 | Benchmark or test helper documented in repo |
| PTY boundary compliance | README currently drifts | 0 M1 crates or stories create PTY runtime | End of PRD | Workspace members and dependency inspection |

## Open Questions

- Should snapshot files store raw cell arrays or a compressed line representation? Owner: implementing engineer. Needed before US-010.
- What exact byte limit should M1 use for OSC/DCS/APC/PM payloads? Owner: implementing engineer. Needed before US-003.
- Should `terminal-cli` be a binary crate named `terminal-cli` or expose a binary named `hera-terminal`? Owner: Arthur. Needed before US-014.
- Should `vte` semantic helpers under its `ansi` feature be used as reference only or enabled as a dependency feature? Owner: implementing engineer. Needed before US-004.
[/PRD]

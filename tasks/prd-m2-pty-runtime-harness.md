[PRD]
# PRD: M2 PTY Runtime Harness

## Changelog

| Version | Date | Author | Summary |
|---------|------|--------|---------|
| 1.0 | 2026-07-03 | Arthur Jean | Initial draft for Hera M2 real PTY runtime harness |

## Problem Statement

1. Hera M1 proves a headless terminal core, fixture runner and debug CLI, but it still cannot execute a real process through a pseudo-terminal.
2. `terminal-cli run <command>` currently returns the expected M1 refusal message, so no end-to-end command output can be captured from a shell, parsed through `terminal-core` and dumped as a snapshot.
3. PTY behavior has platform-specific failure modes that pure byte fixtures do not cover: Windows ConPTY pipe lifecycle, POSIX PTY EOF, resize propagation, child exit status, blocked readers and command startup environment.
4. Without an M2 runtime boundary, future Paneflow dogfood would either couple directly to platform PTY code or push process orchestration concerns back into `terminal-core`.

**Why now:** M1 is marked DONE and verified. The research map explicitly says PTY belongs in M2 after the core can parse bytes, maintain visible state, switch alternate screen, resize predictably and serialize snapshots. The next useful proof is a real PTY harness that keeps `terminal-core` platform-neutral while making `terminal-cli run <command>` produce deterministic snapshot dumps.

## Overview

This PRD defines Hera M2: add a `terminal-pty` crate and connect real local processes to the M1 terminal core. The output is still not a desktop terminal app. It is a runtime harness that can spawn a command or default shell through a PTY, read output bytes into `terminal-core`, propagate resize, collect exit status, enforce bounded IO, and dump final snapshots through `terminal-cli`.

The first backend should use `portable-pty` because its documented model already matches Hera's desired boundary: `PtySystem` opens a `PtyPair`, `MasterPty` exposes resize, size, reader and writer, `SlavePty` spawns commands, and `Child` exposes wait, try_wait and process id. Windows ConPTY remains a transport behind the PTY crate, not authoritative terminal state.

M2 also adds live PTY recording and replay fixtures. Live tests must be isolated because shell availability differs by platform. Deterministic replay must not depend on the same local shell being installed later.

## Goals

| Goal | Month-1 Target | Month-6 Target |
|------|---------------|----------------|
| PTY workspace boundary | `terminal-pty` crate compiles and `terminal-core` has 0 dependency on it | PTY crate supports local and remote-like byte transports behind the same session event model |
| CLI run proof | `terminal-cli run <command>` executes 3 smoke commands on the host platform and emits a snapshot JSON | CLI supports recorded session replay and at least 10 cross-platform smoke fixtures |
| Lifecycle correctness | 100% of M2 lifecycle tests cover spawn, resize, EOF, exit status and timeout | Live PTY test matrix runs on Windows, Linux and macOS CI lanes |
| Replay determinism | 3 live command recordings replay to byte-identical snapshots across 2 runs | 20 recorded PTY sessions replay with 100% deterministic final snapshots |
| Backpressure safety | PTY reader drains 1 MB output without deadlock within 5 seconds in local smoke tests | 50 MB long-output session drains with bounded memory and no core API change |

## Target Users

### Hera Runtime Implementer

- **Role:** Arthur or an implementation agent adding process orchestration after M1.
- **Behaviors:** Works from PRD stories, existing M1 APIs, `docs/research-map.md` and PTY reference inventories.
- **Pain points:** PTY logic can easily leak into `terminal-core` or overfit to Windows PowerShell on the current machine.
- **Current workaround:** Feed byte files into `terminal-cli inject` instead of running real commands.
- **Success looks like:** `terminal-cli run <command>` produces a final snapshot and exit metadata without adding platform dependencies to `terminal-core`.

### Future Host Adapter

- **Role:** Future Paneflow, GUI, TUI or remote-session integration consuming Hera runtime pieces.
- **Behaviors:** Needs a typed session that exposes output chunks, input writes, resize, close, exit status and snapshots.
- **Pain points:** Mature terminal products often mix PTY, pane, renderer and UI concerns behind one runtime surface.
- **Current workaround:** Reuse product-specific process management or call platform PTY APIs directly.
- **Success looks like:** The host can use `terminal-pty` and `terminal-core` as separate crates, with no Paneflow-specific type in either public API.

## Research Findings

Key findings that informed this PRD:

### Competitive Context

- WezTerm: Keeps the terminal engine GUI-free and PTY-free, then uses `portable-pty` as a separate trait crate with `PtySystem`, `MasterPty`, `SlavePty` and `Child` boundaries.
- Windows Terminal: Separates terminal core, connection layer, renderer and ConPTY implementation. Its ConPTY docs and source stress pipe IO, resize, close lifecycle and output-thread draining.
- xterm.js: Treats the terminal as a browser/headless emulator and expects an external addon or transport to provide a real PTY or remote process stream.
- emacs-libvterm: Shows a real host-owned PTY path, but its Emacs process filter model is Unix/editor-shaped and not a cross-platform Hera runtime model.
- **Market gap:** A Rust-first PTY harness that can be embedded without becoming a terminal application, while preserving deterministic snapshot and replay hooks.

### Best Practices Applied

- Keep PTY behind a dedicated crate. `portable-pty` documents a runtime-selectable PTY API and exposes the exact controls Hera needs: open PTY, spawn command, clone reader, take writer, resize, get size and wait for child exit: https://docs.rs/portable-pty/.
- Treat Windows ConPTY as byte transport, not terminal state. Microsoft documents that ConPTY hosts character-mode applications through input/output streams encoded as UTF-8 text plus VT sequences, while the terminal application remains responsible for presentation and input serialization: https://learn.microsoft.com/en-us/windows/console/createpseudoconsole.
- Service ConPTY communication channels on separate threads or equivalent non-blocking architecture. Microsoft warns that single-threaded synchronous servicing can deadlock when buffers fill: https://learn.microsoft.com/en-us/windows/console/creating-a-pseudoconsole-session.
- Resize must update both the PTY and `terminal-core`. `MasterPty::resize` informs the child about new window size, while `Terminal::resize` updates Hera's state model.
- Close lifecycle needs explicit tests. Microsoft documents that `ClosePseudoConsole` can require output draining and has version-specific deadlock behavior on older Windows builds: https://learn.microsoft.com/en-us/windows/console/closepseudoconsole.

*Full local research sources are in `docs/research-map.md`, especially PTY And Process Lifecycle, the WezTerm inventory and the Windows Terminal inventory. Context7 lookup used `/websites/rs_portable-pty` for `portable-pty` API planning.*

## Assumptions & Constraints

### Assumptions (to validate)

- `portable-pty` 0.9.x is sufficient for the first local PTY backend on Windows, Linux and macOS.
- Synchronous PTY reader and writer handles can be wrapped in dedicated worker threads plus bounded channels without introducing Tokio into M2.
- Host smoke commands can be selected per platform with zero external dependencies: Windows uses `cmd /C` or PowerShell only when present, Unix uses `/bin/sh -lc`.
- Final snapshot determinism is enough for M2; interactive frame-by-frame rendering is deferred.
- A 5-second default command timeout is enough for smoke tests and can be overridden by CLI flags.

### Hard Constraints

- `terminal-core` must not depend on `terminal-pty`, `portable-pty`, `windows`, `nix`, `rustix`, `libc`, renderer crates, windowing crates or Paneflow types.
- `terminal-pty` must not depend on `terminal-render-model`; it owns process IO and session events, not rendering.
- `terminal-cli` may depend on `terminal-core`, `terminal-fixtures` and `terminal-pty` to assemble the debug harness.
- No GUI, font stack, renderer, Paneflow integration or terminal multiplexer in M2.
- Windows ConPTY details stay behind `terminal-pty`; POSIX details stay behind `terminal-pty`.
- New files use ASCII text unless editing a file that already intentionally uses non-ASCII.

## Quality Gates

These commands must pass for every user story:

- `cargo fmt --all -- --check` - Rust formatting is stable across the workspace.
- `cargo check --workspace` - All workspace crates typecheck.
- `cargo clippy --workspace --all-targets -- -D warnings` - Lints are treated as blocking defects.
- `cargo test --workspace` - Unit tests, golden fixtures, replay tests and non-live PTY tests pass.
- `cargo test --workspace --features live-pty-tests -- --ignored` - Host-dependent PTY smoke tests pass when the local machine supports them.

## Epics & User Stories

### EP-001: M2 Workspace Boundary

Add the PTY runtime crate and dependencies without weakening the M1 crate boundaries.

**Definition of Done:** The workspace contains `terminal-pty`, M1 still passes, `terminal-core` remains PTY-free, and README/research-map status reflects M2.

#### US-001: Add `terminal-pty` Crate And Dependency Direction

**Description:** As a Hera runtime implementer, I want a dedicated `terminal-pty` crate so that process orchestration has an owned boundary outside `terminal-core`.

**Priority:** P0
**Size:** S (2 pts)
**Dependencies:** None

**Acceptance Criteria:**

- [ ] Given `cargo metadata --no-deps`, when workspace packages are listed, then `terminal-pty` appears alongside the five M1 crates.
- [ ] Given `terminal-core` manifest is inspected, when dependencies are read, then it does not depend on `terminal-pty` or `portable-pty`.
- [ ] Given `terminal-pty` manifest is inspected, when dependencies are read, then `portable-pty` is present behind the PTY boundary.
- [ ] Given a platform-specific dependency is added outside `terminal-pty`, when review checks dependency boundaries, then the story fails.

#### US-002: Update M2 Documentation Boundary

**Description:** As a future implementation agent, I want README and project guidance to show that M2 is the real PTY harness so that the next work does not jump to renderer or Paneflow dogfood.

**Priority:** P0
**Size:** XS (1 pt)
**Dependencies:** None

**Acceptance Criteria:**

- [ ] Given README M2 section is read, when it describes the output, then it says `terminal-cli run <command>` executes through PTY and dumps a snapshot.
- [ ] Given README and `docs/research-map.md` are compared, when PTY scope is mentioned, then both keep PTY outside `terminal-core`.
- [ ] Given docs mention M3, when they are read, then Paneflow dogfood remains deferred until after M2.
- [ ] Given stale wording says `run` is out of scope, when docs are searched, then only M1 PRD historical text keeps that statement.

### EP-002: PTY API And Backend

Define the public `terminal-pty` API, implement the initial `portable-pty` backend and make failures typed.

**Definition of Done:** A caller can configure a command, open a PTY, spawn a child, get reader/writer handles, resize and observe typed lifecycle errors.

#### US-003: Define PTY Domain Types And Errors

**Description:** As a runtime implementer, I want typed PTY configuration, dimensions, events and errors so that CLI and future hosts do not depend on `portable-pty` internals.

**Priority:** P0
**Size:** M (3 pts)
**Dependencies:** Blocked by US-001

**Acceptance Criteria:**

- [ ] Given `terminal-pty` public API is inspected, then it exposes Hera-owned types for `PtyCommand`, `PtySize`, `PtySessionConfig`, `PtyExit`, `PtyEvent` and `PtyError`.
- [ ] Given invalid dimensions such as 0 columns or 0 rows, when a PTY size is created, then it returns a typed validation error.
- [ ] Given a command has an empty program, when it is converted to a spawn config, then it returns a typed validation error.
- [ ] Given public API is searched for `portable_pty::`, when rustdoc-visible items are inspected, then no public Hera type exposes `portable-pty` types directly.

#### US-004: Implement Portable PTY Backend

**Description:** As a runtime implementer, I want a `portable-pty` backed implementation so that M2 can spawn local commands on Windows, Linux and macOS through one crate boundary.

**Priority:** P0
**Size:** M (3 pts)
**Dependencies:** Blocked by US-003

**Acceptance Criteria:**

- [ ] Given a valid command and size, when the backend spawns it, then a session handle is returned with reader, writer, child and platform metadata.
- [ ] Given the platform backend cannot open a PTY, when spawn is attempted, then `PtyError` includes operation name and source message.
- [ ] Given `MasterPty::take_writer` can only be called once, when the backend initializes a session, then the writer is owned exactly once and subsequent public calls do not expose double-take behavior.
- [ ] Given `portable-pty` changes error formatting, when tests run, then assertions check Hera error kind and operation, not fragile full source strings.

#### US-005: Implement Command Builder Policy

**Description:** As a CLI and runtime consumer, I want explicit command, cwd and environment policy so that command startup is predictable and testable.

**Priority:** P0
**Size:** M (3 pts)
**Dependencies:** Blocked by US-003, US-004

**Acceptance Criteria:**

- [ ] Given `PtyCommand` includes program, args, cwd and env overrides, when converted to backend command, then all fields are applied deterministically.
- [ ] Given cwd does not exist, when spawn is requested, then the error is returned before opening PTY handles.
- [ ] Given env override sets a variable to an empty value, when command runs, then the child receives the override according to documented M2 policy.
- [ ] Given command args contain spaces or shell metacharacters, when using direct command mode, then they are passed as arguments and not joined into a shell string.

### EP-003: Session Runtime And Core Bridge

Bridge PTY bytes into `terminal-core`, write input back to the PTY and maintain lifecycle state.

**Definition of Done:** A local command can run through PTY, drain output into `Terminal`, accept input writes, resize, close and return exit metadata without deadlock in smoke tests.

#### US-006: Add PTY Session Event Loop

**Description:** As a runtime implementer, I want a bounded session loop so that PTY output is drained into events without blocking command lifecycle.

**Priority:** P0
**Size:** L (5 pts)
**Dependencies:** Blocked by US-004

**Acceptance Criteria:**

- [ ] Given a command writes output, when the session runs, then output bytes are emitted as ordered `PtyEvent::Output` chunks.
- [ ] Given the output reader returns EOF, when the session observes it, then it emits a terminal EOF event and stops the reader worker.
- [ ] Given the command writes 1 MB to stdout, when the session runs with default bounds, then output drains within 5 seconds in the live smoke test.
- [ ] Given the event channel reaches capacity, when more output arrives, then the session applies the documented backpressure policy without unbounded memory growth.

#### US-007: Bridge PTY Output Into Terminal Core

**Description:** As a Hera runtime consumer, I want PTY output chunks applied to `terminal-core` so that real commands produce render snapshots.

**Priority:** P0
**Size:** M (3 pts)
**Dependencies:** Blocked by US-006

**Acceptance Criteria:**

- [ ] Given a command prints `hello`, when the bridge drains PTY output, then `Terminal::advance_bytes` receives the command output bytes and final snapshot contains `hello`.
- [ ] Given PTY output arrives in multiple chunks, when the bridge applies them, then final snapshot equals replaying the concatenated byte recording through `terminal-cli inject`.
- [ ] Given PTY output contains malformed or partial UTF-8, when chunks are bridged, then `terminal-core` handles it through existing M1 parser behavior without runtime panic.
- [ ] Given the terminal core returns a typed resize error during session setup, when bridge starts, then PTY spawn is not attempted.

#### US-008: Implement Input Write API

**Description:** As a future host adapter, I want a writer API so that user input or scripted stdin can be sent to the child process.

**Priority:** P0
**Size:** M (3 pts)
**Dependencies:** Blocked by US-006

**Acceptance Criteria:**

- [ ] Given a running session, when `write_input` is called with bytes ending in `\r`, then the bytes are written to the PTY writer.
- [ ] Given the PTY writer has been closed, when `write_input` is called, then it returns a typed closed-session error.
- [ ] Given input exceeds the configured M2 max write chunk, when writing is requested, then it is rejected or chunked according to documented policy.
- [ ] Given the command exits before input is written, when input is attempted, then the caller receives a typed lifecycle error and no panic occurs.

#### US-009: Implement Resize Propagation

**Description:** As a runtime consumer, I want resize to update both PTY and terminal core so that child applications and Hera state agree on dimensions.

**Priority:** P0
**Size:** M (3 pts)
**Dependencies:** Blocked by US-006, US-007

**Acceptance Criteria:**

- [ ] Given a running session, when resize to 100x30 is requested, then `terminal-core.resize(100, 30)` succeeds and the PTY backend receives equivalent character dimensions.
- [ ] Given `terminal-core.resize` rejects dimensions, when resize is requested, then PTY resize is not called.
- [ ] Given PTY backend resize fails after core resize succeeds, when resize is requested, then the session emits a typed partial-resize error with both requested dimensions and source operation.
- [ ] Given multiple resize requests occur in sequence, when the session completes, then final core dimensions match the last successful PTY resize.

#### US-010: Implement Child Exit And Timeout Handling

**Description:** As a CLI user, I want exit status and timeout handling so that command runs terminate predictably and report failures.

**Priority:** P0
**Size:** M (3 pts)
**Dependencies:** Blocked by US-006, US-007

**Acceptance Criteria:**

- [ ] Given a command exits 0, when session completes, then result includes exit code 0 and final snapshot.
- [ ] Given a command exits non-zero, when session completes, then result includes the non-zero exit code and still dumps the final snapshot.
- [ ] Given a command exceeds the configured timeout, when timeout elapses, then the session terminates or kills the child and returns a typed timeout error.
- [ ] Given child exit happens before reader EOF, when session completes, then output draining continues until EOF or drain timeout before final snapshot is produced.

### EP-004: CLI Run And Snapshot Artifacts

Replace the M1 refusal path with a real PTY-backed `terminal-cli run` command that remains a debug harness.

**Definition of Done:** CLI can run commands, default shells and scripted input, then emit snapshot and metadata artifacts with stable exit behavior.

#### US-011: Implement `terminal-cli run <command>`

**Description:** As a Hera implementer, I want `terminal-cli run <command>` to execute a real command through PTY so that M2 has an end-to-end proof.

**Priority:** P0
**Size:** M (3 pts)
**Dependencies:** Blocked by US-007, US-010

**Acceptance Criteria:**

- [ ] Given `terminal-cli run <command>`, when the command exits, then stdout contains a JSON object with final snapshot and run metadata.
- [ ] Given command exits non-zero, when CLI finishes, then process exit code matches documented policy and the snapshot is still printed or written to the requested output path.
- [ ] Given no command is supplied, when `terminal-cli run` is invoked, then it returns usage text and exit code 2.
- [ ] Given the platform cannot spawn the command, when CLI runs, then stderr includes the failing operation and program name without a Rust panic backtrace.

#### US-012: Add Default Shell Mode

**Description:** As a CLI user, I want a default shell mode so that M2 can start the host shell without hardcoding one platform globally.

**Priority:** P1
**Size:** M (3 pts)
**Dependencies:** Blocked by US-011

**Acceptance Criteria:**

- [ ] Given `terminal-cli run --shell`, when invoked on Windows, then it selects a documented default shell candidate and records the selected program in metadata.
- [ ] Given `terminal-cli run --shell`, when invoked on Unix, then it selects `$SHELL` or `/bin/sh` according to documented policy.
- [ ] Given no shell candidate exists, when shell mode runs, then it returns a typed shell-not-found error.
- [ ] Given shell mode receives `--command`, when invoked, then the command is passed through the shell only in shell mode and direct command mode remains argument-safe.

#### US-013: Add Scripted Input And Resize CLI Flags

**Description:** As a test author, I want CLI flags for input and resize so that interactive-like programs can be smoke tested without a GUI.

**Priority:** P1
**Size:** M (3 pts)
**Dependencies:** Blocked by US-011, US-009

**Acceptance Criteria:**

- [ ] Given `--input-file <path>`, when CLI runs, then file bytes are written to the PTY after spawn and before stdin-close policy is applied.
- [ ] Given `--cols` and `--rows`, when CLI runs, then initial PTY and terminal dimensions use those values.
- [ ] Given a resize script is provided, when CLI runs, then each resize event is applied in order and recorded in metadata.
- [ ] Given input file exceeds the configured M2 input cap, when CLI runs, then it fails before spawning the command.

#### US-014: Emit Run Metadata And Snapshot Files

**Description:** As a maintainer, I want stable run artifacts so that live command behavior can become replayable evidence.

**Priority:** P0
**Size:** M (3 pts)
**Dependencies:** Blocked by US-011

**Acceptance Criteria:**

- [ ] Given `--output <path>`, when CLI run completes, then it writes snapshot JSON to that path and prints a concise success line.
- [ ] Given `--record <path>`, when CLI run completes, then it writes raw output chunks, timestamps, resize events, input events and exit metadata to a replayable recording file.
- [ ] Given output path parent directory does not exist, when CLI runs, then it returns a path error before spawning the command.
- [ ] Given recording serialization fails, when CLI completes, then command exit metadata is still reported and the serialization error is visible.

### EP-005: Live PTY Fixtures And Replay

Turn live PTY smoke runs into deterministic replay artifacts and platform-specific tests.

**Definition of Done:** M2 has host-gated live tests, recorded replay tests and docs for refreshing PTY recordings.

#### US-015: Define PTY Recording Schema

**Description:** As a fixture maintainer, I want a PTY recording schema so that live process output can be replayed without rerunning the process.

**Priority:** P0
**Size:** M (3 pts)
**Dependencies:** Blocked by US-014

**Acceptance Criteria:**

- [ ] Given a recording file, when it is loaded, then schema validation checks version, initial dimensions, output chunks, resize events, input events, exit metadata and final snapshot.
- [ ] Given a recording omits required fields, when loader runs, then it returns a path and field-specific validation error.
- [ ] Given recording contains timestamps, when deterministic replay runs, then timestamps are preserved as metadata but do not affect final snapshot comparison.
- [ ] Given recording exceeds configured M2 byte cap, when loader runs, then it fails before allocating full payload buffers.

#### US-016: Add Cross-Platform Live Smoke Tests

**Description:** As a Hera maintainer, I want platform-gated live tests so that PTY spawn, output, resize and exit behavior are proven on the current host.

**Priority:** P1
**Size:** L (5 pts)
**Dependencies:** Blocked by US-011, US-013

**Acceptance Criteria:**

- [ ] Given live PTY tests are enabled on Windows, when smoke tests run, then a command prints a known token and exits 0 through ConPTY.
- [ ] Given live PTY tests are enabled on Unix, when smoke tests run, then `/bin/sh -lc` prints a known token and exits 0 through POSIX PTY.
- [ ] Given required shell is unavailable, when live tests run, then they skip with a documented reason rather than failing unrelated platforms.
- [ ] Given a live resize smoke test runs, when command probes terminal size, then output or metadata confirms the requested dimensions.

#### US-017: Add Recorded Replay Regression Pack

**Description:** As a regression tester, I want recorded PTY sessions replayed through `terminal-core` so that live command behavior remains testable offline.

**Priority:** P0
**Size:** M (3 pts)
**Dependencies:** Blocked by US-015

**Acceptance Criteria:**

- [ ] Given the M2 replay pack runs, then it includes at least 3 recordings: plain command output, non-zero exit command and resize-aware command.
- [ ] Given each recording is replayed twice, when final snapshots are serialized, then snapshot bytes are identical across both runs.
- [ ] Given a recording expected snapshot is wrong, when tests run, then failure output identifies the first differing field.
- [ ] Given live recordings are unavailable on a platform, when offline replay tests run, then they still pass from checked-in recordings.

## Functional Requirements

- FR-01: The workspace must include a `terminal-pty` crate for M2.
- FR-02: `terminal-core` must remain independent from PTY and platform dependencies.
- FR-03: `terminal-pty` must expose Hera-owned command, size, session, event, exit and error types.
- FR-04: `terminal-pty` must support a `portable-pty` backend for local commands.
- FR-05: The runtime must drain PTY output into ordered chunks and apply them to `terminal-core`.
- FR-06: The runtime must provide input write and close behavior.
- FR-07: The runtime must propagate resize to both `terminal-core` and the PTY backend.
- FR-08: The runtime must report child exit status, timeout and spawn errors.
- FR-09: `terminal-cli run <command>` must execute real commands through PTY.
- FR-10: `terminal-cli run` must be able to write final snapshot JSON and optional recording artifacts.
- FR-11: M2 must include live PTY smoke tests behind an explicit feature or ignored-test gate.
- FR-12: M2 must include deterministic replay tests for recorded PTY sessions.

## Non-Functional Requirements

- **Performance:** A command that writes 1 MB of output must drain and produce a final snapshot in <=5 seconds on Arthur's current dev machine.
- **Memory:** During the 1 MB output smoke test, terminal plus PTY recording buffers must stay <=128 MB heap growth, measured by test helper or documented benchmark.
- **Backpressure:** Event channels must have a configured capacity <=1024 events by default; overflow behavior must be tested.
- **Timeouts:** Default CLI command timeout must be 5 seconds for tests and configurable up to at least 120 seconds.
- **Reliability:** Recorded PTY replay snapshots must be byte-identical across 2 consecutive runs.
- **Portability:** Non-live tests must pass on any host target supported by the workspace without requiring a shell.
- **Security:** Direct command mode must pass program and args without shell joining; shell interpretation may happen only under explicit shell mode.
- **API Hygiene:** Public API search for `portable_pty::`, `windows::`, `nix::`, `libc::`, `rustix::` in `terminal-core` public items must return 0 matches.

## Edge Cases & Error States

Systematic coverage of unhappy paths. Evidence shows earlier defect discovery significantly reduces cost.

| # | Scenario | Trigger | Expected Behavior | User Message |
|---|----------|---------|-------------------|--------------|
| 1 | Empty command | `terminal-cli run` without program | Exit 2 with usage | "usage: terminal-cli run <command>" |
| 2 | Missing executable | Command program not found | Typed spawn error, no snapshot panic | "spawn failed" |
| 3 | Invalid PTY dimensions | 0 rows or 0 columns | Reject before PTY open | "invalid PTY dimensions" |
| 4 | Reader EOF | Child closes output pipe | Emit EOF event and finish drain path | N/A |
| 5 | Writer closed | Input write after exit | Typed closed-session error | "session is closed" |
| 6 | Command timeout | Child exceeds timeout | Kill or terminate child, return timeout error | "command timed out" |
| 7 | Output flood | Child writes 1 MB or more | Bounded channel drains without unbounded memory | N/A |
| 8 | Resize failure | PTY backend rejects resize | Typed partial-resize error | "PTY resize failed" |
| 9 | Recording too large | Recording exceeds byte cap | Reject before full allocation | "recording exceeds limit" |
| 10 | ConPTY close drain | Windows output pipe still has data during close | Drain until EOF or drain timeout | "close drain timed out" if exceeded |
| 11 | Shell unavailable | `--shell` has no candidate | Typed shell-not-found error | "default shell not found" |
| 12 | Output path invalid | `--output` parent missing | Fail before spawn | "output directory not found" |

## Risks & Mitigations

| # | Risk | Probability | Impact | Mitigation |
|---|------|------------|--------|------------|
| 1 | ConPTY deadlocks on synchronous IO or close | Med | High | Use separate reader/writer/lifecycle workers and add close drain tests |
| 2 | PTY code leaks into `terminal-core` | Med | High | Add dependency and public API search gates for every story |
| 3 | Live tests are flaky across shells | High | Med | Gate live tests behind feature/ignored flag and keep offline replay fixtures mandatory |
| 4 | Command mode accidentally invokes shell semantics | Med | High | Separate direct command mode from explicit `--shell` mode and test metacharacter args |
| 5 | Backpressure policy drops bytes silently | Med | High | Emit explicit overflow events or errors and assert byte accounting in tests |
| 6 | M2 grows into interactive terminal app | Med | Med | Keep non-goals explicit: no renderer, GUI, input UI, pane model or Paneflow adapter |

## Non-Goals

Explicit boundaries: what this version does NOT include:

- No desktop terminal window, GPU renderer, font stack or keyboard UI.
- No Paneflow integration.
- No multiplexer, panes, tabs, domains or remote sessions.
- No authoritative semantic command detection.
- No full interactive TUI event loop beyond scripted input and command execution.
- No shell integration protocol beyond recording raw bytes and optional metadata.
- No direct Windows ConPTY public API exposed outside `terminal-pty`.
- No replacement of M1 snapshot or fixture schema unless required by PTY recording support.

## Files NOT to Modify

- `crates/terminal-core/Cargo.toml` - Do not add PTY, `portable-pty`, platform or runtime dependencies.
- `crates/terminal-core/src/vte_adapter.rs` - Parser adapter should not learn about PTY.
- `crates/terminal-render-model/*` - Renderer-neutral model should not own process lifecycle.
- `docs/reference-inventory/*` - Evidence files should not be rewritten unless a new focused research pass changes a decision.
- Reference repos under `C:\dev\terminal-research`, `C:\dev\wezterm`, `C:\dev\terminal` and related local clones - Read-only references, not vendored Hera code.

## Technical Considerations

Frame as questions for engineering input, not mandates:

- **Backend:** Should M2 depend directly on `portable-pty` or wrap it behind Hera traits from day one? Recommended: wrap it behind Hera types, use `portable-pty` as the only backend implementation for M2.
- **Runtime Model:** Should M2 use standard threads and bounded channels or introduce Tokio? Recommended: standard threads plus bounded channels, because `portable-pty` exposes blocking `Read` and `Write` handles.
- **CLI Output:** Should `terminal-cli run` print snapshot JSON by default or require `--output`? Recommended: print JSON by default for parity with `inject`, with `--output` for artifacts.
- **Exit Code Policy:** Should CLI return the child exit code or 0 when snapshot capture succeeds? Recommended: return child exit code, while preserving snapshot output.
- **Recording Format:** Should PTY recordings extend M1 fixture JSON or use a separate schema? Recommended: separate `pty-recording` schema that can be converted into fixture replay.
- **Timeout Policy:** Should timeout kill the child process group or only the direct child? Recommended: use backend-supported kill first, document process-tree limitation for M2.
- **Windows Specifics:** Should M2 call Windows ConPTY APIs directly? Recommended: no, use `portable-pty` first; direct `windows-rs` adapter becomes a later story only if `portable-pty` blocks required behavior.

## Success Metrics

| Metric | Baseline (current) | Target | Timeframe | How Measured |
|--------|-------------------|--------|-----------|-------------|
| PTY crate presence | 0 PTY crates | 1 `terminal-pty` crate | End of US-001 | `cargo metadata --no-deps` |
| Core PTY isolation | 0 current PTY deps | 0 `terminal-core` PTY/platform deps after M2 | End of every story | Manifest inspection and `rg` API hygiene checks |
| CLI run behavior | `PTY execution is M2` refusal | Real command executes and emits snapshot JSON | End of US-011 | `terminal-cli run <smoke-command>` |
| Live smoke coverage | 0 live PTY tests | 3 host-gated live tests | End of US-016 | `cargo test --workspace --features live-pty-tests -- --ignored` |
| Recorded replay coverage | 0 PTY recordings | 3 replay recordings | End of US-017 | `cargo test --workspace` |
| Backpressure smoke | No PTY output path | 1 MB output drains in <=5 seconds | End of US-006 | Live test or benchmark helper |
| Snapshot determinism | M1 fixtures only | 100% deterministic PTY replay snapshots across 2 runs | End of US-017 | Recording replay test |

## Open Questions

- Should `terminal-cli run` default timeout be exactly 5 seconds for all commands or only for tests? Owner: implementing engineer. Needed before US-011.
- Which Windows shell should `--shell` prefer: `pwsh`, `powershell`, then `cmd`, or `cmd` first for availability? Owner: Arthur or implementing engineer. Needed before US-012.
- Should PTY recordings store raw input events as bytes only, or include semantic labels such as "stdin file" and "resize script"? Owner: implementing engineer. Needed before US-015.
- Should M2 add CI live PTY jobs now, or keep live tests local until the project has public CI? Owner: Arthur. Needed before US-016.
[/PRD]

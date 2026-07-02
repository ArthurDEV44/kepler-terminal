# Windows Terminal Reference Inventory

Status: initial focused pass
Date: 2026-07-02
Reference source: `C:\dev\terminal`
Scope: Windows Terminal lessons for Hera, especially ConPTY, TextBuffer/ROW,
reflow, renderer invalidation, command marks, parser fuzzing and Windows host
constraints.

## Executive Takeaways

Windows Terminal is a high-value Windows compatibility and behavior reference,
not a language or platform template for Hera.

The high-value lessons:

1. Treat ConPTY as a Windows transport and lifecycle adapter, not as Hera's
   authoritative terminal state.
2. Keep the terminal core independent from the PTY runtime. Windows Terminal has
   separate `TerminalCore`, `TerminalConnection`, `TerminalControl`,
   `TextBuffer`, `renderer` and `winconpty` areas.
3. Copy the behavior lessons around resize, alternate screen, row wrap markers,
   circular scrollback, marks and renderer invalidation. Do not copy the
   C++/WinRT app architecture.
4. Model reflow as a fixture-driven feature. Windows Terminal has dedicated
   `TextBuffer::Reflow` tests and still documents semantic mark pain under
   reflow.
5. The renderer boundary is useful because it pulls from an `IRenderData`
   surface and pushes into `IRenderEngine`, but the control layer also documents
   a circular terminal-renderer dependency that Hera should avoid.
6. The Windows app stack exists for historical and platform reasons. Hera
   should remain Rust-first, with a narrow Windows PTY adapter using Win32 FFI.
7. Windows Terminal's fuzzers and ConPTY tests should become fixture sources for
   Hera's parser, resize and Windows adapter validation.

Confidence: high for source topology, ConPTY, TextBuffer/reflow, renderer and
marks because this pass covered local source, specs, samples and tests under
`C:\dev\terminal`. Medium for full app-layer conclusions because this pass did
not audit every settings, XAML, shell integration or packaging file.

## Why Windows Terminal Matters

The repository contains Windows Terminal, Windows Terminal Preview, Windows
Console Host and shared command-line components
(`C:\dev\terminal\README.md:42`, `C:\dev\terminal\README.md:44`). The README
also states that the Console Host still contains the Console API server, input
engine, rendering engine and user preferences (`C:\dev\terminal\README.md:195`,
`C:\dev\terminal\README.md:199`). ConPTY was added as the Windows pseudo-console
bridge (`C:\dev\terminal\README.md:209`).

The shared components matter for Hera because they include a DirectWrite text
layout/rendering engine, a text buffer capable of storing UTF-16 and UTF-8 and a
VT parser/emitter (`C:\dev\terminal\README.md:236`,
`C:\dev\terminal\README.md:237`). This makes Windows Terminal one of the best
references for Windows-specific terminal behavior.

The caveat is just as important. The README says the project explored options
and decided its goals were best met by continuing investment in the C++ codebase
(`C:\dev\terminal\README.md:241`, `C:\dev\terminal\README.md:243`). It also
notes that Terminal is a C++ application, not a C# one
(`C:\dev\terminal\README.md:392`, `C:\dev\terminal\README.md:393`). That is a
coherent decision for a Windows product with conhost history. It is not a reason
for Hera to add C++, C# or WinUI to the core.

Hera implication: use Windows Terminal as the Windows oracle, not as the
architecture stack. The owned engine should stay Rust and cross-platform.

## Codebase Map

| Area | Local path | What it contains | Hera relevance |
|---|---|---|---|
| Terminal core | `C:\dev\terminal\src\cascadia\TerminalCore` | `Terminal`, API dispatch, primary/alternate buffers, resize, shell integration callbacks, render data. | State and boundary reference, with Windows-specific caveats. |
| Text buffer | `C:\dev\terminal\src\buffer\out` | `TextBuffer`, `ROW`, row storage, wrap flags, image slices, marks, reflow and serialization. | Strong source for row invariants, resize and semantic metadata tests. |
| Terminal control | `C:\dev\terminal\src\cascadia\TerminalControl` | UI-facing control core, renderer ownership, connection events, mark UI, search, resize. | Useful host orchestration reference, but not a core template. |
| Connection layer | `C:\dev\terminal\src\cascadia\TerminalConnection` | `ITerminalConnection`, `ConptyConnection`, pipe IO, write, resize, close, handoff. | Windows PTY adapter behavior. |
| ConPTY implementation | `C:\dev\terminal\src\winconpty` | Pseudo console struct, signal pipe, resize, clear, show/hide, reparent, close, tests. | Primary Windows FFI and lifecycle reference. |
| Renderer | `C:\dev\terminal\src\renderer` | `IRenderData`, `IRenderEngine`, `Renderer`, Atlas engine, invalidation, dirty area, frame painting. | Renderer snapshot/invalidation reference. |
| Parser | `C:\dev\terminal\src\terminal\parser` | VT state machine engine, dispatch methods, parser fuzzing project. | Secondary parser behavior and fuzz corpus source. |
| Host/server | `C:\dev\terminal\src\host`, `C:\dev\terminal\src\server` | Console host internals, API server, fuzzer harness. | Compatibility oracle, not Hera ownership model. |
| Specs | `C:\dev\terminal\doc\specs` | Command marks, in-process ConPTY, keyboard handling, show/hide ConPTY details. | Design notes for backlog and failure modes. |
| Samples | `C:\dev\terminal\samples\ConPTY` | C++ and C# MiniTerm/EchoCon samples for pseudo-console creation and pipe IO. | Minimal Windows adapter examples. |

## Language And Platform Decision

Windows Terminal strengthens the Rust-first decision by showing what must be
isolated, not by showing what language Hera should use.

The repo is overwhelmingly C++/C++ headers with IDL and some C# tests or
samples. A quick source extension scan found `.cpp`, `.h`, `.hpp`, `.idl` and
`.cs` files as the dominant implementation surface. The build is Visual Studio,
MSBuild, C++/WinRT, WinUI and vcpkg oriented
(`C:\dev\terminal\README.md:360`, `C:\dev\terminal\README.md:366`,
`C:\dev\terminal\vcpkg.json:2`). That is expected for a Windows platform repo.

| Hera zone | Language | Windows Terminal evidence | Decision |
|---|---|---|---|
| Terminal state core | Rust | `TerminalCore` owns state, buffers, APIs and render data in C++ (`C:\dev\terminal\src\cascadia\TerminalCore\Terminal.hpp:86`, `C:\dev\terminal\src\cascadia\TerminalCore\Terminal.cpp:43`). | Keep Hera core Rust-only. |
| VT parser boundary | Rust wrapper | Windows Terminal has a VT state machine and dispatch layer (`C:\dev\terminal\src\terminal\parser\OutputStateMachineEngine.hpp:20`, `C:\dev\terminal\src\terminal\parser\OutputStateMachineEngine.cpp:411`). | Wrap `alacritty/vte`; use Windows Terminal as compatibility reference. |
| Text buffer and reflow | Rust | `TextBuffer` owns rows, wrap flags, serialization and `Reflow` (`C:\dev\terminal\src\buffer\out\textBuffer.hpp:288`, `C:\dev\terminal\src\buffer\out\textBuffer.cpp:2718`). | Implement in Rust with fixture parity. |
| Semantic marks | Rust observer sidecar | Marks attach to rows and are reflow-sensitive (`C:\dev\terminal\src\buffer\out\Marks.hpp:11`, `C:\dev\terminal\src\buffer\out\textBuffer.cpp:2807`). | Store semantic metadata beside terminal correctness, not inside parser correctness. |
| Windows PTY | Rust plus Win32 FFI | ConPTY APIs expose create, resize, clear, show/hide, reparent and close (`C:\dev\terminal\src\winconpty\winconpty.h:65`, `C:\dev\terminal\src\winconpty\winconpty.cpp:462`). | Use `windows-rs` or raw Win32 FFI behind `terminal-pty`. |
| Unix and macOS PTY | Rust plus Unix syscalls | Windows Terminal does not cover Unix PTY semantics. | Use WezTerm/Contour/Ghostty as Unix/macOS references. |
| Renderer model | Rust render snapshot types | `IRenderData` pulls viewport, buffer, cursor and colors; `IRenderEngine` receives invalidation and paint commands (`C:\dev\terminal\src\renderer\inc\IRenderData.hpp:51`, `C:\dev\terminal\src\renderer\inc\IRenderEngine.hpp:62`). | Keep a renderer-neutral Rust model, then host adapters. |
| Native Windows host | Rust host adapter unless proven insufficient | Windows Terminal uses C++/WinRT and WinUI because it is a Windows app. | Do not add C# or C++ for Hera M1. Reassess only for a Windows-native host shell. |

Bottom line: Windows Terminal is the strongest Windows behavior reference, but
it reinforces a Rust cross-platform core with thin OS-specific adapters.

## Terminal Core And Screens

`Terminal` is the central state object. It can be created with a viewport,
scrollback size and renderer (`C:\dev\terminal\src\cascadia\TerminalCore\Terminal.hpp:86`,
`C:\dev\terminal\src\cascadia\TerminalCore\Terminal.hpp:87`). Construction
allocates the main `TextBuffer` and attaches the renderer
(`C:\dev\terminal\src\cascadia\TerminalCore\Terminal.cpp:43`,
`C:\dev\terminal\src\cascadia\TerminalCore\Terminal.cpp:51`).

Primary and alternate screen buffers are explicit. `Terminal` exposes alternate
and main screen APIs (`C:\dev\terminal\src\cascadia\TerminalCore\Terminal.hpp:157`,
`C:\dev\terminal\src\cascadia\TerminalCore\Terminal.hpp:158`) and selects the
active buffer based on `_inAltBuffer`
(`C:\dev\terminal\src\cascadia\TerminalCore\Terminal.cpp:1327`,
`C:\dev\terminal\src\cascadia\TerminalCore\Terminal.cpp:1330`).
`UseAlternateScreenBuffer` allocates `_altBuffer`, copies cursor state, makes the
main buffer inactive and triggers redraw
(`C:\dev\terminal\src\cascadia\TerminalCore\TerminalApi.cpp:242`,
`C:\dev\terminal\src\cascadia\TerminalCore\TerminalApi.cpp:289`).
`UseMainScreenBuffer` restores the main screen and can replay a deferred resize
(`C:\dev\terminal\src\cascadia\TerminalCore\TerminalApi.cpp:293`,
`C:\dev\terminal\src\cascadia\TerminalCore\TerminalApi.cpp:344`).

Hera implication: alternate screen is a distinct screen state, not a viewport
flag. Resize and scrollback semantics must branch on that state.

## TextBuffer, ROW And Scrollback

The `TextBuffer` design is the densest part of the reference. It manages rows,
cursor, attributes, circular storage, serialization, selection behavior, image
slices and semantic marks.

Important source signals:

- `TextBuffer` has a virtual-memory-oriented layout for `ROW` structs, chars,
  offsets and a scratch row (`C:\dev\terminal\src\buffer\out\textBuffer.cpp:85`,
  `C:\dev\terminal\src\buffer\out\textBuffer.cpp:112`).
- Row access accounts for circular storage and a scratch row
  (`C:\dev\terminal\src\buffer\out\textBuffer.cpp:177`,
  `C:\dev\terminal\src\buffer\out\textBuffer.cpp:206`).
- `ROW` and `TextBuffer` must agree on row stride, chars buffer and offsets
  (`C:\dev\terminal\src\buffer\out\Row.hpp:101`,
  `C:\dev\terminal\src\buffer\out\Row.hpp:111`).
- Rows can contain more characters than columns through character offsets
  (`C:\dev\terminal\src\buffer\out\Row.hpp:280`,
  `C:\dev\terminal\src\buffer\out\Row.hpp:293`).
- Forced wrap is explicit (`C:\dev\terminal\src\buffer\out\Row.hpp:133`,
  `C:\dev\terminal\src\buffer\out\Row.cpp:177`).
- Image slices are row-attached metadata (`C:\dev\terminal\src\buffer\out\Row.hpp:161`,
  `C:\dev\terminal\src\buffer\out\Row.cpp:973`).
- The buffer increments `_firstRow` as circular history advances
  (`C:\dev\terminal\src\buffer\out\textBuffer.cpp:723`,
  `C:\dev\terminal\src\buffer\out\textBuffer.cpp:743`).
- Serialization intentionally avoids line breaks for wrapped rows unless the row
  is terminal for the selection/export path
  (`C:\dev\terminal\src\buffer\out\textBuffer.cpp:2379`,
  `C:\dev\terminal\src\buffer\out\textBuffer.cpp:2401`).

Hera implication: row metadata must be first-class from M1. Wrap flags, row
identity, semantic marks, image placeholders and export rules should not be
bolted on after a plain `Vec<Vec<Cell>>` model ships.

## Resize And Reflow

Windows Terminal gives a clear resize split:

- User resize enters `Terminal::UserResize`
  (`C:\dev\terminal\src\cascadia\TerminalCore\Terminal.cpp:292`).
- Resize while in alternate buffer is deferred because ConPTY reflow is
  destructive (`C:\dev\terminal\src\cascadia\TerminalCore\Terminal.cpp:303`,
  `C:\dev\terminal\src\cascadia\TerminalCore\Terminal.cpp:307`).
- Alternate buffer uses a traditional resize path with no reflow
  (`C:\dev\terminal\src\cascadia\TerminalCore\Terminal.cpp:309`,
  `C:\dev\terminal\src\cascadia\TerminalCore\Terminal.cpp:314`).
- Main buffer resize creates a new buffer with viewport plus scrollback height
  (`C:\dev\terminal\src\cascadia\TerminalCore\Terminal.cpp:318`,
  `C:\dev\terminal\src\cascadia\TerminalCore\Terminal.cpp:333`).
- Reflow is delegated to `TextBuffer::Reflow`
  (`C:\dev\terminal\src\cascadia\TerminalCore\Terminal.cpp:350`,
  `C:\dev\terminal\src\buffer\out\textBuffer.cpp:2718`).

`TextBuffer::Reflow` preserves wrapped logical lines into the new buffer
(`C:\dev\terminal\src\buffer\out\textBuffer.cpp:2706`) and handles double-height
rows as a union (`C:\dev\terminal\src\buffer\out\textBuffer.cpp:2755`). Marks
are explicitly tricky: a mark on a wrapped row can move when rows de-flow into
the previous line (`C:\dev\terminal\src\buffer\out\textBuffer.cpp:2807`,
`C:\dev\terminal\src\buffer\out\textBuffer.cpp:2820`).

The test suite has dedicated reflow fixtures for normal wrapping, shrinking,
growing, cursor behavior and marks
(`C:\dev\terminal\src\buffer\out\ut_textbuffer\ReflowTests.cpp:89`,
`C:\dev\terminal\src\buffer\out\ut_textbuffer\ReflowTests.cpp:138`,
`C:\dev\terminal\src\buffer\out\ut_textbuffer\ReflowTests.cpp:403`,
`C:\dev\terminal\src\buffer\out\ut_textbuffer\ReflowTests.cpp:645`).

Hera implication: reflow should be implemented behind tests before public API
stabilization. The core should distinguish primary buffer reflow, alternate
buffer resize and ConPTY-induced resize effects.

## Renderer Boundary

Windows Terminal's renderer is valuable because the shape is almost the API
Hera wants, even if the implementation is Windows-specific.

`IRenderData` is the renderer's view of terminal state: viewport, text buffer,
text buffer end position, font info, search highlights, selection spans, blink
interval, cursor width, title, hyperlinks and colors
(`C:\dev\terminal\src\renderer\inc\IRenderData.hpp:45`,
`C:\dev\terminal\src\renderer\inc\IRenderData.hpp:51`,
`C:\dev\terminal\src\renderer\inc\IRenderData.hpp:53`,
`C:\dev\terminal\src\renderer\inc\IRenderData.hpp:71`).
`terminalrenderdata.cpp` implements that surface over `Terminal`
(`C:\dev\terminal\src\cascadia\TerminalCore\terminalrenderdata.cpp:11`,
`C:\dev\terminal\src\cascadia\TerminalCore\terminalrenderdata.cpp:24`,
`C:\dev\terminal\src\cascadia\TerminalCore\terminalrenderdata.cpp:105`).

`IRenderEngine` receives frame lifecycle, invalidation, paint and font methods:
`StartPaint`, `EndPaint`, `Present`, `Invalidate`, `InvalidateScroll`,
`PrepareRenderInfo`, `PaintBufferLine`, `PaintImageSlice`, `UpdateFont` and
`GetDirtyArea` (`C:\dev\terminal\src\renderer\inc\IRenderEngine.hpp:62`,
`C:\dev\terminal\src\renderer\inc\IRenderEngine.hpp:77`,
`C:\dev\terminal\src\renderer\inc\IRenderEngine.hpp:81`,
`C:\dev\terminal\src\renderer\inc\IRenderEngine.hpp:92`).

`Renderer` owns invalidation and frame painting. It adds render engines
(`C:\dev\terminal\src\renderer\base\renderer.hpp:69`), triggers redraws
(`C:\dev\terminal\src\renderer\base\renderer.hpp:39`), paints frames
(`C:\dev\terminal\src\renderer\base\renderer.cpp:331`) and calls
`_PaintBufferOutput` over dirty areas and text-buffer iterators
(`C:\dev\terminal\src\renderer\base\renderer.cpp:1012`,
`C:\dev\terminal\src\renderer\base\renderer.cpp:1023`,
`C:\dev\terminal\src\renderer\base\renderer.cpp:1049`,
`C:\dev\terminal\src\renderer\base\renderer.cpp:1092`).

The warning is in `ControlCore`: it documents that `_terminal` depends on
`_renderer` for invalidations and `_renderer` depends on `_terminal` for
`IRenderData` (`C:\dev\terminal\src\cascadia\TerminalControl\ControlCore.h:443`,
`C:\dev\terminal\src\cascadia\TerminalControl\ControlCore.h:450`). This is the
boundary Hera should improve.

Hera implication: expose immutable or versioned render snapshots from
`terminal-core`; let renderers consume snapshots and damage lists. Do not create
a cycle where core mutation requires a concrete renderer and the renderer needs
mutable core access.

## ConPTY And TerminalConnection

Windows Terminal's connection abstraction is a direct reference for
`terminal-pty`.

`ITerminalConnection` exposes `Start`, `WriteInput`, `Resize`, `Close`,
`TerminalOutput` and `StateChanged`
(`C:\dev\terminal\src\cascadia\TerminalConnection\ITerminalConnection.idl:18`,
`C:\dev\terminal\src\cascadia\TerminalConnection\ITerminalConnection.idl:22`,
`C:\dev\terminal\src\cascadia\TerminalConnection\ITerminalConnection.idl:27`).
`ConptyConnection` implements the same surface
(`C:\dev\terminal\src\cascadia\TerminalConnection\ConptyConnection.idl:10`,
`C:\dev\terminal\src\cascadia\TerminalConnection\ConptyConnection.h:23`).

Important behaviors:

- Starting creates or receives a ConPTY and owns the pipe
  (`C:\dev\terminal\src\cascadia\TerminalConnection\ConptyConnection.cpp:400`,
  `C:\dev\terminal\src\cascadia\TerminalConnection\ConptyConnection.cpp:409`,
  `C:\dev\terminal\src\cascadia\TerminalConnection\ConptyConnection.cpp:413`).
- Writes convert input and use overlapped `WriteFile`
  (`C:\dev\terminal\src\cascadia\TerminalConnection\ConptyConnection.cpp:553`,
  `C:\dev\terminal\src\cascadia\TerminalConnection\ConptyConnection.cpp:590`).
- Resize clamps rows and columns before calling `ConptyResizePseudoConsole`
  (`C:\dev\terminal\src\cascadia\TerminalConnection\ConptyConnection.cpp:602`,
  `C:\dev\terminal\src\cascadia\TerminalConnection\ConptyConnection.cpp:610`).
- Close resets the pseudo console, cancels IO, waits for the output thread and
  releases the pipe (`C:\dev\terminal\src\cascadia\TerminalConnection\ConptyConnection.cpp:667`,
  `C:\dev\terminal\src\cascadia\TerminalConnection\ConptyConnection.cpp:697`).
- The output thread reads from the pipe and raises `TerminalOutput`
  (`C:\dev\terminal\src\cascadia\TerminalConnection\ConptyConnection.cpp:737`,
  `C:\dev\terminal\src\cascadia\TerminalConnection\ConptyConnection.cpp:797`).

The lower `winconpty` layer wraps pseudo console creation and signal operations.
`_PseudoConsole` stores signal, PTY reference and process handles
(`C:\dev\terminal\src\winconpty\winconpty.h:11`,
`C:\dev\terminal\src\winconpty\winconpty.h:41`). It supports resize, clear,
show/hide, reparent, release and close
(`C:\dev\terminal\src\winconpty\winconpty.h:65`,
`C:\dev\terminal\src\winconpty\winconpty.cpp:288`,
`C:\dev\terminal\src\winconpty\winconpty.cpp:311`,
`C:\dev\terminal\src\winconpty\winconpty.cpp:335`,
`C:\dev\terminal\src\winconpty\winconpty.cpp:361`,
`C:\dev\terminal\src\winconpty\winconpty.cpp:559`,
`C:\dev\terminal\src\winconpty\winconpty.cpp:586`).

The samples are useful as minimal adapters. `EchoCon` creates pipes and a
pseudo-console, attaches it through `PROC_THREAD_ATTRIBUTE_PSEUDOCONSOLE`, then
listens to the pipe (`C:\dev\terminal\samples\ConPTY\EchoCon\EchoCon\EchoCon.cpp:91`,
`C:\dev\terminal\samples\ConPTY\EchoCon\EchoCon\EchoCon.cpp:126`,
`C:\dev\terminal\samples\ConPTY\EchoCon\EchoCon\EchoCon.cpp:167`). `MiniTerm`
shows the same shape from C# with P/Invoke
(`C:\dev\terminal\samples\ConPTY\MiniTerm\MiniTerm\Native\PseudoConsoleApi.cs:12`,
`C:\dev\terminal\samples\ConPTY\MiniTerm\MiniTerm\Terminal.cs:50`).

Hera implication: model PTY as async byte streams plus resize/close lifecycle.
The Windows implementation can be Rust plus Win32 FFI. C# samples are examples,
not an implementation direction.

## Semantic Marks And Shell Integration

Windows Terminal is especially useful because it exposes the exact tension
Hera will face: semantic command metadata is valuable, but terminal correctness
cannot depend on it.

The marks spec says marks are buffer metadata and are used for command
starts/stops and user navigation (`C:\dev\terminal\doc\specs\#11000 - Marks\Shell-Integration-Marks.md:18`,
`C:\dev\terminal\doc\specs\#11000 - Marks\Shell-Integration-Marks.md:54`). It
also references FinalTerm and OSC 663, then OSC 133 A/B/C/D shell integration
(`C:\dev\terminal\doc\specs\#11000 - Marks\Shell-Integration-Marks.md:95`,
`C:\dev\terminal\doc\specs\#11000 - Marks\Shell-Integration-Marks.md:116`).

The spec documents that an initial vector-of-marks model did not reflow well
(`C:\dev\terminal\doc\specs\#11000 - Marks\Shell-Integration-Marks.md:144`,
`C:\dev\terminal\doc\specs\#11000 - Marks\Shell-Integration-Marks.md:146`) and
that reflow makes this difficult
(`C:\dev\terminal\doc\specs\#11000 - Marks\Shell-Integration-Marks.md:179`,
`C:\dev\terminal\doc\specs\#11000 - Marks\Shell-Integration-Marks.md:180`).

In source, `Terminal` exposes mark APIs and current command APIs
(`C:\dev\terminal\src\cascadia\TerminalCore\Terminal.hpp:123`,
`C:\dev\terminal\src\cascadia\TerminalCore\Terminal.hpp:129`,
`C:\dev\terminal\src\cascadia\TerminalCore\Terminal.hpp:162`). `TextBuffer`
has prompt/output/current command operations
(`C:\dev\terminal\src\buffer\out\textBuffer.hpp:306`,
`C:\dev\terminal\src\buffer\out\textBuffer.cpp:3432`,
`C:\dev\terminal\src\buffer\out\textBuffer.cpp:3492`,
`C:\dev\terminal\src\buffer\out\textBuffer.cpp:3501`).

Hera implication: semantic session intelligence should be a sidecar over row
handles and byte offsets. It may help agents navigate output, but it must never
be required for VT correctness, resize correctness or replay correctness.

## Parser And Fuzzing

Windows Terminal is not the recommended parser seed for Hera, but its parser
and fuzzers are valuable oracles.

`OutputStateMachineEngine` owns dispatch for execute, print, ESC, CSI, DCS, OSC
and SS3 pathways (`C:\dev\terminal\src\terminal\parser\OutputStateMachineEngine.cpp:54`,
`C:\dev\terminal\src\terminal\parser\OutputStateMachineEngine.cpp:132`,
`C:\dev\terminal\src\terminal\parser\OutputStateMachineEngine.cpp:197`,
`C:\dev\terminal\src\terminal\parser\OutputStateMachineEngine.cpp:411`,
`C:\dev\terminal\src\terminal\parser\OutputStateMachineEngine.cpp:709`,
`C:\dev\terminal\src\terminal\parser\OutputStateMachineEngine.cpp:763`,
`C:\dev\terminal\src\terminal\parser\OutputStateMachineEngine.cpp:934`).

There are two fuzzing surfaces worth tracking:

- Host output fuzzer entry through `LLVMFuzzerTestOneInput`
  (`C:\dev\terminal\src\host\ft_fuzzer\fuzzmain.cpp:127`).
- VT command fuzzer token generators for CSI/OSC and parameter variation
  (`C:\dev\terminal\src\terminal\parser\ft_fuzzer\VTCommandFuzzer.cpp:72`,
  `C:\dev\terminal\src\terminal\parser\ft_fuzzer\VTCommandFuzzer.cpp:141`,
  `C:\dev\terminal\src\terminal\parser\ft_fuzzer\VTCommandFuzzer.cpp:165`).

Hera implication: keep `alacritty/vte` as the M1 parser implementation, then
use Windows Terminal fuzzing ideas and crash cases to harden Hera's parser
adapter and terminal state application.

## In-Process ConPTY Spec Warning

The in-process ConPTY spec is valuable because it states the architectural pain
plainly. It says the ConPTY buffer can go out of sync
(`C:\dev\terminal\doc\specs\#13000 - In-process ConPTY.md:142`,
`C:\dev\terminal\doc\specs\#13000 - In-process ConPTY.md:148`) and that ConPTY
fulfilled needs while also causing software decay, so a cleaner architecture was
needed (`C:\dev\terminal\doc\specs\#13000 - In-process ConPTY.md:151`,
`C:\dev\terminal\doc\specs\#13000 - In-process ConPTY.md:154`).

It also discusses synchronization around buffer reflow and internal input ring
buffer ideas (`C:\dev\terminal\doc\specs\#13000 - In-process ConPTY.md:157`,
`C:\dev\terminal\doc\specs\#13000 - In-process ConPTY.md:515`).

Hera implication: never make ConPTY internals part of Hera's public state
model. The adapter should emit bytes and lifecycle events; the core should own
the model.

## What Hera Should Copy

Copy these:

1. Distinct `TerminalCore`, PTY connection, text buffer and renderer boundaries.
2. Explicit primary/alternate screen state and alternate resize rules.
3. Row wrap metadata and reflow fixtures.
4. Dirty-area renderer invalidation and a pull-based render data surface.
5. ConPTY lifecycle tests: create, attach, resize, release, close, broken output
   and pipe shutdown.
6. Semantic marks as row-attached metadata with known reflow edge cases.
7. Parser and host fuzzing as fixture generators.

Do not copy these:

1. C++/WinRT as the default stack.
2. A renderer-core ownership cycle.
3. A Windows-only app structure for a cross-platform engine.
4. C# samples as production direction.
5. ConPTY as authoritative state.

## Hera Backlog Seeds

1. Add Windows PTY adapter spike using Rust Win32 bindings and the
   `ITerminalConnection` shape as behavioral reference.
2. Build reflow fixtures from Windows Terminal `ReflowTests.cpp`, especially
   mark reflow cases.
3. Add tests that prove alternate screen resize does not reflow scrollback.
4. Add row metadata for wrap, semantic marks and image placeholders before
   public render snapshot stabilization.
5. Add parser fuzz corpus ideas from Windows Terminal's VT command fuzzer.
6. Add renderer API tests around damage rectangles, dirty rows, cursor updates
   and image slice placeholders.

## Bottom Line

Windows Terminal is not a cross-platform design to imitate. It is a Windows
compatibility oracle and a source of hard-earned failure modes. For Hera, the
correct extraction is: Rust core, Rust render model, Rust PTY traits, Win32 FFI
only at the Windows adapter boundary, and fixture parity against Windows
Terminal's ConPTY, reflow, renderer and marks behavior.

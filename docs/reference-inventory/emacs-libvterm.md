# emacs-libvterm Reference Inventory

Status: focused local inventory for Hera.

Reference source: `C:\dev\emacs-libvterm`

Scope: emacs-libvterm lessons for Hera, especially host embedding,
libvterm module boundaries, PTY/process coupling, damage-driven redraw,
scrollback, copy mode, and shell integration.

## Executive Takeaways

emacs-libvterm is a strong reference for embedding a compiled terminal core in a
host editor. It is not a terminal-core architecture to copy wholesale.

1. The project is explicitly a GNU Emacs terminal emulator backed by the C
   `libvterm` library, chosen for compatibility and large-output performance
   (`C:\dev\emacs-libvterm\README.md:5`,
   `C:\dev\emacs-libvterm\README.md:8`,
   `C:\dev\emacs-libvterm\README.md:23`).
2. The runtime depends on Emacs dynamic modules and errors early when module
   support is missing (`C:\dev\emacs-libvterm\README.md:65`,
   `C:\dev\emacs-libvterm\README.md:67`,
   `C:\dev\emacs-libvterm\vterm.el:61`).
3. The Elisp surface declares native entrypoints and loads `vterm-module`, while
   the C module exports those functions through the Emacs module API
   (`C:\dev\emacs-libvterm\vterm.el:136`,
   `C:\dev\emacs-libvterm\vterm.el:148`,
   `C:\dev\emacs-libvterm\vterm.el:156`,
   `C:\dev\emacs-libvterm\vterm-module.c:1451`,
   `C:\dev\emacs-libvterm\vterm-module.c:1533`).
4. The native `Term` object owns `VTerm`, `VTermScreen`, scrollback, invalid row
   ranges, cursor state, title, directory metadata, selection data, command
   buffer and PTY fd (`C:\dev\emacs-libvterm\vterm-module.h:69`,
   `C:\dev\emacs-libvterm\vterm-module.h:70`,
   `C:\dev\emacs-libvterm\vterm-module.h:71`,
   `C:\dev\emacs-libvterm\vterm-module.h:75`,
   `C:\dev\emacs-libvterm\vterm-module.h:88`,
   `C:\dev\emacs-libvterm\vterm-module.h:123`).
5. libvterm is configured with UTF-8, state fallbacks, screen callbacks, damage
   merge, alternate screen and reflow when the linked libvterm supports it
   (`C:\dev\emacs-libvterm\vterm-module.c:1253`,
   `C:\dev\emacs-libvterm\vterm-module.c:1259`,
   `C:\dev\emacs-libvterm\vterm-module.c:1267`,
   `C:\dev\emacs-libvterm\vterm-module.c:1268`,
   `C:\dev\emacs-libvterm\vterm-module.c:1269`,
   `C:\dev\emacs-libvterm\vterm-module.c:1270`,
   `C:\dev\emacs-libvterm\vterm-module.c:1272`).
6. The host process path is Emacs-specific and Unix-biased: `vterm-mode`
   creates an Emacs process through `/bin/sh -c stty ... exec`, uses
   `:connection-type 'pty`, installs `vterm--filter`, and gives native code the
   process TTY name (`C:\dev\emacs-libvterm\vterm.el:797`,
   `C:\dev\emacs-libvterm\vterm.el:801`,
   `C:\dev\emacs-libvterm\vterm.el:815`,
   `C:\dev\emacs-libvterm\vterm.el:817`,
   `C:\dev\emacs-libvterm\vterm.el:832`).
7. The output path is clean conceptually: process bytes flow into
   `vterm--write-input`, C calls `vterm_input_write`, then flushes screen damage
   (`C:\dev\emacs-libvterm\vterm.el:1549`,
   `C:\dev\emacs-libvterm\vterm.el:1609`,
   `C:\dev\emacs-libvterm\vterm-module.c:1373`,
   `C:\dev\emacs-libvterm\vterm-module.c:1374`).
8. The input path is also instructive: keys enter `vterm--update`, C maps named
   keys and Unicode through libvterm keyboard APIs, and libvterm output is
   flushed back to the Emacs process writer
   (`C:\dev\emacs-libvterm\vterm.el:1058`,
   `C:\dev\emacs-libvterm\vterm.el:1064`,
   `C:\dev\emacs-libvterm\vterm-module.c:917`,
   `C:\dev\emacs-libvterm\vterm-module.c:1027`,
   `C:\dev\emacs-libvterm\vterm-module.c:1348`,
   `C:\dev\emacs-libvterm\vterm-module.c:900`,
   `C:\dev\emacs-libvterm\vterm.el:1518`).
9. Redraw is damage-driven, but the renderer target is an Emacs text buffer:
   libvterm callbacks invalidate rows, delayed redraw calls native redraw, C
   refreshes scrollback and screen, and rendering inserts strings with Emacs text
   properties (`C:\dev\emacs-libvterm\vterm-module.c:575`,
   `C:\dev\emacs-libvterm\vterm-module.c:580`,
   `C:\dev\emacs-libvterm\vterm-module.c:586`,
   `C:\dev\emacs-libvterm\vterm.el:1403`,
   `C:\dev\emacs-libvterm\vterm.el:1434`,
   `C:\dev\emacs-libvterm\vterm-module.c:631`,
   `C:\dev\emacs-libvterm\vterm-module.c:777`,
   `C:\dev\emacs-libvterm\vterm-module.c:831`).
10. Shell integration is powerful but privileged. OSC 51;A sets directory and
    prompt metadata, OSC 51;E queues Elisp for redraw-time execution through a
    whitelist, and OSC 52 selection updates are disabled by default
    (`C:\dev\emacs-libvterm\vterm-module.c:1085`,
    `C:\dev\emacs-libvterm\vterm-module.c:1108`,
    `C:\dev\emacs-libvterm\vterm-module.c:1115`,
    `C:\dev\emacs-libvterm\vterm-module.c:651`,
    `C:\dev\emacs-libvterm\vterm.el:322`,
    `C:\dev\emacs-libvterm\vterm.el:341`,
    `C:\dev\emacs-libvterm\vterm.el:356`).

Confidence: high for module boundary, process flow, libvterm callbacks, redraw,
scrollback and shell integration because this pass covered README, CMake, Elisp,
C headers, C implementation and shell scripts. Medium for runtime behavior under
real Emacs sessions because this pass did not build or run the module.

## Why emacs-libvterm Matters

emacs-libvterm is the best local reference for one narrow Hera question: how a
terminal engine survives inside a powerful host that already has its own buffer,
process, keymap, selection, cursor, clipboard and directory semantics.

Hera should read it as a host-adapter reference. It proves that a native terminal
core can remain authoritative while the host owns display and process lifecycle,
but it also shows how quickly the boundary becomes host-specific.

## Codebase Map

| Area | Files | What It Shows | Hera Relevance |
|---|---|---|---|
| Elisp host API | `C:\dev\emacs-libvterm\vterm.el` | Mode setup, process creation, keymaps, copy mode, redraw scheduling, shell integration. | Strong host adapter reference. |
| Native module | `C:\dev\emacs-libvterm\vterm-module.c`, `vterm-module.h` | Emacs module API, libvterm screen callbacks, scrollback, render conversion, OSC handlers. | Strong embedding and damage boundary reference. |
| Build | `C:\dev\emacs-libvterm\CMakeLists.txt` | Dynamic module build, system or vendored libvterm, feature probes. | Useful warning for native dependency management. |
| Shell scripts | `C:\dev\emacs-libvterm\etc` | OSC 51 directory/prompt and command helpers for bash, zsh and fish. | Useful semantic side-channel reference. |
| Tests | README and CMake `run` target | Manual Emacs launch for smoke testing. | Weak fixture source. |

## Module Boundary

The Elisp side refuses to run without module support, compiles the module on
demand, declares native functions and then calls into `vterm-module`
(`C:\dev\emacs-libvterm\vterm.el:61`,
`C:\dev\emacs-libvterm\vterm.el:105`,
`C:\dev\emacs-libvterm\vterm.el:136`,
`C:\dev\emacs-libvterm\vterm.el:148`,
`C:\dev\emacs-libvterm\vterm.el:156`). The C side initializes symbols and
host function refs, then binds exported functions back into Elisp
(`C:\dev\emacs-libvterm\vterm-module.c:1451`,
`C:\dev\emacs-libvterm\vterm-module.c:1491`,
`C:\dev\emacs-libvterm\vterm-module.c:1498`,
`C:\dev\emacs-libvterm\vterm-module.c:1517`,
`C:\dev\emacs-libvterm\vterm-module.c:1533`,
`C:\dev\emacs-libvterm\vterm-module.c:1538`).

Hera implication: keep the core API host-neutral, then build adapters that own
host-specific callbacks. Do not let the first UI host define the core boundary.

## libvterm Core Ownership

`Term` owns libvterm state plus host metadata in one native object:
`VTerm *vt`, `VTermScreen *vts`, scrollback, invalid rows, cursor, title,
directory, selection and PTY fd
(`C:\dev\emacs-libvterm\vterm-module.h:69`,
`C:\dev\emacs-libvterm\vterm-module.h:70`,
`C:\dev\emacs-libvterm\vterm-module.h:71`,
`C:\dev\emacs-libvterm\vterm-module.h:75`,
`C:\dev\emacs-libvterm\vterm-module.h:88`,
`C:\dev\emacs-libvterm\vterm-module.h:91`,
`C:\dev\emacs-libvterm\vterm-module.h:95`,
`C:\dev\emacs-libvterm\vterm-module.h:104`,
`C:\dev\emacs-libvterm\vterm-module.h:123`).

`Fvterm_new` allocates libvterm, enables UTF-8, registers fallbacks and screen
callbacks, enables alternate screen and enables reflow when available
(`C:\dev\emacs-libvterm\vterm-module.c:1240`,
`C:\dev\emacs-libvterm\vterm-module.c:1253`,
`C:\dev\emacs-libvterm\vterm-module.c:1254`,
`C:\dev\emacs-libvterm\vterm-module.c:1256`,
`C:\dev\emacs-libvterm\vterm-module.c:1259`,
`C:\dev\emacs-libvterm\vterm-module.c:1268`,
`C:\dev\emacs-libvterm\vterm-module.c:1270`,
`C:\dev\emacs-libvterm\vterm-module.c:1272`).

Hera implication: the terminal core should own terminal state and emit typed
host effects. It should not embed host concepts like Emacs text properties or
process names in core state.

## Process And PTY Flow

`vterm-mode` builds the session environment, including `TERM`,
`EMACS_VTERM_PATH`, `INSIDE_EMACS`, `LINES` and `COLUMNS`
(`C:\dev\emacs-libvterm\vterm.el:760`,
`C:\dev\emacs-libvterm\vterm.el:761`,
`C:\dev\emacs-libvterm\vterm.el:763`,
`C:\dev\emacs-libvterm\vterm.el:765`,
`C:\dev\emacs-libvterm\vterm.el:766`). It then creates a PTY-backed Emacs
process using `/bin/sh -c stty ... exec`, installs `vterm--filter`, and passes
the process TTY name to native code
(`C:\dev\emacs-libvterm\vterm.el:797`,
`C:\dev\emacs-libvterm\vterm.el:801`,
`C:\dev\emacs-libvterm\vterm.el:803`,
`C:\dev\emacs-libvterm\vterm.el:815`,
`C:\dev\emacs-libvterm\vterm.el:817`,
`C:\dev\emacs-libvterm\vterm.el:832`).

Process output enters `vterm--filter`, is split around control sequences and
undecoded partial bytes, then goes through `vterm--write-input`
(`C:\dev\emacs-libvterm\vterm.el:1549`,
`C:\dev\emacs-libvterm\vterm.el:1568`,
`C:\dev\emacs-libvterm\vterm.el:1573`,
`C:\dev\emacs-libvterm\vterm.el:1603`,
`C:\dev\emacs-libvterm\vterm.el:1609`). Native code writes bytes into
libvterm and flushes screen damage
(`C:\dev\emacs-libvterm\vterm-module.c:1364`,
`C:\dev\emacs-libvterm\vterm-module.c:1373`,
`C:\dev\emacs-libvterm\vterm-module.c:1374`).

Key input flows the other way: Elisp calls `vterm--update`, C maps named keys or
Unicode to libvterm keyboard calls, then flushes libvterm output back to
`process-send-string`
(`C:\dev\emacs-libvterm\vterm.el:1058`,
`C:\dev\emacs-libvterm\vterm.el:1064`,
`C:\dev\emacs-libvterm\vterm-module.c:917`,
`C:\dev\emacs-libvterm\vterm-module.c:1027`,
`C:\dev\emacs-libvterm\vterm-module.c:1348`,
`C:\dev\emacs-libvterm\vterm-module.c:900`,
`C:\dev\emacs-libvterm\vterm-module.c:907`,
`C:\dev\emacs-libvterm\vterm.el:1518`).

Hera implication: copy the directional flow, not the implementation. Hera needs
a cross-platform PTY adapter, not `/bin/sh`, Emacs `make-process`, or direct
TTY filename plumbing in core.

## Damage And Rendering

libvterm callbacks invalidate rows for damage, rectangle moves and cursor moves
(`C:\dev\emacs-libvterm\vterm-module.c:575`,
`C:\dev\emacs-libvterm\vterm-module.c:580`,
`C:\dev\emacs-libvterm\vterm-module.c:586`). `term_redraw` refreshes
scrollback, refreshes the visible screen, adjusts the viewport, applies title
and directory changes, executes queued Elisp commands, handles selection data,
and clears invalidation
(`C:\dev\emacs-libvterm\vterm-module.c:628`,
`C:\dev\emacs-libvterm\vterm-module.c:633`,
`C:\dev\emacs-libvterm\vterm-module.c:634`,
`C:\dev\emacs-libvterm\vterm-module.c:636`,
`C:\dev\emacs-libvterm\vterm-module.c:640`,
`C:\dev\emacs-libvterm\vterm-module.c:645`,
`C:\dev\emacs-libvterm\vterm-module.c:651`,
`C:\dev\emacs-libvterm\vterm-module.c:662`,
`C:\dev\emacs-libvterm\vterm-module.c:672`).

Rendering converts `VTermScreenCell` rows into UTF-8 strings, groups style runs,
marks prompts and adds fake newline properties for visual line wraps
(`C:\dev\emacs-libvterm\vterm-module.c:311`,
`C:\dev\emacs-libvterm\vterm-module.c:353`,
`C:\dev\emacs-libvterm\vterm-module.c:369`,
`C:\dev\emacs-libvterm\vterm-module.c:383`,
`C:\dev\emacs-libvterm\vterm-module.c:394`,
`C:\dev\emacs-libvterm\vterm-module.c:847`). Text styling becomes Emacs text
properties for foreground, background, bold, underline, italic, reverse and
strike (`C:\dev\emacs-libvterm\vterm-module.c:787`,
`C:\dev\emacs-libvterm\vterm-module.c:796`,
`C:\dev\emacs-libvterm\vterm-module.c:831`).

Hera implication: damage-driven rendering is the lesson. Emacs buffer text
properties are not the render model Hera wants.

## Scrollback, Resize And Copy Mode

Native scrollback is bounded by `SB_MAX` and the Elisp default
`vterm-max-scrollback` (`C:\dev\emacs-libvterm\vterm-module.h:27`,
`C:\dev\emacs-libvterm\vterm.el:210`). `sb_pushline` copies cells into
`sb_buffer`, shifts rows, carries line metadata, and tracks pending rows for
refresh (`C:\dev\emacs-libvterm\vterm-module.c:29`,
`C:\dev\emacs-libvterm\vterm-module.c:64`,
`C:\dev\emacs-libvterm\vterm-module.c:72`,
`C:\dev\emacs-libvterm\vterm-module.c:95`,
`C:\dev\emacs-libvterm\vterm-module.c:101`,
`C:\dev\emacs-libvterm\vterm-module.c:109`). `refresh_scrollback` inserts
pending rows above the visible section and trims overflow
(`C:\dev\emacs-libvterm\vterm-module.c:480`,
`C:\dev\emacs-libvterm\vterm-module.c:491`,
`C:\dev\emacs-libvterm\vterm-module.c:512`,
`C:\dev\emacs-libvterm\vterm-module.c:517`,
`C:\dev\emacs-libvterm\vterm-module.c:524`).

Resize is coupled to Emacs window sizing. The Elisp adapter skips resize while
copy mode is active, clamps width, then calls native `vterm--set-size`
(`C:\dev\emacs-libvterm\vterm.el:1642`,
`C:\dev\emacs-libvterm\vterm.el:1650`,
`C:\dev\emacs-libvterm\vterm.el:1657`,
`C:\dev\emacs-libvterm\vterm.el:1662`). C records height delta, calls
`vterm_set_size`, flushes damage and redraws
(`C:\dev\emacs-libvterm\vterm-module.c:1386`,
`C:\dev\emacs-libvterm\vterm-module.c:1387`,
`C:\dev\emacs-libvterm\vterm-module.c:1393`,
`C:\dev\emacs-libvterm\vterm-module.c:1394`,
`C:\dev\emacs-libvterm\vterm-module.c:1396`).

Copy mode is a useful host contract: pause terminal output, expose buffer text
for normal selection/search, optionally remove fake newlines, then restore
terminal behavior (`C:\dev\emacs-libvterm\vterm.el:978`,
`C:\dev\emacs-libvterm\vterm.el:980`,
`C:\dev\emacs-libvterm\vterm.el:996`,
`C:\dev\emacs-libvterm\vterm.el:1017`,
`C:\dev\emacs-libvterm\vterm.el:1038`,
`C:\dev\emacs-libvterm\vterm.el:1911`,
`C:\dev\emacs-libvterm\vterm.el:1925`).

Hera implication: copy the user-mode contract for freezing and selecting, but
do not model huge scrollback as a single host text buffer.

## Shell Integration And Trust

Shell-side scripts emit OSC 51 sequences for host integration. `51;A` sets the
current directory and marks prompt end; `51;E` asks Emacs to execute a whitelisted
command (`C:\dev\emacs-libvterm\etc\emacs-vterm-bash.sh:31`,
`C:\dev\emacs-libvterm\etc\emacs-vterm-bash.sh:38`,
`C:\dev\emacs-libvterm\etc\emacs-vterm-bash.sh:52`,
`C:\dev\emacs-libvterm\etc\emacs-vterm-bash.sh:53`). The C handler stores
directory metadata and prompt columns, or queues Elisp code for redraw-time
execution (`C:\dev\emacs-libvterm\vterm-module.c:1085`,
`C:\dev\emacs-libvterm\vterm-module.c:1093`,
`C:\dev\emacs-libvterm\vterm-module.c:1108`,
`C:\dev\emacs-libvterm\vterm-module.c:1115`,
`C:\dev\emacs-libvterm\vterm-module.c:1123`,
`C:\dev\emacs-libvterm\vterm-module.c:651`,
`C:\dev\emacs-libvterm\vterm-module.c:655`).

The Elisp side constrains this with `vterm-eval-cmds`, whose documentation says
the explicit map avoids arbitrary code execution
(`C:\dev\emacs-libvterm\vterm.el:341`,
`C:\dev\emacs-libvterm\vterm.el:344`,
`C:\dev\emacs-libvterm\vterm.el:347`,
`C:\dev\emacs-libvterm\vterm.el:356`). OSC 52 support is disabled by default
for security reasons (`C:\dev\emacs-libvterm\vterm.el:322`,
`C:\dev\emacs-libvterm\vterm.el:333`,
`C:\dev\emacs-libvterm\vterm.el:334`,
`C:\dev\emacs-libvterm\vterm.el:335`).

Hera implication: semantic shell integration should emit typed events with an
explicit trust model. Treat shell-sourced commands as untrusted input unless the
user has opted into a narrow allowlist.

## Build And Test Surface

CMake builds `vterm-module` from C sources and links it against either system or
vendored libvterm (`C:\dev\emacs-libvterm\CMakeLists.txt:12`,
`C:\dev\emacs-libvterm\CMakeLists.txt:28`,
`C:\dev\emacs-libvterm\CMakeLists.txt:63`,
`C:\dev\emacs-libvterm\CMakeLists.txt:78`,
`C:\dev\emacs-libvterm\CMakeLists.txt:97`,
`C:\dev\emacs-libvterm\CMakeLists.txt:102`). It probes libvterm features and
sets compatibility defines for missing APIs
(`C:\dev\emacs-libvterm\CMakeLists.txt:40`,
`C:\dev\emacs-libvterm\CMakeLists.txt:45`,
`C:\dev\emacs-libvterm\CMakeLists.txt:50`,
`C:\dev\emacs-libvterm\CMakeLists.txt:54`). The test surface in this checkout
is essentially a `make run` Emacs smoke target
(`C:\dev\emacs-libvterm\CMakeLists.txt:104`,
`C:\dev\emacs-libvterm\CMakeLists.txt:106`,
`C:\dev\emacs-libvterm\README.md:290`,
`C:\dev\emacs-libvterm\README.md:296`).

Hera implication: this repo is not a fixture source. Use it for integration
shape and trust boundaries, not parser compatibility tests.

## What Hera Should Copy

1. Use a narrow host adapter around a core-owned terminal state
   (`C:\dev\emacs-libvterm\vterm-module.h:69`,
   `C:\dev\emacs-libvterm\vterm-module.c:1253`,
   `C:\dev\emacs-libvterm\vterm-module.c:1268`).
2. Keep process bytes, terminal state mutation, render invalidation and host
   writes as separate steps (`C:\dev\emacs-libvterm\vterm.el:1549`,
   `C:\dev\emacs-libvterm\vterm-module.c:1373`,
   `C:\dev\emacs-libvterm\vterm-module.c:575`,
   `C:\dev\emacs-libvterm\vterm-module.c:900`).
3. Use damage ranges and style-run batching before crossing into the UI
   (`C:\dev\emacs-libvterm\vterm-module.c:575`,
   `C:\dev\emacs-libvterm\vterm-module.c:353`,
   `C:\dev\emacs-libvterm\vterm-module.c:777`).
4. Treat copy/select mode as a first-class host mode that freezes terminal
   mutation from the user's perspective (`C:\dev\emacs-libvterm\vterm.el:996`,
   `C:\dev\emacs-libvterm\vterm.el:1017`).
5. Model shell integration as structured, optional semantics
   (`C:\dev\emacs-libvterm\vterm-module.c:1085`,
   `C:\dev\emacs-libvterm\vterm-module.c:1108`,
   `C:\dev\emacs-libvterm\vterm.el:1778`).

## What Hera Should Not Copy

1. Do not make the first render model a mutable host text buffer with injected
   fake newline markers (`C:\dev\emacs-libvterm\vterm-module.c:847`,
   `C:\dev\emacs-libvterm\vterm.el:1911`,
   `C:\dev\emacs-libvterm\vterm.el:1925`).
2. Do not put `/bin/sh -c stty ... exec` assumptions in core
   (`C:\dev\emacs-libvterm\vterm.el:801`,
   `C:\dev\emacs-libvterm\vterm.el:803`).
3. Do not use OSC command execution as a default product feature without a
   strict allowlist and user-controlled trust boundary
   (`C:\dev\emacs-libvterm\vterm-module.c:1115`,
   `C:\dev\emacs-libvterm\vterm.el:341`,
   `C:\dev\emacs-libvterm\vterm.el:356`).
4. Do not adopt its scrollback ceiling as Hera's long-session design
   (`C:\dev\emacs-libvterm\vterm-module.h:27`,
   `C:\dev\emacs-libvterm\vterm.el:210`).
5. Do not rely on its test surface for compatibility confidence
   (`C:\dev\emacs-libvterm\CMakeLists.txt:104`,
   `C:\dev\emacs-libvterm\README.md:296`).

## Open Questions For Hera

1. Should shell-sourced semantic commands be allowed only from local trusted PTYs,
   or also from remote SSH sessions?
2. Should copy mode freeze rendering, PTY reads, or only viewport mutation?
3. What is Hera's equivalent of host directory synchronization: terminal event,
   pane metadata, or external session index?
4. Where does the trust policy for OSC 52 and command-like OSCs live:
   `terminal-core`, `terminal-protocol`, or the app host?

## Bottom Line

emacs-libvterm should shape Hera's host adapter and shell-integration trust
model. Use it as evidence that embedding matters and that host UX can be rich,
but keep Hera's core renderer-neutral, PTY-neutral, fixture-driven and more
strict about privileged terminal-originated commands.

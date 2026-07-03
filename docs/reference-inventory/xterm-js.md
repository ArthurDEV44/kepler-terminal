# xterm.js Reference Inventory

Status: initial focused pass
Date: 2026-07-03
Reference source: `C:\dev\xterm.js`
Scope: xterm.js lessons for Hera, especially public embedding API, headless
state tracking, parser hooks, buffer API, addon boundaries, async write
pipeline, browser renderer separation, serialization, Unicode, image protocols
and fixture sources.

## Executive Takeaways

xterm.js is a high-value embedding and API reference, not a stack template for
Hera.

The high-value lessons:

1. The public API is deliberately stable and narrower than implementation
   internals. The declaration file says it declares the public API consumed by
   external programs (`C:\dev\xterm.js\typings\xterm.d.ts:1`), and the README
   points users to the declaration file as the canonical API surface
   (`C:\dev\xterm.js\README.md:122`).
2. xterm.js separates a common terminal core from browser and headless public
   wrappers. `CoreTerminal` wires services, parser, input handler and write
   buffer (`C:\dev\xterm.js\src\common\CoreTerminal.ts:123`,
   `C:\dev\xterm.js\src\common\CoreTerminal.ts:143`,
   `C:\dev\xterm.js\src\common\CoreTerminal.ts:158`), while
   `@xterm/headless` is packaged as a Node.js headless terminal
   (`C:\dev\xterm.js\headless\package.json:2`,
   `C:\dev\xterm.js\headless\README.md:5`).
3. The browser package is explicitly a frontend component, not a terminal app
   and not a shell (`C:\dev\xterm.js\README.md:3`,
   `C:\dev\xterm.js\README.md:16`, `C:\dev\xterm.js\README.md:17`).
   That matches Hera's split between `terminal-core` and `terminal-pty`.
4. The buffer API exposes active, normal and alternate buffers, and documents
   that alternate buffer activation happens through DECSET 47
   (`C:\dev\xterm.js\typings\xterm.d.ts:1697`,
   `C:\dev\xterm.js\typings\xterm.d.ts:1699`,
   `C:\dev\xterm.js\typings\xterm.d.ts:1704`,
   `C:\dev\xterm.js\typings\xterm.d.ts:1709`). This is a useful API model for
   Hera's render and inspection surface.
5. The parser hook API is powerful but also a warning. xterm.js implements
   parser actions synchronously for speed, while public hooks may return
   `Promise<boolean>` and pause input processing
   (`C:\dev\xterm.js\typings\xterm.d.ts:1937`,
   `C:\dev\xterm.js\typings\xterm.d.ts:1941`). Hera should avoid async parser
   hooks inside the correctness path.
6. The write pipeline is a major lesson. `WriteBuffer` aggregates chunks,
   emits `onWriteParsed`, has a write timeout and explicitly handles async
   parser continuations (`C:\dev\xterm.js\src\common\input\WriteBuffer.ts:27`,
   `C:\dev\xterm.js\src\common\input\WriteBuffer.ts:47`,
   `C:\dev\xterm.js\src\common\input\WriteBuffer.ts:188`,
   `C:\dev\xterm.js\src\common\input\WriteBuffer.ts:219`). Hera needs a clear
   ingress/backpressure contract before Paneflow dogfood.
7. Addons are the strongest transferable product idea. xterm.js keeps WebGL,
   serialize, image, search, links, Unicode providers and fit logic outside the
   core package (`C:\dev\xterm.js\README.md:78`,
   `C:\dev\xterm.js\README.md:101`,
   `C:\dev\xterm.js\README.md:105`,
   `C:\dev\xterm.js\README.md:110`).
8. Serialization plus headless mode directly supports reconnectable sessions:
   the README names headless state tracking with the serialize addon as a
   reconnection use case (`C:\dev\xterm.js\README.md:118`).
9. xterm.js is a strong fixture source. It ships escape sequence fixtures,
   parser integration tests, input handler tests and benchmarks
   (`C:\dev\xterm.js\test\fixtures\escape_sequence_files\NOTES:1`,
   `C:\dev\xterm.js\test\playwright\Parser.test.ts:30`,
   `C:\dev\xterm.js\test\playwright\InputHandler.test.ts:17`,
   `C:\dev\xterm.js\test\benchmark\EscapeSequenceParser.benchmark.ts:45`).
10. The TypeScript/browser implementation should not change Hera's Rust-first
    decision. Its reusable value is API shape, host separation, addon seams and
    tests.
11. The WebGL renderer is useful evidence for renderer replacement, but not a
    clean plugin contract: the addon reaches through `unsafeCore` to private
    browser services before calling `setRenderer`
    (`C:\dev\xterm.js\addons\addon-webgl\src\WebglAddon.ts:62`,
    `C:\dev\xterm.js\addons\addon-webgl\src\WebglAddon.ts:88`).

Confidence: high for public API, package shape, headless, core services,
parser hooks, buffer/reflow, renderer, addons, serialization and fixtures
because this pass covered local README, package manifests, TypeScript typings,
common core files, browser renderer files, headless wrapper, addons and tests
under `C:\dev\xterm.js`. Medium for full runtime performance conclusions
because benchmarks were sampled as structure and not executed.

## Why xterm.js Matters

xterm.js matters because it is the clearest reference for embedding a terminal
engine into another product. Its README positions it as a browser frontend
component used by VS Code, Tabby and Hyper (`C:\dev\xterm.js\README.md:3`).
It also states that most real use cases connect it to a pseudoterminal API such
as node-pty (`C:\dev\xterm.js\README.md:41`). This is exactly the mental model
Hera wants: terminal state core plus external IO transport plus host renderer.

The project is also useful because it has a public API bias. The package
exports `@xterm/xterm` as browser-oriented JS, module, style and typings
(`C:\dev\xterm.js\package.json:2`, `C:\dev\xterm.js\package.json:5`,
`C:\dev\xterm.js\package.json:6`, `C:\dev\xterm.js\package.json:7`,
`C:\dev\xterm.js\package.json:8`). The separate headless package exports
`@xterm/headless` with its own JS/module/typings files
(`C:\dev\xterm.js\headless\package.json:2`,
`C:\dev\xterm.js\headless\package.json:5`,
`C:\dev\xterm.js\headless\package.json:6`,
`C:\dev\xterm.js\headless\package.json:7`).

Hera implication: design the Rust API as if external hosts matter from day one.
Do not wait until after the core exists to decide what embedders can observe.

## Codebase Map

| Area | Local path | What it contains | Hera relevance |
|---|---|---|---|
| Public browser API | `C:\dev\xterm.js\typings\xterm.d.ts`, `src\browser\public` | `Terminal`, buffer, parser, addons, links, decorations, options. | Best API ergonomics reference in this set. |
| Headless API | `C:\dev\xterm.js\headless`, `src\headless` | Node package and public wrapper around common core. | Direct reference for Hera headless-first design. |
| Common core | `C:\dev\xterm.js\src\common` | `CoreTerminal`, services, input handler, parser, buffers, write queue. | State/core split and async ingress lessons. |
| Parser | `C:\dev\xterm.js\src\common\parser` | VT500 transition table, CSI/DCS/OSC/APC parsers, async handler stack. | Parser hook caution and protocol fixture source. |
| Buffer model | `C:\dev\xterm.js\src\common\buffer` | normal/alt buffers, circular list, lines, cells, reflow. | Useful row and reflow behavior reference. |
| Browser host | `C:\dev\xterm.js\src\browser` | DOM open path, render service, selection, viewport, accessibility, links. | Host adapter reference only, not core model. |
| Renderers | `C:\dev\xterm.js\src\browser\renderer`, `addons\addon-webgl` | DOM renderer, WebGL renderer, render service, render model. | Confirms renderer as swappable adapter. |
| Addons | `C:\dev\xterm.js\addons` | attach, clipboard, fit, image, search, serialize, unicode, web links, WebGL. | Strong optional capability boundary, with some private seams to avoid. |
| Fixtures and tests | `C:\dev\xterm.js\test` | escape sequence files, Playwright integration tests, unit tests, benchmarks. | High-value compatibility and regression corpus. |

## PTY And Transport Boundary

xterm.js does not own process lifecycle. The README states it is not a shell
and that applications connect it to processes through a library such as
node-pty (`C:\dev\xterm.js\README.md:17`,
`C:\dev\xterm.js\README.md:41`). The attach addon reinforces this boundary:
it attaches the terminal to a WebSocket stream, writes incoming socket data into
the terminal and forwards terminal `onData` back to the socket
(`C:\dev\xterm.js\addons\addon-attach\src\AttachAddon.ts:5`,
`C:\dev\xterm.js\addons\addon-attach\src\AttachAddon.ts:27`,
`C:\dev\xterm.js\addons\addon-attach\src\AttachAddon.ts:31`,
`C:\dev\xterm.js\addons\addon-attach\src\AttachAddon.ts:36`,
`C:\dev\xterm.js\addons\addon-attach\src\AttachAddon.ts:54`).

xterm.js also has Windows PTY compatibility knobs, but these are transport
heuristics, not a core architecture target. `CoreTerminal` watches the
`windowsPty` option and toggles behavior for older ConPTY builds
(`C:\dev\xterm.js\src\common\CoreTerminal.ts:152`,
`C:\dev\xterm.js\src\common\CoreTerminal.ts:279`,
`C:\dev\xterm.js\src\common\CoreTerminal.ts:283`). `Buffer` checks
`windowsPty` during resize/reflow decisions
(`C:\dev\xterm.js\src\common\buffer\Buffer.ts:199`,
`C:\dev\xterm.js\src\common\buffer\Buffer.ts:316`,
`C:\dev\xterm.js\src\common\buffer\Buffer.ts:318`), and `WindowsMode`
documents wraparound issues in winpty-like backends
(`C:\dev\xterm.js\src\common\WindowsMode.ts:10`,
`C:\dev\xterm.js\src\common\WindowsMode.ts:16`).

Hera implication: keep process lifecycle and ConPTY quirks in `terminal-pty`.
Use xterm.js Windows behavior as fixture input and compatibility notes, not as
evidence that core state should know the host transport.

## Public API And Headless Boundary

The browser API exposes a `Terminal` class with `buffer`, `parser`, events,
write, resize, selection, markers, decorations, links and addons
(`C:\dev\xterm.js\typings\xterm.d.ts:925`,
`C:\dev\xterm.js\typings\xterm.d.ts:959`,
`C:\dev\xterm.js\typings\xterm.d.ts:970`,
`C:\dev\xterm.js\typings\xterm.d.ts:1093`,
`C:\dev\xterm.js\typings\xterm.d.ts:1381`,
`C:\dev\xterm.js\typings\xterm.d.ts:1430`). The headless declaration exposes
the same core ideas without browser-only concerns
(`C:\dev\xterm.js\typings\xterm-headless.d.ts:646`,
`C:\dev\xterm.js\typings\xterm-headless.d.ts:664`,
`C:\dev\xterm.js\typings\xterm-headless.d.ts:675`,
`C:\dev\xterm.js\typings\xterm-headless.d.ts:898`,
`C:\dev\xterm.js\typings\xterm-headless.d.ts:924`).

The concrete headless public wrapper delegates to a core object for resize,
write, writeln, clear, reset and addons
(`C:\dev\xterm.js\src\headless\public\Terminal.ts:27`,
`C:\dev\xterm.js\src\headless\public\Terminal.ts:129`,
`C:\dev\xterm.js\src\headless\public\Terminal.ts:161`,
`C:\dev\xterm.js\src\headless\public\Terminal.ts:164`,
`C:\dev\xterm.js\src\headless\public\Terminal.ts:171`,
`C:\dev\xterm.js\src\headless\public\Terminal.ts:174`). The internal headless
terminal extends `CoreTerminal`, exposes active buffer convenience access and
implements marker, resize, clear and reset operations
(`C:\dev\xterm.js\src\headless\Terminal.ts:30`,
`C:\dev\xterm.js\src\headless\Terminal.ts:62`,
`C:\dev\xterm.js\src\headless\Terminal.ts:73`,
`C:\dev\xterm.js\src\headless\Terminal.ts:90`,
`C:\dev\xterm.js\src\headless\Terminal.ts:101`,
`C:\dev\xterm.js\src\headless\Terminal.ts:122`).

Hera implication: keep `terminal-core` public enough for embedders, but hide
implementation details behind stable Rust interfaces. A future desktop adapter,
Paneflow adapter and CLI replay tool should all consume the same core API.

## Core Terminal And Write Pipeline

`CoreTerminal` is the key common object. It creates options, log, buffer,
charset, core, mouse, Unicode, OSC link, input handler and write buffer services
(`C:\dev\xterm.js\src\common\CoreTerminal.ts:123`,
`C:\dev\xterm.js\src\common\CoreTerminal.ts:127`,
`C:\dev\xterm.js\src\common\CoreTerminal.ts:129`,
`C:\dev\xterm.js\src\common\CoreTerminal.ts:133`,
`C:\dev\xterm.js\src\common\CoreTerminal.ts:143`,
`C:\dev\xterm.js\src\common\CoreTerminal.ts:158`). Writes flow into
`WriteBuffer`, which calls `InputHandler.parse`
(`C:\dev\xterm.js\src\common\CoreTerminal.ts:158`,
`C:\dev\xterm.js\src\common\CoreTerminal.ts:162`).

Resize flushes pending writes before applying dimensions, which documents a
real race: resize while writes are pending can make buffer state inconsistent
(`C:\dev\xterm.js\src\common\CoreTerminal.ts:187`,
`C:\dev\xterm.js\src\common\CoreTerminal.ts:195`,
`C:\dev\xterm.js\src\common\CoreTerminal.ts:197`). `writeSync` exists but is
warned as unreliable and planned for removal
(`C:\dev\xterm.js\src\common\CoreTerminal.ts:167`,
`C:\dev\xterm.js\src\common\CoreTerminal.ts:175`,
`C:\dev\xterm.js\src\common\CoreTerminal.ts:177`).

`WriteBuffer` is worth studying for Hera's long-session ingress policy. It
tracks pending data, provides flush/writeSync/write paths, uses a write timeout,
handles async parser promise continuations and emits `onWriteParsed`
(`C:\dev\xterm.js\src\common\input\WriteBuffer.ts:39`,
`C:\dev\xterm.js\src\common\input\WriteBuffer.ts:64`,
`C:\dev\xterm.js\src\common\input\WriteBuffer.ts:106`,
`C:\dev\xterm.js\src\common\input\WriteBuffer.ts:152`,
`C:\dev\xterm.js\src\common\input\WriteBuffer.ts:188`,
`C:\dev\xterm.js\src\common\input\WriteBuffer.ts:219`,
`C:\dev\xterm.js\src\common\input\WriteBuffer.ts:314`).

Hera implication: byte ingestion should have explicit modes:

- synchronous test/replay path
- bounded async PTY path
- resize barrier
- parsed-event notification
- backpressure strategy for huge agent output

Do not bury these policies inside a renderer or PTY adapter.

## Parser Boundary And Hooks

xterm.js has a custom `EscapeSequenceParser` with a VT500 transition table
(`C:\dev\xterm.js\src\common\parser\EscapeSequenceParser.ts:97`,
`C:\dev\xterm.js\src\common\parser\EscapeSequenceParser.ts:263`,
`C:\dev\xterm.js\src\common\parser\EscapeSequenceParser.ts:301`). It exposes
registrations for ESC, CSI, DCS, OSC and APC handlers
(`C:\dev\xterm.js\src\common\parser\EscapeSequenceParser.ts:391`,
`C:\dev\xterm.js\src\common\parser\EscapeSequenceParser.ts:426`,
`C:\dev\xterm.js\src\common\parser\EscapeSequenceParser.ts:447`,
`C:\dev\xterm.js\src\common\parser\EscapeSequenceParser.ts:457`,
`C:\dev\xterm.js\src\common\parser\EscapeSequenceParser.ts:467`) and parses
UTF-32 codepoint arrays (`C:\dev\xterm.js\src\common\parser\EscapeSequenceParser.ts:574`).

The `InputHandler` constructs that parser and registers the semantic actions
that mutate terminal state (`C:\dev\xterm.js\src\common\InputHandler.ts:99`,
`C:\dev\xterm.js\src\common\InputHandler.ts:162`,
`C:\dev\xterm.js\src\common\InputHandler.ts:200`,
`C:\dev\xterm.js\src\common\InputHandler.ts:205`,
`C:\dev\xterm.js\src\common\InputHandler.ts:267`,
`C:\dev\xterm.js\src\common\InputHandler.ts:286`,
`C:\dev\xterm.js\src\common\InputHandler.ts:430`,
`C:\dev\xterm.js\src\common\InputHandler.ts:517`). Public parser hooks are
forwarded through `InputHandler` and `ParserApi`
(`C:\dev\xterm.js\src\common\InputHandler.ts:676`,
`C:\dev\xterm.js\src\common\InputHandler.ts:694`,
`C:\dev\xterm.js\src\common\InputHandler.ts:708`,
`C:\dev\xterm.js\src\common\public\ParserApi.ts:13`,
`C:\dev\xterm.js\src\common\public\ParserApi.ts:19`,
`C:\dev\xterm.js\src\common\public\ParserApi.ts:31`,
`C:\dev\xterm.js\src\common\public\ParserApi.ts:37`).

Hera implication: keep `alacritty/vte` as the first Rust parser seed, but copy
the API lesson: a clean internal action layer plus optional extension points.
Async parser hooks should not be in the hot correctness path. If extension
hooks exist, they should observe or emit side effects through bounded queues.

## Buffer, Scrollback And Reflow

The public buffer API is a useful model because it exposes stable concepts
without exposing raw storage. `BufferNamespaceApi` wraps active, normal and
alternate views (`C:\dev\xterm.js\src\common\public\BufferNamespaceApi.ts:12`,
`C:\dev\xterm.js\src\common\public\BufferNamespaceApi.ts:21`,
`C:\dev\xterm.js\src\common\public\BufferNamespaceApi.ts:25`,
`C:\dev\xterm.js\src\common\public\BufferNamespaceApi.ts:30`,
`C:\dev\xterm.js\src\common\public\BufferNamespaceApi.ts:33`). `BufferApiView`
exposes cursor, viewport, base row and line access
(`C:\dev\xterm.js\src\common\public\BufferApiView.ts:11`,
`C:\dev\xterm.js\src\common\public\BufferApiView.ts:22`,
`C:\dev\xterm.js\src\common\public\BufferApiView.ts:24`,
`C:\dev\xterm.js\src\common\public\BufferApiView.ts:25`,
`C:\dev\xterm.js\src\common\public\BufferApiView.ts:27`).

Internally, `BufferSet` owns normal, alt and active buffer changes
(`C:\dev\xterm.js\src\common\buffer\BufferSet.ts:16`,
`C:\dev\xterm.js\src\common\buffer\BufferSet.ts:23`,
`C:\dev\xterm.js\src\common\buffer\BufferSet.ts:82`,
`C:\dev\xterm.js\src\common\buffer\BufferSet.ts:103`). `Buffer` owns lines,
cursor, ybase, ydisp, scroll margins and markers
(`C:\dev\xterm.js\src\common\buffer\Buffer.ts:30`,
`C:\dev\xterm.js\src\common\buffer\Buffer.ts:31`,
`C:\dev\xterm.js\src\common\buffer\Buffer.ts:32`,
`C:\dev\xterm.js\src\common\buffer\Buffer.ts:33`,
`C:\dev\xterm.js\src\common\buffer\Buffer.ts:34`,
`C:\dev\xterm.js\src\common\buffer\Buffer.ts:36`,
`C:\dev\xterm.js\src\common\buffer\Buffer.ts:47`).

Rows are compact typed-array cell storage. `BufferLine` uses `Uint32Array`,
stores `isWrapped`, avoids repeated allocation through reusable `CellData`, and
handles wide/null cells during insert/delete/replace/resize
(`C:\dev\xterm.js\src\common\buffer\BufferLine.ts:63`,
`C:\dev\xterm.js\src\common\buffer\BufferLine.ts:74`,
`C:\dev\xterm.js\src\common\buffer\BufferLine.ts:75`,
`C:\dev\xterm.js\src\common\buffer\BufferLine.ts:87`,
`C:\dev\xterm.js\src\common\buffer\BufferLine.ts:203`,
`C:\dev\xterm.js\src\common\buffer\BufferLine.ts:287`,
`C:\dev\xterm.js\src\common\buffer\BufferLine.ts:315`,
`C:\dev\xterm.js\src\common\buffer\BufferLine.ts:382`,
`C:\dev\xterm.js\src\common\buffer\BufferLine.ts:536`).

Scrollback is a circular list with trim events. `CircularList.push` trims when
the max length is reached, `recycle` requires a full buffer, and `trimStart`
fires trim events (`C:\dev\xterm.js\src\common\CircularList.ts:45`,
`C:\dev\xterm.js\src\common\CircularList.ts:54`,
`C:\dev\xterm.js\src\common\CircularList.ts:129`,
`C:\dev\xterm.js\src\common\CircularList.ts:131`,
`C:\dev\xterm.js\src\common\CircularList.ts:144`,
`C:\dev\xterm.js\src\common\CircularList.ts:213`).

Reflow is implemented as explicit line-layout transforms. Larger resize finds
wrapped blocks, can skip the cursor line unless `reflowCursorLine` is enabled,
copies cells between rows and removes unwrapped rows
(`C:\dev\xterm.js\src\common\buffer\BufferReflow.ts:25`,
`C:\dev\xterm.js\src\common\buffer\BufferReflow.ts:31`,
`C:\dev\xterm.js\src\common\buffer\BufferReflow.ts:45`,
`C:\dev\xterm.js\src\common\buffer\BufferReflow.ts:65`,
`C:\dev\xterm.js\src\common\buffer\BufferReflow.ts:93`). Smaller resize
precomputes new line lengths, including wide-character wrapping cases
(`C:\dev\xterm.js\src\common\buffer\BufferReflow.ts:166`,
`C:\dev\xterm.js\src\common\buffer\BufferReflow.ts:179`,
`C:\dev\xterm.js\src\common\buffer\BufferReflow.ts:203`). Tests cover wide
characters and cursor-line behavior
(`C:\dev\xterm.js\src\common\buffer\BufferReflow.test.ts:25`,
`C:\dev\xterm.js\src\common\buffer\BufferReflow.test.ts:113`).

Hera implication: xterm.js is not the best huge-scrollback storage model, but
it is a good M1 API and reflow fixture reference. Hera should combine this API
clarity with Ghostty-style page/chunk storage and explicit byte budgets.

## Renderer Boundary And Browser Host

The browser terminal extends `CoreTerminal` and creates browser-only services
in `open`: DOM elements, render service, viewport, selection, linkifier,
composition helper, accessibility manager and overview ruler
(`C:\dev\xterm.js\src\browser\CoreBrowserTerminal.ts:63`,
`C:\dev\xterm.js\src\browser\CoreBrowserTerminal.ts:438`,
`C:\dev\xterm.js\src\browser\CoreBrowserTerminal.ts:535`,
`C:\dev\xterm.js\src\browser\CoreBrowserTerminal.ts:559`,
`C:\dev\xterm.js\src\browser\CoreBrowserTerminal.ts:584`,
`C:\dev\xterm.js\src\browser\CoreBrowserTerminal.ts:590`,
`C:\dev\xterm.js\src\browser\CoreBrowserTerminal.ts:632`,
`C:\dev\xterm.js\src\browser\CoreBrowserTerminal.ts:639`).

`RenderService` owns the renderer instance, debounces refreshes and delegates
row painting through the `IRenderer` interface
(`C:\dev\xterm.js\src\browser\services\RenderService.ts:25`,
`C:\dev\xterm.js\src\browser\services\RenderService.ts:28`,
`C:\dev\xterm.js\src\browser\services\RenderService.ts:74`,
`C:\dev\xterm.js\src\browser\services\RenderService.ts:156`,
`C:\dev\xterm.js\src\browser\services\RenderService.ts:184`,
`C:\dev\xterm.js\src\browser\services\RenderService.ts:203`,
`C:\dev\xterm.js\src\browser\services\RenderService.ts:247`). The renderer
interface is row based (`C:\dev\xterm.js\src\browser\renderer\shared\Types.ts:53`,
`C:\dev\xterm.js\src\browser\renderer\shared\Types.ts:72`).

The DOM renderer and WebGL addon are separate consumers of the same terminal
state. DOM rendering builds rows and selection overlays
(`C:\dev\xterm.js\src\browser\renderer\dom\DomRenderer.ts:39`,
`C:\dev\xterm.js\src\browser\renderer\dom\DomRenderer.ts:527`,
`C:\dev\xterm.js\src\browser\renderer\dom\DomRendererRowFactory.ts:38`).
The WebGL addon installs a canvas WebGL2 renderer and uses its own render model
(`C:\dev\xterm.js\addons\addon-webgl\src\WebglAddon.ts:17`,
`C:\dev\xterm.js\addons\addon-webgl\src\WebglAddon.ts:41`,
`C:\dev\xterm.js\addons\addon-webgl\src\WebglRenderer.ts:35`,
`C:\dev\xterm.js\addons\addon-webgl\src\WebglRenderer.ts:45`,
`C:\dev\xterm.js\addons\addon-webgl\src\RenderModel.ts:19`,
`C:\dev\xterm.js\addons\addon-webgl\src\WebglRenderer.ts:354`).
The caveat: WebGL gets there through private `unsafeCore` access before
installing the renderer (`C:\dev\xterm.js\addons\addon-webgl\src\WebglAddon.ts:62`,
`C:\dev\xterm.js\addons\addon-webgl\src\WebglAddon.ts:63`,
`C:\dev\xterm.js\addons\addon-webgl\src\WebglAddon.ts:88`,
`C:\dev\xterm.js\addons\addon-webgl\src\WebglAddon.ts:95`).

Hera implication: copy the renderer swappability, not the DOM/WebGL model. The
Rust core should expose a pull snapshot and damage model. GPUI, CLI snapshot
dumps and future remote renderers consume that model.

## Links, Decorations, Selection And Accessibility

xterm.js stores OSC 8 hyperlink metadata separately from link activation policy.
`InputHandler` registers OSC 8 and writes hyperlink IDs into line/cell metadata
(`C:\dev\xterm.js\src\common\InputHandler.ts:297`,
`C:\dev\xterm.js\src\common\InputHandler.ts:298`,
`C:\dev\xterm.js\src\common\InputHandler.ts:570`,
`C:\dev\xterm.js\src\common\InputHandler.ts:3088`,
`C:\dev\xterm.js\src\common\InputHandler.ts:3104`). `OscLinkService` maps link
IDs to data and markers (`C:\dev\xterm.js\src\common\services\OscLinkService.ts:20`,
`C:\dev\xterm.js\src\common\services\OscLinkService.ts:31`,
`C:\dev\xterm.js\src\common\services\OscLinkService.ts:70`,
`C:\dev\xterm.js\src\common\services\OscLinkService.ts:82`). The public API
warns embedders to validate link protocols
(`C:\dev\xterm.js\typings\xterm.d.ts:153`,
`C:\dev\xterm.js\typings\xterm.d.ts:156`,
`C:\dev\xterm.js\typings\xterm.d.ts:159`,
`C:\dev\xterm.js\typings\xterm.d.ts:160`).

Decorations are marker-based and line-indexed, with explicit sync after trim,
insert and delete events
(`C:\dev\xterm.js\src\common\services\DecorationService.ts:20`,
`C:\dev\xterm.js\src\common\services\DecorationService.ts:24`,
`C:\dev\xterm.js\src\common\services\DecorationService.ts:54`,
`C:\dev\xterm.js\src\common\services\DecorationService.ts:118`,
`C:\dev\xterm.js\src\common\services\DecorationService.ts:153`,
`C:\dev\xterm.js\src\common\services\DecorationService.ts:259`,
`C:\dev\xterm.js\src\common\services\DecorationService.ts:298`). This is
directly relevant to Hera's future semantic timeline.

Selection is a host/browser service over buffer coordinates, not part of
parser correctness. It adjusts for buffer trim, wrapped lines and wide cells
(`C:\dev\xterm.js\src\browser\services\SelectionService.ts:71`,
`C:\dev\xterm.js\src\browser\services\SelectionService.ts:144`,
`C:\dev\xterm.js\src\browser\services\SelectionService.ts:203`,
`C:\dev\xterm.js\src\browser\services\SelectionService.ts:224`,
`C:\dev\xterm.js\src\browser\services\SelectionService.ts:386`,
`C:\dev\xterm.js\src\browser\services\SelectionService.ts:1048`).

Accessibility is explicitly browser-host work. The README lists screen reader
mode and contrast support (`C:\dev\xterm.js\README.md:11`), while the
`AccessibilityManager` owns live-region and ARIA behavior
(`C:\dev\xterm.js\src\browser\AccessibilityManager.ts:28`,
`C:\dev\xterm.js\src\browser\AccessibilityManager.ts:85`,
`C:\dev\xterm.js\src\browser\AccessibilityManager.ts:196`,
`C:\dev\xterm.js\src\browser\AccessibilityManager.ts:423`).

Hera implication: command markers, hyperlinks, selections, decorations and
accessibility all need stable row/cell identity. They should observe core
state without becoming parser truth.

## Addons, Serialization, Images And Unicode

`AddonManager` is tiny and disciplined: load, activate, dispose
(`C:\dev\xterm.js\src\common\public\AddonManager.ts:14`,
`C:\dev\xterm.js\src\common\public\AddonManager.ts:17`,
`C:\dev\xterm.js\src\common\public\AddonManager.ts:23`,
`C:\dev\xterm.js\src\common\public\AddonManager.ts:31`). This is the right
shape for future Hera optional crates and host extensions.

The serialize addon is a strong snapshot/reconnect reference. It serializes
buffer content to VT sequences or HTML (`C:\dev\xterm.js\addons\addon-serialize\README.md:3`),
uses buffer and cell APIs internally (`C:\dev\xterm.js\addons\addon-serialize\src\SerializeAddon.ts:30`,
`C:\dev\xterm.js\addons\addon-serialize\src\SerializeAddon.ts:48`), exposes
`serialize` and `serializeAsHTML` (`C:\dev\xterm.js\addons\addon-serialize\typings\addon-serialize.d.ts:33`,
`C:\dev\xterm.js\addons\addon-serialize\typings\addon-serialize.d.ts:42`),
and handles normal plus alternate buffers
(`C:\dev\xterm.js\addons\addon-serialize\src\SerializeAddon.ts:594`,
`C:\dev\xterm.js\addons\addon-serialize\src\SerializeAddon.ts:600`,
`C:\dev\xterm.js\addons\addon-serialize\src\SerializeAddon.ts:607`,
`C:\dev\xterm.js\addons\addon-serialize\src\SerializeAddon.ts:622`).

The image addon confirms that image protocols are stateful, memory-heavy and
optional. It supports Sixel, iTerm2 inline images and Kitty graphics through
addon handlers (`C:\dev\xterm.js\addons\addon-image\src\ImageAddon.ts:23`,
`C:\dev\xterm.js\addons\addon-image\src\ImageAddon.ts:35`,
`C:\dev\xterm.js\addons\addon-image\src\ImageAddon.ts:54`,
`C:\dev\xterm.js\addons\addon-image\src\ImageAddon.ts:178`,
`C:\dev\xterm.js\addons\addon-image\src\ImageAddon.ts:193`,
`C:\dev\xterm.js\addons\addon-image\src\ImageAddon.ts:205`). Its README calls
out memory limits, eviction, placeholders, resize issues and beta quality
(`C:\dev\xterm.js\addons\addon-image\README.md:123`,
`C:\dev\xterm.js\addons\addon-image\README.md:132`,
`C:\dev\xterm.js\addons\addon-image\README.md:152`,
`C:\dev\xterm.js\addons\addon-image\README.md:198`,
`C:\dev\xterm.js\addons\addon-image\README.md:221`).

Unicode support is service/provider-based. The common service has active
versions, registration and width functions
(`C:\dev\xterm.js\src\common\services\UnicodeService.ts:9`,
`C:\dev\xterm.js\src\common\services\UnicodeService.ts:44`,
`C:\dev\xterm.js\src\common\services\UnicodeService.ts:53`,
`C:\dev\xterm.js\src\common\services\UnicodeService.ts:63`,
`C:\dev\xterm.js\src\common\services\UnicodeService.ts:67`). Addons can
register Unicode 11 or grapheme providers
(`C:\dev\xterm.js\addons\addon-unicode11\src\Unicode11Addon.ts:14`,
`C:\dev\xterm.js\addons\addon-unicode-graphemes\src\UnicodeGraphemesAddon.ts:23`,
`C:\dev\xterm.js\addons\addon-unicode-graphemes\src\UnicodeGraphemesAddon.ts:26`).

Hera implication: optional protocols and Unicode policy should not bloat
`terminal-core`. Put extension data in `terminal-protocol` and capability
crates. The core must remain correct when these addons are absent.

## Fixtures And Tests

The fixture directory is immediately reusable as a behavior corpus. Its notes
state that tests assume an 80x25 terminal and are generated from xterm expected
output (`C:\dev\xterm.js\test\fixtures\escape_sequence_files\NOTES:1`,
`C:\dev\xterm.js\test\fixtures\escape_sequence_files\NOTES:3`,
`C:\dev\xterm.js\test\fixtures\escape_sequence_files\NOTES:6`). The fixture
set includes line wrap, history, alt screen, SGR, vttest, long bash output and
vim examples (`C:\dev\xterm.js\test\fixtures\escape_sequence_files\t0003-line_wrap.in`,
`C:\dev\xterm.js\test\fixtures\escape_sequence_files\t0002-history.in`,
`C:\dev\xterm.js\test\fixtures\escape_sequence_files\t0090-alt_screen.in`,
`C:\dev\xterm.js\test\fixtures\escape_sequence_files\t0200-SGR.html`,
`C:\dev\xterm.js\test\fixtures\escape_sequence_files\t0300-vttest1.in`,
`C:\dev\xterm.js\test\fixtures\escape_sequence_files\t0500-bash_long_line.in`,
`C:\dev\xterm.js\test\fixtures\escape_sequence_files\t0504-vim.in`).

Integration tests cover parser hook behavior and terminal input handling
(`C:\dev\xterm.js\test\playwright\Parser.test.ts:41`,
`C:\dev\xterm.js\test\playwright\Parser.test.ts:93`,
`C:\dev\xterm.js\test\playwright\Parser.test.ts:192`,
`C:\dev\xterm.js\test\playwright\InputHandler.test.ts:1546`,
`C:\dev\xterm.js\test\playwright\InputHandler.test.ts:1692`,
`C:\dev\xterm.js\test\playwright\InputHandler.test.ts:1830`). Benchmarks
separate terminal throughput and parser throughput
(`C:\dev\xterm.js\test\benchmark\Terminal.benchmark.ts:53`,
`C:\dev\xterm.js\test\benchmark\Terminal.benchmark.ts:64`,
`C:\dev\xterm.js\test\benchmark\EscapeSequenceParser.benchmark.ts:132`).

Hera implication: import fixture concepts, not necessarily file formats. The
first Hera fixture harness should support xterm.js escape fixtures, VTTEST,
Alacritty refs, WezTerm terminal tests, libvterm reflow and ConPTY replays.

## What Hera Should Copy

1. Stable public API separated from internal state.
2. A real headless package/API, not a browser-first core.
3. Normal/alternate buffer visibility in the inspection API.
4. `onWriteParsed`-style event after a write affects buffer state.
5. Addon boundary for optional protocols and renderer implementations.
6. Serialization as a first-class reconnection/replay story.
7. Row/cell API that hides raw storage but supports inspection.
8. Explicit link/decoration/marker model over buffer lines.
9. Fixture families for parser hooks, input handling, alt screen, reflow,
   SGR, long lines and real app output.

## What Hera Should Not Copy

1. TypeScript as implementation language for the core.
2. Browser `Document`, DOM, canvas or WebGL concepts inside `terminal-core`.
3. Async parser hooks in the correctness path.
4. Scrollback by line count only as the long-term storage answer.
5. Image rendering inside M1 core.
6. Public API access to internals that makes future storage changes harder.
7. A synchronous write API that bypasses backpressure and async invariants.
8. Private renderer plugin seams like WebGL's `unsafeCore` access.

## Open Questions For Hera

1. Should Hera expose parser extension hooks at all in M1, or only typed
   protocol observer events?
2. Should serialization output VT, structured snapshots, or both? Prefer both:
   VT for interoperability, structured snapshots for deterministic replay.
3. What is the minimum stable `BufferApi` needed for Paneflow without exposing
   storage details?
4. Should Unicode width providers be pluggable in M1, or fixed until the core
   fixture matrix is reliable?
5. Which xterm.js escape fixtures should become the first imported Hera corpus:
   line wrap, alt screen, DECSTBM, SGR, long bash output and vim are the best
   starting subset.

## Bottom Line

xterm.js should shape Hera's external surface more than its internals. Use it
to design embeddability, headless state tracking, parser extension policy,
addon boundaries, serialization, link/decoration APIs and fixture imports. Do
not use it as a reason to weaken the Rust-first, renderer-agnostic,
headless-core thesis.

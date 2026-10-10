# GPUI upgrade and feature review

Reviewed on 2026-10-10 against JLiverTool's current UI, upstream release notes,
and the downloaded crate sources. Recommendations below are project-specific
judgments; available features are not automatically enabled or implemented.

## Dependency choice

| Dependency | Before | After |
| --- | --- | --- |
| GPUI | Zed's `gpui` 0.2.2 | `gpui-pre` =0.3.8, imported as `gpui` |
| Platform backends | Included in GPUI | `gpui-pre-platform` =0.3.8 |
| Components | `gpui-component` 0.5.0 | =0.7.1 |
| Rust | Unpinned; local 1.90.0 | 1.98.1 |

Zed's [latest published `gpui` crate](https://docs.rs/crate/gpui/latest) is still
0.2.2. The component maintainer publishes newer Zed snapshots as `gpui-pre`.
This update uses the exact snapshot selected by
[GPUI Kit 0.7.1](https://github.com/longbridge/gpui-kit/blob/v0.7.1/Cargo.toml),
rather than mixing incompatible framework versions. Its Zed revision is
`279fe070bb389b79652e52065b2f001edcc0b11b`; the
[upstream toolchain](https://github.com/zed-industries/zed/blob/279fe070bb389b79652e52065b2f001edcc0b11b/rust-toolchain.toml)
specifies Rust 1.98.1. Keep the framework and platform pins together when updating.

Direct dependencies preserve the existing imports. The optional `gpui-kit`
facade could consolidate these dependencies later, but is not required.
`font-kit`, X11, Wayland, and runtime Metal shaders remain enabled. The
`core-text` pin remains necessary on macOS. `Cargo.lock` is now included for
reproducible application builds, and release CI uses the pinned toolchain and
`--locked`.

## Changes adopted in this migration

- Start the desktop application with `gpui_platform::application()`.
- Keep every window inside `Root`; the new Root automatically hosts overlays.
- Synchronize the custom palette with `Theme::update`, including Base's theme
  projection and refresh of all windows.
- Replace `DockItem` with `DockLayout`, install `DockSkin`, split panel behavior
  from presentation, and retain `PanelHandle` for both new and restored panels.
  Existing panel names and layout version 2 stay compatible with saved layouts.
- Extract dates from the new date/time picker event values; gift queries remain
  date-only and statistics retain their separate hour/minute controls.
- Adapt borrowed list scroll handles and point-shaped scroll limits while
  preserving the existing 50-pixel auto-scroll threshold.
- Use executor timers, remove the obsolete HTTP client method, and handle only
  slider change events when saving settings.
- Add a headless regression test for legacy dock layout restoration, including
  panel titles, nonclosable/nonzoomable constraints and split sizes.

## Framework additions relevant to this application

These were checked against the old and new GPUI crate sources.

| Addition | JLiverTool use | Assessment |
| --- | --- | --- |
| AccessKit roles, labels, IDs and action handlers | Give custom toolbar icons, message rows and window controls accessible names and activation behavior | High priority; component controls improve automatically, custom `div` controls need application work |
| `UniformListScrollHandle::scroll_to_bottom` and `is_scrolled_to_end` | Simplify chat-follow logic in the main and dashboard lists | Useful; the exact-end predicate cannot replace the current near-bottom tolerance directly |
| Pinch gestures, pressure and auxiliary clicks | Font zoom, alternate link activation or touch interaction | Optional; validate platform behavior before exposing shortcuts |
| External drag payloads and file-drop exit events | Drag exported records out or drop a plugin archive into settings | Optional; requires an explicit import/export flow |
| Text font-family overrides and separate background/foreground painting | Emoji/CJK font fallback and richer medal/message styling | Useful if current rendering shows font inconsistencies |
| Inset/spread box shadows and richer grid sizing | Improve card separation or dashboard layout | Optional; measure rendering cost for high message rates |
| Profiler/debug overlay and separated platform implementations | Investigate frame stalls and isolate platform-specific problems | Useful during development; keep instrumentation opt-in |

Source entry points:
[GPUI exports](https://github.com/zed-industries/zed/blob/279fe070bb389b79652e52065b2f001edcc0b11b/crates/gpui/src/gpui.rs),
[interactive elements](https://github.com/zed-industries/zed/blob/279fe070bb389b79652e52065b2f001edcc0b11b/crates/gpui/src/elements/div.rs),
[uniform lists](https://github.com/zed-industries/zed/blob/279fe070bb389b79652e52065b2f001edcc0b11b/crates/gpui/src/elements/uniform_list.rs).

## Component additions and improvements

The following covers the new feature families and relevant behavior changes
since 0.5.0, including optional additions that need separate dependencies.

| Feature family | Where it could help | Recommendation |
| --- | --- | --- |
| Command palette and keyboard-navigable Toolbar/ToolbarGroup | Main window's growing command row: open dashboard/windows, pin, switch room | High priority; use existing `UiCommand` routes |
| Input IME, undo, selection, focus and caret fixes | Chinese danmaku composer, room title, searches, plugin URL fields | Benefits from upgrading; manually exercise Chinese composition |
| Separate Textarea with auto-grow; atomic inline tokens | Multiline drafting and `/title` or mention chips | Useful; adapt Enter-to-send and preserve plain-text backend commands |
| Combobox and improved Select search/dismissal | Gift category/type filters and a future saved-room selector | Useful when option lists grow; current filtering is already functional |
| Improved variable-height virtual-list measurement and cross-region selectable text | Long danmaku, gifts and SC; selecting/copying across messages | High-value follow-up; prototype against the existing wrapping, eviction and auto-scroll behavior |
| Message, MessageScroller, Bubble, Marker, Attachment | Reply grouping, separators and richer message presentation | Optional; preserve compact overlay mode and existing message semantics |
| Declarative Table and enhanced DataTable selection, sizing/export | Audience/guard rankings and online gift history | Useful for sorting/copy/export; evaluate virtualization for large histories |
| Chart tooltips, axes, formatting, fixed domains/slots and reference lines | Existing statistics LineCharts | High priority; native axes could replace manually approximated labels; reference lines could show targets |
| Chart appearance/hover motion and faster series geometry | Statistics panels refreshed every ten seconds | Available after upgrade; set stable IDs if charts share a construction site; disable appearance animation with `appear(false)` if distracting |
| Bar, pie, radar, Sankey and candlestick capabilities | Gift leaderboards, revenue mix or room comparisons | Bar/pie are useful future additions; the other chart types need data this project does not currently collect |
| Dock layout APIs, split restoration, panel visibility and activation hooks | Existing five-panel dashboard | Migration adopted; `set_split_sizes` could support layout presets without rebuilding views; activation hooks could reduce hidden-panel work |
| Theme synchronization, gradients, focus rings and motion | Seven existing themes, transparent windows and interactive controls | Synchronization adopted; add animation selectively to keep overlays calm |
| HoverCard, improved popover placement and NativeMenu | User info card and message context actions | Useful; existing custom card could share placement/focus behavior |
| AlertDialog, FocusTrap and automatic Root overlays | Plugin deletion, logout and clearing saved data | Useful for consistent confirmation and keyboard focus |
| StatusBar, ProgressCircle, Shimmer and notification lifecycle controls | Connection status, QR login, plugin import and gift queries | Useful for clearer pending/error states; consolidate with current OS notifications before adding another delivery path |
| TimeField and date/time DatePicker | Statistics custom period controls | Useful replacement for four separate time selectors; keep online gift date ranges date-only |
| ColorSelect/ColorPicker | Optional custom theme or overlay color settings | Useful only if configurable colors are added; stock themes already cover current needs |
| Settings/Form/GroupBox refinements, sidebar/tree menus, pagination and stepper | Long settings pages and paginated guard/history screens | Incremental cleanup; avoid rebuilding forms solely to adopt components |
| Headless interaction tests and FPS/resource HUD | Dashboard restore, composer behavior, resizing and heavy chat traffic | Headless coverage started; FPS HUD is a separate development dependency |
| SpeechState/SpeechButton/SpeechWaveform | Optional voice dictation into the danmaku composer | Low priority; speech recognition differs by OS and needs permissions/providers. Existing TTS is speech output and is unrelated |
| Editor/LSP, multicursor editing, decorations and language grammars | Possible future in-app plugin editor | Low priority; plugin editing currently happens outside this app |
| Markdown streaming/search/source mapping, math/frontmatter and document extensions | In-app help, plugin README or release-note viewer | Low priority; current messages should retain plain-text semantics |
| gpui-shell, gpui-wry, NavStack, web/mobile support | Native script plugins, embedded pages or a different navigation model | Defer; each entails architectural work beyond the current browser-based plugin system |
| Questionnaire, Rating and Carousel | Onboarding, feedback or media browsing | No compelling current use |

Feature references:
[0.6.0 architecture and components](https://github.com/longbridge/gpui-kit/releases/tag/v0.6.0),
[0.6.1 editing and headless tests](https://github.com/longbridge/gpui-kit/releases/tag/v0.6.1),
[0.7.0 charts, docking and Root migration](https://github.com/longbridge/gpui-kit/releases/tag/v0.7.0),
[0.7.1 speech, colors and input fixes](https://github.com/longbridge/gpui-kit/releases/tag/v0.7.1).
The intervening patch releases primarily fix behavior and pin dependencies;
[0.5.1](https://github.com/longbridge/gpui-kit/releases/tag/v0.5.1) fixes macOS
`core-text` compatibility.

## Suggested next work

1. Prototype variable-height, selectable message rows using mock traffic.
2. Replace manual statistics axes with native axes and add formatted tooltips.
3. Add accessible names and keyboard navigation to the custom command row.
4. Add dashboard interaction and composer IME tests before larger UI refactors.

## Validation

Verified locally on macOS arm64 with Rust 1.98.1:

- `cargo check --workspace --all-targets --locked`: passed.
- `cargo build -p jlivertool --locked`: passed with production features.
- `cargo test --workspace --locked`: all 26 tests passed, including the new
  headless legacy-layout regression and the five existing message wrapping tests.
- `cargo clippy --workspace --all-targets --locked`: passed with warnings in
  existing code (including legacy Objective-C macros and existing lint findings).
- `git diff --check`: passed.
- `cargo fmt --all -- --check`: fails on repository formatting differences.
  A separate copy of the original HEAD also fails this check with Rust 1.98.1;
  this update does not reformat the entire repository.

Windows/Linux runtime behavior and real IME/graphics behavior remain unverified
locally; headless tests do not validate GPU rendering. Release installers were
not built. CI is configured for all three desktop platforms.

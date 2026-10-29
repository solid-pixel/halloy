# Halloy settings GUI fork

This fork extends Halloy with graphical editing for every supported configuration
area: General, Networks, Proxies, Appearance, Navigation and panes, Chat display,
Sidebar, Notifications and sounds, Highlight rules, Keyboard shortcuts, Commands,
Previews, and Transfers and uploads. The user authorized these areas and explicitly
requested Impeccable and Halloy’s existing design language.

The initial audience is desktop Halloy users who want to configure the client
through forms. This audience is inferred from the request. Halloy supports
macOS, Windows, and Linux; native verification for this change runs on macOS.

## Scope and behavior

- Networks: add/rename/remove servers; connection, identity, timing, TLS,
  WebSocket, password references, SASL PLAIN/EXTERNAL, NickServ, channels/queries,
  upload endpoints, proxies, filters, reroutes, metadata, delivery, protocol logs
  and supported bouncer overrides use grouped forms and structured rows.
- Proxies: global HTTP/SOCKS5 settings and credential references; Tor explains
  its required build feature. Keyring editors never resolve or migrate secrets.
- General: application/window/platform/runtime/logging preferences, Config Editor,
  context menus, metadata/avatar policies, display symbols, CTCP and monitor rules.
- Commands: aliases, execution limits, sysinfo and default part/quit messages.
- Previews: URL modes/patterns, card/image controls and conditions, requests/cache.
- Transfers/uploads: DCC and acceptance, optional atomic server settings, upload controls.
- Appearance: installed themes, light/dark theme selection, font family, text
  size, line height, and chat interface scale.
- Navigation: chat-opening and click actions, a current-pane preset, split axis,
  pane gaps and restoration, title bar buttons, and chat scrollbars.
- Chat display: messages and timestamps, names and user lists, activity messages,
  read markers, the composer, emoji, and typing indicators.
- Sidebar: position, width, order, labels, spacing, unread and highlight
  indicators, typing indicators, and scrollbars.
- Notifications: all existing general events and per-channel rules, system
  notifications, attention, content, sounds, delays, and scope conditions.
- Highlights: nickname case matching, named word/regex rules, case options,
  sounds, and scope conditions, including structured combined conditions.
- Keyboard: all ordinary and message-focus actions, alternative bindings,
  clearing and platform defaults, parser errors and conflict validation.
- Appearance includes a browser for the official Halloy theme gallery: search,
  light/dark filters and downloads into the themes directory.
  Selecting a theme previews it across all app windows and changes the draft.
  Apply downloads missing themes and saves the selection. Revert or discarding
  restores the saved appearance. Downloads preserve existing files.
- Settings opens from the user menu, command bar, welcome screen, or Cmd/Ctrl+,.
- Keyboard focus reaches every Settings control, stays within the window, and
  scrolls into view. Dropdowns support keyboard selection and cancellation;
  confirmations keep focus on their choices. Validation opens the affected
  section and keeps an explanation beside the field until corrected.
- Apply validates and saves the existing `config.toml`, then reloads Halloy.
  Other options and comments are retained. A backup is written before replacing
  the file, external edits block saving, and symlinks update their target.
- Advanced authentication and randomized single/light/dark theme lists have
  explicit structured controls. Untouched values retain their original format.
  Network renames preserve existing keyring credential references.
- Theme gallery requests follow the current global proxy setting, including on
  Retry. Unsupported Tor requests fail without a direct connection.
- Font changes require a restart; existing server connection changes require
  reconnecting. New automatic connections begin after applying.
- Unsaved drafts prompt before closing or quitting. Revert reloads from disk.
- Settings retains its default scale so the chat scale control stays reachable.

## Visual authority

Extend the existing implementation and `assets/screenshot.png`:
`src/appearance/theme.rs`, its theme modules, `src/widget/pick_list.rs`, and the
existing native theme editor define fonts, colors, controls, and window behavior.
Use Halloy’s configured font and palette, flat surfaces, compact labels, and
plain controls. Settings derives readable text, visibly filled inputs and
pickers, visible control borders, and clear focus states from the chosen
palette. Picker menus and unchecked boxes use the same control fill. These
adjustments are local to Settings; the chat preview uses the theme as supplied.
The Settings surface operates as a tool: clear forms, independent scrolling,
explicit states, and a persistent action footer. Form labels sit beside their
controls in aligned rows. Network form labels move above their controls when
the form has less than 440 pixels of width. Controls adapt to the available
space and stop at widths suited to their content; numeric fields stay short,
while text fields and pickers allow longer values. Density comes from compact
row spacing and padding while retaining the existing text hierarchy.
A vertical navigation pane on the left contains all settings areas.
Networks opens a second sidebar with a scrolling network list and an
**Add network** button anchored at the bottom. The selected network opens in
the form to its right. Preference areas expand their groups beneath the active
heading in the main navigation; indented children distinguish that level from
the area headings. The selected group, Appearance, and its theme browser use
the form pane beside the main navigation. Navigation, the network list, and
forms scroll independently. All three Networks columns remain available at the
minimum window size.

Related settings share flat groups with a faint palette-derived background,
a visible border, and inset separators between rows. The Networks form separates
Connection and Authentication; Appearance separates Theme, Font, and Layout.
Preference areas show the selected group's rows together. Group headings and
space between groups provide hierarchy without glass effects. Draft status,
Revert, Apply, and Close remain below the scrolling content.

No change to Halloy’s brand, chat layout, IRC engine, or configuration format is
part of this request. The fork keeps the upstream license and attribution.

## Configuration coverage

The [model checklist](docs/guides/settings-gui-coverage.md) inventories all 527
public configuration model fields, including nested containers and runtime
projections, plus tagged formats and bouncer overrides. Runtime-only server order
has no numeric setting. Platform/build availability remains explicit.

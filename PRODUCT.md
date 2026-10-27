# Halloy settings GUI fork

This fork extends Halloy with graphical editing for Networks, Appearance,
Navigation and panes, Chat display, and Sidebar. The user authorized these areas
and explicitly requested Impeccable and Halloy’s existing design language.

The initial audience is desktop Halloy users who want to configure the client
through forms. This audience is inferred from the request. Halloy supports
macOS, Windows, and Linux; native verification for this change runs on macOS.

## Scope and behavior

- Networks: add, rename, and remove a server; edit hostname, port, nickname,
  auto-join channels, TLS, automatic connection, SASL PLAIN, and NickServ.
- Appearance: installed themes, light/dark theme selection, font family, text
  size, line height, and chat interface scale.
- Navigation: chat-opening and click actions, a current-pane preset, split axis,
  pane gaps and restoration, title bar buttons, and chat scrollbars.
- Chat display: messages and timestamps, names and user lists, activity messages,
  read markers, the composer, emoji, and typing indicators.
- Sidebar: position, width, order, labels, spacing, unread and highlight
  indicators, typing indicators, and scrollbars.
- Appearance includes a browser for the official Halloy theme gallery: search,
  light/dark filters and downloads into the themes directory.
  Selecting a theme previews it across all app windows and changes the draft.
  Apply downloads missing themes and saves the selection. Revert or discarding
  restores the saved appearance. Downloads preserve existing files.
- Settings opens from the user menu, command bar, welcome screen, or Cmd/Ctrl+,.
- Apply validates and saves the existing `config.toml`, then reloads Halloy.
  Other options and comments are retained. A backup is written before replacing
  the file, external edits block saving, and symlinks update their target.
- Advanced authentication and randomized theme lists remain in the file unless
  the user explicitly selects another configuration method.
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
controls in aligned rows. Controls adapt to the available space and stop at
widths suited to their content; numeric fields stay short, while text fields
and pickers allow longer values. Density comes from compact row spacing and
padding while retaining the existing text hierarchy.
Compact underlined tabs switch settings areas; pane
backgrounds and borders group content, selected groups have a stronger fill,
and Appearance fills the available width.
Navigation, Chat display, and Sidebar use a compact group list alongside the form.
Network and group lists scroll independently from their forms. Draft status,
Revert, Apply, and Close remain below the scrolling content.

No change to Halloy’s brand, chat layout, IRC engine, or configuration format is
part of this request. The fork keeps the upstream license and attribution.

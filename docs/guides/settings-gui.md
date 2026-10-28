# Settings GUI

This fork adds Networks, Appearance, Navigation, Chat display, and Sidebar
to a native Settings window. Open **Settings** from the sidebar user menu,
choose **Configuration: Open Settings**
in the command bar, or press **⌘,** on macOS / **Ctrl+,** on Windows and Linux.
The welcome screen also has an **Open Settings** button.
An explicit keyboard binding for the same shortcut takes priority. Config Editor
remains available from the command bar and can have its own keyboard binding.

Choose an area in the navigation pane on the left. **Networks** opens a second
sidebar with the network list; select a network to open its form on the right.
Preference areas expand their groups beneath the active heading in the main
navigation; choose an indented group to open its form. Appearance and its theme
browser use the pane beside the main navigation. Navigation, the network list,
and forms scroll independently, including at the minimum window size.

Related settings appear in flat groups with a faint background, a border, and
inset separators between rows. Form labels sit beside their controls, with short
numeric fields and wider text fields and pickers. In a narrow Networks form,
labels move above their controls. Controls adjust to the available width, with
a limit suited to their content.

The footer keeps the draft status, **Revert changes**, **Apply**, and **Close**
visible while forms scroll.

Use **Tab / Shift+Tab** to move through Settings, and **Enter / Space** to
activate buttons and checkboxes. Focus stays in Settings and scrolls into view.
In a dropdown, use the arrow keys or Home/End, then Enter to select. Typing a
letter jumps to a matching option. Escape cancels the open dropdown; Tab closes
it and moves to the next control. An open confirmation keeps focus on its choices.

## Networks

Use **Add network** at the bottom of the network sidebar to create a network.
Select a network in the list to rename or remove it and edit its connection or
authentication settings. Each network has a server hostname, an optional
port, a nickname, channels to join on connection, TLS, and automatic connection
on launch. An empty port uses Halloy's default for the configured transport.

SASL PLAIN and NickServ authentication can be edited here. Passwords entered in
Settings are stored in `config.toml`. Certificate authentication, keyring entries,
password commands, server passwords, proxies, and other advanced options remain
available in the configuration file. Existing advanced authentication is shown
as **Configured in file**; selecting another method replaces that network's SASL
and NickServ authentication settings.
Renaming a network keeps its existing keyring credentials by retaining explicit
references to their original entries.

New networks with automatic connection enabled connect after applying.
Reconnect existing networks to use changes to the server or authentication.
Removing a network takes effect when you apply and disconnects that network.

## Appearance

Choose an installed theme, or separate themes for light and dark system
appearance. Theme selections preview across all app windows before applying.
Existing randomized theme lists are shown as **Configured in file** and kept
until you choose another theme mode.

Select **Browse and download themes** to search the official Halloy gallery
and filter light or dark themes. Selecting a theme previews it throughout Halloy
and enables **Apply** when it changes the saved selection. **Apply** downloads the theme if needed and saves your choice.
**Revert changes**, or closing Settings and discarding changes, restores the
saved appearance.

**Download theme** also lets you save a theme to the switcher without applying
it. Existing files are kept. With system appearance, selecting a light theme
updates the light choice and selecting a dark theme updates the dark choice.
**Use as light** and **Use as dark** can assign an installed theme to either slot.
The last selection previews regardless of system appearance; applying resumes
the configured system behavior.

The gallery needs an internet connection. A failed request offers **Try again**
and a link to the gallery website. Theme files installed manually are picked up
when Settings is reopened.
Gallery requests use the current global proxy setting. With Halloy's built-in
Tor proxy, gallery requests are unavailable; they do not fall back to a direct connection.

Font family, text size, and line height require an application restart after
applying. Empty font fields use Halloy's defaults. Interface scale applies
immediately. Chat interface scale affects the chat windows;
Settings stays at its default scale so the scale control remains reachable.

## Navigation and panes

Choose how sidebar items, channel links, nicknames, commands, tools, and
notifications open chats. **Reuse the current pane for everything** stages all
these actions to replace the current pane. Press **Apply**, then close existing
extra panes if you want a single chat pane. Individual actions can also open a
new pane or window.

The other groups control the split direction, saved pane layout, spacing,
title bar buttons, and chat scrollbars. The split direction controls future
splits; changing it does not rearrange existing panes.

## Chat display

Use the groups beneath **Chat display** to edit timestamps and date separators,
repeated timestamps and nicknames, message spacing, backlog behavior, nickname
colors and alignment, access levels, the user list, and the topic banner.
Activity controls cover server messages, joins, parts, quits, nickname changes,
and modes.

Read markers, composer visibility, formatting, nickname display, draft
persistence, completion order, emoji, and typing indicators have their own
groups. These are global defaults; channel-specific configuration can override
them. Timestamp and date formats use Halloy's existing format syntax.

## Sidebar

Set the position, width, server and channel ordering, menu buttons, label sizes,
icon sizes, casing, and spacing. Separate groups control unread indicators,
highlights, typing indicators, and scrollbars. Empty optional size fields use
Halloy's defaults. Position changes apply immediately.

Advanced values that do not fit a picker are shown as **Configured in file**.
They stay intact until you choose a replacement. Only edited settings are
written, including when the original file uses an older supported setting name.

## Saving

Press **Apply** or **⌘S / Ctrl+S** to validate, save, and reload the configuration.
If a field is invalid, Settings opens its section, focuses the field, and shows
the correction beside it. The error stays visible until it is corrected.
Settings edits the original TOML document, retaining other settings and comments.
The previous file is backed up next to it as `config.toml.settings-backup`.
Symlinked configurations update their actual target, with the backup beside it.

**Revert changes** reloads the file from disk and discards the current draft.
Closing or quitting with an unsaved draft asks whether to discard it.

If another editor changes the file, saving is refused. Reopen Settings or revert
to load the latest version, then make your changes again. Save or revert unsaved
changes in Halloy's Config Editor before applying Settings.

## Development

With Rust installed, build and run using the existing workspace:

```sh
cargo run --profile ci
cargo test --profile ci -p data config::settings::tests
cargo clippy --locked --profile ci --workspace --all-targets -- -D warnings
cargo +nightly fmt --all -- --check
```

To try the GUI with separate configuration and history, create an empty directory
and set `HALLOY_PORTABLE_DIR` to its absolute path when running. Place a test
`config.toml` there with `autoconnect = false` for every test network if no
connections are desired.

Settings keeps the selected palette while improving text contrast, field and
button outlines, and focus visibility. Inputs and pickers have visible fills
against their panes; picker menus and unchecked boxes use the same control fill.
The active area is bold with a stronger background; its selected network or
preference group has a stronger fill and visible border. Settings groups stay
flat, with space and headings separating them. Chat windows show the supplied
theme colors.

# Settings GUI

This fork adds graphical editing for all supported configuration areas: General,
Networks, Proxies, Appearance, Navigation, Chat display, Sidebar, Notifications
and sounds, Highlight rules, Keyboard shortcuts, Commands, Previews, and
Transfers and uploads. See the [field coverage checklist](settings-gui-coverage.md). Open **Settings** from the sidebar user menu,
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

Use **More network settings** to choose a group for identity, connection limits,
WebSocket/TLS options, server passwords, NickServ, SASL PLAIN or EXTERNAL,
joined channels and queries, upload endpoints, proxies, ignored messages,
reroutes, metadata, capabilities, delivery conditions, protocol logs, and
bouncer overrides. Lists and maps have individual add/edit/remove controls.
Channel entries include their mute state.

Password controls retain stored values without displaying them. A replacement
is stored in `config.toml`; keyring, file and command controls edit references.
Choose one password source. Keyring controls never retrieve or migrate secrets.
SASL and NickServ can be configured independently. Renaming a network retains
explicit references to its original keyring entries.

**Proxies** edits the global proxy using the same controls. An unset network
proxy inherits it. Tor requires a build with the Tor feature; unsupported
selection explains that requirement. Map names are changed through add/remove,
which avoids silently moving a credential to another keyring entry.

New networks with automatic connection enabled connect after applying.
Reconnect existing networks to use changes to the server or authentication.
Removing a network takes effect when you apply and disconnects that network.

## Appearance

Choose an installed theme, or separate themes for light and dark system
appearance. Theme selections preview across all app windows before applying.
The light and dark menus list only themes with matching background brightness,
using the same classification as the gallery. The single-theme menu lists all
themes. Existing selections are retained until you choose a replacement.
Use the theme list mode for a randomized list or separate light/dark lists.
Lists retain their saved choices and order until edited. The last ordinary or
list-mode selection is the one saved by Apply.

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

Font family, code font, weights, stretch, emoji size, text size and line height
require an application restart after
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
Activity groups cover every supported server event and internal message, with
visibility, smart windows, opacity, unread behavior and scope conditions. Ordered
color rules, metadata formatting, replies, reactions, redactions and query closing
also have controls.

Read markers, composer visibility, formatting, nickname display, draft
persistence, completion order, emoji, and typing indicators have their own
groups. These are global defaults; channel-specific configuration can override
them. Timestamp and date formats use Halloy's existing format syntax.

## Sidebar

Set the position, width, server and channel ordering, menu buttons, label sizes,
icon sizes, casing, and spacing. Separate groups control unread indicators,
highlights, typing indicators, scrollbars, internal tool order and muting. Empty optional size fields use
Halloy's defaults. Position changes apply immediately.

Advanced values that do not fit a picker are shown as **Configured in file**.
They stay intact until you choose a replacement. Only edited settings are
written, including when the original file uses an older supported setting name.

## Notifications and sounds

Choose an event beneath **Notifications & sounds** to edit system notifications,
window attention, content visibility, sounds, and the delay between notifications.
All nine general event types are available. **Add channel rule** configures
notifications for every message in a named channel; channel names may include dots.
Select a channel rule to rename or remove it. Removal is staged until Apply and
asks for confirmation.

Sounds accept built-in names or custom sound filenames from Halloy's `sounds`
folder. Use **Add built-in sound** to append a sound, or type names directly.
Separate alternative sounds with semicolons; Halloy chooses one randomly.
Empty notification sounds mean silence. The delay is in milliseconds; an empty
field restores 500 ms. These controls configure future notifications; they do
not send a test notification or play audio.

Conditions have separate channel, user, and server lists. Separate names with
semicolons, or enter `*` for every name in that scope. Includes take priority over
excludes. To notify only for a list, exclude `*` in all relevant scopes, then add
the allowed names to Include. **Add condition** creates combined user, channel,
server and server-message restrictions. All nonempty fields within one condition
must match; an empty condition matches everything. The same editor is used for
Channel Monitor, previews, activity and sidebar conditions.

## Highlight rules

**Nickname** controls case matching and inclusion/exclusion scopes for your
nickname. To disable nickname highlights everywhere, exclude `*` in the channel,
user, and server scopes. Includes can enable selected scopes again.

**Add word rule** and **Add regex rule** create custom matches. A rule has an
optional notification name, words or a regular expression, sounds, and conditions.
Words and phrases use semicolons as separators; **Ignore case** applies to word
rules. Regex rules use the existing regex syntax, including flags such as `(?i)`.
The Match picker switches between the two types. An empty rule sound list uses
the highlight notification sound. Invalid expressions reveal and focus their
field when applying. Rules can be removed after confirmation or restored with
Revert before applying.

For lists in these areas, wrap an entry containing a literal semicolon in double
quotes. Quoted strings use the same escapes as TOML strings. Existing lists are
shown with quoting where needed, including bindings for the semicolon key.

## Keyboard shortcuts

The four groups cover all existing ordinary and message-focus actions. Edit a
binding with Halloy's existing names, such as `command+k`, `ctrl+shift+k`, or
`alt+up`. Separate alternatives with semicolons. Commas in shortcuts are literal:
`command+,` can explicitly override the built-in Settings gesture.

**Clear** disables that action; **Default** removes its override and restores
Halloy's platform default. Conflicts and invalid bindings prevent saving and
reveal the affected field. Resolve conflicts by changing or clearing one of the
listed actions; changes are validated together so bindings can be swapped.
Message selection retains its built-in plain arrow keys. Direction bindings also
enter message selection from chat. Settings always consumes its own Cmd/Ctrl+,
gesture before editable fields receive text.

## General and Commands

General groups cover tooltips, update checks, initial window size and platform
padding/decorations, rendering, logs, Config Editor indentation, context menus,
metadata subscriptions and avatars, global display symbols/colors, CTCP replies,
and Channel Monitor conditions. **Value source** distinguishes inherited defaults
from explicit false, empty text and empty lists. Window, rendering, font and
logging startup changes show restart advice.

Commands groups cover command descriptions, aliases with placeholder arguments,
optional local `/exec` execution and limits, sysinfo content, and default part/quit
messages. Alias removal is staged and can be restored or reverted before Apply.
Configuring an alias does not execute it.

## Previews, transfers and uploads

Preview groups cover link/card/image appearance and actions, animation, URL
pattern filters, scope/compound conditions, request limits, user agent and image
cache. Cache pickers offer Unlimited and First save only where supported.
URL matching uses regular expressions; invalid expressions prevent saving.

Transfers groups cover DCC enablement, passive mode, acceptance timeout,
completion messages, save folder, automatic acceptance restrictions, and an
optional DCC server. Enabling automatic acceptance requires a save folder.
Configuring a DCC server requires both IP addresses and an ordered port range;
these settings are validated together. Global upload controls cover enablement,
button, paste and file drop. Per-network endpoints and credentials are in Networks.

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

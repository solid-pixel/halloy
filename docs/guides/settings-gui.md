# Settings GUI

This fork adds Networks and Appearance to a native Settings window. Open
**Settings** from the sidebar user menu, choose **Configuration: Open Settings**
in the command bar, or press **⌘,** on macOS / **Ctrl+,** on Windows and Linux.
The welcome screen also has an **Open Settings** button.

## Networks

Add, rename, or remove networks. Each network has a server hostname, an optional
port, a nickname, channels to join on connection, TLS, and automatic connection
on launch. An empty port uses Halloy's default for the configured transport.

SASL PLAIN and NickServ authentication can be edited here. Passwords entered in
Settings are stored in `config.toml`. Certificate authentication, keyring entries,
password commands, server passwords, proxies, and other advanced options remain
available in the configuration file. Existing advanced authentication is shown
as **Configured in file**; selecting another method replaces that network's SASL
and NickServ authentication settings.

New networks with automatic connection enabled connect after applying.
Reconnect existing networks to use changes to the server or authentication.
Removing a network takes effect when you apply and disconnects that network.

## Appearance

Choose an installed theme, or separate themes for light and dark system
appearance. A single theme previews within Settings before applying. Existing
randomized theme lists are shown as **Configured in file** and kept until you
choose another theme mode. Install additional theme files in Halloy's `themes`
directory before opening Settings.

Font family, text size, and line height require an application restart after
applying. Empty font fields use Halloy's defaults. Interface scale and sidebar
position apply immediately. Chat interface scale affects the chat windows;
Settings stays at its default scale so the scale control remains reachable.

## Saving

Press **Apply** or **⌘S / Ctrl+S** to validate, save, and reload the configuration.
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

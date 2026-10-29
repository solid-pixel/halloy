# Settings coverage checklist

This checklist follows the actual configuration models in `data/src/config.rs`
and `data/src/config/`. Each field is covered by a form control or a structured
editor, except the runtime-only fields explicitly identified below. Container
fields lead to their nested editors. Supported legacy spellings and untouched
advanced values are preserved when another field changes.

| Configuration family | Settings area and editor |
| --- | --- |
| Application, window, platform, runtime, logs, context menu, metadata, CTCP, monitor | General: grouped controls, explicit defaults, lists and compound conditions |
| Servers, credentials, SASL, upload endpoints, filters, reroutes, bouncer overrides | Networks: connection form and advanced groups; structured map and list rows |
| Global proxy | Proxies: method, address, port and credentials |
| Theme, font, scale | Appearance: theme preview/gallery, separate light/dark choices, randomized lists and font controls |
| Actions and panes | Navigation: opening/click behavior, panes, replies, image actions and URL confirmation |
| Buffer, nickname, display metadata, text input, activity | Chat display: controls, ordered lists, color rules and compound conditions |
| Sidebar | Sidebar: layout, internal tools, indicators and compound conditions |
| Notifications, highlights, keyboard | Existing event/rule/binding editors, retained with their validation |
| Commands | Commands: aliases, execution, sysinfo and default messages |
| Preview | Previews: link/card/image behavior, URL patterns, scope conditions, requests and cache |
| File transfers and global uploads | Transfers & uploads: DCC, acceptance restrictions, optional server and upload controls |

`Server.order` is assigned from document order (`serde(skip)`), and follows the existing
network order; it has no independent configuration override. `file_transfer::Server.bind_ports`
is the runtime range produced from `bind_port_first` and `bind_port_last`.
`Nickname.color_overrides` is loaded through the supported `color_override` key.
Deprecated aliases use the same control as their canonical field. Platform
controls remain visible with platform labels; availability of rendering APIs,
notifications and drag/drop follows the platform and compiled features.
Keyring editors change references only; they do not retrieve or migrate secrets.
The older documented `sidebar.unread_indicator.highlight_exclude` key is ignored
by this runtime; highlight scopes use `sidebar.highlight_indicator.exclude`.
Cache numeric modes require positive values: zero described in older prose is
rejected by the actual parser.

## Model inventory

The checked entries below include containers and the runtime projections described
above. Enum alternatives are represented by their corresponding choice or mode
control, including booleans, optional values, scalar/list forms and legacy aliases.

### `data/src/config.rs`

- [x] `Config`: `appearance`, `context_menu`, `servers`, `proxy`, `font`, `scale_factor`, `buffer`, `channel_monitor`, `pane`, `sidebar`, `keyboard`, `notifications`, `file_transfer`, `filehost`, `tooltips`, `window`, `preview`, `highlights`, `actions`, `ctcp`, `display`, `logs`, `platform_specific`, `runtime`, `check_for_update_on_launch`, `metadata`, `config_editor`.
- [x] `Scrollbar`: `width`, `scroller_width`, `hidden`.
- [x] `Font`: `family`, `code`, `stretch`, `size`, `line_height`, `weight`, `bold_weight`, `only_emojis_size`.
- [x] `Configuration`: `logs`, `font`.
- [x] `ParseError`: `message`, `details`, `line`.

### `data/src/config/keys.rs`

- [x] `Keyboard`: `move_up`, `move_down`, `move_left`, `move_right`, `new_horizontal_buffer`, `new_vertical_buffer`, `close_buffer`, `maximize_buffer`, `restore_buffer`, `cycle_next_buffer`, `cycle_previous_buffer`, `leave_buffer`, `toggle_nick_list`, `toggle_topic`, `toggle_sidebar`, `toggle_fullscreen`, `command_bar`, `reload_configuration`, `file_transfers`, `logs`, `theme_editor`, `highlights`, `search`, `scroll_up_page`, `scroll_down_page`, `scroll_to_top`, `scroll_to_bottom`, `scroll_to_backlog`, `cycle_next_unread_buffer`, `cycle_previous_unread_buffer`, `mark_as_read`, `config_editor_save`, `quit_application`, `open_config_editor`, `open_config_file`, `show_muted_buffers`, `hide_muted_buffers`, `focus_up`, `focus_down`, `focus_left`, `focus_right`, `focus_activate`, `focus_activate_alt`, `focus_reply`, `focus_react`, `focus_redact`.

### `data/src/config/actions.rs`

- [x] `Actions`: `sidebar`, `buffer`, `nicklist`, `notification`.
- [x] `Buffer`: `click_channel_name`, `click_highlight`, `click_channel_discovery`, `click_username`, `join_channel`, `open_internal`, `message_channel`, `message_user`, `only_contract_expanded_message`, `click_image_url`.
- [x] `Sidebar`: `buffer`, `channel`, `query`, `internal`, `server`, `focused_buffer`, `cycle`.
- [x] `Nicklist`: `click_username`.
- [x] `Notification`: `default`, `open_buffer`.

### `data/src/config/filehost.rs`

- [x] `Filehost`: `enabled`, `button`, `paste`, `file_drop`.

### `data/src/config/runtime.rs`

- [x] `Runtime`: `backend`, `power_preference`, `vsync`, `antialiasing`, `metrics_hinting`, `smooth_scrolling`.

### `data/src/config/display.rs`

- [x] `Display`: `direction_arrows`, `decode_urls`, `truncation_character`, `nickname`, `nicklist_nickname`, `adapt_metadata_colors`.
- [x] `DirectionArrows`: `left`, `right`.

### `data/src/config/config_editor.rs`

- [x] `ConfigEditor`: `indent`.

### `data/src/config/notification.rs`

- [x] `Notification`: `show_toast`, `request_attention`, `show_content`, `sound`, `delay`, `exclude`, `include`.
- [x] `Notifications`: `connected`, `disconnected`, `reconnected`, `direct_message`, `highlight`, `file_transfer_request`, `monitored_online`, `monitored_offline`, `channels`, `reaction`.

### `data/src/config/file_transfer.rs`

- [x] `FileTransfer`: `enabled`, `save_directory`, `passive`, `timeout`, `send_completion_message`, `auto_accept`, `server`.
- [x] `AutoAccept`: `enabled`, `nicks`, `masks`.
- [x] `Server`: `public_address`, `bind_address`, `bind_ports`.

### `data/src/config/context_menu.rs`

- [x] `ContextMenu`: `padding`, `show_user_metadata`.
- [x] `Padding`: `entry`.

### `data/src/config/inclusivities.rs`

- [x] `Inclusivities`: `users`, `channels`, `servers`, `server_messages`, `criteria`.

### `data/src/config/metadata.rs`

- [x] `Metadata`: `preferred_keys`, `avatar`.
- [x] `Avatar`: `enabled`, `exclude`, `size`.

### `data/src/config/ctcp.rs`

- [x] `Ctcp`: `ping`, `source`, `time`, `version`, `userinfo`.

### `data/src/config/highlights.rs`

- [x] `Highlights`: `nickname`, `matches`.
- [x] `Nickname`: `exclude`, `include`, `case_insensitive`.
- [x] `Match`: `regex`, `name`, `exclude`, `include`, `sound`.

### `data/src/config/window.rs`

- [x] `Window`: `initial_height`, `initial_width`.

### `data/src/config/sidebar.rs`

- [x] `Sidebar`: `max_width`, `unread_indicator`, `highlight_indicator`, `typing_indicator`, `position`, `order_by`, `scrollbar`, `secondary_font_size`, `primary_icon`, `primary_font_size`, `user_menu`, `collapse_button`, `padding`, `spacing`, `order_channels_by`, `channel_name_casing`, `internal_buffers`.
- [x] `InternalBuffers`: `mute`, `position`, `buffers`.
- [x] `Padding`: `buffer`.
- [x] `Spacing`: `server`.
- [x] `UserMenu`: `enabled`.
- [x] `CollapseButton`: `enabled`.
- [x] `UnreadIndicator`: `title`, `icon`, `icon_size`, `show_on_open_buffers`, `query_as_highlight`, `exclude`, `include`.
- [x] `HighlightIndicator`: `title`, `icon`, `icon_size`, `show_on_open_buffers`, `exclude`, `include`.
- [x] `TypingIndicator`: `show`, `icon_height`.

### `data/src/config/preview.rs`

- [x] `Preview`: `enabled`, `exclude`, `max_per_message`, `request`, `card`, `image`.
- [x] `Request`: `user_agent`, `timeout_ms`, `max_image_size`, `max_scrape_size`, `concurrency`, `delay_ms`, `image_cache`.
- [x] `ImageCache`: `max_size`, `trim_interval`.
- [x] `Card`: `exclude`, `include`, `hide_url`, `show_image`, `round_image_corners`, `max_width`, `description_max_height`, `image_max_height`, `description_decode_html`, `image_action`.
- [x] `Image`: `action`, `exclude`, `include`, `hide_url`, `round_corners`, `max_width`, `max_height`, `animation`.

### `data/src/config/channel_monitor.rs`

- [x] `ChannelMonitor`: `exclude`, `include`.

### `data/src/config/server.rs`

- [x] `Server`: `nickname`, `nick_password`, `nick_password_keyring`, `nick_password_file`, `nick_password_file_first_line_only`, `nick_password_command`, `nick_identify_syntax`, `alt_nicks`, `username`, `realname`, `server`, `port`, `password`, `password_keyring`, `password_file`, `password_file_first_line_only`, `password_command`, `filters`, `reroute`, `channels`, `channel_keys`, `channel_keys_keyring`, `order_channels_by`, `queries`, `ping_time`, `ping_timeout`, `reconnect_delay`, `max_connection_attempts`, `should_ghost`, `ghost_sequence`, `umodes`, `use_tls`, `use_websocket`, `websocket_path`, `websocket_ping_time`, `dangerously_accept_invalid_certs`, `root_cert_path`, `sasl`, `on_connect`, `who_poll_enabled`, `who_poll_interval`, `monitor`, `automated_chathistory`, `anti_flood`, `order`, `proxy`, `confirm_message_delivery`, `autoconnect`, `typing`, `filehost`, `metadata`, `icon`, `do_not_request`, `irc_protocol_log`, `sidebar_visibility`, `networks`.
- [x] `Muteable`: `name`, `mute`.
- [x] `ConfirmMessageDelivery`: `enabled`, `exclude`, `include`.
- [x] `OptionalTyping`: `share`, `show`.
- [x] `AntiFlood`: `rate`, `burst`.
- [x] `IrcProtocolLog`: `enabled`, `format`, `timestamp`.

### `data/src/config/platform_specific.rs`

- [x] `PlatformSpecific`: `linux`, `macos`, `windows`.
- [x] `Windows`: `decorations`.
- [x] `Linux`: `decorations`.
- [x] `MacOS`: `decorations`, `content_padding`, `sidebar_padding`.

### `data/src/config/buffer.rs`

- [x] `Buffer`: `timestamp`, `nickname`, `text_input`, `channel`, `server_messages`, `internal_messages`, `status_message_prefix`, `chathistory`, `backlog_separator`, `date_separators`, `commands`, `emojis`, `mark_as_read`, `url`, `line_spacing`, `scroll_position_on_open`, `typing`, `redaction`, `close`, `reply`.
- [x] `Close`: `query`.
- [x] `Reply`: `enabled`, `show_icon`, `icon_size`, `insert_nick`, `hide_redundant_nicks`, `highlight_hovered_message`, `tooltip`.
- [x] `ReplyTooltip`: `enabled`, `delay`, `max_width`, `max_chars`.
- [x] `Emojis`: `show_picker`, `skin_tone`, `auto_replace`, `characters_to_trigger_picker`.
- [x] `Url`: `prompt_before_open`.
- [x] `MarkAsRead`: `on_application_exit`, `on_buffer_close`, `on_scroll_to_bottom`, `on_message_sent`, `on_message`.
- [x] `Quit`: `default_reason`.
- [x] `Part`: `default_reason`.
- [x] `Exec`: `enabled`, `timeout`, `max_output_bytes`.
- [x] `SysInfo`: `cpu`, `memory`, `gpu`, `os`, `uptime`.
- [x] `Commands`: `show_description`, `exec`, `sysinfo`, `quit`, `part`, `aliases`.
- [x] `ServerMessages`: `condense`, `join_topic`, `join`, `part`, `quit`, `change_host`, `change_mode`, `change_nick`, `monitored_online`, `monitored_offline`, `standard_reply_fail`, `standard_reply_warn`, `standard_reply_note`, `wallops`, `kick`, `change_topic`, `request_topic`, `away`, `invite`, `default`.
- [x] `Condensation`: `messages`, `format`, `icon`, `timestamp`, `dimmed`, `max`.
- [x] `ServerMessage`: `enabled`, `smart`, `username_format`, `exclude`, `include`, `dimmed`, `triggers_unread`.
- [x] `ServerMessageDefault`: `enabled`, `smart`, `username_format`, `exclude`, `include`, `dimmed`, `triggers_unread`.
- [x] `InternalMessages`: `success`, `error`, `default`.
- [x] `InternalMessage`: `enabled`, `smart`.
- [x] `InternalMessageDefault`: `enabled`, `smart`.
- [x] `Chathistory`: `infinite_scroll`.

### `data/src/config/pane.rs`

- [x] `Pane`: `split_axis`, `scrollbar`, `restore_on_launch`, `gap`, `always_show_title_bar_buttons`.
- [x] `Gap`: `inner`, `outer`.

### `data/src/config/logs.rs`

- [x] `Logs`: `file_level`, `file_timestamp`, `pane_level`, `max_file_count`.

### `data/src/config/server/filehost.rs`

- [x] `Filehost`: `enabled`, `override_url`, `credentials`.

### `data/src/config/server/icon.rs`

- [x] `Icon`: `enabled`, `override_url`.

### `data/src/config/server/filters.rs`

- [x] `Filters`: `ignore`, `regex`.

### `data/src/config/server/reroute.rs`

- [x] `Reroute`: `query`, `notice`.
- [x] `RerouteRule`: `user`, `target`, `follow`.

### `data/src/config/buffer/channel.rs`

- [x] `Channel`: `nicklist`, `topic_banner`, `message`, `channel_name_casing`.
- [x] `Message`: `show_emoji_reacts`, `max_reaction_display`, `max_reaction_chars`.
- [x] `Nicklist`: `enabled`, `position`, `width`, `alignment`, `show_access_levels`, `show_bot_icon`, `truncate`, `scrollbar`.
- [x] `TopicBanner`: `enabled`, `max_lines`.

### `data/src/config/buffer/hide_consecutive.rs`

- [x] `HideConsecutive`: `enabled`, `show_after_previews`.

### `data/src/config/buffer/redaction.rs`

- [x] `Redaction`: `display`.

### `data/src/config/buffer/text_input.rs`

- [x] `TextInput`: `visibility`, `auto_format`, `autocomplete`, `nickname`, `key_bindings`, `kill_to_clipboard`, `max_lines`, `send_line_delay`, `persist`, `upload_long_paste`.
- [x] `Nickname`: `enabled`, `show_access_levels`.
- [x] `Autocomplete`: `order_by`, `sort_direction`, `completion_suffixes`, `include_ignored`.

### `data/src/config/buffer/typing.rs`

- [x] `Typing`: `share`, `show`, `style`, `font_size`, `animation`.
- [x] `Animation`: `enabled`, `size`.

### `data/src/config/buffer/nickname.rs`

- [x] `Nickname`: `away`, `offline`, `color`, `color_overrides`, `brackets`, `alignment`, `show_access_levels`, `show_bot_icon`, `shown_status`, `truncate`, `hide_consecutive`.
- [x] `OfflineStyle`: `color`, `alpha`.

## Additional tagged and nested formats

- [x] SASL PLAIN and EXTERNAL, password source variants, disconnect behavior.
- [x] Global/per-network HTTP, SOCKS5 and optional Tor proxies.
- [x] Filehost inherited/none/PLAIN/EXTERNAL credentials.
- [x] All supported bouncer network overrides, including enabled state.
- [x] Scope wildcards, lists and compound criteria (users/channels/server/message).
- [x] Repeated-name/timestamp booleans and smart windows with preview behavior.
- [x] Dimmed/solid/custom opacity and offline name color modes.
- [x] Ordered nickname colors, regex rules and metadata display selections.
- [x] Random and separate light/dark theme lists; scalar themes and pending gallery themes.
- [x] Cache unlimited/first-save modes and atomic optional DCC server settings.

Verification details and native evidence are recorded in
`.impeccable/review/remaining-settings-2026-10-29/` (local artifacts).

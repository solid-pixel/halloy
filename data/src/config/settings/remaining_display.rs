//! Additional display preferences. Register `schema()` after the incumbent
//! preference schema; `font_schema(area)` supplies Appearance's Font rows.
//! Structured collections are edited separately by `DisplayEditors` below.
use toml_edit::{
    Array, ArrayOfTables, DocumentMut, InlineTable, Item, Table, TableLike,
    Value, value,
};

use super::preferences::{Area, Choice, Kind, Spec};
use super::rules::{display_list, parse_list};
use super::{child_table, set};

macro_rules! choices {
    ($($label:literal => $value:literal),+ $(,)?) => {
        &[$(Choice { label: $label, value: $value }),+]
    };
}
const INHERIT_BOOL: &[Choice] =
    choices!("Use default" => "", "Enabled" => "true", "Disabled" => "false");
const EVENT: &[Choice] = choices!("Use default" => "", "Show" => "true", "Hide" => "false", "Do not store" => "\"drop\"");
const FORMAT: &[Choice] = choices!("Use default" => "", "Nickname only" => "\"short\"", "Nickname and user host" => "\"full\"");
const ACCESS: &[Choice] = choices!("Highest level" => "\"highest\"", "All levels" => "\"all\"", "Hidden" => "\"none\"");
const WEIGHT: &[Choice] = choices!("Thin" => "\"thin\"", "Extra light" => "\"extra-light\"", "Light" => "\"light\"", "Normal" => "\"normal\"", "Medium" => "\"medium\"", "Semibold" => "\"semibold\"", "Bold" => "\"bold\"", "Extra bold" => "\"extra-bold\"", "Black" => "\"black\"");
fn spec(
    area: Area,
    group: &'static str,
    path: &'static str,
    label: &'static str,
    kind: Kind,
    default: &'static str,
    help: &'static str,
) -> Spec {
    Spec {
        area,
        group,
        path,
        label,
        kind,
        default,
        help,
        aliases: &[],
    }
}
fn integer(optional: bool, max: i64) -> Kind {
    Kind::Integer {
        min: if optional { 1 } else { 0 },
        max,
        optional,
    }
}
fn positive(max: i64) -> Kind {
    Kind::Integer {
        min: 1,
        max,
        optional: false,
    }
}
fn float(optional: bool) -> Kind {
    Kind::Float {
        min: 0.01,
        max: 65535.0,
        optional,
    }
}

/// Pass the integration's Appearance area here. This keeps the module usable
/// with the existing Area enum until Appearance is registered centrally.
pub fn font_schema(area: Area) -> Vec<Spec> {
    // Static array avoids allocation/lifetime leakage in Spec's choice slice.
    const BOLD: &[Choice] = choices!("Automatic bold weight" => "", "Thin" => "\"thin\"", "Extra light" => "\"extra-light\"", "Light" => "\"light\"", "Normal" => "\"normal\"", "Medium" => "\"medium\"", "Semibold" => "\"semibold\"", "Bold" => "\"bold\"", "Extra bold" => "\"extra-bold\"", "Black" => "\"black\"");
    let mut result = vec![
        spec(
            area,
            "Font",
            "font.weight",
            "Regular weight",
            Kind::Choice(WEIGHT),
            "\"normal\"",
            "Requires restart.",
        ),
        spec(
            area,
            "Font",
            "font.bold_weight",
            "Bold weight",
            Kind::Choice(BOLD),
            "",
            "Requires restart.",
        ),
        spec(
            area,
            "Font",
            "font.stretch",
            "Font stretch",
            Kind::Choice(
                choices!("Ultra condensed" => "\"ultra-condensed\"", "Extra condensed" => "\"extra-condensed\"", "Condensed" => "\"condensed\"", "Semi condensed" => "\"semi-condensed\"", "Normal" => "\"normal\"", "Semi expanded" => "\"semi-expanded\"", "Expanded" => "\"expanded\"", "Extra expanded" => "\"extra-expanded\"", "Ultra expanded" => "\"ultra-expanded\""),
            ),
            "\"normal\"",
            "Requires restart.",
        ),
        spec(
            area,
            "Font",
            "font.only_emojis_size",
            "Emoji-only message size",
            integer(true, 255),
            "",
            "Leave empty for the font size. Requires restart.",
        ),
    ];
    result[1].aliases = &["font.bold-weight"];
    result
}
pub fn schema() -> Vec<Spec> {
    use Area::*;
    let mut fields = vec![
        spec(
            Navigation,
            "Opening chats",
            "actions.notification.default",
            "Clicking notifications",
            Kind::Choice(
                choices!("Activate Halloy" => "\"activate-application\"", "Open the conversation" => "\"open-buffer\""),
            ),
            "\"activate-application\"",
            "",
        ),
        spec(
            Chat,
            "Messages",
            "buffer.timestamp.context_menu_format",
            "Timestamp context menu format",
            Kind::Text,
            "\"%x\"",
            "Use Halloy's strftime format syntax.",
        ),
        spec(
            Chat,
            "Messages",
            "buffer.timestamp.brackets.left",
            "Left timestamp bracket",
            Kind::Text,
            "\"\"",
            "",
        ),
        spec(
            Chat,
            "Messages",
            "buffer.timestamp.brackets.right",
            "Right timestamp bracket",
            Kind::Text,
            "\"\"",
            "",
        ),
        spec(
            Navigation,
            "Click actions",
            "actions.buffer.only_contract_expanded_message",
            "Only contract an expanded message",
            Kind::Boolean,
            "true",
            "",
        ),
        spec(
            Navigation,
            "Click actions",
            "actions.buffer.click_image_url",
            "Clicking image links",
            Kind::Choice(
                choices!("Open URL" => "\"open-url\"", "Preview image" => "\"preview\""),
            ),
            "\"open-url\"",
            "",
        ),
        spec(
            Chat,
            "Names and users",
            "buffer.nickname.brackets.left",
            "Left nickname bracket",
            Kind::Text,
            "\"\"",
            "",
        ),
        spec(
            Chat,
            "Names and users",
            "buffer.nickname.brackets.right",
            "Right nickname bracket",
            Kind::Text,
            "\"\"",
            "",
        ),
        spec(
            Chat,
            "Names and users",
            "buffer.nickname.truncate",
            "Truncate nickname after",
            Kind::Integer {
                min: 0,
                max: 65535,
                optional: true,
            },
            "",
            "",
        ),
        spec(
            Chat,
            "Names and users",
            "buffer.nickname.shown_status",
            "Nickname status",
            Kind::Choice(
                choices!("Current status" => "\"current\"", "Status when sent" => "\"historical\""),
            ),
            "\"current\"",
            "",
        ),
        spec(
            Chat,
            "Names and users",
            "buffer.channel.nicklist.alignment",
            "User list alignment",
            Kind::Choice(
                choices!("Left" => "\"left\"", "Right" => "\"right\""),
            ),
            "\"left\"",
            "",
        ),
        spec(
            Chat,
            "Names and users",
            "buffer.channel.nicklist.show_access_levels",
            "User list access levels",
            Kind::Choice(ACCESS),
            "\"highest\"",
            "",
        ),
        spec(
            Chat,
            "Names and users",
            "buffer.channel.nicklist.show_bot_icon",
            "User list bot icon",
            Kind::Boolean,
            "true",
            "",
        ),
        spec(
            Chat,
            "Names and users",
            "buffer.channel.nicklist.truncate",
            "Truncate user list names after",
            Kind::Integer {
                min: 0,
                max: 65535,
                optional: true,
            },
            "",
            "",
        ),
        spec(
            Chat,
            "Names and users",
            "buffer.channel.nicklist.scrollbar.hidden",
            "Hide user list scrollbar",
            Kind::Boolean,
            "true",
            "",
        ),
        spec(
            Chat,
            "Names and users",
            "buffer.channel.nicklist.scrollbar.width",
            "User list scrollbar width",
            integer(false, 4294967295),
            "5",
            "",
        ),
        spec(
            Chat,
            "Names and users",
            "buffer.channel.nicklist.scrollbar.scroller_width",
            "User list scrollbar thumb width",
            integer(false, 4294967295),
            "5",
            "",
        ),
        spec(
            Chat,
            "Names and users",
            "buffer.channel.channel_name_casing",
            "Channel name casing",
            Kind::Choice(
                choices!("Use server casing" => "", "Lowercase" => "\"lowercase\""),
            ),
            "",
            "",
        ),
        spec(
            Chat,
            "Messages",
            "buffer.chathistory.infinite_scroll",
            "Load older history while scrolling",
            Kind::Boolean,
            "true",
            "",
        ),
        spec(
            Chat,
            "Messages",
            "buffer.status_message_prefix.brackets.left",
            "Left status prefix bracket",
            Kind::Text,
            "\"\"",
            "",
        ),
        spec(
            Chat,
            "Messages",
            "buffer.status_message_prefix.brackets.right",
            "Right status prefix bracket",
            Kind::Text,
            "\"\"",
            "",
        ),
        spec(
            Chat,
            "Replies and reactions",
            "buffer.reply.enabled",
            "Enable replies",
            Kind::Boolean,
            "true",
            "",
        ),
        spec(
            Chat,
            "Replies and reactions",
            "buffer.reply.show_icon",
            "Show reply icon",
            Kind::Boolean,
            "false",
            "",
        ),
        spec(
            Chat,
            "Replies and reactions",
            "buffer.reply.insert_nick",
            "Insert nickname when replying",
            Kind::Boolean,
            "true",
            "",
        ),
        spec(
            Chat,
            "Replies and reactions",
            "buffer.reply.hide_redundant_nicks",
            "Hide repeated reply nicknames",
            Kind::Boolean,
            "true",
            "",
        ),
        spec(
            Chat,
            "Replies and reactions",
            "buffer.reply.highlight_hovered_message",
            "Highlight reply target on hover",
            Kind::Boolean,
            "true",
            "",
        ),
        spec(
            Chat,
            "Replies and reactions",
            "buffer.reply.tooltip.enabled",
            "Show reply tooltip",
            Kind::Boolean,
            "true",
            "",
        ),
        spec(
            Chat,
            "Replies and reactions",
            "buffer.reply.icon_size",
            "Reply icon size",
            float(false),
            "10.0",
            "Pixels",
        ),
        spec(
            Chat,
            "Replies and reactions",
            "buffer.reply.tooltip.max_width",
            "Reply tooltip width",
            float(false),
            "600.0",
            "Pixels",
        ),
        spec(
            Chat,
            "Replies and reactions",
            "buffer.reply.tooltip.delay",
            "Reply tooltip delay",
            integer(false, 9223372036854775807),
            "500",
            "",
        ),
        spec(
            Chat,
            "Replies and reactions",
            "buffer.reply.tooltip.max_chars",
            "Reply tooltip character limit",
            integer(false, 9223372036854775807),
            "1024",
            "",
        ),
        spec(
            Chat,
            "Replies and reactions",
            "buffer.channel.message.show_emoji_reacts",
            "Show reactions",
            Kind::Boolean,
            "true",
            "",
        ),
        spec(
            Chat,
            "Replies and reactions",
            "buffer.channel.message.max_reaction_display",
            "Visible reactions",
            positive(4294967295),
            "5",
            "",
        ),
        spec(
            Chat,
            "Replies and reactions",
            "buffer.channel.message.max_reaction_chars",
            "Reaction character limit",
            positive(4294967295),
            "64",
            "",
        ),
        spec(
            Chat,
            "Replies and reactions",
            "buffer.redaction.display",
            "Redacted messages",
            Kind::Choice(
                choices!("Hide" => "\"none\"", "Show redaction" => "\"redacted\"", "Dim original" => "\"dimmed\""),
            ),
            "\"redacted\"",
            "",
        ),
        spec(
            Chat,
            "Replies and reactions",
            "buffer.close.query",
            "Closing private conversations",
            Kind::Choice(
                choices!("Keep conversation" => "\"keep\"", "Close with pane" => "\"close-pane\"", "Close with buffer" => "\"close-buffer\""),
            ),
            "\"keep\"",
            "",
        ),
        spec(
            Chat,
            "Composer",
            "buffer.text_input.key_bindings",
            "Composer keys",
            Kind::Choice(
                choices!("Platform default" => "", "Standard" => "\"default\"", "Emacs" => "\"emacs\""),
            ),
            "",
            "",
        ),
        spec(
            Chat,
            "Composer",
            "buffer.text_input.kill_to_clipboard",
            "Copy killed text to clipboard",
            Kind::Boolean,
            "true",
            "",
        ),
        spec(
            Chat,
            "Composer",
            "buffer.text_input.send_line_delay",
            "Delay between sent lines (ms)",
            integer(false, 9223372036854775807),
            "100",
            "",
        ),
        spec(
            Chat,
            "Composer",
            "buffer.text_input.upload_long_paste",
            "Upload long pastes",
            Kind::Boolean,
            "false",
            "",
        ),
        spec(
            Chat,
            "Composer",
            "buffer.text_input.nickname.show_access_levels",
            "Composer nickname access levels",
            Kind::Choice(ACCESS),
            "\"highest\"",
            "",
        ),
        spec(
            Chat,
            "Composer",
            "buffer.text_input.autocomplete.sort_direction",
            "Completion direction",
            Kind::Choice(
                choices!("Ascending" => "\"asc\"", "Descending" => "\"desc\""),
            ),
            "\"asc\"",
            "",
        ),
        spec(
            Chat,
            "Composer",
            "buffer.text_input.autocomplete.include_ignored",
            "Include ignored users in completion",
            Kind::Boolean,
            "false",
            "",
        ),
        spec(
            Chat,
            "Composer",
            "buffer.emojis.skin_tone",
            "Emoji skin tone",
            Kind::Choice(
                choices!("Default" => "\"default\"", "Light" => "\"light\"", "Medium Light" => "\"medium-light\"", "Medium" => "\"medium\"", "Medium Dark" => "\"medium-dark\"", "Dark" => "\"dark\""),
            ),
            "\"default\"",
            "",
        ),
        spec(
            Chat,
            "Composer",
            "buffer.emojis.characters_to_trigger_picker",
            "Characters before emoji suggestions",
            integer(false, 9223372036854775807),
            "2",
            "",
        ),
        spec(
            Chat,
            "Typing",
            "buffer.typing.style",
            "Typing indicator style",
            Kind::Choice(
                choices!("Padded" => "\"padded\"", "Popped" => "\"popped\""),
            ),
            "\"popped\"",
            "",
        ),
        spec(
            Chat,
            "Typing",
            "buffer.typing.font_size",
            "Typing text size",
            integer(true, 255),
            "",
            "",
        ),
        spec(
            Chat,
            "Typing",
            "buffer.typing.animation.size",
            "Typing animation size",
            integer(true, 255),
            "",
            "",
        ),
        spec(
            Chat,
            "Links",
            "buffer.url.prompt_before_open",
            "Confirm before opening links",
            Kind::Boolean,
            "false",
            "",
        ),
        spec(
            Sidebar,
            "Layout",
            "sidebar.internal_buffers.position",
            "Internal tools position",
            Kind::Choice(
                choices!("Before servers" => "\"before-servers\"", "After servers" => "\"after-servers\""),
            ),
            "\"after-servers\"",
            "",
        ),
        spec(
            Sidebar,
            "Highlights and typing",
            "sidebar.typing_indicator.icon_height",
            "Typing icon height",
            positive(4294967295),
            "4",
            "Pixels",
        ),
        spec(
            Chat,
            "Activity: Default",
            "buffer.server_messages.default.smart",
            "Smart activity window (seconds)",
            Kind::Integer {
                min: 0,
                max: 9223372036854775807,
                optional: true,
            },
            "",
            "Leave empty to inherit. Show activity for recently active users; Away instead suppresses repeated notices.",
        ),
        spec(
            Chat,
            "Activity: Default",
            "buffer.server_messages.default.username_format",
            "User details",
            Kind::Choice(FORMAT),
            "\"full\"",
            "",
        ),
        spec(
            Chat,
            "Activity: Default",
            "buffer.server_messages.default.triggers_unread",
            "Trigger unread indicator",
            Kind::Choice(INHERIT_BOOL),
            "",
            "",
        ),
        spec(
            Chat,
            "Activity: Join topic",
            "buffer.server_messages.join_topic.enabled",
            "Show message",
            Kind::Choice(EVENT),
            "",
            "",
        ),
        spec(
            Chat,
            "Activity: Join topic",
            "buffer.server_messages.join_topic.smart",
            "Smart activity window (seconds)",
            Kind::Integer {
                min: 0,
                max: 9223372036854775807,
                optional: true,
            },
            "",
            "Leave empty to inherit. Show activity for recently active users; Away instead suppresses repeated notices.",
        ),
        spec(
            Chat,
            "Activity: Join topic",
            "buffer.server_messages.join_topic.username_format",
            "User details",
            Kind::Choice(FORMAT),
            "",
            "",
        ),
        spec(
            Chat,
            "Activity: Join topic",
            "buffer.server_messages.join_topic.triggers_unread",
            "Trigger unread indicator",
            Kind::Choice(INHERIT_BOOL),
            "",
            "",
        ),
        spec(
            Chat,
            "Activity: Join",
            "buffer.server_messages.join.smart",
            "Smart activity window (seconds)",
            Kind::Integer {
                min: 0,
                max: 9223372036854775807,
                optional: true,
            },
            "",
            "Leave empty to inherit. Show activity for recently active users; Away instead suppresses repeated notices.",
        ),
        spec(
            Chat,
            "Activity: Join",
            "buffer.server_messages.join.username_format",
            "User details",
            Kind::Choice(FORMAT),
            "",
            "",
        ),
        spec(
            Chat,
            "Activity: Join",
            "buffer.server_messages.join.triggers_unread",
            "Trigger unread indicator",
            Kind::Choice(INHERIT_BOOL),
            "",
            "",
        ),
        spec(
            Chat,
            "Activity: Part",
            "buffer.server_messages.part.smart",
            "Smart activity window (seconds)",
            Kind::Integer {
                min: 0,
                max: 9223372036854775807,
                optional: true,
            },
            "",
            "Leave empty to inherit. Show activity for recently active users; Away instead suppresses repeated notices.",
        ),
        spec(
            Chat,
            "Activity: Part",
            "buffer.server_messages.part.username_format",
            "User details",
            Kind::Choice(FORMAT),
            "",
            "",
        ),
        spec(
            Chat,
            "Activity: Part",
            "buffer.server_messages.part.triggers_unread",
            "Trigger unread indicator",
            Kind::Choice(INHERIT_BOOL),
            "",
            "",
        ),
        spec(
            Chat,
            "Activity: Quit",
            "buffer.server_messages.quit.smart",
            "Smart activity window (seconds)",
            Kind::Integer {
                min: 0,
                max: 9223372036854775807,
                optional: true,
            },
            "",
            "Leave empty to inherit. Show activity for recently active users; Away instead suppresses repeated notices.",
        ),
        spec(
            Chat,
            "Activity: Quit",
            "buffer.server_messages.quit.username_format",
            "User details",
            Kind::Choice(FORMAT),
            "",
            "",
        ),
        spec(
            Chat,
            "Activity: Quit",
            "buffer.server_messages.quit.triggers_unread",
            "Trigger unread indicator",
            Kind::Choice(INHERIT_BOOL),
            "",
            "",
        ),
        spec(
            Chat,
            "Activity: Change host",
            "buffer.server_messages.change_host.enabled",
            "Show message",
            Kind::Choice(EVENT),
            "",
            "",
        ),
        spec(
            Chat,
            "Activity: Change host",
            "buffer.server_messages.change_host.smart",
            "Smart activity window (seconds)",
            Kind::Integer {
                min: 0,
                max: 9223372036854775807,
                optional: true,
            },
            "",
            "Leave empty to inherit. Show activity for recently active users; Away instead suppresses repeated notices.",
        ),
        spec(
            Chat,
            "Activity: Change host",
            "buffer.server_messages.change_host.username_format",
            "User details",
            Kind::Choice(FORMAT),
            "",
            "",
        ),
        spec(
            Chat,
            "Activity: Change host",
            "buffer.server_messages.change_host.triggers_unread",
            "Trigger unread indicator",
            Kind::Choice(INHERIT_BOOL),
            "",
            "",
        ),
        spec(
            Chat,
            "Activity: Change mode",
            "buffer.server_messages.change_mode.smart",
            "Smart activity window (seconds)",
            Kind::Integer {
                min: 0,
                max: 9223372036854775807,
                optional: true,
            },
            "",
            "Leave empty to inherit. Show activity for recently active users; Away instead suppresses repeated notices.",
        ),
        spec(
            Chat,
            "Activity: Change mode",
            "buffer.server_messages.change_mode.username_format",
            "User details",
            Kind::Choice(FORMAT),
            "",
            "",
        ),
        spec(
            Chat,
            "Activity: Change mode",
            "buffer.server_messages.change_mode.triggers_unread",
            "Trigger unread indicator",
            Kind::Choice(INHERIT_BOOL),
            "",
            "",
        ),
        spec(
            Chat,
            "Activity: Change nick",
            "buffer.server_messages.change_nick.smart",
            "Smart activity window (seconds)",
            Kind::Integer {
                min: 0,
                max: 9223372036854775807,
                optional: true,
            },
            "",
            "Leave empty to inherit. Show activity for recently active users; Away instead suppresses repeated notices.",
        ),
        spec(
            Chat,
            "Activity: Change nick",
            "buffer.server_messages.change_nick.username_format",
            "User details",
            Kind::Choice(FORMAT),
            "",
            "",
        ),
        spec(
            Chat,
            "Activity: Change nick",
            "buffer.server_messages.change_nick.triggers_unread",
            "Trigger unread indicator",
            Kind::Choice(INHERIT_BOOL),
            "",
            "",
        ),
        spec(
            Chat,
            "Activity: Monitored online",
            "buffer.server_messages.monitored_online.enabled",
            "Show message",
            Kind::Choice(EVENT),
            "",
            "",
        ),
        spec(
            Chat,
            "Activity: Monitored online",
            "buffer.server_messages.monitored_online.smart",
            "Smart activity window (seconds)",
            Kind::Integer {
                min: 0,
                max: 9223372036854775807,
                optional: true,
            },
            "",
            "Leave empty to inherit. Show activity for recently active users; Away instead suppresses repeated notices.",
        ),
        spec(
            Chat,
            "Activity: Monitored online",
            "buffer.server_messages.monitored_online.username_format",
            "User details",
            Kind::Choice(FORMAT),
            "",
            "",
        ),
        spec(
            Chat,
            "Activity: Monitored online",
            "buffer.server_messages.monitored_online.triggers_unread",
            "Trigger unread indicator",
            Kind::Choice(INHERIT_BOOL),
            "",
            "",
        ),
        spec(
            Chat,
            "Activity: Monitored offline",
            "buffer.server_messages.monitored_offline.enabled",
            "Show message",
            Kind::Choice(EVENT),
            "",
            "",
        ),
        spec(
            Chat,
            "Activity: Monitored offline",
            "buffer.server_messages.monitored_offline.smart",
            "Smart activity window (seconds)",
            Kind::Integer {
                min: 0,
                max: 9223372036854775807,
                optional: true,
            },
            "",
            "Leave empty to inherit. Show activity for recently active users; Away instead suppresses repeated notices.",
        ),
        spec(
            Chat,
            "Activity: Monitored offline",
            "buffer.server_messages.monitored_offline.username_format",
            "User details",
            Kind::Choice(FORMAT),
            "",
            "",
        ),
        spec(
            Chat,
            "Activity: Monitored offline",
            "buffer.server_messages.monitored_offline.triggers_unread",
            "Trigger unread indicator",
            Kind::Choice(INHERIT_BOOL),
            "",
            "",
        ),
        spec(
            Chat,
            "Activity: Standard reply fail",
            "buffer.server_messages.standard_reply_fail.enabled",
            "Show message",
            Kind::Choice(EVENT),
            "",
            "",
        ),
        spec(
            Chat,
            "Activity: Standard reply fail",
            "buffer.server_messages.standard_reply_fail.smart",
            "Smart activity window (seconds)",
            Kind::Integer {
                min: 0,
                max: 9223372036854775807,
                optional: true,
            },
            "",
            "Leave empty to inherit. Show activity for recently active users; Away instead suppresses repeated notices.",
        ),
        spec(
            Chat,
            "Activity: Standard reply fail",
            "buffer.server_messages.standard_reply_fail.username_format",
            "User details",
            Kind::Choice(FORMAT),
            "",
            "",
        ),
        spec(
            Chat,
            "Activity: Standard reply fail",
            "buffer.server_messages.standard_reply_fail.triggers_unread",
            "Trigger unread indicator",
            Kind::Choice(INHERIT_BOOL),
            "",
            "",
        ),
        spec(
            Chat,
            "Activity: Standard reply warn",
            "buffer.server_messages.standard_reply_warn.enabled",
            "Show message",
            Kind::Choice(EVENT),
            "",
            "",
        ),
        spec(
            Chat,
            "Activity: Standard reply warn",
            "buffer.server_messages.standard_reply_warn.smart",
            "Smart activity window (seconds)",
            Kind::Integer {
                min: 0,
                max: 9223372036854775807,
                optional: true,
            },
            "",
            "Leave empty to inherit. Show activity for recently active users; Away instead suppresses repeated notices.",
        ),
        spec(
            Chat,
            "Activity: Standard reply warn",
            "buffer.server_messages.standard_reply_warn.username_format",
            "User details",
            Kind::Choice(FORMAT),
            "",
            "",
        ),
        spec(
            Chat,
            "Activity: Standard reply warn",
            "buffer.server_messages.standard_reply_warn.triggers_unread",
            "Trigger unread indicator",
            Kind::Choice(INHERIT_BOOL),
            "",
            "",
        ),
        spec(
            Chat,
            "Activity: Standard reply note",
            "buffer.server_messages.standard_reply_note.enabled",
            "Show message",
            Kind::Choice(EVENT),
            "",
            "",
        ),
        spec(
            Chat,
            "Activity: Standard reply note",
            "buffer.server_messages.standard_reply_note.smart",
            "Smart activity window (seconds)",
            Kind::Integer {
                min: 0,
                max: 9223372036854775807,
                optional: true,
            },
            "",
            "Leave empty to inherit. Show activity for recently active users; Away instead suppresses repeated notices.",
        ),
        spec(
            Chat,
            "Activity: Standard reply note",
            "buffer.server_messages.standard_reply_note.username_format",
            "User details",
            Kind::Choice(FORMAT),
            "",
            "",
        ),
        spec(
            Chat,
            "Activity: Standard reply note",
            "buffer.server_messages.standard_reply_note.triggers_unread",
            "Trigger unread indicator",
            Kind::Choice(INHERIT_BOOL),
            "",
            "",
        ),
        spec(
            Chat,
            "Activity: Wallops",
            "buffer.server_messages.wallops.enabled",
            "Show message",
            Kind::Choice(EVENT),
            "",
            "",
        ),
        spec(
            Chat,
            "Activity: Wallops",
            "buffer.server_messages.wallops.smart",
            "Smart activity window (seconds)",
            Kind::Integer {
                min: 0,
                max: 9223372036854775807,
                optional: true,
            },
            "",
            "Leave empty to inherit. Show activity for recently active users; Away instead suppresses repeated notices.",
        ),
        spec(
            Chat,
            "Activity: Wallops",
            "buffer.server_messages.wallops.username_format",
            "User details",
            Kind::Choice(FORMAT),
            "",
            "",
        ),
        spec(
            Chat,
            "Activity: Wallops",
            "buffer.server_messages.wallops.triggers_unread",
            "Trigger unread indicator",
            Kind::Choice(INHERIT_BOOL),
            "",
            "",
        ),
        spec(
            Chat,
            "Activity: Kick",
            "buffer.server_messages.kick.enabled",
            "Show message",
            Kind::Choice(EVENT),
            "",
            "",
        ),
        spec(
            Chat,
            "Activity: Kick",
            "buffer.server_messages.kick.smart",
            "Smart activity window (seconds)",
            Kind::Integer {
                min: 0,
                max: 9223372036854775807,
                optional: true,
            },
            "",
            "Leave empty to inherit. Show activity for recently active users; Away instead suppresses repeated notices.",
        ),
        spec(
            Chat,
            "Activity: Kick",
            "buffer.server_messages.kick.username_format",
            "User details",
            Kind::Choice(FORMAT),
            "",
            "",
        ),
        spec(
            Chat,
            "Activity: Kick",
            "buffer.server_messages.kick.triggers_unread",
            "Trigger unread indicator",
            Kind::Choice(INHERIT_BOOL),
            "",
            "",
        ),
        spec(
            Chat,
            "Activity: Change topic",
            "buffer.server_messages.change_topic.enabled",
            "Show message",
            Kind::Choice(EVENT),
            "",
            "",
        ),
        spec(
            Chat,
            "Activity: Change topic",
            "buffer.server_messages.change_topic.smart",
            "Smart activity window (seconds)",
            Kind::Integer {
                min: 0,
                max: 9223372036854775807,
                optional: true,
            },
            "",
            "Leave empty to inherit. Show activity for recently active users; Away instead suppresses repeated notices.",
        ),
        spec(
            Chat,
            "Activity: Change topic",
            "buffer.server_messages.change_topic.username_format",
            "User details",
            Kind::Choice(FORMAT),
            "",
            "",
        ),
        spec(
            Chat,
            "Activity: Change topic",
            "buffer.server_messages.change_topic.triggers_unread",
            "Trigger unread indicator",
            Kind::Choice(INHERIT_BOOL),
            "",
            "",
        ),
        spec(
            Chat,
            "Activity: Request topic",
            "buffer.server_messages.request_topic.enabled",
            "Show message",
            Kind::Choice(EVENT),
            "",
            "",
        ),
        spec(
            Chat,
            "Activity: Request topic",
            "buffer.server_messages.request_topic.smart",
            "Smart activity window (seconds)",
            Kind::Integer {
                min: 0,
                max: 9223372036854775807,
                optional: true,
            },
            "",
            "Leave empty to inherit. Show activity for recently active users; Away instead suppresses repeated notices.",
        ),
        spec(
            Chat,
            "Activity: Request topic",
            "buffer.server_messages.request_topic.username_format",
            "User details",
            Kind::Choice(FORMAT),
            "",
            "",
        ),
        spec(
            Chat,
            "Activity: Request topic",
            "buffer.server_messages.request_topic.triggers_unread",
            "Trigger unread indicator",
            Kind::Choice(INHERIT_BOOL),
            "",
            "",
        ),
        spec(
            Chat,
            "Activity: Away",
            "buffer.server_messages.away.enabled",
            "Show message",
            Kind::Choice(EVENT),
            "",
            "",
        ),
        spec(
            Chat,
            "Activity: Away",
            "buffer.server_messages.away.smart",
            "Smart activity window (seconds)",
            Kind::Integer {
                min: 0,
                max: 9223372036854775807,
                optional: true,
            },
            "",
            "Leave empty to inherit. Show activity for recently active users; Away instead suppresses repeated notices.",
        ),
        spec(
            Chat,
            "Activity: Away",
            "buffer.server_messages.away.username_format",
            "User details",
            Kind::Choice(FORMAT),
            "",
            "",
        ),
        spec(
            Chat,
            "Activity: Away",
            "buffer.server_messages.away.triggers_unread",
            "Trigger unread indicator",
            Kind::Choice(INHERIT_BOOL),
            "",
            "",
        ),
        spec(
            Chat,
            "Activity: Invite",
            "buffer.server_messages.invite.enabled",
            "Show message",
            Kind::Choice(EVENT),
            "",
            "",
        ),
        spec(
            Chat,
            "Activity: Invite",
            "buffer.server_messages.invite.smart",
            "Smart activity window (seconds)",
            Kind::Integer {
                min: 0,
                max: 9223372036854775807,
                optional: true,
            },
            "",
            "Leave empty to inherit. Show activity for recently active users; Away instead suppresses repeated notices.",
        ),
        spec(
            Chat,
            "Activity: Invite",
            "buffer.server_messages.invite.username_format",
            "User details",
            Kind::Choice(FORMAT),
            "",
            "",
        ),
        spec(
            Chat,
            "Activity: Invite",
            "buffer.server_messages.invite.triggers_unread",
            "Trigger unread indicator",
            Kind::Choice(INHERIT_BOOL),
            "",
            "",
        ),
        spec(
            Chat,
            "Internal activity",
            "buffer.internal_messages.default.enabled",
            "Show internal messages",
            Kind::Boolean,
            "true",
            "",
        ),
        spec(
            Chat,
            "Internal activity",
            "buffer.internal_messages.default.smart",
            "Default smart window (seconds)",
            Kind::Integer {
                min: 0,
                max: 9223372036854775807,
                optional: true,
            },
            "",
            "",
        ),
        spec(
            Chat,
            "Internal activity",
            "buffer.internal_messages.success.enabled",
            "Show success messages",
            Kind::Choice(INHERIT_BOOL),
            "",
            "",
        ),
        spec(
            Chat,
            "Internal activity",
            "buffer.internal_messages.success.smart",
            "Success smart window (seconds)",
            Kind::Integer {
                min: 0,
                max: 9223372036854775807,
                optional: true,
            },
            "",
            "",
        ),
        spec(
            Chat,
            "Internal activity",
            "buffer.internal_messages.error.enabled",
            "Show error messages",
            Kind::Choice(INHERIT_BOOL),
            "",
            "",
        ),
        spec(
            Chat,
            "Internal activity",
            "buffer.internal_messages.error.smart",
            "Error smart window (seconds)",
            Kind::Integer {
                min: 0,
                max: 9223372036854775807,
                optional: true,
            },
            "",
            "",
        ),
        spec(
            Chat,
            "Condensed activity",
            "buffer.server_messages.condense.format",
            "Format",
            Kind::Choice(
                choices!("Brief" => "\"brief\"", "Detailed" => "\"detailed\"", "Full" => "\"full\"", "Summary" => "\"summary\""),
            ),
            "\"brief\"",
            "",
        ),
        spec(
            Chat,
            "Condensed activity",
            "buffer.server_messages.condense.icon",
            "Icon",
            Kind::Choice(
                choices!("None" => "\"none\"", "Chevron" => "\"chevron\"", "Dot" => "\"dot\""),
            ),
            "\"none\"",
            "",
        ),
        spec(
            Chat,
            "Condensed activity",
            "buffer.server_messages.condense.timestamp",
            "Timestamp",
            Kind::Choice(
                choices!("Range" => "\"range\"", "Start" => "\"start\"", "End" => "\"end\"", "None" => "\"none\""),
            ),
            "\"range\"",
            "",
        ),
        spec(
            Chat,
            "Condensed activity",
            "buffer.server_messages.condense.max",
            "Maximum messages in one group",
            Kind::Integer {
                min: 0,
                max: 65535,
                optional: true,
            },
            "",
            "",
        ),
    ];
    for field in &mut fields {
        if field.path == "buffer.text_input.nickname.show_access_levels" {
            field.aliases = &["buffer.text_input.nickname.show_access_level"];
        }
    }
    fields
}

/// A list is an ordered set of individual controls, never TOML source text.
#[derive(Debug, Clone)]
pub struct StringList {
    pub area: Option<Area>,
    pub group: &'static str,
    pub label: &'static str,
    pub path: &'static str,
    pub entries: Vec<String>,
    /// Empty for free text; these are exact runtime enum spellings otherwise.
    pub options: &'static [&'static str],
    pub fixed_len: Option<usize>,
    pub wrapper: Option<&'static str>,
    original: Vec<String>,
}
impl StringList {
    #[allow(clippy::too_many_arguments)]
    fn new(
        document: &DocumentMut,
        area: Option<Area>,
        group: &'static str,
        label: &'static str,
        path: &'static str,
        default: &[&str],
        options: &'static [&'static str],
        fixed_len: Option<usize>,
        wrapper: Option<&'static str>,
    ) -> Self {
        let item = at(document.as_item(), path);
        let item =
            wrapper.map_or(item, |key| item.and_then(|item| item.get(key)));
        let entries: Vec<String> = item.and_then(Item::as_array).map_or_else(
            || default.iter().map(|s| (*s).into()).collect(),
            |array| {
                array
                    .iter()
                    .filter_map(Value::as_str)
                    .map(str::to_owned)
                    .collect()
            },
        );
        Self {
            area,
            group,
            label,
            path,
            original: entries.clone(),
            entries,
            options,
            fixed_len,
            wrapper,
        }
    }
    pub fn is_dirty(&self) -> bool {
        self.entries != self.original
    }
    fn render_into(&self, document: &mut DocumentMut) -> Result<(), String> {
        if !self.is_dirty() {
            return Ok(());
        }
        if self.wrapper == Some("palette")
            && self.entries.iter().any(|entry| !valid_color(entry))
        {
            return Err("Enter a color as #RRGGBB or #RRGGBBAA.".into());
        }
        if self.fixed_len.is_some_and(|len| self.entries.len() != len) {
            return Err(format!(
                "{} needs exactly {} entries.",
                self.label,
                self.fixed_len.unwrap()
            ));
        }
        if !self.options.is_empty()
            && self.entries.iter().any(|entry| {
                !self.options.contains(&entry.as_str())
                    && !self.original.contains(entry)
            })
        {
            return Err(format!(
                "{}: choose a supported value for each entry.",
                self.label
            ));
        }
        let previous = at(document.as_item(), self.path);
        let source = self
            .wrapper
            .map_or(previous, |key| previous.and_then(|item| item.get(key)));
        let mut array = Array::new();
        // Retain value decorations for untouched list entries, even after moving.
        let mut used = vec![];
        for entry in &self.entries {
            let original = source.and_then(Item::as_array).and_then(|old| {
                old.iter()
                    .enumerate()
                    .find(|(index, value)| {
                        !used.contains(index)
                            && value.as_str() == Some(entry.as_str())
                    })
                    .map(|(index, value)| {
                        used.push(index);
                        value.clone()
                    })
            });
            array.push_formatted(
                original.unwrap_or_else(|| Value::from(entry.clone())),
            );
        }
        let item = if let Some(key) = self.wrapper {
            let mut item = previous
                .filter(|item| item.is_table_like())
                .cloned()
                .unwrap_or_else(|| {
                    Item::Value(Value::InlineTable(InlineTable::new()))
                });
            set(item.as_table_like_mut().unwrap(), key, value(array));
            item
        } else {
            value(array)
        };
        write_at(document, self.path, item)
    }
}
fn at<'a>(mut item: &'a Item, path: &str) -> Option<&'a Item> {
    for key in path.split('.') {
        item = item.as_table_like()?.get(key)?;
    }
    Some(item)
}
fn write_at(
    document: &mut DocumentMut,
    path: &str,
    item: Item,
) -> Result<(), String> {
    let mut keys = path.split('.').collect::<Vec<_>>();
    let last = keys.pop().ok_or("Missing setting name")?;
    let mut parent: &mut dyn TableLike = document.as_table_mut();
    for key in keys {
        if matches!(key, "unread_indicator" | "highlight_indicator")
            && let Some(mode) =
                parent.get(key).and_then(Item::as_str).map(str::to_owned)
        {
            let mut table = Table::new();
            table.insert("title", value(mode == "title"));
            table.insert(
                "icon",
                value(if mode == "dot" { "dot" } else { "none" }),
            );
            set(
                parent,
                key,
                Item::Value(Value::InlineTable(table.into_inline_table())),
            );
        }
        parent = child_table(parent, key)?;
    }
    super::preferences::complete_brackets(parent, path);
    set(parent, last, item);
    Ok(())
}
fn string(item: Option<&Item>, default: &str) -> String {
    item.and_then(Item::as_str).unwrap_or(default).into()
}
fn strings(item: Option<&Item>) -> Vec<String> {
    item.and_then(Item::as_array)
        .map(|array| {
            array
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}
fn list_item(entries: Vec<String>) -> Item {
    let mut array = Array::new();
    for entry in entries {
        array.push(entry);
    }
    value(array)
}
fn records(item: Option<&Item>) -> Vec<Item> {
    if let Some(array) = item.and_then(Item::as_array_of_tables) {
        return array.iter().cloned().map(Item::Table).collect();
    }
    item.and_then(Item::as_array)
        .map(|array| array.iter().cloned().map(Item::Value).collect())
        .unwrap_or_default()
}
fn render_records(source: Option<&Item>, records: Vec<Item>) -> Item {
    if source.is_some_and(Item::is_array_of_tables) {
        let mut array = ArrayOfTables::new();
        let positions = source
            .and_then(Item::as_array_of_tables)
            .map(|array| array.iter().map(Table::position).collect::<Vec<_>>())
            .unwrap_or_default();
        for (index, item) in records.into_iter().enumerate() {
            if let Ok(mut table) = item.into_table() {
                // Document serialization sorts by original table position.
                // Keep positions tied to slots so reordering changes priority.
                table.set_position(positions.get(index).copied().flatten());
                array.push(table);
            }
        }
        Item::ArrayOfTables(array)
    } else {
        let mut array = Array::new();
        for item in records {
            if let Ok(value) = item.into_value() {
                array.push_formatted(value);
            }
        }
        value(array)
    }
}
fn optional_text(table: &mut dyn TableLike, key: &str, text: &str) {
    if text.is_empty() {
        table.remove(key);
    } else {
        set(table, key, value(text));
    }
}
fn optional_list(
    table: &mut dyn TableLike,
    key: &str,
    input: &str,
) -> Result<(), String> {
    let entries = parse_list(input)?;
    if entries.is_empty() {
        table.remove(key);
    } else {
        set(table, key, list_item(entries));
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CriterionValues {
    pub users: String,
    pub user: String,
    pub channels: String,
    pub channel: String,
    pub server: String,
    pub server_message: String,
}
#[derive(Debug, Clone)]
pub struct Criterion {
    pub values: CriterionValues,
    original_values: CriterionValues,
    original: Item,
}
impl Criterion {
    fn parse(item: Item) -> Self {
        let values = CriterionValues {
            users: display_list(&strings(item.get("users"))),
            user: string(item.get("user"), ""),
            channels: display_list(&strings(item.get("channels"))),
            channel: string(item.get("channel"), ""),
            server: string(item.get("server"), ""),
            server_message: string(item.get("server_message"), ""),
        };
        Self {
            original_values: values.clone(),
            values,
            original: item,
        }
    }
    fn render(&self) -> Result<Item, String> {
        let mut item = self.original.clone();
        let table = item
            .as_table_like_mut()
            .ok_or("Condition must be a table")?;
        for (key, next, previous) in [
            ("users", &self.values.users, &self.original_values.users),
            (
                "channels",
                &self.values.channels,
                &self.original_values.channels,
            ),
        ] {
            if next != previous {
                optional_list(table, key, next)?;
            }
        }
        for (key, next, previous) in [
            ("user", &self.values.user, &self.original_values.user),
            (
                "channel",
                &self.values.channel,
                &self.original_values.channel,
            ),
            ("server", &self.values.server, &self.original_values.server),
            (
                "server_message",
                &self.values.server_message,
                &self.original_values.server_message,
            ),
        ] {
            if next != previous {
                optional_text(table, key, next);
            }
        }
        Ok(item)
    }
}
#[derive(Debug, Clone)]
pub struct Filter {
    pub area: Area,
    pub group: &'static str,
    pub label: String,
    pub path: String,
    /// Users, channels, servers, server-message kinds. "*" covers a scope.
    pub scopes: [String; 4],
    pub criteria: Vec<Criterion>,
    original_scopes: [String; 4],
    original_criteria: Vec<CriterionValues>,
    normalized: Item,
}
const SCOPES: [&str; 4] = ["users", "channels", "servers", "server_messages"];
impl Filter {
    fn parse(
        document: &DocumentMut,
        area: Area,
        group: &'static str,
        label: String,
        path: String,
    ) -> Result<Self, String> {
        use crate::config::inclusivities::{Inclusivities, Inclusivity};
        let original = at(document.as_item(), &path);
        let normalized =
            if let Some(item) = original.filter(|item| item.is_table_like()) {
                item.clone()
            } else {
                let mut table = Table::new();
                if let Some(item) = original {
                    let parsed: toml::Table =
                        toml::from_str(&format!("v = {item}"))
                            .map_err(|error| error.to_string())?;
                    let inclusivities: Inclusivities = parsed["v"]
                        .clone()
                        .try_into()
                        .map_err(|error: toml::de::Error| error.to_string())?;
                    for (key, scope) in SCOPES.into_iter().zip([
                        inclusivities.users,
                        inclusivities.channels,
                        inclusivities.servers,
                        inclusivities.server_messages,
                    ]) {
                        match scope {
                            Some(Inclusivity::All) => {
                                table.insert(key, value("*"));
                            }
                            Some(Inclusivity::Any(entries)) => {
                                table.insert(key, list_item(entries));
                            }
                            None => {}
                        }
                    }
                }
                Item::Value(Value::InlineTable(table.into_inline_table()))
            };
        let scopes = SCOPES.map(|key| match normalized.get(key) {
            Some(item) if item.is_str() => string(Some(item), ""),
            other => display_list(&strings(other)),
        });
        let criteria = records(normalized.get("criteria"))
            .into_iter()
            .map(Criterion::parse)
            .collect::<Vec<_>>();
        Ok(Self {
            area,
            group,
            label,
            path,
            original_scopes: scopes.clone(),
            scopes,
            original_criteria: criteria
                .iter()
                .map(|criterion| criterion.values.clone())
                .collect(),
            criteria,
            normalized,
        })
    }
    pub fn is_dirty(&self) -> bool {
        self.scopes != self.original_scopes
            || self
                .criteria
                .iter()
                .map(|criterion| &criterion.values)
                .ne(self.original_criteria.iter())
    }
    fn render_into(&self, document: &mut DocumentMut) -> Result<(), String> {
        if !self.is_dirty() {
            return Ok(());
        }
        let mut item = at(document.as_item(), &self.path)
            .filter(|item| item.is_table_like())
            .cloned()
            .unwrap_or_else(|| self.normalized.clone());
        let table = item
            .as_table_like_mut()
            .ok_or("Conditions must be a table")?;
        for (index, key) in SCOPES.iter().enumerate() {
            if self.scopes[index] != self.original_scopes[index] {
                if matches!(self.scopes[index].trim(), "*" | "all") {
                    set(table, key, value("*"));
                } else {
                    optional_list(table, key, &self.scopes[index])?;
                }
            }
        }
        if self
            .criteria
            .iter()
            .map(|criterion| &criterion.values)
            .ne(self.original_criteria.iter())
        {
            let rendered = self
                .criteria
                .iter()
                .map(Criterion::render)
                .collect::<Result<Vec<_>, _>>()?;
            let next = render_records(table.get("criteria"), rendered);
            set(table, "criteria", next);
        }
        write_at(document, &self.path, item)
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColorValues {
    pub regex_mode: bool,
    pub regex: String,
    pub nicknames: String,
    pub ignore_case: bool,
    pub color: String,
}
#[derive(Debug, Clone)]
pub struct ColorRule {
    pub values: ColorValues,
    original_values: ColorValues,
    original: Item,
}
impl ColorRule {
    fn parse(item: Item) -> Self {
        let values = ColorValues {
            regex_mode: item.get("regex").is_some(),
            regex: string(item.get("regex"), ""),
            nicknames: display_list(&strings(item.get("nicknames"))),
            ignore_case: item
                .get("case_insensitive")
                .and_then(Item::as_bool)
                .unwrap_or(true),
            color: string(item.get("color"), "#ffffff"),
        };
        Self {
            original_values: values.clone(),
            values,
            original: item,
        }
    }
    fn render(&self) -> Result<Item, String> {
        if self.values == self.original_values {
            return Ok(self.original.clone());
        }
        if self.values.regex_mode {
            fancy_regex::Regex::new(&self.values.regex)
                .map_err(|error| error.to_string())?;
        } else if parse_list(&self.values.nicknames)?.is_empty() {
            return Err(
                "Enter at least one nickname for this color rule.".into()
            );
        }
        if !valid_color(&self.values.color) {
            return Err("Enter a color as #RRGGBB or #RRGGBBAA.".into());
        }
        let mut item = self.original.clone();
        let table = item
            .as_table_like_mut()
            .ok_or("Nickname color rule must be a table")?;
        let next = &self.values;
        let previous = &self.original_values;
        if next.regex_mode != previous.regex_mode {
            for key in ["regex", "nicknames", "case_insensitive"] {
                table.remove(key);
            }
        }
        if next.regex_mode {
            if next.regex != previous.regex
                || next.regex_mode != previous.regex_mode
            {
                set(table, "regex", value(&next.regex));
            }
        } else {
            if next.nicknames != previous.nicknames
                || next.regex_mode != previous.regex_mode
            {
                set(
                    table,
                    "nicknames",
                    list_item(parse_list(&next.nicknames)?),
                );
            }
            if next.ignore_case != previous.ignore_case
                || next.regex_mode != previous.regex_mode
            {
                set(table, "case_insensitive", value(next.ignore_case));
            }
        }
        if next.color != previous.color {
            set(table, "color", value(&next.color));
        }
        Ok(item)
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeListMode {
    Keep,
    Random,
    Adaptive,
}
#[derive(Debug, Clone)]
pub struct ThemeLists {
    pub mode: ThemeListMode,
    pub random: Vec<String>,
    pub light: Vec<String>,
    pub dark: Vec<String>,
    original: (ThemeListMode, Vec<String>, Vec<String>, Vec<String>),
}
impl ThemeLists {
    fn parse(document: &DocumentMut) -> Self {
        fn names(item: Option<&Item>) -> Vec<String> {
            if let Some(name) = item.and_then(Item::as_str) {
                vec![name.into()]
            } else {
                strings(item)
            }
        }
        let theme = document.get("theme");
        let mode = if theme.is_some_and(Item::is_array) {
            ThemeListMode::Random
        } else if theme.is_some_and(Item::is_table_like)
            && ["light", "dark"].iter().any(|key| {
                theme
                    .and_then(|item| item.get(key))
                    .is_some_and(Item::is_array)
            })
        {
            ThemeListMode::Adaptive
        } else {
            ThemeListMode::Keep
        };
        let random = names(theme);
        let light = names(theme.and_then(|item| item.get("light")));
        let dark = names(theme.and_then(|item| item.get("dark")));
        Self {
            mode,
            original: (mode, random.clone(), light.clone(), dark.clone()),
            random,
            light,
            dark,
        }
    }
    pub fn is_dirty(&self) -> bool {
        (self.mode, &self.random, &self.light, &self.dark)
            != (
                self.original.0,
                &self.original.1,
                &self.original.2,
                &self.original.3,
            )
    }
    fn render_into(&self, document: &mut DocumentMut) -> Result<(), String> {
        if !self.is_dirty() || self.mode == ThemeListMode::Keep {
            return Ok(());
        }
        let check = |list: &[String]| {
            if list.is_empty() || list.iter().any(|name| name.trim().is_empty())
            {
                Err("Choose at least one theme in each active theme list."
                    .to_owned())
            } else {
                Ok(())
            }
        };
        match self.mode {
            ThemeListMode::Random => {
                check(&self.random)?;
                let next = decorated_list(&self.random, document.get("theme"));
                write_at(document, "theme", next)
            }
            ThemeListMode::Adaptive => {
                check(&self.light)?;
                check(&self.dark)?;
                let mut item = document
                    .get("theme")
                    .filter(|item| item.is_table_like())
                    .cloned()
                    .unwrap_or_else(|| {
                        Item::Value(Value::InlineTable(InlineTable::new()))
                    });
                let table = item.as_table_like_mut().unwrap();
                // Update only the edited side, keeping scalar slots and comments.
                if self.mode != self.original.0 || self.light != self.original.2
                {
                    let next = decorated_list(&self.light, table.get("light"));
                    set(table, "light", next);
                }
                if self.mode != self.original.0 || self.dark != self.original.3
                {
                    let next = decorated_list(&self.dark, table.get("dark"));
                    set(table, "dark", next);
                }
                write_at(document, "theme", item)
            }
            ThemeListMode::Keep => Ok(()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct BacklogLabel {
    pub show: bool,
    pub text: String,
    original: (bool, String),
}
#[derive(Debug, Clone)]
pub struct DisplayEditors {
    pub lists: Vec<StringList>,
    pub filters: Vec<Filter>,
    pub opacities: Vec<Opacity>,
    pub repeats: Vec<Repeats>,
    pub optional_text: Vec<OptionalText>,
    pub colors: Vec<ColorRule>,
    pub themes: ThemeLists,
    pub backlog: BacklogLabel,
    original_colors: Vec<ColorValues>,
}
const SERVER_EVENTS: &[(&str, &str)] = &[
    ("default", "Activity: Default"),
    ("join_topic", "Activity: Join topic"),
    ("join", "Activity: Join"),
    ("part", "Activity: Part"),
    ("quit", "Activity: Quit"),
    ("change_host", "Activity: Change host"),
    ("change_mode", "Activity: Change mode"),
    ("change_nick", "Activity: Change nick"),
    ("monitored_online", "Activity: Monitored online"),
    ("monitored_offline", "Activity: Monitored offline"),
    ("standard_reply_fail", "Activity: Standard reply fail"),
    ("standard_reply_warn", "Activity: Standard reply warn"),
    ("standard_reply_note", "Activity: Standard reply note"),
    ("wallops", "Activity: Wallops"),
    ("kick", "Activity: Kick"),
    ("change_topic", "Activity: Change topic"),
    ("request_topic", "Activity: Request topic"),
    ("away", "Activity: Away"),
    ("invite", "Activity: Invite"),
];
impl DisplayEditors {
    pub fn parse(document: &DocumentMut) -> Result<Self, String> {
        let lists = vec![
            StringList::new(
                document,
                Some(Area::Chat),
                "Names and users",
                "Names in messages",
                "display.nickname",
                &["display-name"],
                &["display-name", "pronouns", "color"],
                None,
                None,
            ),
            StringList::new(
                document,
                Some(Area::Chat),
                "Names and users",
                "Names in the user list",
                "display.nicklist_nickname",
                &["display-name"],
                &["display-name", "pronouns", "color"],
                None,
                None,
            ),
            StringList::new(
                document,
                Some(Area::Chat),
                "Composer",
                "Completion suffixes (start / elsewhere)",
                "buffer.text_input.autocomplete.completion_suffixes",
                &[": ", " "],
                &[],
                Some(2),
                None,
            ),
            StringList::new(
                document,
                Some(Area::Chat),
                "Names and users",
                "Nickname palette",
                "buffer.nickname.color",
                &[],
                &[],
                None,
                Some("palette"),
            ),
            StringList::new(
                document,
                Some(Area::Chat),
                "Condensed activity",
                "Condensed message kinds",
                "buffer.server_messages.condense.messages",
                &["change-host", "change-nick", "join", "part", "quit"],
                &["join", "part", "quit", "change-host", "change-nick", "kick"],
                None,
                None,
            ),
            StringList::new(
                document,
                Some(Area::Sidebar),
                "Layout",
                "Internal tools in display order",
                "sidebar.internal_buffers.buffers",
                &[],
                &[
                    "config-editor",
                    "file-transfers",
                    "channel-discovery",
                    "channel-monitor",
                    "highlights",
                    "logs",
                    "search",
                ],
                None,
                None,
            ),
            StringList::new(
                document,
                Some(Area::Sidebar),
                "Layout",
                "Muted internal tools",
                "sidebar.internal_buffers.mute",
                &[],
                &["highlights", "logs"],
                None,
                None,
            ),
        ];
        let mut filters = vec![];
        for (event, group) in SERVER_EVENTS {
            for side in ["include", "exclude"] {
                filters.push(Filter::parse(
                    document,
                    Area::Chat,
                    group,
                    format!("{} {side}", event.replace('_', " ")),
                    format!("buffer.server_messages.{event}.{side}"),
                )?);
            }
        }
        for (indicator, group) in [
            ("unread_indicator", "Unread indicators"),
            ("highlight_indicator", "Highlights and typing"),
        ] {
            for side in ["include", "exclude"] {
                filters.push(Filter::parse(
                    document,
                    Area::Sidebar,
                    group,
                    format!("{} {side}", indicator.replace('_', " ")),
                    format!("sidebar.{indicator}.{side}"),
                )?);
            }
        }
        for (area, group, base) in [
            (Area::General, "Channel Monitor", "channel_monitor"),
            (Area::Previews, "Link cards", "preview.card"),
            (Area::Previews, "Images", "preview.image"),
        ] {
            for side in ["include", "exclude"] {
                filters.push(Filter::parse(
                    document,
                    area,
                    group,
                    format!("{side} conditions"),
                    format!("{base}.{side}"),
                )?);
            }
        }
        let mut opacities = SERVER_EVENTS
            .iter()
            .map(|(event, group)| {
                Opacity::parse(
                    document,
                    Area::Chat,
                    group,
                    format!("buffer.server_messages.{event}.dimmed"),
                    OpacityKind::Activity,
                )
            })
            .collect::<Vec<_>>();
        opacities.push(Opacity::parse(
            document,
            Area::Chat,
            "Condensed activity",
            "buffer.server_messages.condense.dimmed".into(),
            OpacityKind::Activity,
        ));
        opacities.push(Opacity::parse(
            document,
            Area::Chat,
            "Names and users",
            "buffer.nickname.away".into(),
            OpacityKind::Away,
        ));
        opacities.push(Opacity::parse(
            document,
            Area::Chat,
            "Names and users",
            "buffer.nickname.offline".into(),
            OpacityKind::Offline,
        ));
        let colors =
            records(at(document.as_item(), "buffer.nickname.color_override"))
                .into_iter()
                .map(ColorRule::parse)
                .collect::<Vec<_>>();
        let backlog = at(document.as_item(), "buffer.backlog_separator.text");
        let show = backlog.and_then(Item::as_bool) != Some(false);
        let text = string(backlog, "backlog");
        Ok(Self {
            lists,
            filters,
            opacities,
            optional_text: vec![
                OptionalText::parse(
                    document,
                    None,
                    "Font",
                    "font.code",
                    "Code font",
                    "Leave empty to use the main font. Requires restart.",
                ),
                OptionalText::parse(
                    document,
                    Some(Area::Chat),
                    "Messages",
                    "buffer.timestamp.copy_format",
                    "Copied timestamp format",
                    "Leave empty for the standard IRCv3 UTC timestamp.",
                ),
                OptionalText::parse(
                    document,
                    Some(Area::Chat),
                    "Messages",
                    "buffer.timestamp.locale",
                    "Timestamp locale",
                    "BCP 47 language tag, such as en-US. Empty uses the system locale.",
                ),
            ],
            repeats: vec![
                Repeats::parse(
                    document,
                    "buffer.timestamp.hide_consecutive",
                    "Messages",
                    "Repeated timestamps",
                ),
                Repeats::parse(
                    document,
                    "buffer.nickname.hide_consecutive",
                    "Names and users",
                    "Repeated nicknames",
                ),
            ],
            original_colors: colors
                .iter()
                .map(|rule| rule.values.clone())
                .collect(),
            colors,
            themes: ThemeLists::parse(document),
            backlog: BacklogLabel {
                original: (show, text.clone()),
                show,
                text,
            },
        })
    }
    pub fn is_dirty(&self) -> bool {
        self.optional_text.iter().any(OptionalText::is_dirty)
            || self.repeats.iter().any(Repeats::is_dirty)
            || self.opacities.iter().any(Opacity::is_dirty)
            || self.lists.iter().any(StringList::is_dirty)
            || self.filters.iter().any(Filter::is_dirty)
            || self
                .colors
                .iter()
                .map(|rule| &rule.values)
                .ne(self.original_colors.iter())
            || self.themes.is_dirty()
            || (self.backlog.show, &self.backlog.text)
                != (self.backlog.original.0, &self.backlog.original.1)
    }
    /// Call after preference leaves have been written, before Config::validate.
    pub fn render_into(
        &self,
        document: &mut DocumentMut,
    ) -> Result<(), String> {
        for list in &self.lists {
            list.render_into(document)?;
        }
        for opacity in &self.opacities {
            opacity.render_into(document)?;
        }
        for repeat in &self.repeats {
            repeat.render_into(document)?;
        }
        for field in &self.optional_text {
            field.render_into(document)?;
        }
        for filter in &self.filters {
            filter.render_into(document)?;
        }
        if self
            .colors
            .iter()
            .map(|rule| &rule.values)
            .ne(self.original_colors.iter())
        {
            let source =
                at(document.as_item(), "buffer.nickname.color_override");
            let rendered = self
                .colors
                .iter()
                .map(ColorRule::render)
                .collect::<Result<Vec<_>, _>>()?;
            let next = render_records(source, rendered);
            write_at(document, "buffer.nickname.color_override", next)?;
        }
        if (self.backlog.show, &self.backlog.text)
            != (self.backlog.original.0, &self.backlog.original.1)
        {
            write_at(
                document,
                "buffer.backlog_separator.text",
                if self.backlog.show {
                    value(&self.backlog.text)
                } else {
                    value(false)
                },
            )?;
        }
        self.themes.render_into(document)
    }
    /// Reuse the same condition controls within a notification/highlight rule.
    pub fn conditions(source: &Item) -> Result<Self, String> {
        let mut document = DocumentMut::new();
        if let Some(table) = source.as_table_like() {
            for (key, item) in table.iter() {
                document
                    .as_table_mut()
                    .insert_formatted(table.key(key).unwrap(), item.clone());
            }
        }
        let mut editor = Self::parse(&DocumentMut::new())?;
        editor.lists.clear();
        editor.filters.clear();
        editor.opacities.clear();
        editor.repeats.clear();
        editor.optional_text.clear();
        for side in ["include", "exclude"] {
            editor.filters.push(Filter::parse(
                &document,
                Area::General,
                "Conditions",
                format!("{side} conditions"),
                side.into(),
            )?);
        }
        Ok(editor)
    }
    pub fn add_color(&mut self) {
        let mut item = InlineTable::new();
        item.insert("nicknames", Value::Array(Array::new()));
        item.insert("color", Value::from("#ffffff"));
        self.colors
            .push(ColorRule::parse(Item::Value(Value::InlineTable(item))));
    }
    pub fn add_criterion(&mut self, filter: usize) {
        if let Some(filter) = self.filters.get_mut(filter) {
            filter.criteria.push(Criterion::parse(Item::Value(
                Value::InlineTable(InlineTable::new()),
            )));
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpacityKind {
    Activity,
    Away,
    Offline,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpacityMode {
    Default,
    Dimmed,
    Solid,
    Hidden,
    Custom,
}
#[derive(Debug, Clone)]
pub struct Opacity {
    pub area: Area,
    pub group: &'static str,
    pub path: String,
    pub kind: OpacityKind,
    pub mode: OpacityMode,
    pub alpha: String,
    pub offline_color: String,
    original: (OpacityMode, String, String),
}
impl Opacity {
    fn parse(
        document: &DocumentMut,
        area: Area,
        group: &'static str,
        path: String,
        kind: OpacityKind,
    ) -> Self {
        let item = at(document.as_item(), &path);
        let mut alpha = String::new();
        let mut offline_color = "theme".to_owned();
        let mode = match kind {
            OpacityKind::Activity => match item {
                None => OpacityMode::Default,
                Some(item) if item.as_bool() == Some(true) => {
                    OpacityMode::Dimmed
                }
                Some(item) if item.as_bool() == Some(false) => {
                    OpacityMode::Solid
                }
                Some(item) => {
                    alpha = item
                        .as_float()
                        .map(|number| number.to_string())
                        .unwrap_or_default();
                    OpacityMode::Custom
                }
            },
            OpacityKind::Away => match item.and_then(Item::as_str) {
                Some("solid" | "none") => OpacityMode::Solid,
                Some("dimmed") | None
                    if item.is_none_or(toml_edit::Item::is_str) =>
                {
                    OpacityMode::Dimmed
                }
                _ => {
                    alpha = item
                        .and_then(|item| item.get("dimmed"))
                        .and_then(Item::as_float)
                        .map(|number| number.to_string())
                        .unwrap_or_default();
                    OpacityMode::Custom
                }
            },
            OpacityKind::Offline => match item.and_then(Item::as_str) {
                Some("none") => OpacityMode::Hidden,
                Some("solid") => OpacityMode::Solid,
                Some("dimmed") => {
                    offline_color = "nickname".into();
                    OpacityMode::Dimmed
                }
                _ => {
                    if let Some(item) = item {
                        offline_color = string(
                            item.get("color"),
                            if item.get("dimmed").is_some() {
                                "nickname"
                            } else {
                                "theme"
                            },
                        );
                        let alpha_item = item.get("dimmed").or_else(|| {
                            item.get("alpha")
                                .and_then(|alpha| alpha.get("dimmed"))
                        });
                        alpha = alpha_item
                            .and_then(Item::as_float)
                            .map(|number| number.to_string())
                            .unwrap_or_default();
                        if item.get("alpha").and_then(Item::as_str).is_some_and(
                            |value| matches!(value, "solid" | "none"),
                        ) {
                            OpacityMode::Solid
                        } else if alpha.is_empty() {
                            OpacityMode::Dimmed
                        } else {
                            OpacityMode::Custom
                        }
                    } else {
                        OpacityMode::Dimmed
                    }
                }
            },
        };
        Self {
            area,
            group,
            path,
            kind,
            mode,
            original: (mode, alpha.clone(), offline_color.clone()),
            alpha,
            offline_color,
        }
    }
    pub fn is_dirty(&self) -> bool {
        (self.mode, &self.alpha, &self.offline_color)
            != (self.original.0, &self.original.1, &self.original.2)
    }
    fn render_into(&self, document: &mut DocumentMut) -> Result<(), String> {
        if !self.is_dirty() {
            return Ok(());
        }
        let number = if self.mode == OpacityMode::Custom {
            Some(
                self.alpha
                    .parse::<f64>()
                    .ok()
                    .filter(|alpha| {
                        alpha.is_finite() && (0.0..=1.0).contains(alpha)
                    })
                    .ok_or("Opacity must be a number from 0 to 1.")?,
            )
        } else {
            None
        };
        let item = match self.kind {
            OpacityKind::Activity => match self.mode {
                OpacityMode::Default => { remove_at(document, &self.path)?; return Ok(()); },
                OpacityMode::Dimmed => value(true), OpacityMode::Solid => value(false), OpacityMode::Custom => value(number.unwrap()), OpacityMode::Hidden => return Err("Activity opacity cannot be hidden; use the Show message setting.".into()),
            },
            OpacityKind::Away => match self.mode {
                OpacityMode::Dimmed | OpacityMode::Default => value("dimmed"), OpacityMode::Solid => value("solid"), OpacityMode::Custom => { let mut table = InlineTable::new(); table.insert("dimmed", Value::from(number.unwrap())); value(table) }, OpacityMode::Hidden => return Err("Away appearance cannot be hidden.".into()),
            },
            OpacityKind::Offline => {
                if self.mode == OpacityMode::Hidden { value("none") } else {
                    if !matches!(self.offline_color.as_str(), "theme" | "nickname") { return Err("Choose theme or nickname for the offline color.".into()); }
                    let mut table = InlineTable::new();
                    table.insert("color", Value::from(self.offline_color.clone()));
                    if let Some(number) = number { let mut alpha = InlineTable::new(); alpha.insert("dimmed", Value::from(number)); table.insert("alpha", Value::InlineTable(alpha)); }
                    else { table.insert("alpha", Value::from(if self.mode == OpacityMode::Solid { "solid" } else { "dimmed" })); }
                    value(table)
                }
            },
        };
        write_at(document, &self.path, item)
    }
}
fn remove_at(document: &mut DocumentMut, path: &str) -> Result<(), String> {
    let mut keys = path.split('.').collect::<Vec<_>>();
    let last = keys.pop().ok_or("Missing setting name")?;
    let mut parent: &mut dyn TableLike = document.as_table_mut();
    for key in keys {
        parent = child_table(parent, key)?;
    }
    parent.remove(last);
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    List(usize),
    Filter(usize),
    Colors,
    Themes,
    Opacity(usize),
    Repeat(usize),
    OptionalText(usize),
    Backlog,
}
#[derive(Debug, Clone)]
pub struct ValidationError {
    pub target: Target,
    pub message: String,
}
impl Target {
    pub fn input_id(self) -> String {
        if self == Self::Themes {
            "display-theme-mode".into()
        } else {
            format!("settings-display-{self:?}")
        }
    }
}
fn valid_color(value: &str) -> bool {
    matches!(value.len(), 7 | 9)
        && value.starts_with('#')
        && value.as_bytes()[1..].iter().all(u8::is_ascii_hexdigit)
}

impl DisplayEditors {
    /// Locate the editable leaf inside a structured group, including rows that
    /// are below the viewport. Validation and the UI use these same stable IDs.
    pub fn focus_id(&self, target: Target) -> String {
        match target {
            Target::List(index) => {
                if let Some(list) = self.lists.get(index) {
                    let bad = list.entries.iter().position(|entry| {
                        if list.wrapper == Some("palette") {
                            !valid_color(entry)
                        } else {
                            !list.options.is_empty()
                                && !list.options.contains(&entry.as_str())
                                && !list.original.contains(entry)
                        }
                    });
                    if let Some(row) = bad.filter(|row| *row > 0) {
                        return format!("settings-display-list-{index}-{row}");
                    }
                }
            }
            Target::Filter(index) => {
                if let Some(filter) = self.filters.get(index) {
                    for (scope, value) in filter.scopes.iter().enumerate() {
                        if parse_list(value).is_err() {
                            return if scope == 0 {
                                target.input_id()
                            } else {
                                format!("display-filter-{index}-{scope}")
                            };
                        }
                    }
                    for (row, criterion) in filter.criteria.iter().enumerate() {
                        for (field, value) in [
                            (0, &criterion.values.users),
                            (2, &criterion.values.channels),
                        ] {
                            if parse_list(value).is_err() {
                                return format!(
                                    "display-criterion-{index}-{row}-{field}"
                                );
                            }
                        }
                    }
                }
            }
            Target::Colors => {
                for (index, rule) in self.colors.iter().enumerate() {
                    let values = &rule.values;
                    if values.regex_mode
                        && fancy_regex::Regex::new(&values.regex).is_err()
                    {
                        return format!("display-color-regex-{index}");
                    }
                    if !values.regex_mode
                        && parse_list(&values.nicknames)
                            .map_or(true, |names| names.is_empty())
                    {
                        return format!("display-color-nicks-{index}");
                    }
                    if !valid_color(&values.color) {
                        return format!("display-color-value-{index}");
                    }
                }
            }
            Target::Opacity(index)
                if self.opacities.get(index).is_some_and(|opacity| {
                    opacity.mode == OpacityMode::Custom
                }) =>
            {
                return format!("display-opacity-alpha-{index}");
            }
            Target::Repeat(index)
                if self
                    .repeats
                    .get(index)
                    .is_some_and(|repeat| repeat.mode == RepeatMode::Smart) =>
            {
                return format!("display-repeat-seconds-{index}");
            }
            _ => {}
        }
        target.input_id()
    }
}

impl DisplayEditors {
    pub fn validation_error(
        &self,
        source: &DocumentMut,
    ) -> Option<ValidationError> {
        fn validate(document: &DocumentMut) -> Result<(), String> {
            crate::Config::validate(&document.to_string())
                .map_err(|error| error.to_string())
        }
        for (index, list) in self
            .lists
            .iter()
            .enumerate()
            .filter(|(_, list)| list.is_dirty())
        {
            let mut document = source.clone();
            if let Err(message) = list
                .render_into(&mut document)
                .and_then(|()| validate(&document))
            {
                return Some(ValidationError {
                    target: Target::List(index),
                    message,
                });
            }
        }
        for (index, filter) in self
            .filters
            .iter()
            .enumerate()
            .filter(|(_, filter)| filter.is_dirty())
        {
            let mut document = source.clone();
            if let Err(message) = filter
                .render_into(&mut document)
                .and_then(|()| validate(&document))
            {
                return Some(ValidationError {
                    target: Target::Filter(index),
                    message,
                });
            }
        }
        for (index, field) in self
            .optional_text
            .iter()
            .enumerate()
            .filter(|(_, field)| field.is_dirty())
        {
            let mut document = source.clone();
            if let Err(message) = field
                .render_into(&mut document)
                .and_then(|()| validate(&document))
            {
                return Some(ValidationError {
                    target: Target::OptionalText(index),
                    message,
                });
            }
        }
        for (index, repeat) in self
            .repeats
            .iter()
            .enumerate()
            .filter(|(_, repeat)| repeat.is_dirty())
        {
            let mut document = source.clone();
            if let Err(message) = repeat
                .render_into(&mut document)
                .and_then(|()| validate(&document))
            {
                return Some(ValidationError {
                    target: Target::Repeat(index),
                    message,
                });
            }
        }
        for (index, opacity) in self
            .opacities
            .iter()
            .enumerate()
            .filter(|(_, opacity)| opacity.is_dirty())
        {
            let mut document = source.clone();
            if let Err(message) = opacity
                .render_into(&mut document)
                .and_then(|()| validate(&document))
            {
                return Some(ValidationError {
                    target: Target::Opacity(index),
                    message,
                });
            }
        }
        if self
            .colors
            .iter()
            .map(|rule| &rule.values)
            .ne(self.original_colors.iter())
        {
            let mut document = source.clone();
            let check = self
                .colors
                .iter()
                .map(ColorRule::render)
                .collect::<Result<Vec<_>, _>>()
                .and_then(|records| {
                    let item = render_records(
                        at(
                            document.as_item(),
                            "buffer.nickname.color_override",
                        ),
                        records,
                    );
                    write_at(
                        &mut document,
                        "buffer.nickname.color_override",
                        item,
                    )
                })
                .and_then(|()| validate(&document));
            if let Err(message) = check {
                return Some(ValidationError {
                    target: Target::Colors,
                    message,
                });
            }
        }
        let mut document = source.clone();
        if let Err(message) = self
            .themes
            .render_into(&mut document)
            .and_then(|()| validate(&document))
        {
            return Some(ValidationError {
                target: Target::Themes,
                message,
            });
        }
        None
    }
    pub fn location(&self, target: Target) -> (Option<Area>, &'static str) {
        match target {
            Target::List(index) => self
                .lists
                .get(index)
                .map_or((Some(Area::Chat), "Composer"), |list| {
                    (list.area, list.group)
                }),
            Target::Filter(index) => self
                .filters
                .get(index)
                .map_or((Some(Area::Chat), "Activity"), |filter| {
                    (Some(filter.area), filter.group)
                }),
            Target::OptionalText(index) => self
                .optional_text
                .get(index)
                .map_or((None, "Font"), |field| (field.area, field.group)),
            Target::Repeat(index) => self
                .repeats
                .get(index)
                .map_or((Some(Area::Chat), "Messages"), |repeat| {
                    (Some(repeat.area), repeat.group)
                }),
            Target::Opacity(index) => self
                .opacities
                .get(index)
                .map_or((Some(Area::Chat), "Activity"), |opacity| {
                    (Some(opacity.area), opacity.group)
                }),
            Target::Colors => (Some(Area::Chat), "Names and users"),
            Target::Themes => (None, "Theme"),
            Target::Backlog => (Some(Area::Chat), "Messages"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepeatMode {
    Never,
    Always,
    Smart,
}
#[derive(Debug, Clone)]
pub struct Repeats {
    pub area: Area,
    pub group: &'static str,
    pub label: &'static str,
    pub path: &'static str,
    pub mode: RepeatMode,
    pub seconds: String,
    pub show_after_previews: bool,
    original: (RepeatMode, String, bool),
}
impl Repeats {
    fn parse(
        document: &DocumentMut,
        path: &'static str,
        group: &'static str,
        label: &'static str,
    ) -> Self {
        let item = at(document.as_item(), path);
        let enabled = item.and_then(|item| item.get("enabled")).or(item);
        let smart = enabled
            .and_then(|item| item.get("smart"))
            .and_then(Item::as_integer);
        let mode = if smart.is_some() {
            RepeatMode::Smart
        } else if enabled.and_then(Item::as_bool) == Some(true) {
            RepeatMode::Always
        } else {
            RepeatMode::Never
        };
        let seconds =
            smart.map_or_else(|| "120".into(), |seconds| seconds.to_string());
        let show_after_previews = item
            .and_then(|item| item.get("show_after_previews"))
            .and_then(Item::as_bool)
            .unwrap_or(false);
        Self {
            area: Area::Chat,
            group,
            label,
            path,
            mode,
            original: (mode, seconds.clone(), show_after_previews),
            seconds,
            show_after_previews,
        }
    }
    pub fn is_dirty(&self) -> bool {
        (self.mode, &self.seconds, self.show_after_previews)
            != (self.original.0, &self.original.1, self.original.2)
    }
    fn render_into(&self, document: &mut DocumentMut) -> Result<(), String> {
        if !self.is_dirty() {
            return Ok(());
        }
        let enabled = match self.mode {
            RepeatMode::Never => value(false),
            RepeatMode::Always => value(true),
            RepeatMode::Smart => {
                let seconds = self.seconds.parse::<i64>().ok().filter(|seconds| *seconds >= 0).ok_or("Repeat window must be a non-negative whole number of seconds.")?;
                let mut table = InlineTable::new();
                table.insert("smart", Value::from(seconds));
                value(table)
            }
        };
        let mut item = at(document.as_item(), self.path)
            .filter(|item| item.get("enabled").is_some())
            .cloned()
            .unwrap_or_else(|| {
                Item::Value(Value::InlineTable(InlineTable::new()))
            });
        let table = item
            .as_table_like_mut()
            .ok_or("Repeated-message settings must be a table")?;
        if self.mode != self.original.0
            || self.seconds != self.original.1
            || !table.contains_key("enabled")
        {
            set(table, "enabled", enabled);
        }
        if self.show_after_previews != self.original.2 {
            set(
                table,
                "show_after_previews",
                value(self.show_after_previews),
            );
        }
        write_at(document, self.path, item)
    }
}
/// The structured repeat controls replace these two incumbent simple pickers.
/// Filter them when extending preferences::schema, rather than showing duplicates.
pub fn replaces(path: &str) -> bool {
    matches!(
        path,
        "buffer.nickname.hide_consecutive"
            | "buffer.timestamp.hide_consecutive"
    )
}

#[derive(Debug, Clone)]
pub struct OptionalText {
    pub area: Option<Area>,
    pub group: &'static str,
    pub path: &'static str,
    pub label: &'static str,
    pub help: &'static str,
    pub value: String,
    original: String,
}
impl OptionalText {
    fn parse(
        document: &DocumentMut,
        area: Option<Area>,
        group: &'static str,
        path: &'static str,
        label: &'static str,
        help: &'static str,
    ) -> Self {
        let value = string(at(document.as_item(), path), "");
        Self {
            area,
            group,
            path,
            label,
            help,
            original: value.clone(),
            value,
        }
    }
    pub fn is_dirty(&self) -> bool {
        self.value != self.original
    }
    fn render_into(&self, document: &mut DocumentMut) -> Result<(), String> {
        if !self.is_dirty() {
            return Ok(());
        }
        if self.value.is_empty() {
            remove_at(document, self.path)
        } else {
            write_at(document, self.path, value(&self.value))
        }
    }
}

fn decorated_list(entries: &[String], source: Option<&Item>) -> Item {
    let mut array = Array::new();
    let mut used = vec![];
    for entry in entries {
        let old = source.and_then(Item::as_array).and_then(|array| {
            array
                .iter()
                .enumerate()
                .find(|(index, value)| {
                    !used.contains(index)
                        && value.as_str() == Some(entry.as_str())
                })
                .map(|(index, value)| {
                    used.push(index);
                    value.clone()
                })
        });
        array.push_formatted(old.unwrap_or_else(|| Value::from(entry.clone())));
    }
    value(array)
}

impl DisplayEditors {
    /// Invoke when the incumbent nickname-color picker explicitly chooses
    /// solid/unique; the later user action cancels a staged palette edit.
    pub fn reset_palette(&mut self) {
        if let Some(list) = self
            .lists
            .iter_mut()
            .find(|list| list.path == "buffer.nickname.color")
        {
            list.entries.clone_from(&list.original);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn document(source: &str) -> DocumentMut {
        source.parse().unwrap()
    }
    fn render(source: &str, edit: impl FnOnce(&mut DisplayEditors)) -> String {
        let mut document = document(source);
        let mut editor = DisplayEditors::parse(&document).unwrap();
        edit(&mut editor);
        editor.render_into(&mut document).unwrap();
        crate::Config::validate(&document.to_string()).unwrap();
        document.to_string()
    }
    fn runtime_accepts_every_key(document: &DocumentMut) {
        let mut ignored = vec![];
        let _: crate::config::Configuration = serde_ignored::deserialize(
            toml::Deserializer::parse(&document.to_string()).unwrap(),
            |path| ignored.push(path.to_string()),
        )
        .unwrap();
        assert!(
            ignored.is_empty(),
            "Ignored settings: {ignored:?}\n{document}"
        );
    }
    #[test]
    fn new_schema_choices_defaults_and_paths_are_runtime_supported() {
        let mut paths = std::collections::BTreeSet::new();
        for spec in schema().into_iter().chain(font_schema(Area::Chat)) {
            assert!(paths.insert(spec.path), "Duplicate {}", spec.path);
            let choices = match spec.kind {
                Kind::Choice(choices) => choices
                    .iter()
                    .map(|choice| choice.value)
                    .collect::<Vec<_>>(),
                _ => vec![spec.default],
            };
            for literal in choices.into_iter().filter(|value| !value.is_empty())
            {
                let mut document = document("");
                write_at(
                    &mut document,
                    spec.path,
                    Item::Value(literal.parse::<Value>().unwrap()),
                )
                .unwrap();
                crate::Config::validate(&document.to_string()).unwrap_or_else(
                    |error| panic!("{} = {literal}: {error}", spec.path),
                );
                runtime_accepts_every_key(&document);
            }
        }
    }
    #[test]
    fn schema_does_not_duplicate_incumbent_controls() {
        let incumbent = super::super::preferences::schema();
        for spec in schema().into_iter().chain(font_schema(Area::Chat)) {
            assert!(
                incumbent
                    .iter()
                    .filter(|field| field.path == spec.path)
                    .count()
                    <= 1,
                "Duplicate control {}",
                spec.path
            );
        }
    }
    #[test]
    fn unchanged_advanced_values_and_comments_round_trip_exactly() {
        let source = "# Keep my palette\ntheme = { light = ['one', 'two'], dark = 'dark' } # theme\n[buffer.nickname]\ncolor = { palette = ['#123456', '#abcdef'] }\naway = { dimmed = 0.42 }\noffline = { color = 'theme', alpha = { dimmed = 0.31 } }\ncolor_override = [{ regex = '^Admin', color = '#112233' }]\nhide_consecutive = { enabled = { smart = 120 }, show_after_previews = true }\n[buffer.server_messages.join]\nexclude = { channels = ['#quiet'], criteria = [{ user = 'bot', server = 'test' }] } # conditions\n";
        let document = document(source);
        let editor = DisplayEditors::parse(&document).unwrap();
        assert!(!editor.is_dirty());
        let mut result = document.clone();
        editor.render_into(&mut result).unwrap();
        assert_eq!(result.to_string(), source);
    }
    #[test]
    fn legacy_conditions_convert_only_when_edited_and_keep_suffix_comments() {
        let source = "[buffer.server_messages.join]\nexclude = ['#quiet', 'bot'] # scopes\ninclude = '*' # untouched\n";
        let result = render(source, |editor| {
            let filter = editor
                .filters
                .iter_mut()
                .find(|filter| {
                    filter.path == "buffer.server_messages.join.exclude"
                })
                .unwrap();
            assert_eq!(filter.scopes[0], "bot");
            assert_eq!(filter.scopes[1], "#quiet");
            filter.scopes[2] = "test".into();
        });
        assert!(result.contains("# scopes"));
        assert!(result.contains("include = '*' # untouched"));
        let parsed: toml::Value = toml::from_str(&result).unwrap();
        assert_eq!(
            parsed["buffer"]["server_messages"]["join"]["exclude"]["channels"]
                [0]
            .as_str(),
            Some("#quiet")
        );
    }
    #[test]
    fn compound_condition_edits_keep_siblings_and_quoted_entries() {
        let source = "[buffer.server_messages.part]\nexclude = { users = ['bot'], criteria = [{ user = 'old', users = ['extra'], channel = '#quiet', server = 'test' }] } # retain\n";
        let result = render(source, |editor| {
            let filter = editor
                .filters
                .iter_mut()
                .find(|filter| {
                    filter.path == "buffer.server_messages.part.exclude"
                })
                .unwrap();
            filter.criteria[0].values.user = "new".into();
            filter.scopes[1] = "\"#semi;colon\"; #two".into();
        });
        assert!(result.contains("# retain"));
        assert!(result.contains("server = 'test'"));
        assert!(result.contains("users = ['extra']"));
        let parsed: toml::Value = toml::from_str(&result).unwrap();
        assert_eq!(
            parsed["buffer"]["server_messages"]["part"]["exclude"]["criteria"]
                [0]["user"]
                .as_str(),
            Some("new")
        );
    }
    #[test]
    fn nickname_color_order_and_table_comments_survive_edits() {
        let source = "[[buffer.nickname.color_override]]\n# First rule\nnicknames = ['admin']\ncolor = '#123456' # first color\n[[buffer.nickname.color_override]]\nregex = '^Guest'\ncolor = '#abcdef' # second color\n";
        let result = render(source, |editor| {
            editor.colors.swap(0, 1);
            editor.colors[1].values.color = "#654321".into();
        });
        assert!(result.find("^Guest").unwrap() < result.find("admin").unwrap());
        assert!(result.contains("# First rule"));
        assert!(result.contains("# first color"));
        assert!(result.contains("# second color"));
    }
    #[test]
    fn nickname_color_mode_switch_removes_competing_match_fields() {
        let result = render(
            "buffer.nickname.color_override = [{ nicknames = ['old'], case_insensitive = false, color = '#123456' }]\n",
            |editor| {
                editor.colors[0].values.regex_mode = true;
                editor.colors[0].values.regex = "^New".into();
            },
        );
        let parsed: toml::Value = toml::from_str(&result).unwrap();
        let rule = &parsed["buffer"]["nickname"]["color_override"][0];
        assert!(rule.get("nicknames").is_none());
        assert!(rule.get("case_insensitive").is_none());
        assert_eq!(rule["color"].as_str(), Some("#123456"));
    }
    #[test]
    fn alpha_modes_cover_activity_away_and_offline_formats() {
        for path in [
            "buffer.server_messages.join.dimmed",
            "buffer.server_messages.condense.dimmed",
            "buffer.nickname.away",
            "buffer.nickname.offline",
        ] {
            let result = render("", |editor| {
                let opacity = editor
                    .opacities
                    .iter_mut()
                    .find(|opacity| opacity.path == path)
                    .unwrap();
                opacity.mode = OpacityMode::Custom;
                opacity.alpha = "0.37".into();
            });
            runtime_accepts_every_key(&document(&result));
        }
    }
    #[test]
    fn repeat_window_preserves_preview_setting_and_shorthand_semantics() {
        let result = render(
            "buffer.timestamp.hide_consecutive = { smart = 120 } # shorthand\n",
            |editor| {
                editor.repeats[0].show_after_previews = true;
            },
        );
        assert!(result.contains("# shorthand"));
        let parsed: toml::Value = toml::from_str(&result).unwrap();
        assert_eq!(parsed["buffer"]["timestamp"]["hide_consecutive"]["enabled"]["smart"].as_integer(), Some(120));
        assert_eq!(parsed["buffer"]["timestamp"]["hide_consecutive"]["show_after_previews"].as_bool(), Some(true));
    }
    #[test]
    fn adaptive_theme_list_edit_preserves_the_other_slot_and_comments() {
        let source =
            "theme = { light = ['one', 'two'], dark = 'dark' } # keep\n";
        let result =
            render(source, |editor| editor.themes.light.push("three".into()));
        assert!(result.contains("dark = 'dark'"));
        assert!(result.contains("# keep"));
        let parsed: toml::Value = toml::from_str(&result).unwrap();
        assert_eq!(parsed["theme"]["light"][2].as_str(), Some("three"));
    }
    #[test]
    fn completion_suffix_whitespace_and_literal_semicolons_are_preserved() {
        let result = render("", |editor| {
            let list = editor
                .lists
                .iter_mut()
                .find(|list| list.path.ends_with("completion_suffixes"))
                .unwrap();
            list.entries = vec!["; ".into(), " ".into()];
        });
        let parsed: toml::Value = toml::from_str(&result).unwrap();
        assert_eq!(parsed["buffer"]["text_input"]["autocomplete"]["completion_suffixes"][0].as_str(), Some("; "));
        assert_eq!(parsed["buffer"]["text_input"]["autocomplete"]["completion_suffixes"][1].as_str(), Some(" "));
    }
    #[test]
    fn optional_font_and_timestamp_text_clear_to_runtime_defaults() {
        let result = render(
            "font.code = 'Other'\nbuffer.timestamp.copy_format = '%R'\nbuffer.timestamp.locale = 'en-US'\n",
            |editor| {
                for field in &mut editor.optional_text {
                    field.value.clear();
                }
            },
        );
        let document = document(&result);
        assert!(at(document.as_item(), "font.code").is_none());
        assert!(
            at(document.as_item(), "buffer.timestamp.copy_format").is_none()
        );
        assert!(at(document.as_item(), "buffer.timestamp.locale").is_none());
    }
    #[test]
    fn invalid_alpha_regex_color_conditions_and_empty_theme_lists_are_located()
    {
        for case in 0..5 {
            let source = document("");
            let mut editor = DisplayEditors::parse(&source).unwrap();
            match case {
                0 => {
                    editor.opacities[0].mode = OpacityMode::Custom;
                    editor.opacities[0].alpha = "NaN".into();
                }
                1 => {
                    editor.add_color();
                    editor.colors[0].values.regex_mode = true;
                    editor.colors[0].values.regex = "[".into();
                }
                2 => {
                    editor.add_color();
                    editor.colors[0].values.nicknames = "name".into();
                    editor.colors[0].values.color = "bad".into();
                }
                3 => {
                    editor.filters[0].scopes[0] = "\"unterminated".into();
                }
                _ => editor.themes.mode = ThemeListMode::Adaptive,
            }
            assert!(
                editor.validation_error(&source).is_some(),
                "Case {case} unexpectedly valid"
            );
        }
    }
}

/// Register once, after constructing the incumbent scalar schema. Appearance
/// can supply its own area or render `font_schema` separately.
pub fn extend_schema(fields: &mut Vec<Spec>, appearance_area: Option<Area>) {
    fields.retain(|field| !replaces(field.path));
    for field in fields.iter_mut() {
        if let Some(event) = field
            .path
            .strip_prefix("buffer.server_messages.")
            .and_then(|path| path.strip_suffix(".enabled"))
            && let Some((_, group)) =
                SERVER_EVENTS.iter().find(|(name, _)| *name == event)
        {
            field.group = group;
        }
    }
    fields.extend(schema());
    if let Some(area) = appearance_area {
        fields.extend(font_schema(area));
    }
}
impl ThemeLists {
    /// Names needed by the existing Apply download/install phase. Keep-mode
    /// delegates to the incumbent Appearance selection entirely.
    pub fn selected(&self) -> Vec<&str> {
        match self.mode {
            ThemeListMode::Keep => vec![],
            ThemeListMode::Random => {
                self.random.iter().map(String::as_str).collect()
            }
            ThemeListMode::Adaptive => self
                .light
                .iter()
                .chain(&self.dark)
                .map(String::as_str)
                .collect(),
        }
    }
}

#[cfg(test)]
mod sidebar_regressions {
    use super::*;
    #[test]
    fn indicator_scope_edits_keep_legacy_mode_and_suffix_comment() {
        let mut document: DocumentMut =
            "[sidebar]\nunread_indicator = 'none' # keep mode\n"
                .parse()
                .unwrap();
        let mut editor = DisplayEditors::parse(&document).unwrap();
        let filter = editor
            .filters
            .iter_mut()
            .find(|filter| filter.path == "sidebar.unread_indicator.exclude")
            .unwrap();
        filter.scopes[1] = "#quiet".into();
        editor.render_into(&mut document).unwrap();
        crate::Config::validate(&document.to_string()).unwrap();
        assert!(document.to_string().contains("# keep mode"));
        assert_eq!(
            document["sidebar"]["unread_indicator"]["icon"].as_str(),
            Some("none")
        );
        assert_eq!(
            document["sidebar"]["unread_indicator"]["title"].as_bool(),
            Some(false)
        );
    }
    #[test]
    fn condensation_aliases_survive_adding_another_message_kind() {
        let mut document: DocumentMut =
            "buffer.server_messages.condense.messages = ['change_nick']\n"
                .parse()
                .unwrap();
        let mut editor = DisplayEditors::parse(&document).unwrap();
        let list = editor
            .lists
            .iter_mut()
            .find(|list| list.path.ends_with("condense.messages"))
            .unwrap();
        list.entries.push("kick".into());
        editor.render_into(&mut document).unwrap();
        crate::Config::validate(&document.to_string()).unwrap();
        assert_eq!(
            document["buffer"]["server_messages"]["condense"]["messages"][0]
                .as_str(),
            Some("change_nick")
        );
    }
}

#[cfg(test)]
mod validation_leaf_regressions {
    use super::*;
    #[test]
    fn validation_focuses_nested_inputs_and_rejects_unicode_colors_without_panicking()
     {
        let source = DocumentMut::new();
        let mut editor = DisplayEditors::parse(&source).unwrap();
        editor.add_criterion(0);
        editor.filters[0].criteria[0].values.channels = "\"unterminated".into();
        let error = editor.validation_error(&source).unwrap();
        assert_eq!(editor.focus_id(error.target), "display-criterion-0-0-2");
        editor = DisplayEditors::parse(&source).unwrap();
        editor.opacities[0].mode = OpacityMode::Custom;
        editor.opacities[0].alpha = "bad".into();
        let error = editor.validation_error(&source).unwrap();
        assert_eq!(editor.focus_id(error.target), "display-opacity-alpha-0");
        editor = DisplayEditors::parse(&source).unwrap();
        editor.add_color();
        editor.colors[0].values.nicknames = "qa".into();
        editor.colors[0].values.color = "#aéaaa".into();
        let error = editor.validation_error(&source).unwrap();
        assert_eq!(editor.focus_id(error.target), "display-color-value-0");
        editor = DisplayEditors::parse(&source).unwrap();
        let index = editor
            .lists
            .iter()
            .position(|list| list.wrapper == Some("palette"))
            .unwrap();
        editor.lists[index].entries = vec!["#ffffff".into(), "#aéaaa".into()];
        let error = editor.validation_error(&source).unwrap();
        assert_eq!(
            editor.focus_id(error.target),
            format!("settings-display-list-{index}-1")
        );
    }
}

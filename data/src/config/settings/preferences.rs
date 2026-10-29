//! GUI-editable global preferences. Only changed leaves are written, retaining
//! comments, custom conditions, aliases, and options outside the edited field.
use toml_edit::{DocumentMut, Item, Table, TableLike, Value, value};

use super::{child_table, set};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Area {
    General,
    Appearance,
    Commands,
    Previews,
    Transfers,
    Navigation,
    Chat,
    Sidebar,
}

#[derive(Debug, Clone, Copy)]
pub struct Choice {
    pub label: &'static str,
    pub value: &'static str,
}
#[derive(Debug, Clone, Copy)]
pub enum Kind {
    Boolean,
    Text,
    Integer { min: i64, max: i64, optional: bool },
    Float { min: f64, max: f64, optional: bool },
    Pair,
    Choice(&'static [Choice]),
}
#[derive(Debug, Clone)]
pub struct Spec {
    pub area: Area,
    pub group: &'static str,
    pub path: &'static str,
    pub label: &'static str,
    pub kind: Kind,
    pub default: &'static str,
    pub help: &'static str,
    pub aliases: &'static [&'static str],
}
#[derive(Debug, Clone)]
pub struct Field {
    pub spec: Spec,
    pub value: String,
    original: String,
    write_path: String,
}
#[derive(Debug, Clone)]
pub struct Preferences {
    pub fields: Vec<Field>,
}

const OPEN: &[Choice] = &[
    Choice {
        label: "Reuse current pane",
        value: "\"replace-pane\"",
    },
    Choice {
        label: "Open a new split",
        value: "\"new-pane\"",
    },
    Choice {
        label: "Open a new window",
        value: "\"new-window\"",
    },
];
const OVERRIDE: &[Choice] = &[
    Choice {
        label: "Use default opening behavior",
        value: "",
    },
    OPEN[0],
    OPEN[1],
    OPEN[2],
];
const CLICK: &[Choice] = &[
    OPEN[0],
    OPEN[1],
    OPEN[2],
    Choice {
        label: "Do nothing",
        value: "\"noop\"",
    },
];
const NICK: &[Choice] = &[
    OPEN[0],
    OPEN[1],
    OPEN[2],
    Choice {
        label: "Insert nickname in message",
        value: "\"insert-nickname\"",
    },
    Choice {
        label: "Do nothing",
        value: "\"noop\"",
    },
];
const NICK_OVERRIDE: &[Choice] = &[
    Choice {
        label: "Use chat nickname behavior",
        value: "",
    },
    NICK[0],
    NICK[1],
    NICK[2],
    NICK[3],
    NICK[4],
];
const REPEAT: &[Choice] = &[
    Choice {
        label: "Show every message",
        value: "false",
    },
    Choice {
        label: "Hide consecutive repeats",
        value: "true",
    },
];
const ACCESS: &[Choice] = &[
    Choice {
        label: "Highest level only",
        value: "\"highest\"",
    },
    Choice {
        label: "All levels",
        value: "\"all\"",
    },
    Choice {
        label: "Hidden",
        value: "\"none\"",
    },
];
const EVENTS: &[Choice] = &[
    Choice {
        label: "Use server-message default",
        value: "",
    },
    Choice {
        label: "Show",
        value: "true",
    },
    Choice {
        label: "Hide",
        value: "false",
    },
    Choice {
        label: "Do not store",
        value: "\"drop\"",
    },
];
const ICONS: &[Choice] = &[
    Choice {
        label: "None",
        value: "\"none\"",
    },
    Choice {
        label: "Dot",
        value: "\"dot\"",
    },
    Choice {
        label: "Hollow circle",
        value: "\"circle-empty\"",
    },
    Choice {
        label: "Circled dot",
        value: "\"dot-circled\"",
    },
    Choice {
        label: "Star",
        value: "\"star\"",
    },
    Choice {
        label: "Asterisk",
        value: "\"asterisk\"",
    },
    Choice {
        label: "Speaker",
        value: "\"speaker\"",
    },
    Choice {
        label: "Lightbulb",
        value: "\"lightbulb\"",
    },
    Choice {
        label: "Certificate",
        value: "\"certificate\"",
    },
];

pub fn schema() -> Vec<Spec> {
    use Area::*;
    macro_rules! s {
        ($area:ident,$group:literal,$path:literal,$label:literal,$kind:expr,$default:literal,$help:literal) => {
            Spec {
                area: $area,
                group: $group,
                path: $path,
                label: $label,
                kind: $kind,
                default: $default,
                help: $help,
                aliases: &[],
            }
        };
    }
    macro_rules! b {
        ($a:ident,$g:literal,$p:literal,$l:literal,$d:literal) => {
            s!($a, $g, $p, $l, Kind::Boolean, $d, "")
        };
    }
    macro_rules! n {
        ($a:ident,$g:literal,$p:literal,$l:literal,$d:literal) => {
            s!(
                $a,
                $g,
                $p,
                $l,
                Kind::Integer {
                    min: 0,
                    max: 4_294_967_295,
                    optional: false
                },
                $d,
                "Pixels"
            )
        };
    }
    macro_rules! o {
        ($a:ident,$g:literal,$p:literal,$l:literal,$max:literal) => {
            s!(
                $a,
                $g,
                $p,
                $l,
                Kind::Integer {
                    min: 1,
                    max: $max,
                    optional: true
                },
                "",
                "Leave empty for Halloy's default."
            )
        };
    }
    macro_rules! t {
        ($a:ident,$g:literal,$p:literal,$l:literal,$d:literal,$h:literal) => {
            s!($a, $g, $p, $l, Kind::Text, $d, $h)
        };
    }
    macro_rules! c {
        ($a:ident,$g:literal,$p:literal,$l:literal,$choices:expr,$d:literal) => {
            s!($a, $g, $p, $l, Kind::Choice($choices), $d, "")
        };
    }
    let mut specs = vec![
        c!(
            Navigation,
            "Opening chats",
            "actions.sidebar.buffer",
            "Default sidebar action",
            OPEN,
            "\"new-pane\""
        ),
        c!(
            Navigation,
            "Opening chats",
            "actions.sidebar.channel",
            "Channels",
            OVERRIDE,
            ""
        ),
        c!(
            Navigation,
            "Opening chats",
            "actions.sidebar.query",
            "Private conversations",
            OVERRIDE,
            ""
        ),
        c!(
            Navigation,
            "Opening chats",
            "actions.sidebar.server",
            "Servers",
            OVERRIDE,
            ""
        ),
        c!(
            Navigation,
            "Opening chats",
            "actions.sidebar.internal",
            "Tools and internal views",
            OVERRIDE,
            ""
        ),
        c!(
            Navigation,
            "Opening chats",
            "actions.sidebar.focused_buffer",
            "Clicking the already focused chat",
            &[
                Choice {
                    label: "Keep current chat",
                    value: ""
                },
                Choice {
                    label: "Close its pane",
                    value: "\"close-pane\""
                }
            ],
            ""
        ),
        c!(
            Navigation,
            "Click actions",
            "actions.buffer.click_channel_name",
            "Channel names in messages",
            CLICK,
            "\"new-pane\""
        ),
        c!(
            Navigation,
            "Click actions",
            "actions.buffer.click_highlight",
            "Channels in highlights",
            CLICK,
            "\"new-pane\""
        ),
        c!(
            Navigation,
            "Click actions",
            "actions.buffer.click_channel_discovery",
            "Channels in discovery",
            CLICK,
            "\"new-pane\""
        ),
        c!(
            Navigation,
            "Click actions",
            "actions.buffer.click_username",
            "Nicknames in messages",
            NICK,
            "\"new-pane\""
        ),
        c!(
            Navigation,
            "Click actions",
            "actions.nicklist.click_username",
            "Nicknames in the user list",
            NICK_OVERRIDE,
            ""
        ),
        c!(
            Navigation,
            "Click actions",
            "actions.buffer.join_channel",
            "Joining a channel",
            OPEN,
            "\"new-pane\""
        ),
        c!(
            Navigation,
            "Click actions",
            "actions.buffer.open_internal",
            "Opening tools and internal views",
            OPEN,
            "\"new-pane\""
        ),
        c!(
            Navigation,
            "Click actions",
            "actions.buffer.message_channel",
            "Opening a channel with /msg",
            OPEN,
            "\"new-pane\""
        ),
        c!(
            Navigation,
            "Click actions",
            "actions.buffer.message_user",
            "Opening a private conversation",
            OPEN,
            "\"new-pane\""
        ),
        c!(
            Navigation,
            "Click actions",
            "actions.notification.open_buffer",
            "Opening chats from notifications",
            OPEN,
            "\"new-pane\""
        ),
        c!(
            Navigation,
            "Click actions",
            "actions.sidebar.cycle",
            "Cycling through collapsed servers",
            &[
                Choice {
                    label: "Include their chats",
                    value: "\"into-collapsed\""
                },
                Choice {
                    label: "Skip their chats",
                    value: "\"skip-collapsed\""
                }
            ],
            "\"into-collapsed\""
        ),
        c!(
            Navigation,
            "Pane layout",
            "pane.split_axis",
            "Split direction",
            &[
                Choice {
                    label: "Automatic, shorter side",
                    value: "\"shorter\""
                },
                Choice {
                    label: "Automatic, largest pane",
                    value: "\"largest-shorter\""
                },
                Choice {
                    label: "Horizontal divider",
                    value: "\"horizontal\""
                },
                Choice {
                    label: "Vertical divider",
                    value: "\"vertical\""
                }
            ],
            "\"shorter\""
        ),
        b!(
            Navigation,
            "Pane layout",
            "pane.restore_on_launch",
            "Restore pane layout on launch",
            "true"
        ),
        n!(
            Navigation,
            "Pane layout",
            "pane.gap.inner",
            "Space between panes",
            "4"
        ),
        n!(
            Navigation,
            "Pane layout",
            "pane.gap.outer",
            "Space around the chat layout",
            "8"
        ),
        b!(
            Navigation,
            "Pane layout",
            "pane.always_show_title_bar_buttons",
            "Always show pane title-bar buttons",
            "false"
        ),
        b!(
            Navigation,
            "Scrollbars",
            "pane.scrollbar.hidden",
            "Hide chat scrollbars",
            "false"
        ),
        n!(
            Navigation,
            "Scrollbars",
            "pane.scrollbar.width",
            "Scrollbar width",
            "5"
        ),
        n!(
            Navigation,
            "Scrollbars",
            "pane.scrollbar.scroller_width",
            "Scrollbar thumb width",
            "5"
        ),
        t!(
            Chat,
            "Messages",
            "buffer.timestamp.format",
            "Timestamp format",
            "\"%R\"",
            "For example %H:%M or %H:%M:%S. An empty format hides timestamps."
        ),
        c!(
            Chat,
            "Messages",
            "buffer.timestamp.hide_consecutive",
            "Repeated timestamps",
            REPEAT,
            "false"
        ),
        b!(
            Chat,
            "Messages",
            "buffer.date_separators.show",
            "Show date separators",
            "true"
        ),
        t!(
            Chat,
            "Messages",
            "buffer.date_separators.format",
            "Date format",
            "\"%A, %B %-d\"",
            "For example %Y-%m-%d."
        ),
        n!(
            Chat,
            "Messages",
            "buffer.line_spacing",
            "Extra space between messages",
            "0"
        ),
        c!(
            Chat,
            "Messages",
            "buffer.scroll_position_on_open",
            "Scroll position when opening a chat",
            &[
                Choice {
                    label: "Oldest unread message",
                    value: "\"oldest-unread\""
                },
                Choice {
                    label: "Newest message",
                    value: "\"newest\""
                }
            ],
            "\"oldest-unread\""
        ),
        b!(
            Chat,
            "Messages",
            "buffer.backlog_separator.hide_when_all_read",
            "Hide unread separator when everything is read",
            "true"
        ),
        c!(
            Chat,
            "Names and users",
            "buffer.nickname.color",
            "Nickname colors",
            &[
                Choice {
                    label: "Unique colors",
                    value: "\"unique\""
                },
                Choice {
                    label: "One theme color",
                    value: "\"solid\""
                }
            ],
            "\"unique\""
        ),
        c!(
            Chat,
            "Names and users",
            "buffer.nickname.alignment",
            "Nickname alignment",
            &[
                Choice {
                    label: "Left",
                    value: "\"left\""
                },
                Choice {
                    label: "Right",
                    value: "\"right\""
                },
                Choice {
                    label: "Above the message",
                    value: "\"top\""
                }
            ],
            "\"left\""
        ),
        c!(
            Chat,
            "Names and users",
            "buffer.nickname.hide_consecutive",
            "Repeated nicknames",
            REPEAT,
            "false"
        ),
        c!(
            Chat,
            "Names and users",
            "buffer.nickname.show_access_levels",
            "Channel privileges beside nicknames",
            ACCESS,
            "\"highest\""
        ),
        b!(
            Chat,
            "Names and users",
            "buffer.nickname.show_bot_icon",
            "Show bot icons",
            "true"
        ),
        b!(
            Chat,
            "Names and users",
            "buffer.channel.nicklist.enabled",
            "Show the channel user list",
            "true"
        ),
        c!(
            Chat,
            "Names and users",
            "buffer.channel.nicklist.position",
            "User-list position",
            &[
                Choice {
                    label: "Left",
                    value: "\"left\""
                },
                Choice {
                    label: "Right",
                    value: "\"right\""
                }
            ],
            "\"right\""
        ),
        s!(
            Chat,
            "Names and users",
            "buffer.channel.nicklist.width",
            "User-list width",
            Kind::Float {
                min: 1.0,
                max: 10000.0,
                optional: true
            },
            "",
            "Pixels; leave empty for automatic width."
        ),
        b!(
            Chat,
            "Names and users",
            "buffer.channel.topic_banner.enabled",
            "Show the channel topic",
            "true"
        ),
        s!(
            Chat,
            "Names and users",
            "buffer.channel.topic_banner.max_lines",
            "Maximum topic lines",
            Kind::Integer {
                min: 1,
                max: 65535,
                optional: false
            },
            "2",
            ""
        ),
        b!(
            Chat,
            "Activity",
            "buffer.server_messages.default.enabled",
            "Show server activity messages by default",
            "true"
        ),
        c!(
            Chat,
            "Activity",
            "buffer.server_messages.join.enabled",
            "People joining",
            EVENTS,
            ""
        ),
        c!(
            Chat,
            "Activity",
            "buffer.server_messages.part.enabled",
            "People leaving a channel",
            EVENTS,
            ""
        ),
        c!(
            Chat,
            "Activity",
            "buffer.server_messages.quit.enabled",
            "People disconnecting",
            EVENTS,
            ""
        ),
        c!(
            Chat,
            "Activity",
            "buffer.server_messages.change_nick.enabled",
            "Nickname changes",
            EVENTS,
            ""
        ),
        c!(
            Chat,
            "Activity",
            "buffer.server_messages.change_mode.enabled",
            "Channel-mode changes",
            EVENTS,
            ""
        ),
        c!(
            Chat,
            "Read markers",
            "buffer.mark_as_read.on_message",
            "Mark arriving messages as read",
            &[
                Choice {
                    label: "In the focused chat",
                    value: "\"focused\""
                },
                Choice {
                    label: "In every open chat",
                    value: "\"open\""
                },
                Choice {
                    label: "Never automatically",
                    value: "\"none\""
                }
            ],
            "\"focused\""
        ),
        b!(
            Chat,
            "Read markers",
            "buffer.mark_as_read.on_scroll_to_bottom",
            "Mark as read when scrolling to the bottom",
            "true"
        ),
        b!(
            Chat,
            "Read markers",
            "buffer.mark_as_read.on_message_sent",
            "Mark as read when sending a message",
            "true"
        ),
        b!(
            Chat,
            "Read markers",
            "buffer.mark_as_read.on_application_exit",
            "Mark as read when quitting Halloy",
            "false"
        ),
        c!(
            Chat,
            "Read markers",
            "buffer.mark_as_read.on_buffer_close",
            "Mark as read when closing a chat",
            &[
                Choice {
                    label: "Only when scrolled to the bottom",
                    value: "\"scrolled-to-bottom\""
                },
                Choice {
                    label: "Always",
                    value: "true"
                },
                Choice {
                    label: "Never",
                    value: "false"
                }
            ],
            "\"scrolled-to-bottom\""
        ),
        c!(
            Chat,
            "Composer",
            "buffer.text_input.visibility",
            "Show the message input",
            &[
                Choice {
                    label: "Always",
                    value: "\"always\""
                },
                Choice {
                    label: "Only in the focused pane",
                    value: "\"focused\""
                }
            ],
            "\"always\""
        ),
        c!(
            Chat,
            "Composer",
            "buffer.text_input.auto_format",
            "Message formatting",
            &[
                Choice {
                    label: "Disabled",
                    value: "\"disabled\""
                },
                Choice {
                    label: "Markdown",
                    value: "\"markdown\""
                },
                Choice {
                    label: "All supported formatting",
                    value: "\"all\""
                }
            ],
            "\"disabled\""
        ),
        b!(
            Chat,
            "Composer",
            "buffer.text_input.nickname.enabled",
            "Show your nickname beside the input",
            "true"
        ),
        s!(
            Chat,
            "Composer",
            "buffer.text_input.max_lines",
            "Maximum input lines",
            Kind::Integer {
                min: 1,
                max: 1000,
                optional: false
            },
            "5",
            ""
        ),
        b!(
            Chat,
            "Composer",
            "buffer.text_input.persist",
            "Keep message drafts between sessions",
            "true"
        ),
        c!(
            Chat,
            "Composer",
            "buffer.text_input.autocomplete.order_by",
            "Nickname completion order",
            &[
                Choice {
                    label: "Recently active",
                    value: "\"recent\""
                },
                Choice {
                    label: "Alphabetical",
                    value: "\"alpha\""
                }
            ],
            "\"recent\""
        ),
        b!(
            Chat,
            "Composer",
            "buffer.emojis.show_picker",
            "Show the emoji picker",
            "true"
        ),
        b!(
            Chat,
            "Composer",
            "buffer.emojis.auto_replace",
            "Replace emoji shortcodes while typing",
            "true"
        ),
        b!(
            Chat,
            "Typing",
            "buffer.typing.show",
            "Show other people's typing indicators",
            "true"
        ),
        b!(
            Chat,
            "Typing",
            "buffer.typing.share",
            "Share your typing status",
            "false"
        ),
        b!(
            Chat,
            "Typing",
            "buffer.typing.animation.enabled",
            "Animate typing indicators",
            "true"
        ),
        o!(
            Sidebar,
            "Layout",
            "sidebar.max_width",
            "Maximum sidebar width",
            65535
        ),
        c!(
            Sidebar,
            "Layout",
            "sidebar.order_by",
            "Server order",
            &[
                Choice {
                    label: "Alphabetical",
                    value: "\"alpha\""
                },
                Choice {
                    label: "Configuration order",
                    value: "\"config\""
                }
            ],
            "\"alpha\""
        ),
        c!(
            Sidebar,
            "Layout",
            "sidebar.order_channels_by",
            "Channel order",
            &[
                Choice {
                    label: "Name",
                    value: "\"name\""
                },
                Choice {
                    label: "Name and prefix",
                    value: "\"name-and-prefix\""
                },
                Choice {
                    label: "Configuration order",
                    value: "\"config\""
                }
            ],
            "\"name\""
        ),
        b!(
            Sidebar,
            "Layout",
            "sidebar.user_menu.enabled",
            "Show the user menu",
            "true"
        ),
        b!(
            Sidebar,
            "Layout",
            "sidebar.collapse_button.enabled",
            "Show the sidebar collapse button",
            "true"
        ),
        o!(
            Sidebar,
            "Labels and spacing",
            "sidebar.primary_font_size",
            "Server label size",
            255
        ),
        o!(
            Sidebar,
            "Labels and spacing",
            "sidebar.secondary_font_size",
            "Channel label size",
            255
        ),
        c!(
            Sidebar,
            "Labels and spacing",
            "sidebar.primary_icon",
            "Server icons",
            &[
                Choice {
                    label: "Hidden",
                    value: "\"hidden\""
                },
                Choice {
                    label: "Small (16 px)",
                    value: "16"
                },
                Choice {
                    label: "Default (20 px)",
                    value: "20"
                },
                Choice {
                    label: "Large (24 px)",
                    value: "24"
                }
            ],
            "20"
        ),
        c!(
            Sidebar,
            "Labels and spacing",
            "sidebar.channel_name_casing",
            "Channel-name casing",
            &[
                Choice {
                    label: "As provided by the server",
                    value: ""
                },
                Choice {
                    label: "Lowercase",
                    value: "\"lowercase\""
                }
            ],
            ""
        ),
        s!(
            Sidebar,
            "Labels and spacing",
            "sidebar.padding.buffer",
            "Item padding",
            Kind::Pair,
            "[5, 4]",
            "Vertical, horizontal pixels; for example 5, 4."
        ),
        n!(
            Sidebar,
            "Labels and spacing",
            "sidebar.spacing.server",
            "Space between server groups",
            "2"
        ),
        c!(
            Sidebar,
            "Unread indicators",
            "sidebar.unread_indicator.icon",
            "Unread icon",
            ICONS,
            "\"dot\""
        ),
        b!(
            Sidebar,
            "Unread indicators",
            "sidebar.unread_indicator.title",
            "Emphasize unread chat names",
            "false"
        ),
        s!(
            Sidebar,
            "Unread indicators",
            "sidebar.unread_indicator.icon_size",
            "Unread icon size",
            Kind::Integer {
                min: 1,
                max: 4294967295,
                optional: false
            },
            "6",
            "Pixels"
        ),
        b!(
            Sidebar,
            "Unread indicators",
            "sidebar.unread_indicator.show_on_open_buffers",
            "Show unread indicators for open chats",
            "true"
        ),
        b!(
            Sidebar,
            "Unread indicators",
            "sidebar.unread_indicator.query_as_highlight",
            "Treat unread private messages as highlights",
            "false"
        ),
        c!(
            Sidebar,
            "Highlights and typing",
            "sidebar.highlight_indicator.icon",
            "Highlight icon",
            ICONS,
            "\"circle-empty\""
        ),
        b!(
            Sidebar,
            "Highlights and typing",
            "sidebar.highlight_indicator.title",
            "Emphasize highlighted chat names",
            "false"
        ),
        s!(
            Sidebar,
            "Highlights and typing",
            "sidebar.highlight_indicator.icon_size",
            "Highlight icon size",
            Kind::Integer {
                min: 1,
                max: 4294967295,
                optional: false
            },
            "6",
            "Pixels"
        ),
        b!(
            Sidebar,
            "Highlights and typing",
            "sidebar.highlight_indicator.show_on_open_buffers",
            "Show highlight indicators for open chats",
            "true"
        ),
        c!(
            Sidebar,
            "Highlights and typing",
            "sidebar.typing_indicator.show",
            "Typing indicators in the sidebar",
            &[
                Choice {
                    label: "Automatic for the sidebar position",
                    value: ""
                },
                Choice {
                    label: "Private conversations",
                    value: "\"queries\""
                },
                Choice {
                    label: "Hidden",
                    value: "\"none\""
                }
            ],
            ""
        ),
        b!(
            Sidebar,
            "Scrollbars",
            "sidebar.scrollbar.hidden",
            "Hide the sidebar scrollbar",
            "false"
        ),
        n!(
            Sidebar,
            "Scrollbars",
            "sidebar.scrollbar.width",
            "Scrollbar width",
            "5"
        ),
        n!(
            Sidebar,
            "Scrollbars",
            "sidebar.scrollbar.scroller_width",
            "Scrollbar thumb width",
            "5"
        ),
    ];
    for spec in &mut specs {
        spec.aliases = match spec.path {
            "actions.buffer.click_username" => {
                &["actions.buffer.click_nickname"]
            }
            "actions.nicklist.click_username" => {
                &["actions.nicklist.click_nickname"]
            }
            "actions.buffer.open_internal" => &["actions.buffer.local"],
            "buffer.channel.topic_banner.enabled" => {
                &["buffer.channel.topic.enabled"]
            }
            "buffer.channel.topic_banner.max_lines" => {
                &["buffer.channel.topic.max_lines"]
            }
            "sidebar.primary_font_size" => &["sidebar.server_font_size"],
            "sidebar.secondary_font_size" => &["sidebar.font_size"],
            "sidebar.primary_icon" => &[
                "sidebar.server_icon",
                "sidebar.server_icon_size",
                "sidebar.primary_icon_size",
            ],
            _ => &[],
        };
    }
    specs.extend(super::remaining_media::schema(
        Area::Previews,
        Area::Transfers,
    ));
    super::remaining_display::extend_schema(&mut specs, Some(Area::Appearance));
    specs
}

fn literal(text: &str) -> Option<toml::Value> {
    if text.is_empty() {
        return None;
    }
    toml::from_str::<toml::Table>(&format!("v = {text}"))
        .ok()?
        .remove("v")
}
fn lookup<'a>(root: &'a toml::Value, path: &str) -> Option<&'a toml::Value> {
    path.split('.')
        .try_fold(root, |value, part| value.get(part))
}
fn legacy_indicator(root: &toml::Value, path: &str) -> Option<toml::Value> {
    for indicator in ["unread_indicator", "highlight_indicator"] {
        let prefix = format!("sidebar.{indicator}.");
        let Some(field) = path.strip_prefix(&prefix) else {
            continue;
        };
        let Some(mode) = lookup(root, &format!("sidebar.{indicator}"))
            .and_then(toml::Value::as_str)
        else {
            continue;
        };
        return match field {
            "title" => Some(toml::Value::Boolean(mode == "title")),
            "icon" => Some(toml::Value::String(
                if mode == "dot" { "dot" } else { "none" }.into(),
            )),
            _ => None,
        };
    }
    None
}
impl Preferences {
    pub fn parse(document: &DocumentMut) -> Result<Self, String> {
        let root: toml::Value = toml::from_str(&document.to_string())
            .map_err(|error| error.to_string())?;
        let fields = schema()
            .into_iter()
            .map(|spec| {
                let write_path = std::iter::once(spec.path)
                    .chain(spec.aliases.iter().copied())
                    .find(|path| lookup(&root, path).is_some())
                    .or_else(|| {
                        std::iter::once(spec.path)
                            .chain(spec.aliases.iter().copied())
                            .find(|path| {
                                path.rsplit_once('.').is_some_and(
                                    |(parent, _)| {
                                        lookup(&root, parent)
                                            .is_some_and(toml::Value::is_table)
                                    },
                                )
                            })
                    })
                    .unwrap_or(spec.path)
                    .to_owned();
                let projected = legacy_indicator(&root, spec.path);
                let value = lookup(&root, &write_path)
                    .cloned()
                    .or(projected)
                    .or_else(|| literal(spec.default));
                let value = match spec.kind {
                    Kind::Text => value
                        .as_ref()
                        .and_then(toml::Value::as_str)
                        .map_or_else(
                            || "Configured in file".into(),
                            str::to_owned,
                        ),
                    Kind::Choice(choices) => choices
                        .iter()
                        .find(|choice| literal(choice.value) == value)
                        .map_or_else(
                            || format!("configured:{value:?}"),
                            |choice| choice.value.to_owned(),
                        ),
                    Kind::Pair => value
                        .as_ref()
                        .and_then(toml::Value::as_array)
                        .map(|items| {
                            items
                                .iter()
                                .map(ToString::to_string)
                                .collect::<Vec<_>>()
                                .join(", ")
                        })
                        .unwrap_or_default(),
                    _ => {
                        value.map(|value| value.to_string()).unwrap_or_default()
                    }
                };
                Field {
                    spec,
                    original: value.clone(),
                    value,
                    write_path,
                }
            })
            .collect();
        Ok(Self { fields })
    }
    pub fn is_dirty(&self) -> bool {
        self.fields
            .iter()
            .any(|field| field.value != field.original)
    }
    pub(super) fn validation_error(
        &self,
        source: &DocumentMut,
    ) -> Option<(usize, String)> {
        self.fields.iter().enumerate().find_map(|(index, field)| {
            if field.value == field.original {
                return None;
            }
            // Use the same writer and runtime validation as Apply. This also
            // locates errors in formats or other values parsed by serde.
            let mut document = source.clone();
            let isolated = Self {
                fields: vec![field.clone()],
            };
            isolated
                .render_into(&mut document)
                .and_then(|()| {
                    super::Config::validate(&document.to_string())
                        .map_err(|error| error.to_string())
                })
                .err()
                .map(|message| (index, message))
        })
    }
    pub fn reuse_current_pane(&mut self) {
        for field in &mut self.fields {
            if matches!(field.spec.kind, Kind::Choice(choices) if choices.iter().any(|choice| choice.value == "\"replace-pane\""))
                && field.spec.path != "actions.sidebar.focused_buffer"
            {
                field.value = "\"replace-pane\"".into();
            }
        }
        if let Some(field) = self
            .fields
            .iter_mut()
            .find(|field| field.spec.path == "actions.sidebar.focused_buffer")
        {
            field.value.clear();
        }
    }
    pub fn render_into(
        &self,
        document: &mut DocumentMut,
    ) -> Result<(), String> {
        for field in &self.fields {
            if field.value == field.original {
                continue;
            }
            let item = field.render()?;
            let mut parts = field.write_path.split('.').collect::<Vec<_>>();
            let key = parts.pop().ok_or("Missing setting name")?;
            let mut parent: &mut dyn TableLike = document.as_table_mut();
            for part in parts {
                if matches!(part, "unread_indicator" | "highlight_indicator")
                    && let Some(mode) = parent
                        .get(part)
                        .and_then(Item::as_str)
                        .map(str::to_owned)
                {
                    let mut table = Table::new();
                    table.insert("title", value(mode == "title"));
                    table.insert(
                        "icon",
                        value(if mode == "dot" { "dot" } else { "none" }),
                    );
                    parent.insert(part, Item::Table(table));
                }
                parent = child_table(parent, part)?;
            }
            if let Some(item) = item {
                complete_brackets(parent, &field.write_path);
                set(parent, key, item);
            } else {
                parent.remove(key);
            }
        }
        Ok(())
    }
}
impl Field {
    pub fn is_custom(&self) -> bool {
        match self.spec.kind {
            Kind::Choice(choices) => {
                !choices.iter().any(|choice| choice.value == self.value)
            }
            Kind::Boolean => !matches!(self.value.as_str(), "true" | "false"),
            _ => false,
        }
    }
    pub fn original_value(&self) -> &str {
        &self.original
    }
    fn render(&self) -> Result<Option<Item>, String> {
        let label = self.spec.label;
        match self.spec.kind {
            Kind::Text => Ok(Some(value(&self.value))),
            Kind::Boolean => self
                .value
                .parse::<bool>()
                .map(value)
                .map(Some)
                .map_err(|_| format!("{label}: choose enabled or disabled.")),
            Kind::Choice(choices) => {
                if !choices.iter().any(|choice| choice.value == self.value) {
                    return Err(format!("{label}: choose an option."));
                }
                if self.value.is_empty() {
                    Ok(None)
                } else {
                    self.value
                        .parse::<Value>()
                        .map(Item::Value)
                        .map(Some)
                        .map_err(|error| error.to_string())
                }
            }
            Kind::Integer { min, max, optional } => {
                if optional && self.value.is_empty() {
                    return Ok(None);
                }
                self.value.parse::<i64>().ok().filter(|value| (min..=max).contains(value)).map(value).map(Some)
                    .ok_or_else(|| if max == i64::MAX && min == 0 { format!("{label} must be a non-negative whole number.") } else { format!("{label} must be a whole number between {min} and {max}.") })
            }
            Kind::Float { min, max, optional } => {
                if optional && self.value.is_empty() {
                    return Ok(None);
                }
                self.value
                    .parse::<f64>()
                    .ok()
                    .filter(|value| {
                        value.is_finite() && (min..=max).contains(value)
                    })
                    .map(value)
                    .map(Some)
                    .ok_or_else(|| {
                        format!("{label} must be between {min} and {max}.")
                    })
            }
            Kind::Pair => {
                let values = self.value.split(',').map(str::trim).map(str::parse::<u16>).collect::<Result<Vec<_>,_>>().ok().filter(|values| values.len()==2).ok_or_else(|| format!("{label}: enter two non-negative whole numbers separated by a comma."))?;
                let mut array = toml_edit::Array::new();
                for item in values {
                    array.push(i64::from(item));
                }
                Ok(Some(value(array)))
            }
        }
    }
}

/// Brackets are a required pair in the runtime model; initialize only the
/// missing side when the first override is introduced.
pub(super) fn complete_brackets(parent: &mut dyn TableLike, path: &str) {
    if path
        .rsplit_once('.')
        .is_some_and(|(parent, _)| parent.ends_with(".brackets"))
    {
        for key in ["left", "right"] {
            if !parent.contains_key(key) {
                set(parent, key, value(""));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Config;
    use crate::config::settings::Settings;

    fn edit(settings: &mut Settings, path: &str, value: &str) {
        settings
            .preferences
            .fields
            .iter_mut()
            .find(|field| field.spec.path == path)
            .unwrap()
            .value = value.into();
    }

    #[test]
    fn every_exposed_choice_and_default_is_accepted_by_the_runtime_config() {
        let settings = Settings::parse(String::new()).unwrap();
        for field in &settings.preferences.fields {
            let values = match field.spec.kind {
                Kind::Choice(choices) => choices
                    .iter()
                    .map(|choice| choice.value.to_owned())
                    .collect::<Vec<_>>(),
                Kind::Boolean => vec!["true".into(), "false".into()],
                _ => vec![field.value.clone()],
            };
            for option in values {
                let mut settings = settings.clone();
                if field.spec.path == "file_transfer.auto_accept.enabled" {
                    settings
                        .media
                        .fields
                        .iter_mut()
                        .find(|field| {
                            field.path == "file_transfer.save_directory"
                        })
                        .unwrap()
                        .value = "/tmp/halloy-transfer-test".into();
                }
                let target = settings
                    .preferences
                    .fields
                    .iter_mut()
                    .find(|target| target.spec.path == field.spec.path)
                    .unwrap();
                target.original = "force write".into();
                target.value = option.clone();
                let result = settings.render();
                assert!(
                    result.is_ok(),
                    "{} = {}: {:?}",
                    field.spec.path,
                    option,
                    result.err()
                );
                let rendered = result.unwrap();
                let mut ignored = vec![];
                let _: crate::config::Configuration =
                    serde_ignored::deserialize(
                        toml::Deserializer::parse(&rendered).unwrap(),
                        |path| ignored.push(path.to_string()),
                    )
                    .unwrap();
                assert!(
                    ignored.is_empty(),
                    "{} was ignored by the runtime: {ignored:?}",
                    field.spec.path
                );
            }
        }
    }

    #[test]
    fn edits_preserve_comments_inline_tables_aliases_and_custom_rules() {
        let source = "# Keep this\n[actions.buffer]\nclick_nickname = { open-query = 'new-window' } # click rule\n[buffer.nickname]\ncolor = { palette = ['#112233', '#445566'] }\n[buffer.server_messages.join]\nexclude = ['#quiet'] # keep rule\n[sidebar]\nserver_font_size = 15\n[servers.test]\nserver = 'irc.example.test'\nnickname = 'test'\npassword_command = 'secret-tool'\n";
        let mut settings = Settings::parse(source.into()).unwrap();
        assert_eq!(settings.render().unwrap(), source);
        edit(
            &mut settings,
            "actions.buffer.click_username",
            "\"replace-pane\"",
        );
        edit(&mut settings, "sidebar.primary_font_size", "17");
        edit(
            &mut settings,
            "buffer.server_messages.join.enabled",
            "false",
        );
        let rendered = settings.render().unwrap();
        assert!(rendered.contains("# Keep this"));
        assert!(rendered.contains("# click rule"));
        assert!(rendered.contains("# keep rule"));
        assert!(rendered.contains("password_command = 'secret-tool'"));
        assert!(
            rendered.contains("color = { palette = ['#112233', '#445566'] }")
        );
        assert!(rendered.contains("server_font_size = 17"));
        assert!(!rendered.contains("primary_font_size"));
        assert!(rendered.contains("click_nickname = \"replace-pane\""));
        assert!(!rendered.contains("click_username"));
        Config::validate(&rendered).unwrap();
        let reparsed = Settings::parse(rendered).unwrap();
        assert!(!reparsed.is_dirty());
        assert!(
            reparsed
                .preferences
                .fields
                .iter()
                .find(|field| field.spec.path == "buffer.nickname.color")
                .unwrap()
                .is_custom()
        );
    }

    #[test]
    fn missing_siblings_use_the_existing_legacy_parent() {
        for (source, path, value) in [
            (
                "[buffer.channel.topic]\nenabled = false\n",
                "buffer.channel.topic_banner.max_lines",
                "3",
            ),
            (
                "[buffer.channel.topic]\nmax_lines = 3\n",
                "buffer.channel.topic_banner.enabled",
                "false",
            ),
            (
                "buffer = { channel = { topic = { enabled = false } } }\n",
                "buffer.channel.topic_banner.max_lines",
                "3",
            ),
        ] {
            let mut settings = Settings::parse(source.into()).unwrap();
            edit(&mut settings, path, value);
            let rendered = settings.render().unwrap();
            Config::validate(&rendered).unwrap();
            assert!(!rendered.contains("topic_banner"), "{rendered}");
            let document = rendered.parse::<DocumentMut>().unwrap();
            assert_eq!(
                document["buffer"]["channel"]["topic"]["enabled"].as_bool(),
                Some(false)
            );
            assert_eq!(
                document["buffer"]["channel"]["topic"]["max_lines"]
                    .as_integer(),
                Some(3)
            );
        }
    }

    #[test]
    fn preference_edits_preserve_leading_and_suffix_comments() {
        let source = "[buffer.timestamp]\n# Timestamp explanation\n'format' = '%R' # timestamp suffix\n";
        let mut settings = Settings::parse(source.into()).unwrap();
        edit(&mut settings, "buffer.timestamp.format", "%H:%M:%S");
        let rendered = settings.render().unwrap();
        assert!(rendered.contains("# Timestamp explanation\n'format' = \"%H:%M:%S\" # timestamp suffix"));
    }

    #[test]
    fn one_pane_preset_overrides_per_kind_click_actions_without_changing_layout()
     {
        let mut settings = Settings::parse("[actions.sidebar]\nbuffer = 'new-window'\nchannel = 'new-pane'\nfocused_buffer = 'close-pane'\n[actions.buffer]\nclick_nickname = 'insert-nickname'\n[pane]\nsplit_axis = 'vertical'\n".into()).unwrap();
        settings.preferences.reuse_current_pane();
        assert!(settings.is_dirty());
        let rendered = settings.render().unwrap();
        let config: toml::Value = toml::from_str(&rendered).unwrap();
        assert_eq!(
            config["actions"]["sidebar"]["channel"].as_str(),
            Some("replace-pane")
        );
        assert!(config["actions"]["sidebar"].get("focused_buffer").is_none());
        assert_eq!(
            config["actions"]["buffer"]["click_nickname"].as_str(),
            Some("replace-pane")
        );
        assert_eq!(config["pane"]["split_axis"].as_str(), Some("vertical"));
    }

    #[test]
    fn legacy_indicator_forms_keep_their_meaning_when_editing_other_fields() {
        let mut settings = Settings::parse("[sidebar]\nunread_indicator = 'none'\nhighlight_indicator = 'title'\n".into()).unwrap();
        let unread = settings
            .preferences
            .fields
            .iter()
            .find(|field| field.spec.path == "sidebar.unread_indicator.icon")
            .unwrap();
        assert_eq!(unread.value, "\"none\"");
        let highlight = settings
            .preferences
            .fields
            .iter()
            .find(|field| {
                field.spec.path == "sidebar.highlight_indicator.title"
            })
            .unwrap();
        assert_eq!(highlight.value, "true");
        edit(&mut settings, "sidebar.unread_indicator.icon_size", "9");
        edit(&mut settings, "sidebar.highlight_indicator.icon_size", "9");
        let output: toml::Value =
            toml::from_str(&settings.render().unwrap()).unwrap();
        assert_eq!(
            output["sidebar"]["unread_indicator"]["icon"].as_str(),
            Some("none")
        );
        assert_eq!(
            output["sidebar"]["highlight_indicator"]["title"].as_bool(),
            Some(true)
        );
    }

    #[test]
    fn optional_overrides_can_be_removed_without_removing_siblings() {
        let mut settings = Settings::parse("actions.sidebar = { buffer = 'new-pane', channel = 'new-window', query = 'replace-pane' }\n".into()).unwrap();
        edit(&mut settings, "actions.sidebar.channel", "");
        let output: toml::Value =
            toml::from_str(&settings.render().unwrap()).unwrap();
        assert!(output["actions"]["sidebar"].get("channel").is_none());
        assert_eq!(
            output["actions"]["sidebar"]["query"].as_str(),
            Some("replace-pane")
        );
    }

    #[test]
    fn malformed_numbers_padding_and_date_formats_are_rejected_before_saving() {
        for (path, input) in [
            ("pane.gap.inner", "-1"),
            ("sidebar.secondary_font_size", "0"),
            ("sidebar.padding.buffer", "5"),
            ("buffer.channel.nicklist.width", "inf"),
            ("buffer.timestamp.format", "%Q"),
        ] {
            let mut settings = Settings::parse(String::new()).unwrap();
            edit(&mut settings, path, input);
            assert!(settings.render().is_err(), "{path} accepted {input}");
        }
    }
}

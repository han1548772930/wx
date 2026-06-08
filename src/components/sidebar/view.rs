use gpui::{
    Anchor, App, AppContext, Context, Entity, InteractiveElement, IntoElement, ParentElement,
    Render, RenderOnce, SharedString, StatefulInteractiveElement, Styled, Window, div,
    prelude::FluentBuilder,
};
use gpui_component::{
    ActiveTheme, Icon, Selectable, WindowExt, popover::Popover,
    v_flex,
};

use crate::ui::theme::Theme;

use crate::models::ToolbarItem;

use crate::app::actions::ToolbarClicked;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PhonePopoverHover {
    None,
    Video,
    Voice,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MenuPopoverHover {
    None,
    VideoLive,
    ChatFiles,
    ChatHistory,
    Lock,
    Feedback,
    Settings,
}

pub struct ToolBar {
    active_item: ToolbarItem,
    phone_hover: PhonePopoverHover,
    menu_hover: MenuPopoverHover,
}

#[derive(IntoElement)]
struct ToolbarPopoverTrigger {
    id: SharedString,
    icon_path: SharedString,
    icon_color: gpui::Hsla,
    hover_bg: gpui::Hsla,
    active_bg: gpui::Hsla,
    selected: bool,
}

impl ToolbarPopoverTrigger {
    fn new(
        id: impl Into<SharedString>,
        icon_path: impl Into<SharedString>,
        icon_color: gpui::Hsla,
        hover_bg: gpui::Hsla,
        active_bg: gpui::Hsla,
    ) -> Self {
        Self {
            id: id.into(),
            icon_path: icon_path.into(),
            icon_color,
            hover_bg,
            active_bg,
            selected: false,
        }
    }
}

impl Selectable for ToolbarPopoverTrigger {
    fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    fn is_selected(&self) -> bool {
        self.selected
    }
}

impl RenderOnce for ToolbarPopoverTrigger {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let selected = self.selected;
        let hover_bg = self.hover_bg;
        let active_bg = self.active_bg;

        div()
            .id(self.id)
            .flex()
            .items_center()
            .justify_center()
            .rounded(crate::ui::constants::radius_md())
            .w(crate::ui::constants::toolbar_trigger_size())
            .h(crate::ui::constants::toolbar_trigger_size())
            .cursor_pointer()
            .when(selected, |this| this.bg(active_bg))
            .hover(move |this| this.bg(hover_bg))
            .active(move |this| this.bg(active_bg))
            .child(
                Icon::default()
                    .path(self.icon_path)
                    .size(crate::ui::constants::icon_md())
                    .text_color(self.icon_color),
            )
    }
}

impl ToolBar {
    pub fn new(_window: &mut Window, _cx: &mut Context<Self>) -> Self {
        Self {
            active_item: ToolbarItem::Chat,
            phone_hover: PhonePopoverHover::None,
            menu_hover: MenuPopoverHover::None,
        }
    }

    pub fn view(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx))
    }

    pub fn set_active_item(&mut self, item: ToolbarItem, cx: &mut Context<Self>) {
        self.active_item = item;
        cx.notify();
    }

    fn render_toolbar_button(
        &self,
        item: ToolbarItem,
        id: &'static str,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let is_active = self.active_item == item;
        let theme = cx.theme();
        let weixin_colors = Theme::weixin_colors(cx);

        let icon_path = if is_active {
            item.icon_path_fill().unwrap_or(item.icon_path())
        } else {
            item.icon_path()
        };

        let icon_color = if is_active && item.has_fill() {
            weixin_colors.weixin_green
        } else {
            theme.muted_foreground
        };
        let hover_bg = weixin_colors.toolbar_button_hover;
        let active_bg = weixin_colors.toolbar_button_active;

        div()
            .w_full()
            .flex()
            .items_center()
            .justify_center()
            .py(crate::ui::constants::toolbar_button_padding_y())
            .child(
                div()
                    .id(id)
                    .flex()
                    .items_center()
                    .justify_center()
                    .p(crate::ui::constants::toolbar_item_padding())
                    .rounded(crate::ui::constants::radius_md())
                    .cursor_pointer()
                    .hover(move |this| this.bg(hover_bg))
                    .active(move |this| this.bg(active_bg))
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.active_item = item;
                        window.dispatch_action(Box::new(ToolbarClicked { item }), cx);
                        cx.notify();
                    }))
                    .child(
                        Icon::default()
                            .path(icon_path)
                            .size(crate::ui::constants::icon_md())
                            .text_color(icon_color),
                    ),
            )
    }

    fn render_menu_item_helper<F, C>(
        id: &'static str,
        label: &'static str,
        is_hovered: bool,
        on_hover: F,
        on_click: C,
    ) -> impl IntoElement
    where
        F: Fn(bool, &mut App) + Clone + 'static,
        C: Fn(&gpui::MouseDownEvent, &mut Window, &mut App) + 'static,
    {
        crate::ui::base::menu_item::MenuItem::new(id, label)
            .hovered(is_hovered)
            .on_hover(on_hover)
            .on_click(on_click)
    }

    fn render_phone_button(
        &self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let theme = cx.theme();
        let weixin_colors = Theme::weixin_colors(cx);
        let toolbar = cx.entity();
        let hover_bg = weixin_colors.toolbar_button_hover;
        let active_bg = weixin_colors.toolbar_button_active;

        div().w_full().flex().items_center().justify_center().child(
            Popover::new("toolbar-phone")
                .appearance(false)
                .anchor(Anchor::BottomRight)
                .trigger(ToolbarPopoverTrigger::new(
                    "phone-trigger",
                    "phone.svg",
                    theme.muted_foreground,
                    hover_bg,
                    active_bg,
                ))
                .content(move |_, _window, cx| {
                    let weixin_colors = Theme::weixin_colors(cx);

                    let phone_hover = toolbar.read(cx).phone_hover;
                    let video_hovered = matches!(phone_hover, PhonePopoverHover::Video);
                    let voice_hovered = matches!(phone_hover, PhonePopoverHover::Voice);

                    let toolbar_for_video = toolbar.clone();
                    let set_video_hover = move |is_hovering: bool, cx: &mut App| {
                        _ = toolbar_for_video.update(cx, |this: &mut ToolBar, cx| {
                            this.phone_hover = if is_hovering {
                                PhonePopoverHover::Video
                            } else if matches!(this.phone_hover, PhonePopoverHover::Video) {
                                PhonePopoverHover::None
                            } else {
                                this.phone_hover
                            };
                            cx.notify();
                        });
                    };

                    let toolbar_for_voice = toolbar.clone();
                    let set_voice_hover = move |is_hovering: bool, cx: &mut App| {
                        _ = toolbar_for_voice.update(cx, |this: &mut ToolBar, cx| {
                            this.phone_hover = if is_hovering {
                                PhonePopoverHover::Voice
                            } else if matches!(this.phone_hover, PhonePopoverHover::Voice) {
                                PhonePopoverHover::None
                            } else {
                                this.phone_hover
                            };
                            cx.notify();
                        });
                    };

                    // 点击后重置 hover 状态，避免下次打开弹出层时仍然高亮
                    let toolbar_for_video_click = toolbar.clone();
                    let toolbar_for_voice_click = toolbar.clone();

                    v_flex()
                        .gap_1()
                        .p_2()
                        .bg(weixin_colors.popover_bg)
                        .rounded(crate::ui::constants::radius_md())
                        .shadow_md()
                        .child(Self::render_menu_item_helper(
                            "phone-video-call",
                            "视频通话",
                            video_hovered,
                            set_video_hover,
                            cx.listener(move |_, _, window, cx| {
                                window.push_notification("视频通话功能开发中...", cx);
                                cx.emit(gpui::DismissEvent);
                                _ = toolbar_for_video_click.update(cx, |this: &mut ToolBar, cx| {
                                    this.phone_hover = PhonePopoverHover::None;
                                    cx.notify();
                                });
                            }),
                        ))
                        .child(Self::render_menu_item_helper(
                            "phone-voice-call",
                            "语音通话",
                            voice_hovered,
                            set_voice_hover,
                            cx.listener(move |_, _, window, cx| {
                                window.push_notification("语音通话功能开发中...", cx);
                                cx.emit(gpui::DismissEvent);
                                _ = toolbar_for_voice_click.update(cx, |this: &mut ToolBar, cx| {
                                    this.phone_hover = PhonePopoverHover::None;
                                    cx.notify();
                                });
                            }),
                        ))
                }),
        )
    }

    fn render_menu_button(&self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let weixin_colors = Theme::weixin_colors(cx);
        let toolbar = cx.entity();
        let hover_bg = weixin_colors.toolbar_button_hover;
        let active_bg = weixin_colors.toolbar_button_active;

        div()
            .w_full()
            .flex()
            .items_center()
            .justify_center()
            .py(crate::ui::constants::toolbar_menu_padding_y())
            .child(
                Popover::new("toolbar-menu")
                    .appearance(false)
                    .anchor(Anchor::BottomRight)
                    .trigger(ToolbarPopoverTrigger::new(
                        "menu-trigger",
                        "menu.svg",
                        theme.muted_foreground,
                        hover_bg,
                        active_bg,
                    ))
                    .content(move |_, _window, cx| {
                        let weixin_colors = Theme::weixin_colors(cx);

                        let menu_hover = toolbar.read(cx).menu_hover;
                        let video_live_hovered = matches!(menu_hover, MenuPopoverHover::VideoLive);
                        let chat_files_hovered = matches!(menu_hover, MenuPopoverHover::ChatFiles);
                        let chat_history_hovered =
                            matches!(menu_hover, MenuPopoverHover::ChatHistory);
                        let lock_hovered = matches!(menu_hover, MenuPopoverHover::Lock);
                        let feedback_hovered = matches!(menu_hover, MenuPopoverHover::Feedback);
                        let settings_hovered = matches!(menu_hover, MenuPopoverHover::Settings);

                        let toolbar_for_video_live = toolbar.clone();
                        let set_video_live_hover = move |is_hovering: bool, cx: &mut App| {
                            _ = toolbar_for_video_live.update(cx, |this: &mut ToolBar, cx| {
                                this.menu_hover = if is_hovering {
                                    MenuPopoverHover::VideoLive
                                } else if matches!(this.menu_hover, MenuPopoverHover::VideoLive) {
                                    MenuPopoverHover::None
                                } else {
                                    this.menu_hover
                                };
                                cx.notify();
                            });
                        };

                        let toolbar_for_chat_files = toolbar.clone();
                        let set_chat_files_hover = move |is_hovering: bool, cx: &mut App| {
                            _ = toolbar_for_chat_files.update(cx, |this: &mut ToolBar, cx| {
                                this.menu_hover = if is_hovering {
                                    MenuPopoverHover::ChatFiles
                                } else if matches!(this.menu_hover, MenuPopoverHover::ChatFiles) {
                                    MenuPopoverHover::None
                                } else {
                                    this.menu_hover
                                };
                                cx.notify();
                            });
                        };

                        let toolbar_for_chat_history = toolbar.clone();
                        let set_chat_history_hover = move |is_hovering: bool, cx: &mut App| {
                            _ = toolbar_for_chat_history.update(cx, |this: &mut ToolBar, cx| {
                                this.menu_hover = if is_hovering {
                                    MenuPopoverHover::ChatHistory
                                } else if matches!(this.menu_hover, MenuPopoverHover::ChatHistory) {
                                    MenuPopoverHover::None
                                } else {
                                    this.menu_hover
                                };
                                cx.notify();
                            });
                        };

                        let toolbar_for_lock = toolbar.clone();
                        let set_lock_hover = move |is_hovering: bool, cx: &mut App| {
                            _ = toolbar_for_lock.update(cx, |this: &mut ToolBar, cx| {
                                this.menu_hover = if is_hovering {
                                    MenuPopoverHover::Lock
                                } else if matches!(this.menu_hover, MenuPopoverHover::Lock) {
                                    MenuPopoverHover::None
                                } else {
                                    this.menu_hover
                                };
                                cx.notify();
                            });
                        };

                        let toolbar_for_feedback = toolbar.clone();
                        let set_feedback_hover = move |is_hovering: bool, cx: &mut App| {
                            _ = toolbar_for_feedback.update(cx, |this: &mut ToolBar, cx| {
                                this.menu_hover = if is_hovering {
                                    MenuPopoverHover::Feedback
                                } else if matches!(this.menu_hover, MenuPopoverHover::Feedback) {
                                    MenuPopoverHover::None
                                } else {
                                    this.menu_hover
                                };
                                cx.notify();
                            });
                        };

                        let toolbar_for_settings = toolbar.clone();
                        let set_settings_hover = move |is_hovering: bool, cx: &mut App| {
                            _ = toolbar_for_settings.update(cx, |this: &mut ToolBar, cx| {
                                this.menu_hover = if is_hovering {
                                    MenuPopoverHover::Settings
                                } else if matches!(this.menu_hover, MenuPopoverHover::Settings) {
                                    MenuPopoverHover::None
                                } else {
                                    this.menu_hover
                                };
                                cx.notify();
                            });
                        };

                        // 点击菜单项后重置 hover 状态
                        let toolbar_for_video_live_click = toolbar.clone();
                        let toolbar_for_chat_files_click = toolbar.clone();
                        let toolbar_for_chat_history_click = toolbar.clone();
                        let toolbar_for_lock_click = toolbar.clone();
                        let toolbar_for_feedback_click = toolbar.clone();
                        let toolbar_for_settings_click = toolbar.clone();

                        v_flex()
                            .w(crate::ui::constants::toolbar_popover_width())
                            .gap_0()
                            .py_2()
                            .bg(weixin_colors.popover_bg)
                            .p_1()
                            .rounded(crate::ui::constants::radius_md())
                            .shadow_md()
                            .child(Self::render_menu_item_helper(
                                "menu-video-live",
                                "视频号直播伴侣",
                                video_live_hovered,
                                set_video_live_hover,
                                cx.listener(move |_, _, window, cx| {
                                    window.push_notification("视频号直播伴侣功能开发中...", cx);
                                    cx.emit(gpui::DismissEvent);
                                    _ = toolbar_for_video_live_click.update(
                                        cx,
                                        |this: &mut ToolBar, cx| {
                                            this.menu_hover = MenuPopoverHover::None;
                                            cx.notify();
                                        },
                                    );
                                }),
                            ))
                            .child(Self::render_menu_item_helper(
                                "menu-chat-files",
                                "聊天文件",
                                chat_files_hovered,
                                set_chat_files_hover,
                                cx.listener(move |_, _, window, cx| {
                                    window.push_notification("聊天文件功能开发中...", cx);
                                    cx.emit(gpui::DismissEvent);
                                    _ = toolbar_for_chat_files_click.update(
                                        cx,
                                        |this: &mut ToolBar, cx| {
                                            this.menu_hover = MenuPopoverHover::None;
                                            cx.notify();
                                        },
                                    );
                                }),
                            ))
                            .child(Self::render_menu_item_helper(
                                "menu-chat-history",
                                "聊天记录管理",
                                chat_history_hovered,
                                set_chat_history_hover,
                                cx.listener(move |_, _, window, cx| {
                                    window.push_notification("聊天记录管理功能开发中...", cx);
                                    cx.emit(gpui::DismissEvent);
                                    _ = toolbar_for_chat_history_click.update(
                                        cx,
                                        |this: &mut ToolBar, cx| {
                                            this.menu_hover = MenuPopoverHover::None;
                                            cx.notify();
                                        },
                                    );
                                }),
                            ))
                            .child(Self::render_menu_item_helper(
                                "menu-lock",
                                "锁定",
                                lock_hovered,
                                set_lock_hover,
                                cx.listener(move |_, _, window, cx| {
                                    window.push_notification("锁定功能开发中...", cx);
                                    cx.emit(gpui::DismissEvent);
                                    _ = toolbar_for_lock_click.update(
                                        cx,
                                        |this: &mut ToolBar, cx| {
                                            this.menu_hover = MenuPopoverHover::None;
                                            cx.notify();
                                        },
                                    );
                                }),
                            ))
                            .child(Self::render_menu_item_helper(
                                "menu-feedback",
                                "意见反馈",
                                feedback_hovered,
                                set_feedback_hover,
                                cx.listener(move |_, _, window, cx| {
                                    window.push_notification("意见反馈功能开发中...", cx);
                                    cx.emit(gpui::DismissEvent);
                                    _ = toolbar_for_feedback_click.update(
                                        cx,
                                        |this: &mut ToolBar, cx| {
                                            this.menu_hover = MenuPopoverHover::None;
                                            cx.notify();
                                        },
                                    );
                                }),
                            ))
                            .child(Self::render_menu_item_helper(
                                "menu-settings",
                                "设置",
                                settings_hovered,
                                set_settings_hover,
                                cx.listener(move |_, _, _window, cx| {
                                    crate::app::WeixinApp::open_settings_window(cx);
                                    cx.emit(gpui::DismissEvent);
                                    _ = toolbar_for_settings_click.update(
                                        cx,
                                        |this: &mut ToolBar, cx| {
                                            this.menu_hover = MenuPopoverHover::None;
                                            cx.notify();
                                        },
                                    );
                                }),
                            ))
                    }),
            )
    }
}

impl Render for ToolBar {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let weixin_colors = Theme::weixin_colors(cx);
        let theme = cx.theme();

        let bg_color = if window.is_window_active() {
            theme.transparent
        } else {
            weixin_colors.toolbar_bg
        };

        v_flex()
            .bg(bg_color)
            .w(crate::ui::constants::toolbar_width())
            .h_full()
            .items_center()
            .py_2()
            .child(
                v_flex()
                    .flex_1()
                    .w_full()
                    .gap_0()
                    .items_center()
                    .child(self.render_toolbar_button(ToolbarItem::Chat, "toolbar-chat", cx))
                    .child(self.render_toolbar_button(
                        ToolbarItem::Contacts,
                        "toolbar-contacts",
                        cx,
                    ))
                    .child(self.render_toolbar_button(
                        ToolbarItem::Favorites,
                        "toolbar-favorites",
                        cx,
                    ))
                    .child(self.render_toolbar_button(ToolbarItem::Moments, "toolbar-moments", cx))
                    .child(self.render_toolbar_button(
                        ToolbarItem::Channels,
                        "toolbar-channels",
                        cx,
                    ))
                    .child(self.render_toolbar_button(ToolbarItem::Search, "toolbar-search", cx))
                    .child(self.render_toolbar_button(
                        ToolbarItem::MiniProgram,
                        "toolbar-miniprogram",
                        cx,
                    )),
            )
            .child(
                v_flex()
                    .w_full()
                    .items_center()
                    .gap_0()
                    .mb_2()
                    .child(self.render_phone_button(window, cx))
                    .child(self.render_menu_button(window, cx)),
            )
    }
}

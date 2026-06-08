use crate::ui::theme::Theme;
use crate::ui::composites::setting_card;
use gpui::{
    div, px, Anchor, App, Context, InteractiveElement, IntoElement, MouseButton, MouseDownEvent,
    ParentElement, Styled, Window,
};
use gpui_component::{h_flex, input::Input, popover::Popover, v_flex, ActiveTheme, Icon, Sizable};

use super::window::{ShortcutInputField, ShortcutSendHover, SHORTCUT_INPUT_PLACEHOLDER};
use super::SettingsWindow;

impl SettingsWindow {
    pub(crate) fn render_shortcuts_tab(
        &self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let theme = cx.theme();
        let foreground = theme.foreground;

        let send_row = setting_card::setting_row()
            .py_3()
            .child(div().text_sm().text_color(foreground).child("发送消息"))
            .child(self.render_shortcut_send_selector(cx));

        let mut shortcuts_content = v_flex().gap_0().child(send_row);
        shortcuts_content = shortcuts_content.child(setting_card::SettingDivider::new());

        let action_rows = [
            ("截图", ShortcutInputField::Screenshot),
            ("锁定", ShortcutInputField::Lock),
            ("显示/隐藏微信窗口", ShortcutInputField::ToggleWindow),
        ];

        for (index, (label, field)) in action_rows.iter().enumerate() {
            let row = self.shortcut_input_row(*label, *field, window, cx);
            shortcuts_content = shortcuts_content.child(row);
            if index < action_rows.len() - 1 {
                shortcuts_content = shortcuts_content.child(setting_card::SettingDivider::new());
            }
        }

        let reset_listener = cx.listener(|this: &mut SettingsWindow, _evt, window, cx| {
            this.reset_shortcut_fields(window, cx);
        });
        let reset_row = h_flex().justify_end().py_3().px_4().child(
            crate::ui::base::settings_button::SettingsButton::new("shortcut-reset")
                .label("恢复默认设置")
                .on_click(reset_listener),
        );

        shortcuts_content = shortcuts_content.child(setting_card::SettingDivider::new());
        shortcuts_content = shortcuts_content.child(reset_row);

        v_flex()
            .gap_6()
            .child(setting_card::SettingCard::new(shortcuts_content))
    }
}

impl SettingsWindow {
    fn render_shortcut_send_selector(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let label = self.shortcut_send_selection_label();
        let settings = cx.entity();

        Popover::new("shortcut-send-popover")
            .appearance(false)
            .anchor(Anchor::BottomLeft)
            .trigger(Self::general_select_trigger_button(
                "shortcut-send-btn",
                label,
                cx,
            ))
            .content(move |_, _window, cx| {
                let hover_state = settings.read(cx).shortcut_send_hover_state();
                let enter_hovered = matches!(hover_state, ShortcutSendHover::Enter);
                let ctrl_hovered = matches!(hover_state, ShortcutSendHover::CtrlEnter);

                let settings_for_enter_hover = settings.clone();
                let set_enter_hover = move |is_hovering: bool, cx: &mut App| {
                    _ = settings_for_enter_hover.update(cx, |this: &mut SettingsWindow, cx| {
                        if is_hovering {
                            this.set_shortcut_send_hover(ShortcutSendHover::Enter);
                        } else {
                            this.clear_shortcut_send_hover(ShortcutSendHover::Enter);
                        }
                        cx.notify();
                    });
                };

                let settings_for_ctrl_hover = settings.clone();
                let set_ctrl_hover = move |is_hovering: bool, cx: &mut App| {
                    _ = settings_for_ctrl_hover.update(cx, |this: &mut SettingsWindow, cx| {
                        if is_hovering {
                            this.set_shortcut_send_hover(ShortcutSendHover::CtrlEnter);
                        } else {
                            this.clear_shortcut_send_hover(ShortcutSendHover::CtrlEnter);
                        }
                        cx.notify();
                    });
                };

                let settings_for_enter_click = settings.clone();
                let enter_click = cx.listener(move |_, _, _window, cx| {
                    _ = settings_for_enter_click.update(cx, |this: &mut SettingsWindow, cx| {
                        this.set_shortcut_send_selection("Enter");
                        this.clear_shortcut_send_hover(ShortcutSendHover::Enter);
                        cx.notify();
                    });
                    cx.emit(gpui::DismissEvent);
                });

                let settings_for_ctrl_click = settings.clone();
                let ctrl_click = cx.listener(move |_, _, _window, cx| {
                    _ = settings_for_ctrl_click.update(cx, |this: &mut SettingsWindow, cx| {
                        this.set_shortcut_send_selection("Ctrl + Enter");
                        this.clear_shortcut_send_hover(ShortcutSendHover::CtrlEnter);
                        cx.notify();
                    });
                    cx.emit(gpui::DismissEvent);
                });

                let theme = cx.theme();

                v_flex()
                    .w(crate::ui::constants::popover_width_md())
                    .gap_0()
                    .py_2()
                    .bg(theme.popover)
                    .p_1()
                    .rounded(crate::ui::constants::radius_md())
                    .shadow_md()
                    .child(Self::render_static_select_item(
                        "shortcut-send-enter",
                        "Enter",
                        enter_hovered,
                        set_enter_hover,
                        enter_click,
                    ))
                    .child(Self::render_static_select_item(
                        "shortcut-send-ctrl-enter",
                        "Ctrl\u{00A0}+\u{00A0}Enter",
                        ctrl_hovered,
                        set_ctrl_hover,
                        ctrl_click,
                    ))
            })
    }

    fn shortcut_input_row(
        &self,
        label: &'static str,
        field: ShortcutInputField,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let theme = cx.theme();
        let foreground = theme.foreground;
        let weixin_colors = Theme::weixin_colors(cx);
        let input_bg = weixin_colors.input_field_bg;
        let input_focus_color = weixin_colors.input_field_focus;
        let default_border_color = theme.border;
        let input_border_color = if self.is_shortcut_input_focused(field) {
            input_focus_color
        } else {
            default_border_color
        };
        let blur_field = field;
        let blur_listener = cx.listener(
            move |this: &mut SettingsWindow, _: &MouseDownEvent, window, cx| {
                if this.is_shortcut_input_focused(blur_field) {
                    this.blur_focus(window, cx);
                }
            },
        );
        let input_state = self.shortcut_input_state(field).clone();
        let icon_color = gpui::rgb(0x7b7b7b);
        let display_text = self.shortcut_display_text(field, cx);
        let is_focused = self.is_shortcut_input_focused(field);
        let wrapper_width = self.shortcut_input_width(field);
        let icon_field = field;
        let icon_listener = cx.listener(move |this: &mut SettingsWindow, _, window, cx| {
            this.set_shortcut_field_text(icon_field, "点击设置", window, cx);
        });
        let show_icon =
            !is_focused && display_text != SHORTCUT_INPUT_PLACEHOLDER && display_text != "点击设置";

        setting_card::setting_row()
            .py_3()
            .child(div().text_sm().text_color(foreground).child(label))
            .child(
                h_flex().items_center().child(
                    div()
                        .w(wrapper_width)
                        .rounded(crate::ui::constants::radius_sm())
                        .border_1()
                        .border_color(input_border_color)
                        .bg(input_bg)
                        .on_mouse_down_out(blur_listener)
                        .child({
                            let mut row = h_flex().items_center().gap(px(0.5)).px(px(4.));
                            row = row.child(
                                Input::new(&input_state)
                                    .xsmall()
                                    .flex_1()
                                    .appearance(false)
                                    .focus_bordered(false)
                                    .bordered(false)
                                    .text_xs()
                                    .text_color(foreground)
                                    .pl(px(3.))
                                    .pr(px(1.5)),
                            );
                            if show_icon {
                                row = row.child(
                                    div()
                                        .cursor_pointer()
                                        .rounded(crate::ui::constants::radius_sm())
                                        .px(px(6.))
                                        .py(px(2.))
                                        .on_mouse_down(MouseButton::Left, icon_listener)
                                        .child(
                                            Icon::default()
                                                .path("setting/close_fill.svg")
                                                .text_color(icon_color),
                                        ),
                                );
                            }
                            row
                        }),
                ),
            )
    }
}

use crate::app::config::Preferences;
use crate::ui::composites::setting_card;
use crate::ui::theme::{Theme, ThemeMode};
use gpui::{Bounds, DismissEvent, EventEmitter, InteractiveElement, *};
use gpui_component::{
    ActiveTheme, Icon, Sizable, Size, WindowExt,
    button::{Button, ButtonCustomVariant, ButtonVariants as _},
    h_flex,
    input::{InputEvent, InputState},
    popover::Popover,
    scroll::ScrollableElement,
    switch::Switch,
    v_flex,
};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};

pub(crate) const SHORTCUT_INPUT_PLACEHOLDER: &str = "按下快捷键";
const SHORTCUT_SCREENSHOT_DEFAULT: &str = "Ctrl + Alt + A";
const SHORTCUT_LOCK_DEFAULT: &str = "Ctrl + Alt + L";
const SHORTCUT_TOGGLE_DEFAULT: &str = "Ctrl + Alt + W";

pub static SETTINGS_WINDOW_OPEN: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Clone, Copy, PartialEq)]
enum SettingsTab {
    AccountAndStorage,
    General,
    Shortcuts,
    Notifications,
    Plugins,
    About,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ThemeHover {
    None,
    Light,
    Dark,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LanguageHover {
    None,
    ChineseSimplified,
    ChineseTraditional,
    English,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TranslateLanguageHover {
    None,
    SimplifiedChinese,
    TraditionalChinese,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ShortcutSendHover {
    None,
    Enter,
    CtrlEnter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum ShortcutInputField {
    Screenshot,
    Lock,
    ToggleWindow,
}

pub struct SettingsWindow {
    active_tab_ix: usize,
    root_focus_handle: gpui::FocusHandle,

    /// Layout bounds of the custom font slider bar, used for drag position mapping。
    font_slider_bounds: Bounds<Pixels>,
    /// 当前是否在拖动字体大小滑块。
    is_font_slider_dragging: bool,

    theme_hover: ThemeHover,
    language_hover: LanguageHover,
    translate_language_selection: String,
    translate_language_hover: TranslateLanguageHover,
    auto_download_limit_input: Entity<InputState>,
    auto_download_input_focused: bool,
    _auto_download_input_subscription: gpui::Subscription,
    shortcut_send_selection: String,
    shortcut_send_hover: ShortcutSendHover,
    shortcut_screenshot_input: Entity<InputState>,
    shortcut_lock_input: Entity<InputState>,
    shortcut_toggle_input: Entity<InputState>,
    shortcut_display_texts: HashMap<ShortcutInputField, String>,
    shortcut_input_widths: HashMap<ShortcutInputField, Pixels>,
    focused_shortcut_input: Option<ShortcutInputField>,
    _shortcut_input_subscriptions: Vec<gpui::Subscription>,
}

impl EventEmitter<DismissEvent> for SettingsWindow {}

impl Focusable for SettingsWindow {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.root_focus_handle.clone()
    }
}

impl SettingsWindow {
    pub fn new(_window: &mut Window, cx: &mut Context<Self>) -> Self {
        let root_focus_handle = cx.focus_handle();

        // 字体大小 Slider：0..8 共 9 档，根据当前全局字体大小计算初始档位
        // let current_font_size: f32 = cx.theme().font_size.into();
        // let initial_font_size = current_font_size;

        let auto_download_limit_input = cx.new(|cx| InputState::new(_window, cx).placeholder("20"));
        auto_download_limit_input.update(cx, |state, cx| {
            state.set_value("20", _window, cx);
        });
        let auto_download_input_subscription = cx.subscribe_in(
            &auto_download_limit_input,
            _window,
            |this, _, event: &InputEvent, _window, cx| {
                this.handle_auto_download_input_focus(event, cx);
            },
        );

        gpui_component::Theme::global_mut(cx).ring = Theme::weixin_colors(cx).input_field_focus;

        let shortcut_screenshot_input =
            cx.new(|cx| InputState::new(_window, cx).placeholder(SHORTCUT_INPUT_PLACEHOLDER));
        shortcut_screenshot_input.update(cx, |state, cx| {
            state.set_value(SHORTCUT_SCREENSHOT_DEFAULT, _window, cx);
        });

        let shortcut_lock_input =
            cx.new(|cx| InputState::new(_window, cx).placeholder(SHORTCUT_INPUT_PLACEHOLDER));
        shortcut_lock_input.update(cx, |state, cx| {
            state.set_value(SHORTCUT_LOCK_DEFAULT, _window, cx);
        });

        let shortcut_toggle_input =
            cx.new(|cx| InputState::new(_window, cx).placeholder(SHORTCUT_INPUT_PLACEHOLDER));
        shortcut_toggle_input.update(cx, |state, cx| {
            state.set_value(SHORTCUT_TOGGLE_DEFAULT, _window, cx);
        });

        let mut shortcut_display_texts = HashMap::new();
        shortcut_display_texts.insert(
            ShortcutInputField::Screenshot,
            SHORTCUT_SCREENSHOT_DEFAULT.to_string(),
        );
        shortcut_display_texts.insert(ShortcutInputField::Lock, SHORTCUT_LOCK_DEFAULT.to_string());
        shortcut_display_texts.insert(
            ShortcutInputField::ToggleWindow,
            SHORTCUT_TOGGLE_DEFAULT.to_string(),
        );

        let shortcut_screenshot_subscription = cx.subscribe_in(
            &shortcut_screenshot_input,
            _window,
            |this, _, event: &InputEvent, window, cx| {
                this.handle_shortcut_input_focus(ShortcutInputField::Screenshot, event, window, cx);
            },
        );
        let shortcut_lock_subscription = cx.subscribe_in(
            &shortcut_lock_input,
            _window,
            |this, _, event: &InputEvent, window, cx| {
                this.handle_shortcut_input_focus(ShortcutInputField::Lock, event, window, cx);
            },
        );
        let shortcut_toggle_subscription = cx.subscribe_in(
            &shortcut_toggle_input,
            _window,
            |this, _, event: &InputEvent, window, cx| {
                this.handle_shortcut_input_focus(
                    ShortcutInputField::ToggleWindow,
                    event,
                    window,
                    cx,
                );
            },
        );
        let shortcut_input_subscriptions = vec![
            shortcut_screenshot_subscription,
            shortcut_lock_subscription,
            shortcut_toggle_subscription,
        ];

        let mut this = Self {
            active_tab_ix: 1,
            root_focus_handle,
            theme_hover: ThemeHover::None,
            language_hover: LanguageHover::None,
            translate_language_selection: "简体中文".to_string(),
            translate_language_hover: TranslateLanguageHover::None,
            auto_download_limit_input,
            auto_download_input_focused: false,
            _auto_download_input_subscription: auto_download_input_subscription,
            shortcut_send_selection: "Enter".to_string(),
            shortcut_send_hover: ShortcutSendHover::None,
            shortcut_screenshot_input,
            shortcut_lock_input,
            shortcut_toggle_input,
            shortcut_display_texts,
            shortcut_input_widths: HashMap::new(),
            focused_shortcut_input: None,
            _shortcut_input_subscriptions: shortcut_input_subscriptions,
            font_slider_bounds: Bounds::default(),
            is_font_slider_dragging: false,
        };

        this.refresh_all_shortcut_input_widths(_window, cx);
        cx.defer_in(_window, |this, window, cx| {
            this.refresh_all_shortcut_input_widths(window, cx);
        });
        this
    }

    pub fn view(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx))
    }

    pub(crate) fn auto_download_limit_input(&self) -> &Entity<InputState> {
        &self.auto_download_limit_input
    }

    pub(crate) fn blur_focus(&self, window: &mut Window, cx: &mut App) {
        self.root_focus_handle.focus(window, cx);
    }

    pub(crate) fn is_auto_download_input_focused(&self) -> bool {
        self.auto_download_input_focused
    }

    pub(crate) fn shortcut_send_selection_label(&self) -> String {
        self.shortcut_send_selection.clone()
    }

    pub(crate) fn shortcut_send_hover_state(&self) -> ShortcutSendHover {
        self.shortcut_send_hover
    }

    pub(crate) fn translate_language_selection_label(&self) -> String {
        self.translate_language_selection.clone()
    }

    pub(crate) fn translate_language_hover_state(&self) -> TranslateLanguageHover {
        self.translate_language_hover
    }

    pub(crate) fn set_shortcut_send_hover(&mut self, hover: ShortcutSendHover) {
        self.shortcut_send_hover = hover;
    }

    pub(crate) fn clear_shortcut_send_hover(&mut self, hover: ShortcutSendHover) {
        if self.shortcut_send_hover == hover {
            self.shortcut_send_hover = ShortcutSendHover::None;
        }
    }

    pub(crate) fn set_shortcut_send_selection(&mut self, value: &str) {
        self.shortcut_send_selection = value.to_string();
    }

    pub(crate) fn set_translate_language_selection(&mut self, value: &str) {
        self.translate_language_selection = value.to_string();
    }

    pub(crate) fn set_translate_language_hover(&mut self, hover: TranslateLanguageHover) {
        self.translate_language_hover = hover;
    }

    pub(crate) fn clear_translate_language_hover(&mut self, hover: TranslateLanguageHover) {
        if self.translate_language_hover == hover {
            self.translate_language_hover = TranslateLanguageHover::None;
        }
    }

    pub(crate) fn shortcut_input_state(&self, field: ShortcutInputField) -> &Entity<InputState> {
        match field {
            ShortcutInputField::Screenshot => &self.shortcut_screenshot_input,
            ShortcutInputField::Lock => &self.shortcut_lock_input,
            ShortcutInputField::ToggleWindow => &self.shortcut_toggle_input,
        }
    }

    pub(crate) fn set_shortcut_field_text(
        &mut self,
        field: ShortcutInputField,
        value: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let owned = Self::normalize_shortcut_text(value);
        self.shortcut_display_texts.insert(field, owned.clone());
        let state = self.shortcut_input_state(field).clone();
        let display_text = owned.clone();
        state.update(cx, |input, cx| {
            input.set_value(display_text.clone(), window, cx);
        });
        if self.refresh_shortcut_input_width(field, window, cx) {
            cx.notify();
        }
    }

    pub(crate) fn reset_shortcut_fields(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.set_shortcut_field_text(
            ShortcutInputField::Screenshot,
            SHORTCUT_SCREENSHOT_DEFAULT,
            window,
            cx,
        );
        self.set_shortcut_field_text(ShortcutInputField::Lock, SHORTCUT_LOCK_DEFAULT, window, cx);
        self.set_shortcut_field_text(
            ShortcutInputField::ToggleWindow,
            SHORTCUT_TOGGLE_DEFAULT,
            window,
            cx,
        );
    }

    pub(crate) fn shortcut_input_width(&self, field: ShortcutInputField) -> Pixels {
        self.shortcut_input_widths
            .get(&field)
            .copied()
            .unwrap_or_else(|| crate::ui::constants::settings_shortcut_input_min_width())
    }

    pub(crate) fn shortcut_display_text(
        &self,
        field: ShortcutInputField,
        cx: &mut Context<Self>,
    ) -> String {
        if self.is_shortcut_input_focused(field) {
            let live_text = self.read_shortcut_input_text(field, cx);
            return if live_text.is_empty() {
                SHORTCUT_INPUT_PLACEHOLDER.to_string()
            } else {
                live_text
            };
        }

        let text = self
            .shortcut_display_texts
            .get(&field)
            .cloned()
            .unwrap_or_else(|| self.read_shortcut_input_text(field, cx));
        if text.is_empty() {
            SHORTCUT_INPUT_PLACEHOLDER.to_string()
        } else {
            text
        }
    }

    fn read_shortcut_input_text(
        &self,
        field: ShortcutInputField,
        cx: &mut Context<Self>,
    ) -> String {
        let state = self.shortcut_input_state(field).clone();
        state.read_with(cx, |input, _| input.text().to_string())
    }

    fn normalize_shortcut_text(text: &str) -> String {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            String::new()
        } else {
            trimmed.split_whitespace().collect::<Vec<_>>().join(" ")
        }
    }

    fn commit_shortcut_input_text(
        &mut self,
        field: ShortcutInputField,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        let raw = self.read_shortcut_input_text(field, cx);
        let normalized = Self::normalize_shortcut_text(&raw);
        let current_display = self
            .shortcut_display_texts
            .get(&field)
            .cloned()
            .unwrap_or_default();
        let final_text = if normalized.is_empty() {
            current_display
        } else {
            normalized
        };
        self.shortcut_display_texts
            .insert(field, final_text.clone());
        let state = self.shortcut_input_state(field).clone();
        state.update(cx, |input, cx| {
            input.set_value(final_text.clone(), window, cx);
        });
        self.refresh_shortcut_input_width(field, window, cx)
    }

    fn refresh_all_shortcut_input_widths(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let fields = [
            ShortcutInputField::Screenshot,
            ShortcutInputField::Lock,
            ShortcutInputField::ToggleWindow,
        ];
        for field in fields {
            self.refresh_shortcut_input_width(field, window, cx);
        }
    }

    fn refresh_shortcut_input_width(
        &mut self,
        field: ShortcutInputField,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        let width = self.measure_shortcut_input_width(field, window, cx);
        let changed = self
            .shortcut_input_widths
            .get(&field)
            .map(|current| *current != width)
            .unwrap_or(true);
        self.shortcut_input_widths.insert(field, width);
        changed
    }

    fn font_size_from_index(index: i32) -> f32 {
        match index {
            0 => 15.0,
            1 => 16.0, // 标准
            2 => 17.0,
            3 => 18.0,
            4 => 19.0,
            5 => 20.0,
            6 => 21.0,
            7 => 22.0,
            8 => 23.0,
            _ => 16.0,
        }
    }

    fn font_index_from_size(size: f32) -> i32 {
        let rounded = size.round() as i32;
        match rounded {
            15 => 0,
            16 => 1, // 标准
            17 => 2,
            18 => 3,
            19 => 4,
            20 => 5,
            21 => 6,
            22 => 7,
            23 => 8,
            _ => 1,
        }
    }

    fn measure_shortcut_input_width(
        &self,
        field: ShortcutInputField,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Pixels {
        let theme = cx.theme();
        let foreground = theme.foreground;
        let icon_color = gpui::rgb(0x7b7b7b);
        let min_width = crate::ui::constants::settings_shortcut_input_min_width();
        let max_width = crate::ui::constants::settings_shortcut_input_max_width();
        let display_text = self.shortcut_display_text(field, cx);

        let mut measure_variant = |text: &str, include_icon: bool| {
            let mut row = h_flex().items_center().gap(px(0.5)).px(px(4.));
            row = row.child(
                div()
                    .pl(px(3.))
                    .pr(px(1.5))
                    .whitespace_nowrap()
                    .text_xs()
                    .text_color(foreground)
                    .child(text.to_string()),
            );
            if include_icon {
                row = row.child(
                    div()
                        .rounded(crate::ui::constants::radius_sm())
                        .px(px(6.))
                        .py(px(2.))
                        .child(
                            Icon::default()
                                .path("setting/close_fill.svg")
                                .text_color(icon_color),
                        ),
                );
            }

            let mut element = div()
                .rounded(crate::ui::constants::radius_sm())
                .border_1()
                .border_color(gpui::transparent_black())
                .max_w(max_width)
                .child(row)
                .into_any_element();
            let available_space = size(AvailableSpace::MaxContent, AvailableSpace::MinContent);
            element.layout_as_root(available_space, window, cx).width
        };

        let show_icon = display_text != SHORTCUT_INPUT_PLACEHOLDER && display_text != "点击设置";

        let padding_adjustment = px(10.);
        let mut measured_width =
            measure_variant(display_text.as_str(), show_icon) + padding_adjustment;
        let enforce_min_width =
            display_text != "点击设置" && display_text != SHORTCUT_INPUT_PLACEHOLDER;
        if enforce_min_width {
            measured_width = measured_width.max(min_width);
        }
        measured_width = measured_width.min(max_width);

        measured_width
    }

    pub(crate) fn is_shortcut_input_focused(&self, field: ShortcutInputField) -> bool {
        self.focused_shortcut_input == Some(field)
    }

    fn get_active_tab(&self) -> SettingsTab {
        match self.active_tab_ix {
            0 => SettingsTab::AccountAndStorage,
            1 => SettingsTab::General,
            2 => SettingsTab::Shortcuts,
            3 => SettingsTab::Notifications,
            4 => SettingsTab::Plugins,
            5 => SettingsTab::About,
            _ => SettingsTab::General,
        }
    }

    fn set_active_tab(&mut self, ix: usize, _: &mut Window, cx: &mut Context<Self>) {
        self.active_tab_ix = ix;
        cx.notify();
    }

    fn handle_auto_download_input_focus(&mut self, event: &InputEvent, cx: &mut Context<Self>) {
        match event {
            InputEvent::Focus => {
                if !self.auto_download_input_focused {
                    self.auto_download_input_focused = true;
                    cx.notify();
                }
            }
            InputEvent::Blur => {
                if self.auto_download_input_focused {
                    self.auto_download_input_focused = false;
                    cx.notify();
                }
            }
            _ => {}
        }
    }

    fn handle_shortcut_input_focus(
        &mut self,
        field: ShortcutInputField,
        event: &InputEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let state = self.shortcut_input_state(field).clone();
        match event {
            InputEvent::Focus => {
                let mut should_notify = false;
                if self.focused_shortcut_input != Some(field) {
                    self.focused_shortcut_input = Some(field);
                    should_notify = true;
                }
                state.update(cx, |input, cx| {
                    input.set_value("", window, cx);
                });
                if self.refresh_shortcut_input_width(field, window, cx) {
                    should_notify = true;
                }
                if should_notify {
                    cx.notify();
                }
            }
            InputEvent::Blur => {
                let mut should_notify = false;
                if self.focused_shortcut_input == Some(field) {
                    self.focused_shortcut_input = None;
                    should_notify = true;
                }
                if self.commit_shortcut_input_text(field, window, cx) {
                    should_notify = true;
                }
                if should_notify {
                    cx.notify();
                }
            }
            InputEvent::PressEnter { .. } => {
                if self.commit_shortcut_input_text(field, window, cx) {
                    cx.notify();
                }
            }
            _ => {}
        }
    }

    fn render_content(&self, window: &mut Window, cx: &mut Context<Self>) -> gpui::AnyElement {
        match self.get_active_tab() {
            SettingsTab::AccountAndStorage => {
                self.render_account_and_storage_tab(cx).into_any_element()
            }
            SettingsTab::General => self.render_general_tab(window, cx).into_any_element(),
            SettingsTab::Shortcuts => self.render_shortcuts_tab(window, cx).into_any_element(),
            SettingsTab::Notifications => self.render_notifications_tab(cx).into_any_element(),
            SettingsTab::Plugins => self.render_plugins_tab(cx).into_any_element(),
            SettingsTab::About => self.render_about_tab(cx).into_any_element(),
        }
    }

    pub(crate) fn render_theme_setting(
        &self,
        current_mode: ThemeMode,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> gpui::AnyElement {
        let theme = cx.theme();
        let foreground = theme.foreground;

        setting_card::setting_row()
            .py_2()
            .child(div().text_sm().text_color(foreground).child("外观"))
            .child(self.render_theme_button(current_mode, window, cx))
            .into_any_element()
    }

    /// 通用设置中的下拉按钮触发器（语言 / 主题 / 字体），统一使用同一风格。
    pub(crate) fn general_select_trigger_button(
        id: &'static str,
        label: String,
        cx: &mut Context<Self>,
    ) -> Button {
        let weixin_colors = Theme::weixin_colors(cx);
        let foreground = cx.theme().foreground;

        // 亮色：白底 + 边框，hover f2f2f2 / border ebebeb
        // 深色：沿用微信深色布局的搜索/hover/选中颜色
        let (bg, hover, active, border) = Theme::general_select_button_colors(cx);

        let variant = ButtonCustomVariant::new(cx)
            .color(bg)
            .foreground(foreground)
            .hover(hover)
            .active(active);

        Button::new(id)
            .xsmall()
            .h(px(26.))
            .w(px(90.))
            .border_1()
            .border_color(border)
            .custom(variant)
            .child(
                h_flex()
                    .items_center()
                    .gap_2()
                    .text_color(foreground)
                    .text_size(px(11.))
                    .child(label)
                    .child(
                        div()
                            .p(crate::ui::constants::icon_badge_padding_xs())
                            .rounded_sm()
                            .bg(weixin_colors.weixin_green)
                            .child(
                                Icon::default()
                                    .path("arrow.svg")
                                    .text_color(gpui::rgb(0xffffff)),
                            ),
                    ),
            )
    }

    fn render_theme_button(
        &self,
        current_mode: ThemeMode,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let label = match current_mode {
            ThemeMode::Light => "浅色模式".to_string(),
            ThemeMode::Dark => "深色模式".to_string(),
        };

        let settings = cx.entity();

        Popover::new("theme-popover")
            .appearance(false)
            .anchor(gpui::Anchor::BottomLeft)
            .trigger(Self::general_select_trigger_button("theme-btn", label, cx))
            .content(move |_, _window, cx| {
                let theme = cx.theme();

                let theme_hover = settings.read(cx).theme_hover;
                let light_hovered = matches!(theme_hover, ThemeHover::Light);
                let dark_hovered = matches!(theme_hover, ThemeHover::Dark);

                let settings_for_light = settings.clone();
                let set_light_hover = move |is_hovering: bool, cx: &mut App| {
                    _ = settings_for_light.update(cx, |this: &mut SettingsWindow, cx| {
                        this.theme_hover = if is_hovering {
                            ThemeHover::Light
                        } else if matches!(this.theme_hover, ThemeHover::Light) {
                            ThemeHover::None
                        } else {
                            this.theme_hover
                        };
                        cx.notify();
                    });
                };

                let settings_for_dark = settings.clone();
                let set_dark_hover = move |is_hovering: bool, cx: &mut App| {
                    _ = settings_for_dark.update(cx, |this: &mut SettingsWindow, cx| {
                        this.theme_hover = if is_hovering {
                            ThemeHover::Dark
                        } else if matches!(this.theme_hover, ThemeHover::Dark) {
                            ThemeHover::None
                        } else {
                            this.theme_hover
                        };
                        cx.notify();
                    });
                };

                let settings_for_light_click = settings.clone();
                let settings_for_dark_click = settings.clone();

                v_flex()
                    .w(crate::ui::constants::popover_width_sm())
                    .gap_0()
                    .py_2()
                    .bg(theme.popover)
                    .p_1()
                    .rounded(crate::ui::constants::radius_md())
                    .shadow_md()
                    .child(Self::render_static_theme_item(
                        "theme-light",
                        "浅色模式",
                        light_hovered,
                        set_light_hover,
                        cx.listener(move |_, _, window, cx| {
                            Theme::set_light(cx);
                            Preferences::save_from_app(cx);
                            cx.refresh_windows();
                            window.push_notification("切换到浅色主题", cx);
                            cx.emit(gpui::DismissEvent);
                            _ = settings_for_light_click.update(
                                cx,
                                |this: &mut SettingsWindow, cx| {
                                    this.theme_hover = ThemeHover::None;
                                    cx.notify();
                                },
                            );
                        }),
                    ))
                    .child(Self::render_static_theme_item(
                        "theme-dark",
                        "深色模式",
                        dark_hovered,
                        set_dark_hover,
                        cx.listener(move |_, _, window, cx| {
                            Theme::set_dark(cx);
                            Preferences::save_from_app(cx);
                            cx.refresh_windows();
                            window.push_notification("切换到深色主题", cx);
                            cx.emit(gpui::DismissEvent);
                            _ = settings_for_dark_click.update(
                                cx,
                                |this: &mut SettingsWindow, cx| {
                                    this.theme_hover = ThemeHover::None;
                                    cx.notify();
                                },
                            );
                        }),
                    ))
            })
    }

    fn render_static_theme_item<FSet, L>(
        id: &'static str,
        label: &'static str,
        hovered: bool,
        set_hover: FSet,
        on_mouse_down: L,
    ) -> impl IntoElement
    where
        FSet: Fn(bool, &mut App) + Clone + 'static,
        L: Fn(&gpui::MouseDownEvent, &mut Window, &mut App) + 'static,
    {
        Self::render_static_select_item(id, label, hovered, set_hover, on_mouse_down)
    }

    pub(crate) fn render_static_select_item<FSet, L>(
        id: &'static str,
        label: &'static str,
        hovered: bool,
        set_hover: FSet,
        on_mouse_down: L,
    ) -> impl IntoElement
    where
        FSet: Fn(bool, &mut App) + Clone + 'static,
        L: Fn(&gpui::MouseDownEvent, &mut Window, &mut App) + 'static,
    {
        crate::ui::base::menu_item::MenuItem::new(id, label)
            .hovered(hovered)
            .compact(true)
            .on_hover(set_hover)
            .on_click(on_mouse_down)
    }

    pub(crate) fn render_setting_row(
        &self,
        label: &'static str,
        enabled: bool,
        cx: &Context<Self>,
    ) -> impl IntoElement {
        let theme = cx.theme();

        setting_card::setting_row()
            .py_2()
            .child(div().text_sm().text_color(theme.foreground).child(label))
            .child(Switch::new(label).checked(enabled).with_size(Size::Small))
    }

    fn render_static_language_item<FSet, L>(
        id: &'static str,
        label: &'static str,
        hovered: bool,
        set_hover: FSet,
        on_mouse_down: L,
    ) -> impl IntoElement
    where
        FSet: Fn(bool, &mut App) + Clone + 'static,
        L: Fn(&gpui::MouseDownEvent, &mut Window, &mut App) + 'static,
    {
        Self::render_static_select_item(id, label, hovered, set_hover, on_mouse_down)
    }

    pub(crate) fn render_language_button(
        &self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let current_language = "简体中文".to_string();
        let settings = cx.entity();

        Popover::new("language-popover")
            .appearance(false)
            .anchor(gpui::Anchor::BottomRight)
            .trigger(Self::general_select_trigger_button(
                "language-btn",
                current_language.clone(),
                cx,
            ))
            .content(move |_, _window, cx| {
                let theme = cx.theme();

                let language_hover = settings.read(cx).language_hover;
                let chinese_hovered = matches!(language_hover, LanguageHover::ChineseSimplified);
                let traditional_hovered =
                    matches!(language_hover, LanguageHover::ChineseTraditional);
                let english_hovered = matches!(language_hover, LanguageHover::English);

                let settings_for_chinese = settings.clone();
                let set_chinese_hover = move |is_hovering: bool, cx: &mut App| {
                    _ = settings_for_chinese.update(cx, |this: &mut SettingsWindow, cx| {
                        this.language_hover = if is_hovering {
                            LanguageHover::ChineseSimplified
                        } else if matches!(this.language_hover, LanguageHover::ChineseSimplified) {
                            LanguageHover::None
                        } else {
                            this.language_hover
                        };
                        cx.notify();
                    });
                };

                let settings_for_traditional = settings.clone();
                let set_traditional_hover = move |is_hovering: bool, cx: &mut App| {
                    _ = settings_for_traditional.update(cx, |this: &mut SettingsWindow, cx| {
                        this.language_hover = if is_hovering {
                            LanguageHover::ChineseTraditional
                        } else if matches!(this.language_hover, LanguageHover::ChineseTraditional) {
                            LanguageHover::None
                        } else {
                            this.language_hover
                        };
                        cx.notify();
                    });
                };

                let settings_for_english = settings.clone();
                let set_english_hover = move |is_hovering: bool, cx: &mut App| {
                    _ = settings_for_english.update(cx, |this: &mut SettingsWindow, cx| {
                        this.language_hover = if is_hovering {
                            LanguageHover::English
                        } else if matches!(this.language_hover, LanguageHover::English) {
                            LanguageHover::None
                        } else {
                            this.language_hover
                        };
                        cx.notify();
                    });
                };

                let settings_for_chinese_click = settings.clone();
                let settings_for_traditional_click = settings.clone();
                let settings_for_english_click = settings.clone();

                v_flex()
                    .w(crate::ui::constants::popover_width_md())
                    .gap_0()
                    .py_2()
                    .bg(theme.popover)
                    .p_1()
                    .rounded(crate::ui::constants::radius_md())
                    .shadow_md()
                    .child(Self::render_static_language_item(
                        "lang-chinese",
                        "简体中文",
                        chinese_hovered,
                        set_chinese_hover,
                        cx.listener(move |_, _, window, cx| {
                            window.push_notification("语言切换功能开发中...", cx);
                            cx.emit(gpui::DismissEvent);
                            _ = settings_for_chinese_click.update(
                                cx,
                                |this: &mut SettingsWindow, cx| {
                                    this.language_hover = LanguageHover::None;
                                    cx.notify();
                                },
                            );
                        }),
                    ))
                    .child(Self::render_static_language_item(
                        "lang-traditional",
                        "繁體中文",
                        traditional_hovered,
                        set_traditional_hover,
                        cx.listener(move |_, _, window, cx| {
                            window.push_notification("语言切换功能开发中...", cx);
                            cx.emit(gpui::DismissEvent);
                            _ = settings_for_traditional_click.update(
                                cx,
                                |this: &mut SettingsWindow, cx| {
                                    this.language_hover = LanguageHover::None;
                                    cx.notify();
                                },
                            );
                        }),
                    ))
                    .child(Self::render_static_language_item(
                        "lang-english",
                        "English",
                        english_hovered,
                        set_english_hover,
                        cx.listener(move |_, _, window, cx| {
                            window.push_notification("语言切换功能开发中...", cx);
                            cx.emit(gpui::DismissEvent);
                            _ = settings_for_english_click.update(
                                cx,
                                |this: &mut SettingsWindow, cx| {
                                    this.language_hover = LanguageHover::None;
                                    cx.notify();
                                },
                            );
                        }),
                    ))
            })
    }

    pub(crate) fn render_translate_language_button(
        &self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let label = self.translate_language_selection_label();
        let settings = cx.entity();

        Popover::new("translate-language-popover")
            .appearance(false)
            .anchor(gpui::Anchor::BottomRight)
            .trigger(Self::general_select_trigger_button(
                "translate-language-btn",
                label,
                cx,
            ))
            .content(move |_, _window, cx| {
                let theme = cx.theme();

                let hover_state = settings.read(cx).translate_language_hover_state();
                let simplified_hovered =
                    matches!(hover_state, TranslateLanguageHover::SimplifiedChinese);
                let traditional_hovered =
                    matches!(hover_state, TranslateLanguageHover::TraditionalChinese);

                let settings_for_simplified_hover = settings.clone();
                let set_simplified_hover = move |is_hovering: bool, cx: &mut App| {
                    _ = settings_for_simplified_hover.update(
                        cx,
                        |this: &mut SettingsWindow, cx| {
                            if is_hovering {
                                this.set_translate_language_hover(
                                    TranslateLanguageHover::SimplifiedChinese,
                                );
                            } else {
                                this.clear_translate_language_hover(
                                    TranslateLanguageHover::SimplifiedChinese,
                                );
                            }
                            cx.notify();
                        },
                    );
                };

                let settings_for_traditional_hover = settings.clone();
                let set_traditional_hover = move |is_hovering: bool, cx: &mut App| {
                    _ = settings_for_traditional_hover.update(
                        cx,
                        |this: &mut SettingsWindow, cx| {
                            if is_hovering {
                                this.set_translate_language_hover(
                                    TranslateLanguageHover::TraditionalChinese,
                                );
                            } else {
                                this.clear_translate_language_hover(
                                    TranslateLanguageHover::TraditionalChinese,
                                );
                            }
                            cx.notify();
                        },
                    );
                };

                let settings_for_simplified_click = settings.clone();
                let simplified_click = cx.listener(move |_, _, _window, cx| {
                    _ = settings_for_simplified_click.update(
                        cx,
                        |this: &mut SettingsWindow, cx| {
                            this.set_translate_language_selection("简体中文");
                            this.clear_translate_language_hover(
                                TranslateLanguageHover::SimplifiedChinese,
                            );
                            cx.notify();
                        },
                    );
                    cx.emit(gpui::DismissEvent);
                });

                let settings_for_traditional_click = settings.clone();
                let traditional_click = cx.listener(move |_, _, _window, cx| {
                    _ = settings_for_traditional_click.update(
                        cx,
                        |this: &mut SettingsWindow, cx| {
                            this.set_translate_language_selection("繁体中文");
                            this.clear_translate_language_hover(
                                TranslateLanguageHover::TraditionalChinese,
                            );
                            cx.notify();
                        },
                    );
                    cx.emit(gpui::DismissEvent);
                });

                v_flex()
                    .w(crate::ui::constants::popover_width_sm())
                    .gap_0()
                    .py_2()
                    .bg(theme.popover)
                    .p_1()
                    .rounded(crate::ui::constants::radius_md())
                    .shadow_md()
                    .child(Self::render_static_language_item(
                        "translate-language-simplified",
                        "简体中文",
                        simplified_hovered,
                        set_simplified_hover,
                        simplified_click,
                    ))
                    .child(Self::render_static_language_item(
                        "translate-language-traditional",
                        "繁体中文",
                        traditional_hovered,
                        set_traditional_hover,
                        traditional_click,
                    ))
            })
    }

    /// 根据当前字体大小像素值，返回对应的档位索引（0..=8）。
    fn current_font_size_index(&self, cx: &App) -> usize {
        let size: f32 = cx.theme().font_size.into();
        Self::font_index_from_size(size) as usize
    }

    /// 设置字体大小档位，并同步到全局主题与偏好配置。
    fn set_font_size_index(&mut self, index: usize, cx: &mut Context<Self>) {
        let clamped = if index > 8 { 8 } else { index } as i32;
        let size = Self::font_size_from_index(clamped);

        gpui_component::Theme::global_mut(cx).font_size = px(size);
        Preferences::save_from_app(cx);
        cx.refresh_windows();
        cx.notify();
    }

    /// 根据鼠标在窗口中的位置，计算对应的字体档位索引（0..=8）。
    fn font_slider_index_from_position(&self, position: Point<Pixels>, cx: &App) -> usize {
        let bounds = self.font_slider_bounds;
        let total_width = bounds.size.width;
        if total_width <= px(0.) {
            return self.current_font_size_index(cx);
        }

        // 将全局坐标转换为滑块内部坐标，并限制在 [0, total_width] 范围内。
        let inner_x = position.x - bounds.left();
        let inner_x = inner_x.clamp(px(0.), total_width);

        // 归一化到 [0, 1]，再映射到 0..=8 的 9 个刻度；round 保证“超过一半才切换到下一档”。
        let frac = inner_x / total_width; // 0.0..=1.0
        let pos = frac * 8.0; // 0.0..=8.0 （tick 0..8）

        let idx = pos.round().clamp(0.0, 8.0) as i32;
        idx as usize
    }

    /// 单个段的渲染：用 div 自行绘制一段进度条，并在当前档位上显示滑块和刻度。
    fn render_font_size_step(
        &self,
        step_index: usize,
        total_steps: usize,
        current_index: usize,
        active_color: Hsla,
        inactive_color: Hsla,
        thumb_color: Hsla,
        active_tick_color: Hsla,
        inactive_tick_color: Hsla,
    ) -> impl IntoElement {
        // tick 索引 (0..=8) -> 段索引 (0..=7)：
        //  index=0/1 都落在第 0 段，其余 i 落在第 (i-1) 段

        // 已经过的段：索引 < 当前 tick 索引
        let is_filled_segment = step_index < current_index;
        let bg_color = if is_filled_segment {
            active_color
        } else {
            inactive_color
        };
        // 刻度本身从第 0 根到当前档位都用高亮色，保证当前档位的刻度始终是高亮的。
        let tick_color = if step_index <= current_index {
            active_tick_color
        } else {
            inactive_tick_color
        };

        // 横向一段进度条 + 上方刻度 + 当前档位的白色滑块
        let mut segment = div()
            .flex_1()
            // 中间线更细一点，方便刻度上下居中
            .h(px(3.))
            .relative()
            .bg(bg_color)
            .cursor_pointer()
            // 左侧刻度（每一段都有），高度略大于中线，并上下居中
            .child(
                div()
                    .absolute()
                    .top(px(-2.5)) // 3px 在线上方 + 6px 在线下方，和 3px 线居中
                    .w(px(3.)) // 稍微粗一点
                    .h(px(9.))
                    .left(px(0.))
                    .bg(tick_color),
            );

        // 最右侧再补一个刻度，保证两端都有刻度
        if step_index + 1 == total_steps {
            let end_tick_color = if step_index + 1 <= current_index {
                active_tick_color
            } else {
                inactive_tick_color
            };

            segment = segment.child(
                div()
                    .absolute()
                    .top(px(-2.5))
                    .w(px(2.))
                    .h(px(10.))
                    .right(px(0.))
                    .bg(end_tick_color),
            );
        }

        // 当前档位上的白色椭圆滑块（左右更扁一点）
        // Case 1: Thumb is at the start of this segment (step_index == current_index)
        if step_index == current_index {
            segment = segment.child(
                div()
                    .absolute()
                    .top(px(-6.5))
                    // 白色滑块的中心对齐当前刻度线：放在当前段的左边界，左右稍微超出一点
                    .left(px(-5.))
                    .w(px(10.))
                    .h(px(18.))
                    .rounded_full()
                    .bg(thumb_color),
            );
        } else if current_index == total_steps && step_index + 1 == total_steps {
            // Case 2: Thumb is at the end of the slider (current_index == total_steps),
            // and this is the last segment (step_index == total_steps - 1)
            segment = segment.child(
                div()
                    .absolute()
                    .top(px(-6.5))
                    .right(px(-5.))
                    .w(px(10.))
                    .h(px(18.))
                    .rounded_full()
                    .bg(thumb_color),
            );
        }

        segment
    }

    /// 字体大小设置滑块：使用 div 模拟，不再使用组件库的 Slider。
    /// 共有 9 档（15–23），不能停在每个间隔的中间，只能落在离散档位上。
    pub(crate) fn render_font_size_slider(
        &self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> gpui::AnyElement {
        let weixin_colors = Theme::weixin_colors(cx);
        let settings = cx.entity();
        let settings_for_down = settings.clone();
        let settings_for_move = settings.clone();
        let settings_for_up = settings.clone();

        let active_color = weixin_colors.weixin_green;
        // 中间线基础颜色：使用 light 主题里的 item_hover（EAEAEA），暗色主题继续用 slider_bar 逻辑
        let inactive_color = match cx.theme().mode {
            crate::ui::theme::ThemeMode::Light => weixin_colors.item_hover,
            _ => cx.theme().slider_bar.opacity(0.2),
        };
        // 刻度基础颜色：同样用 item_hover；经过的部分仍然用 primary 绿色
        let inactive_tick_color = match cx.theme().mode {
            crate::ui::theme::ThemeMode::Light => weixin_colors.settings_button_bg,
            _ => cx.theme().slider_bar.opacity(0.4),
        };
        let active_tick_color: Hsla = weixin_colors.weixin_green;
        // 滑块使用纯白色
        let thumb_color: Hsla = rgb(0xFFFFFF).into();
        let label_color = cx.theme().muted_foreground;
        let current_index = self.current_font_size_index(cx);
        // 段数 = 8（刻度 = 段数 + 1 = 9）
        let total_steps = 8usize;

        // 内部真正的细线条和刻度条
        let bar_inner = h_flex()
            .gap_0()
            .w_full()
            .h(px(3.))
            .rounded(crate::ui::constants::radius_sm())
            .children((0..total_steps).map(|i| {
                self.render_font_size_step(
                    i,
                    total_steps,
                    current_index,
                    active_color,
                    inactive_color,
                    thumb_color,
                    active_tick_color,
                    inactive_tick_color,
                )
            }));

        // 外层放大点击/拖动命中区域（高度更高），事件全部绑在外层容器上
        let bar = div()
            .h(px(18.))
            .w_full()
            .flex()
            .items_center()
            // 让内部 absolute 的 canvas 相对于整个条容器定位，
            // 这样记录到的 bounds 就是整条滑块条本身的宽度和位置
            .relative()
            // 按下时开始拖动，并跳转到对应档位
            .on_mouse_down(
                MouseButton::Left,
                move |e: &gpui::MouseDownEvent, _window: &mut Window, cx: &mut App| {
                    _ = settings_for_down.update(cx, |this: &mut SettingsWindow, cx| {
                        this.is_font_slider_dragging = true;
                        let idx = this.font_slider_index_from_position(e.position, cx);
                        this.set_font_size_index(idx, cx);
                    });
                },
            )
            // 鼠标移动时，如果处于拖动状态则更新档位
            .on_mouse_move(
                move |e: &gpui::MouseMoveEvent, _window: &mut Window, cx: &mut App| {
                    _ = settings_for_move.update(cx, |this: &mut SettingsWindow, cx| {
                        if this.is_font_slider_dragging {
                            let idx = this.font_slider_index_from_position(e.position, cx);
                            this.set_font_size_index(idx, cx);
                        }
                    });
                },
            )
            // 松开鼠标时结束拖动
            .on_mouse_up(
                MouseButton::Left,
                move |_e: &gpui::MouseUpEvent, _window: &mut Window, cx: &mut App| {
                    _ = settings_for_up.update(cx, |this: &mut SettingsWindow, _cx| {
                        this.is_font_slider_dragging = false;
                    });
                },
            )
            .child(bar_inner)
            // 使用 canvas 记录当前滑块的布局边界，用于鼠标位置映射
            .child({
                let settings = settings.clone();
                canvas(
                    move |bounds, _, cx| {
                        _ = settings.update(cx, |this: &mut SettingsWindow, _| {
                            this.font_slider_bounds = bounds;
                        });
                    },
                    |_, _, _, _| {},
                )
                .absolute()
                .size_full()
            });

        v_flex()
            .gap_1()
            .w(px(160.))
            .child(bar)
            .child(
                div()
                    .relative()
                    .w_full()
                    .h(px(14.))
                    .text_size(px(10.))
                    .text_color(label_color)
                    .child(
                        div().absolute().left(px(0.)).child(
                            div()
                                .w(px(0.))
                                .flex()
                                .justify_center()
                                .whitespace_nowrap()
                                .child("小"),
                        ),
                    )
                    .child(
                        div().absolute().left(px(20.)).child(
                            div()
                                .w(px(0.))
                                .flex()
                                .justify_center()
                                .whitespace_nowrap()
                                .child("标准"),
                        ),
                    )
                    .child(
                        div().absolute().right(px(0.)).child(
                            div()
                                .w(px(0.))
                                .flex()
                                .justify_center()
                                .whitespace_nowrap()
                                .child("大"),
                        ),
                    ),
            )
            .into_any_element()
    }

    fn render_tab_item(
        &self,
        ix: usize,
        label: &'static str,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let weixin_colors = Theme::weixin_colors(cx);
        let theme = cx.theme();
        let is_active = self.active_tab_ix == ix;
        let active_bg = weixin_colors.item_selected;
        let hover_bg = weixin_colors.item_hover;
        let transparent_bg = gpui::transparent_black();
        let text_color = theme.foreground;

        let tab_id = match ix {
            0 => "tab-0",
            1 => "tab-1",
            2 => "tab-2",
            3 => "tab-3",
            4 => "tab-4",
            5 => "tab-5",
            _ => "tab-unknown",
        };

        // 为每个设置页 tab 配置一个图标，图标文件位于 assets/setting 目录下。
        let icon_path = match ix {
            // 账号与存储
            0 => "setting/user.svg",
            // 通用
            1 => "setting/setting.svg",
            // 快捷键
            2 => "setting/arrow-up9.svg",
            // 通知
            3 => "setting/notifications.svg",
            // 插件
            4 => "setting/risk.svg",
            // 关于微信
            5 => "setting/about.svg",
            _ => "setting/setting.svg",
        };

        let content = h_flex()
            .items_center()
            .gap_2()
            .child(
                Icon::default()
                    .path(icon_path)
                    .w(crate::ui::constants::icon_sm())
                    .h(crate::ui::constants::icon_sm())
                    .text_color(text_color),
            )
            .child(div().text_sm().text_color(text_color).child(label));

        div()
            .id(tab_id)
            .w_full()
            .px_4()
            .py_2()
            .cursor_pointer()
            .rounded(crate::ui::constants::radius_sm())
            .bg(if is_active { active_bg } else { transparent_bg })
            .hover(move |s| if is_active { s } else { s.bg(hover_bg) })
            .on_click(cx.listener(move |this, _ev, _window, cx| {
                this.set_active_tab(ix, _window, cx);
            }))
            .child(content)
    }
}

impl Drop for SettingsWindow {
    fn drop(&mut self) {
        SETTINGS_WINDOW_OPEN.store(false, Ordering::SeqCst);
    }
}

impl Render for SettingsWindow {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // 固定设置窗口的 rem 基础字号，使其不随全局字体缩放变化
        window.set_rem_size(px(16.0));

        let weixin_colors = Theme::weixin_colors(cx);
        let theme = cx.theme();
        let close_hover = rgb(0xe81123);
        let text_color = theme.foreground;
        let left_bg = weixin_colors.session_list_bg;
        let right_bg = weixin_colors.chat_area_bg;
        let border_color = theme.border;

        h_flex()
            .size_full()
            .child(
                div()
                    .w(px(0.))
                    .h(px(0.))
                    .track_focus(&self.root_focus_handle),
            )
            .child(
                v_flex()
                    .w(crate::ui::constants::settings_sidebar_width())
                    .h_full()
                    .bg(left_bg)
                    .border_r_1()
                    .border_color(border_color)
                    .child(
                        div()
                            .window_control_area(WindowControlArea::Drag)
                            .w_full()
                            .h(crate::ui::constants::settings_title_height())
                            .flex()
                            .items_center()
                            .px_4(),
                    )
                    .child(
                        v_flex()
                            .flex_1()
                            .w_full()
                            .py_4()
                            .px_3()
                            .child(self.render_tab_item(0, "账号与存储", cx))
                            .child(self.render_tab_item(1, "通用", cx))
                            .child(self.render_tab_item(2, "快捷键", cx))
                            .child(self.render_tab_item(3, "通知", cx))
                            .child(self.render_tab_item(4, "插件", cx))
                            .child(self.render_tab_item(5, "关于微信", cx)),
                    ),
            )
            .child(
                v_flex()
                    .flex_1()
                    .h_full()
                    .bg(right_bg)
                    .child(
                        h_flex()
                            .w_full()
                            .h(crate::ui::constants::settings_title_height())
                            .items_center()
                            .child(
                                div()
                                    .window_control_area(WindowControlArea::Drag)
                                    .flex_1()
                                    .h_full(),
                            )
                            .child(
                                div()
                                    .id("settings-close-btn")
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .h_full()
                                    .w(crate::ui::constants::settings_close_button_width())
                                    .window_control_area(WindowControlArea::Close)
                                    .cursor_pointer()
                                    .hover(move |s| s.bg(close_hover).text_color(gpui::white()))
                                    .child(
                                        Icon::default()
                                            .path("window-close.svg")
                                            .text_color(text_color)
                                            .small(),
                                    ),
                            ),
                    )
                    .child(
                        div()
                            .flex_1()
                            .w_full()
                            .h(crate::ui::constants::settings_window_content_height())
                            .relative()
                            .child(
                                v_flex()
                                    .size_full()
                                    .overflow_y_scrollbar()
                                    .p_6()
                                    .child(self.render_content(window, cx)),
                            ),
                    ),
            )
    }
}

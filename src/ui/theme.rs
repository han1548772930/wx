use gpui::{App, Hsla, rgb};
use gpui_component::{ActiveTheme as _, Colorize, Theme as GpuiTheme, ThemeMode as GpuiThemeMode};

pub use gpui_component::ThemeMode;

#[derive(Debug, Clone)]
pub struct WeixinThemeColors {
    pub toolbar_bg: Hsla,
    pub toolbar_button_hover: Hsla,
    pub toolbar_button_active: Hsla,
    pub session_list_bg: Hsla,
    pub chat_area_bg: Hsla,
    pub search_bar_bg: Hsla,
    pub item_hover: Hsla,
    pub item_selected: Hsla,
    pub settings_button_bg: Hsla,
    pub settings_button_hover: Hsla,
    pub settings_button_active: Hsla,
    pub message_bubble_self: Hsla,
    pub message_bubble_other: Hsla,
    pub message_text_self: Hsla,
    pub message_text_other: Hsla,
    pub weixin_green: Hsla,
    pub unread_badge: Hsla,
    pub caret: Hsla,
    pub storage_path_text: Hsla,
    pub input_field_bg: Hsla,
    pub input_field_focus: Hsla,
    pub send_button_disabled_bg: Hsla,
    pub send_button_disabled_text: Hsla,
    pub window_button_hover: Hsla,
    pub popover_bg: Hsla,
    pub chat_button_hover: Hsla,
}

impl WeixinThemeColors {
    fn alpha(color: u32, opacity: f32) -> Hsla {
        let color: Hsla = rgb(color).into();
        color.opacity(opacity)
    }

    pub fn light() -> Self {
        Self {
            toolbar_bg: rgb(0xE6E6E6).into(),
            toolbar_button_hover: Self::alpha(0x000000, 0.06),
            toolbar_button_active: Self::alpha(0x000000, 0.10),
            session_list_bg: rgb(0xF7F7F7).into(),
            chat_area_bg: rgb(0xEDEDED).into(),
            search_bar_bg: rgb(0xEDEDED).into(),
            item_hover: rgb(0xEAEAEA).into(),
            item_selected: rgb(0xDEDEDE).into(),
            settings_button_bg: rgb(0xEAEAEA).into(),
            settings_button_hover: rgb(0xE4E4E4).into(),
            settings_button_active: rgb(0xE4E4E4).into(),
            message_bubble_self: rgb(0x95EC69).into(),
            message_bubble_other: rgb(0xFFFFFF).into(),
            message_text_self: rgb(0x000000).into(),
            message_text_other: rgb(0x333333).into(),
            weixin_green: rgb(0x07C160).into(),
            unread_badge: rgb(0xFA5151).into(),
            caret: rgb(0x07C160).into(),
            storage_path_text: rgb(0x576B95).into(),
            input_field_bg: rgb(0xFFFFFF).into(),
            input_field_focus: rgb(0x44D087).into(),
            send_button_disabled_bg: rgb(0xE1E1E1).into(),
            send_button_disabled_text: rgb(0x9D9D9D).into(),
            window_button_hover: rgb(0xE0E0E0).into(),
            popover_bg: rgb(0xFFFFFF).into(),
            chat_button_hover: rgb(0xE0E0E0).into(),
        }
    }

    pub fn dark() -> Self {
        Self {
            toolbar_bg: rgb(0x2C2C2C).into(),
            toolbar_button_hover: Self::alpha(0xFFFFFF, 0.07),
            toolbar_button_active: Self::alpha(0xFFFFFF, 0.11),
            session_list_bg: rgb(0x242424).into(),
            chat_area_bg: rgb(0x191919).into(),
            search_bar_bg: rgb(0x2F2F2F).into(),
            item_hover: rgb(0x2F2F2F).into(),
            item_selected: rgb(0x3A3A3A).into(),
            settings_button_bg: rgb(0x2F2F2F).into(),
            settings_button_hover: rgb(0x353535).into(),
            settings_button_active: rgb(0x353535).into(),
            message_bubble_self: rgb(0x95EC69).into(),
            message_bubble_other: rgb(0x3A3A3A).into(),
            message_text_self: rgb(0x000000).into(),
            message_text_other: rgb(0xE6E6E6).into(),
            weixin_green: rgb(0x07C160).into(),
            unread_badge: rgb(0xFA5151).into(),
            caret: rgb(0x07C160).into(),
            storage_path_text: rgb(0x7D90A9).into(),
            input_field_bg: rgb(0x2E2E2E).into(),
            input_field_focus: rgb(0x0E9A51).into(),
            send_button_disabled_bg: rgb(0x252525).into(),
            send_button_disabled_text: rgb(0x575757).into(),
            window_button_hover: rgb(0x242424).into(),
            popover_bg: rgb(0x242424).into(),
            chat_button_hover: rgb(0x242424).into(),
        }
    }
}

pub struct Theme;

impl Theme {
    pub fn weixin_colors(cx: &App) -> WeixinThemeColors {
        match cx.theme().mode {
            GpuiThemeMode::Light => WeixinThemeColors::light(),
            GpuiThemeMode::Dark => WeixinThemeColors::dark(),
        }
    }

    pub fn set_light(cx: &mut App) {
        GpuiTheme::change(GpuiThemeMode::Light, None, cx);
        let colors = WeixinThemeColors::light();
        let theme = GpuiTheme::global_mut(cx);

        theme.background = colors.toolbar_bg.lighten(0.07).opacity(0.7);
        theme.caret = colors.caret;
        theme.ring = colors.input_field_focus;
        theme.primary = rgb(0x07C160).into();
        theme.primary_hover = rgb(0x06B75B).into();
        theme.primary_active = colors.weixin_green;
        theme.switch = rgb(0xcccccc).into();
        theme.switch_thumb = rgb(0xffffff).into();
    }

    pub fn set_dark(cx: &mut App) {
        GpuiTheme::change(GpuiThemeMode::Dark, None, cx);
        let colors = WeixinThemeColors::dark();
        let theme = GpuiTheme::global_mut(cx);

        theme.background = colors.toolbar_bg.darken(0.45).opacity(0.85);
        theme.caret = colors.caret;
        theme.ring = colors.input_field_focus;
        theme.primary = rgb(0x07C160).into();
        theme.primary_hover = rgb(0x13C468).into();
        theme.primary_active = colors.weixin_green;
        theme.switch = rgb(0xcccccc).into();
        theme.switch_thumb = rgb(0xffffff).into();
    }

    pub fn general_select_button_colors(cx: &App) -> (Hsla, Hsla, Hsla, Hsla) {
        let colors = Self::weixin_colors(cx);
        match cx.theme().mode {
            GpuiThemeMode::Light => (
                rgb(0xFFFFFF).into(),
                rgb(0xF2F2F2).into(),
                rgb(0xF2F2F2).into(),
                rgb(0xEBEBEB).into(),
            ),
            GpuiThemeMode::Dark => (
                colors.search_bar_bg,
                colors.item_hover,
                colors.item_hover,
                colors.item_selected,
            ),
        }
    }
}

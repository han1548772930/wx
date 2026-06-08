use crate::ui::theme::Theme;
use gpui::{prelude::FluentBuilder, *};
use gpui_component::{
    button::{Button, ButtonCustomVariant, ButtonVariants as _},
    ActiveTheme, Sizable,
};
use std::rc::Rc;

#[derive(IntoElement)]
pub struct SettingsButton {
    id: SharedString,
    label: Option<SharedString>,
    on_click: Option<Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>>,
}

impl SettingsButton {
    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: None,
            on_click: None,
        }
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for SettingsButton {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let foreground = theme.foreground;
        let border_color = theme.border;
        let weixin_colors = Theme::weixin_colors(cx);
        let (bg, hover, active) = (
            weixin_colors.settings_button_bg,
            weixin_colors.settings_button_hover,
            weixin_colors.settings_button_active,
        );

        let settings_button_variant = ButtonCustomVariant::new(cx)
            .color(bg)
            .foreground(foreground)
            .hover(hover)
            .active(active);

        Button::new(self.id)
            .xsmall()
            .p_3()
            .border_1()
            .border_color(border_color)
            .custom(settings_button_variant)
            .when_some(self.label, |this, label| this.label(label))
            .when_some(self.on_click, |this, handler| {
                this.on_click(move |event, window, cx| handler(event, window, cx))
            })
    }
}

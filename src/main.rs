pub mod app;
mod assets;
pub mod components;
mod infra;
mod models;
mod ui;
mod utils;

use assets::Assets;
use gpui_platform::application;

fn main() {
    let app = application().with_assets(Assets);

    app.run(move |cx| {
        crate::app::bootstrap::init_app(cx);
        crate::app::bootstrap::open_main_window(cx);
    });
}

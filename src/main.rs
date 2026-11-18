mod app;
mod math;

use gpui_kit::component::{Theme, ThemeMode};
use gpui_kit::*;

use crate::app::AppView;

fn main() {
    gpui_kit::application()
        .with_assets(gpui_kit::assets::Assets)
        .run(|cx| {
            gpui_kit::init(cx);
            install_theme(cx);

            let options = WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds::new(
                    point(px(80.), px(80.)),
                    size(px(1100.), px(720.)),
                ))),
                window_min_size: Some(size(px(640.), px(420.))),
                titlebar: Some(TitlebarOptions {
                    title: Some("typst-math".into()),
                    ..Default::default()
                }),
                ..Default::default()
            };

            gpui_kit::open_window(options, cx, |window, cx| {
                cx.new(|cx| AppView::new(window, cx))
            })
            .expect("failed to open window");
        });
}

/// A flat, dark palette with square corners, close to Zed's editor chrome.
fn install_theme(cx: &mut App) {
    Theme::update(cx, |theme| {
        theme.mode = ThemeMode::Dark;
    });

    Theme::update(cx, |theme| {
        theme.radius = px(0.);
        theme.radius_lg = px(0.);
        theme.window_radius = px(0.);
        theme.shadow = false;
        theme.focus_ring = false;
        theme.font_size = px(14.);
        theme.mono_font_size = px(13.);

        theme.colors.background = rgb(0x1e1e1e).into();
        theme.colors.foreground = rgb(0xd4d4d4).into();
        theme.colors.border = rgb(0x2b2b2b).into();
        theme.colors.muted = rgb(0x262626).into();
        theme.colors.muted_foreground = rgb(0x8a8a8a).into();
        theme.colors.accent = rgb(0x2a2d2e).into();
        theme.colors.accent_foreground = rgb(0xffffff).into();
        theme.colors.caret = rgb(0xd4d4d4).into();
        theme.colors.selection = rgb(0x264f78).into();
        theme.colors.input = rgb(0x1e1e1e).into();
        theme.colors.title_bar = rgb(0x181818).into();
        theme.colors.title_bar_border = rgb(0x2b2b2b).into();
        theme.colors.sidebar = rgb(0x181818).into();
        theme.colors.sidebar_border = rgb(0x2b2b2b).into();
        theme.colors.scrollbar = rgb(0x181818).into();
        theme.colors.scrollbar_thumb = rgb(0x3b3b3b).into();
        theme.colors.scrollbar_thumb_hover = rgb(0x4b4b4b).into();
    });
}

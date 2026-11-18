//! The editor window: a Typst source pane beside a rendered preview.

use std::time::Duration;

use gpui_kit::component::{
    ActiveTheme as _, h_flex,
    input::{Editor, EditorState, InputEvent, TabSize},
    text::{TextView, TextViewState},
    v_flex,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crate::math::{self, MathCache, MathPlugin};

const DEFAULT_SOURCE: &str = "sum_(i=1)^n i^2 = (n (n + 1) (2 n + 1)) / 6";
const DEBOUNCE: Duration = Duration::from_millis(120);

const EDITOR_BACKGROUND: u32 = 0x1e1e1e;
const PANEL_BACKGROUND: u32 = 0x181818;
const ERROR_BACKGROUND: u32 = 0x2a1215;
const ERROR_FOREGROUND: u32 = 0xf48771;

pub struct AppView {
    editor: Entity<EditorState>,
    preview: Entity<TextViewState>,
    cache: MathCache,
    plugin: MathPlugin,
    generation: u64,
    error: Option<String>,
    _subscription: Subscription,
}

impl AppView {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let editor = cx.new(|cx| {
            EditorState::new(window, cx)
                .line_number(false)
                .soft_wrap(true)
                .tab_size(TabSize {
                    tab_size: 2,
                    hard_tabs: false,
                })
                .placeholder("Type a Typst expression")
                .default_value(DEFAULT_SOURCE)
        });

        let cache = MathCache::default();
        let plugin = MathPlugin::new(cache.clone());
        let preview = cx.new(|cx| TextViewState::markdown(&preview_text(DEFAULT_SOURCE), cx));

        let focus = editor.focus_handle(cx);
        window.defer(cx, move |window, cx| focus.focus(window, cx));

        let subscription = cx.subscribe(&editor, |this, _, event: &InputEvent, cx| {
            if matches!(event, InputEvent::Change) {
                this.schedule(cx);
            }
        });

        let mut view = Self {
            editor,
            preview,
            cache,
            plugin,
            generation: 0,
            error: None,
            _subscription: subscription,
        };
        view.schedule(cx);
        view
    }

    /// Compile after a short pause and discard any result that arrived late.
    fn schedule(&mut self, cx: &mut Context<Self>) {
        let source = self.editor.read(cx).value().to_string();
        self.generation = self.generation.wrapping_add(1);
        let generation = self.generation;
        let colour = math::hex(cx.theme().foreground);

        cx.spawn(async move |this, cx| {
            cx.background_executor().timer(DEBOUNCE).await;
            let result = cx
                .background_executor()
                .spawn(async move { math::compile(&source, &colour) })
                .await;

            let _ = this.update(cx, |this, cx| {
                if this.generation != generation {
                    return;
                }

                match result {
                    Ok(math) => {
                        this.cache.set(math);
                        this.error = None;
                    }
                    Err(error) => {
                        this.cache.clear();
                        this.error = Some(error);
                    }
                }

                this.preview
                    .update(cx, |state, cx| state.invalidate_inline_layout(cx));
                cx.notify();
            });
        })
        .detach();
    }
}

impl Render for AppView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let source = self.editor.read(cx).value().to_string();
        let text = preview_text(&source);
        self.preview
            .update(cx, |state, cx| state.set_text(&text, cx));

        let theme = cx.theme();

        v_flex()
            .size_full()
            .bg(theme.background)
            .text_color(theme.foreground)
            .child(
                h_flex()
                    .h(px(34.))
                    .flex_shrink_0()
                    .items_center()
                    .px_3()
                    .bg(rgb(PANEL_BACKGROUND))
                    .border_b_1()
                    .border_color(theme.border)
                    .child(
                        div()
                            .text_sm()
                            .text_color(theme.muted_foreground)
                            .child("typst-math"),
                    ),
            )
            .child(
                h_flex()
                    .flex_1()
                    .min_h_0()
                    .child(
                        div()
                            .h_full()
                            .w(relative(0.46))
                            .min_w_0()
                            .bg(rgb(EDITOR_BACKGROUND))
                            .border_r_1()
                            .border_color(theme.border)
                            .child(
                                div()
                                    .size_full()
                                    .p_4()
                                    .font_family(theme.mono_font_family.clone())
                                    .text_size(theme.mono_font_size)
                                    .child(
                                        Editor::new(&self.editor).h(relative(1.)).p_0().border_0(),
                                    ),
                            ),
                    )
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .h_full()
                            .bg(rgb(PANEL_BACKGROUND))
                            .child(
                                div().size_full().p_6().child(
                                    TextView::new(&self.preview)
                                        .scrollable(true)
                                        .plugin(self.plugin.clone()),
                                ),
                            ),
                    ),
            )
            .when_some(self.error.clone(), |this, error| {
                this.child(
                    div()
                        .flex_shrink_0()
                        .max_h(px(160.))
                        .overflow_hidden()
                        .border_t_1()
                        .border_color(theme.border)
                        .bg(rgb(ERROR_BACKGROUND))
                        .p_3()
                        .font_family(theme.mono_font_family.clone())
                        .text_sm()
                        .text_color(rgb(ERROR_FOREGROUND))
                        .child(error),
                )
            })
    }
}

fn preview_text(source: &str) -> String {
    format!("$$\n{}\n$$", math::body(source))
}

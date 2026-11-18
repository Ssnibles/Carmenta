//! Compile a Typst maths expression and expose the SVG to the text view.

use std::{
    io::Write,
    process::{Command, Stdio},
    sync::{Arc, Mutex},
};

use gpui_kit::component::{
    ActiveTheme as _,
    text::{MarkdownNode, MarkdownParseContext, MarkdownPlugin, markdown_ast},
};
use gpui_kit::*;

const FALLBACK_COLOUR: &str = "#d4d4d4";
const TEXT_SIZE: &str = "22pt";
const FALLBACK_SIZE: (f32, f32) = (160.0, 40.0);

const PREAMBLE: &str = "#set page(width: auto, height: auto, margin: 4pt, fill: none)\n";

/// A compiled expression, ready to be drawn as one inline image.
#[derive(Clone)]
pub struct RenderedMath {
    pub key: String,
    pub image: Arc<Image>,
    pub width: f32,
    pub height: f32,
}

/// The image the editor most recently produced.
///
/// The renderer reads it on every frame, so it is kept separate from the
/// editor state and only swapped after a compile succeeds.
#[derive(Clone, Default)]
pub struct MathCache(Arc<Mutex<Option<RenderedMath>>>);

impl MathCache {
    pub fn get(&self, key: &str) -> Option<RenderedMath> {
        self.0
            .lock()
            .ok()?
            .as_ref()
            .filter(|math| math.key == key)
            .cloned()
    }

    pub fn set(&self, math: RenderedMath) {
        if let Ok(mut slot) = self.0.lock() {
            *slot = Some(math);
        }
    }

    pub fn clear(&self) {
        if let Ok(mut slot) = self.0.lock() {
            *slot = None;
        }
    }
}

/// Renders `$$ ... $$` blocks with the cached Typst result.
#[derive(Clone)]
pub struct MathPlugin {
    cache: MathCache,
}

impl MathPlugin {
    pub fn new(cache: MathCache) -> Self {
        Self { cache }
    }
}

impl MarkdownPlugin for MathPlugin {
    fn is_block(&self) -> bool {
        true
    }

    fn name(&self) -> &str {
        "carmenta-math"
    }

    fn parse(
        &self,
        node: &markdown_ast::Node,
        _cx: &MarkdownParseContext<'_>,
    ) -> Option<MarkdownNode> {
        let markdown_ast::Node::Math(math) = node else {
            return None;
        };
        let key = body(&math.value).to_string();
        Some(MarkdownNode::new("carmenta-math", key.clone()).text(key))
    }

    fn render(&self, node: &MarkdownNode, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let key = node.data::<String>().cloned().unwrap_or_default();

        if let Some(math) = self.cache.get(&key) {
            return div()
                .w_full()
                .flex()
                .justify_center()
                .child(
                    img(math.image)
                        .w(px(math.width))
                        .h(px(math.height))
                        .max_w_full()
                        .object_fit(ObjectFit::Contain),
                )
                .into_any_element();
        }

        div()
            .w_full()
            .flex()
            .justify_center()
            .py_2()
            .font_family(cx.theme().mono_font_family.clone())
            .text_size(cx.theme().mono_font_size)
            .text_color(cx.theme().muted_foreground)
            .child(key)
            .into_any_element()
    }
}

/// Strip one pair of surrounding `$` delimiters and any outer whitespace.
pub fn body(source: &str) -> &str {
    let source = source.trim();
    let source = source.strip_prefix('$').unwrap_or(source);
    source.strip_suffix('$').unwrap_or(source)
}

/// Compile `source` with the Typst CLI and return the SVG as an image.
pub fn compile(source: &str, colour: &str) -> Result<RenderedMath, String> {
    let key = body(source).to_string();
    let document = format!(
        "{PREAMBLE}#set text(size: {TEXT_SIZE}, fill: rgb(\"{}\"))\n$ {key} $\n",
        sanitise_colour(colour)
    );

    let output = run_typst(&document)?;
    let mut svg = svg_from_output(output)?;
    let (width, height) = svg_size(&svg).unwrap_or(FALLBACK_SIZE);

    // A leading XML declaration is noise to the SVG decoder.
    if let Some(start) = svg.find("<svg") {
        svg.drain(..start);
    }

    Ok(RenderedMath {
        key,
        image: Arc::new(Image::from_bytes(ImageFormat::Svg, svg.into_bytes())),
        width,
        height,
    })
}

/// Accept only `#rrggbb`; anything else falls back to the default.
fn sanitise_colour(colour: &str) -> &str {
    let is_hex = colour.len() == 7
        && colour.starts_with('#')
        && colour[1..].chars().all(|c| c.is_ascii_hexdigit());

    if is_hex { colour } else { FALLBACK_COLOUR }
}

fn run_typst(document: &str) -> Result<std::process::Output, String> {
    let mut child = Command::new("typst")
        .args(["compile", "-", "--format", "svg", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| {
            format!("could not run `typst`: {e}\n\nIs the Typst CLI installed and on PATH?")
        })?;

    child
        .stdin
        .take()
        .expect("stdin was piped")
        .write_all(document.as_bytes())
        .map_err(|e| format!("failed to send source to typst: {e}"))?;

    child
        .wait_with_output()
        .map_err(|e| format!("failed to read typst output: {e}"))
}

fn svg_from_output(output: std::process::Output) -> Result<String, String> {
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_owned());
    }

    let svg = String::from_utf8_lossy(&output.stdout).into_owned();
    if svg.trim().is_empty() {
        return Err("typst produced no output".to_owned());
    }

    Ok(svg)
}

/// Read the `width`/`height` attributes Typst writes, in logical pixels.
fn svg_size(svg: &str) -> Option<(f32, f32)> {
    Some((dimension(svg, "width")?, dimension(svg, "height")?))
}

fn dimension(svg: &str, name: &str) -> Option<f32> {
    let needle = format!("{name}=\"");
    let start = svg.find(&needle)? + needle.len();
    let end = start + svg[start..].find('"')?;
    let value = &svg[start..end];

    if let Some(value) = value.strip_suffix("pt") {
        value.parse::<f32>().ok().map(|v| v * 96.0 / 72.0)
    } else {
        value
            .strip_suffix("px")
            .unwrap_or(value)
            .parse::<f32>()
            .ok()
    }
}

/// Format an interface colour as the `#rrggbb` Typst accepts.
pub fn hex(color: Hsla) -> String {
    let rgba = color.to_rgb();
    let channel = |value: f32| (value.clamp(0.0, 1.0) * 255.0).round() as u8;
    format!(
        "#{:02x}{:02x}{:02x}",
        channel(rgba.r),
        channel(rgba.g),
        channel(rgba.b)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    // `gpui_kit::*` re-exports GPUI's test macro, which shadows the built-in
    // `#[test]` attribute and recurses when used here.
    #[::core::prelude::v1::test]
    fn strips_delimiters_and_whitespace() {
        assert_eq!(body("  $x^2$  "), "x^2");
        assert_eq!(body("$ x^2 $"), " x^2 ");
        assert_eq!(body("x^2"), "x^2");
    }

    #[::core::prelude::v1::test]
    fn accepts_only_six_digit_hex_colours() {
        assert_eq!(sanitise_colour("#e6edf3"), "#e6edf3");
        assert_eq!(sanitise_colour("#ABCDEF"), "#ABCDEF");
        assert_eq!(sanitise_colour("#fff"), FALLBACK_COLOUR);
        assert_eq!(sanitise_colour("e6edf3"), FALLBACK_COLOUR);
        assert_eq!(sanitise_colour("#e6edfg"), FALLBACK_COLOUR);
    }

    #[::core::prelude::v1::test]
    fn reads_typst_dimensions_in_pixels() {
        let svg = r#"<svg width="45.5pt" height="20pt" viewBox="0 0 45.5 20"></svg>"#;
        let (width, height) = svg_size(svg).expect("dimensions");
        assert!((width - 60.666).abs() < 0.01);
        assert!((height - 26.666).abs() < 0.01);
    }

    #[::core::prelude::v1::test]
    fn formats_colours_as_hex() {
        assert_eq!(hex(rgb(0x1e1e1e).into()), "#1e1e1e");
    }
}

# Carmenta

A native desktop app for writing Typst maths and seeing it typeset. Source on
the left, the rendered formula on the right, updated as you type.

Named after the Roman goddess credited with inventing the Latin alphabet.

Built with [GPUI Kit](https://gpui-kit.com) on top of GPUI. There is no webview
and no JavaScript. Typst runs as a child process and its SVG is decoded
directly by GPUI.

## Layout

| file           | what it does                                                |
| -------------- | ----------------------------------------------------------- |
| `src/main.rs`  | application start-up, window options, the dark Zed-like theme |
| `src/app.rs`   | the editor pane, the preview pane, debounced recompilation   |
| `src/math.rs`  | Typst CLI -> SVG, the image cache, and the Markdown plugin   |

**Editor.** A GPUI Kit `EditorState` in plain mode: no line numbers, soft wrap,
monospace. It is a normal code editor underneath, so selection, undo, IME and
keyboard navigation already work. Completion providers can be attached to it
later without replacing the widget.

**Preview.** A `TextViewState` in Markdown mode with a block plugin for
`$$ ... $$`. The plugin looks up the compiled SVG in a small cache and draws it
as one inline image. When the cache has no entry yet, it shows the source text
instead, which is also what you see while Typst is still working.

**Compilation.** Every edit schedules a compile 120 ms later. A generation
counter discards results that arrive after a newer edit, so fast typing cannot
make an older formula win. The Typst result is an SVG, which GPUI decodes and
scales without a rasterisation step.

**Theme.** Dark, flat, square corners. The palette and the zero-radius tokens
are set once in `src/main.rs`, so the rest of the app inherits them.

## Requirements

- A Rust toolchain with edition 2024 support.
- The [Typst CLI](https://typst.app) (`typst`) on `PATH`, 0.13 or newer.
- On Linux: Wayland or X11, Vulkan, and the usual fontconfig/freetype stack.
  `shell.nix` provides them.

## Run

With Nix:

```sh
nix-shell
run     # debug build and launch
build   # release build, prints the binary path
```

Non-interactively:

```sh
nix-shell --run run
nix-shell --run build
```

Without Nix:

```sh
cargo run
cargo build --release
```

## Notes

- The app is stateless: nothing is written to disk and no documents are tracked.
- `gpui-kit` is pinned to an exact version because its GPUI backend moves in
  pre-release snapshots. Upgrade it deliberately.
- The next step is a purpose-built editor over the same text renderer, with
  Typst-aware completion, rather than a plain source box.

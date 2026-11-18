{ pkgs ? import <nixpkgs> { } }:

let
  projectRoot = toString ./.;

  # GPUI uses Wayland/X11 through dlopen and renders with Vulkan, so these
  # also need to be on the loader path at runtime, not just at build time.
  runtimeLibs = with pkgs; [
    wayland
    libxkbcommon
    libGL
    mesa
    vulkan-loader
    libX11
    libXcursor
    libXi
    libXrandr
    libXinerama
    libxcb
    fontconfig
    freetype
    expat
    glib
    zlib
    stdenv.cc.cc.lib
  ];
in
pkgs.mkShell {
  nativeBuildInputs = with pkgs; [
    cargo
    rustc
    rustfmt
    pkg-config
    clang
    vulkan-headers
  ];

  buildInputs = runtimeLibs;

  shellHook = ''
    export LD_LIBRARY_PATH=${pkgs.lib.makeLibraryPath runtimeLibs}:$LD_LIBRARY_PATH

    # Convenience commands: `run` builds and launches the app; `build`
    # produces the release binary and prints its path.
    run() {
      cargo run --manifest-path "${projectRoot}/Cargo.toml" "$@"
    }

    build() {
      cargo build --release --manifest-path "${projectRoot}/Cargo.toml" "$@" &&
        printf 'binary: %s\n' "${projectRoot}/target/release/carmenta"
    }

    export -f run build
  '';
}

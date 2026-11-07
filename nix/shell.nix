{ pkgs, lib, mkShell }:

let
  rust = (pkgs.rust-bin.fromRustupToolchainFile ../rust-toolchain.toml).override {
    extensions = [ "rust-src" "rust-analyzer" ];
  };
  libraries = with pkgs; [
    gtk3
    webkitgtk_4_1
    libsoup_3
    glib
    gdk-pixbuf
    cairo
    dbus
    openssl
    librsvg
    libayatana-appindicator
    xdotool
  ];
  runtimeLibraries = libraries ++ (with pkgs; [
    xorg.libX11
    xorg.libxcb
    wayland
    libxkbcommon
  ]);
in
mkShell {
  packages = with pkgs; [
    nodejs_24
    corepack
    rust
    pkg-config
    just
    git
    zsh
  ];

  buildInputs = libraries;

  # Clipboard backends and WebKit load libraries and GIO modules at runtime.
  LD_LIBRARY_PATH = lib.makeLibraryPath runtimeLibraries;
  FONTCONFIG_FILE = pkgs.makeFontsConf {
    fontDirectories = [ "/run/current-system/sw/share/fonts" "~/.fonts" ];
    includes = [ "${pkgs.fontconfig.out}/etc/fonts/conf.d" ];
  };
  FONTCONFIG_PATH = "${pkgs.fontconfig.out}/etc/fonts";
  GIO_EXTRA_MODULES = "${pkgs.glib-networking}/lib/gio/modules";
  GST_PLUGIN_PATH_1_0 = "${pkgs.gst_all_1.gst-plugins-base}/lib/gstreamer-1.0";
  RUST_SRC_PATH = "${rust}/lib/rustlib/src/rust/library";

  shellHook = ''
    # Unlike packaged applications, cargo-built executables have no GApps wrapper.
    export XDG_DATA_DIRS="${pkgs.gsettings-desktop-schemas}/share/gsettings-schemas/${pkgs.gsettings-desktop-schemas.name}:${pkgs.gtk3}/share/gsettings-schemas/${pkgs.gtk3.name}''${XDG_DATA_DIRS:+:$XDG_DATA_DIRS}"
    exec "${pkgs.zsh}/bin/zsh"
  '';
}

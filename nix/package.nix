{
  lib,
  stdenv,
  fetchurl,
  dpkg,
  autoPatchelfHook,
  wrapGAppsHook3,
  gtk3,
  webkitgtk_4_1,
  libsoup_3,
  glib,
  glib-networking,
  dbus,
  gdk-pixbuf,
  cairo,
  xorg,
  wayland,
}:

stdenv.mkDerivation rec {
  pname = "hitsu";
  version = "0.7.0";

  # Upstream currently publishes Linux binaries only for x86_64.
  src = fetchurl {
    url = "https://github.com/ruaylabs/hitsu/releases/download/v${version}/Hitsu_${version}_amd64.deb";
    sha256 = "6e998e9a49c2e555e33cd4dda7bb580b6a6aa2667d7e6d617dab9bb115d2e42d";
  };

  nativeBuildInputs = [
    dpkg
    autoPatchelfHook
    wrapGAppsHook3
  ];

  buildInputs = [
    gtk3
    webkitgtk_4_1
    libsoup_3
    glib
    glib-networking
    dbus
    gdk-pixbuf
    cairo
    stdenv.cc.cc.lib
  ];

  # Clipboard backends load these libraries dynamically.
  runtimeDependencies = map lib.getLib [
    xorg.libX11
    xorg.libxcb
    wayland
  ];

  unpackPhase = ''
    runHook preUnpack
    dpkg-deb -x "$src" .
    runHook postUnpack
  '';

  dontConfigure = true;
  dontBuild = true;
  dontWrapGApps = true;

  installPhase = ''
    runHook preInstall
    mkdir -p "$out"
    cp -r usr/bin usr/share "$out/"
    substituteInPlace "$out/share/applications/Hitsu.desktop" \
      --replace-fail 'Categories=' 'Categories=Utility;Security;'
    runHook postInstall
  '';

  # Do not wrap the native host: Linux peer authentication checks its actual
  # executable path. Keep it beside the wrapped desktop application's ELF.
  postFixup = ''
    wrapGApp "$out/bin/hitsu"
  '';

  meta = {
    description = "Native desktop password manager for KeePass databases";
    homepage = "https://github.com/ruaylabs/hitsu";
    license = lib.licenses.mit;
    platforms = [ "x86_64-linux" ];
    sourceProvenance = [ lib.sourceTypes.binaryNativeCode ];
    mainProgram = "hitsu";
  };
}

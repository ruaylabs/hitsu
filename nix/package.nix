{ lib
, stdenv
, fetchurl
, dpkg
, autoPatchelfHook
, wrapGAppsHook3
, gtk3
, webkitgtk_4_1
, libsoup_3
, glib
, glib-networking
, dbus
, gdk-pixbuf
, cairo
, xorg
, wayland
,
}:

stdenv.mkDerivation rec {
  pname = "hitsu";
  version = "0.12.0";

  # Upstream currently publishes Linux binaries only for x86_64.
  src = fetchurl {
    url = "https://github.com/ruaylabs/hitsu/releases/download/v${version}/Hitsu_${version}_amd64.deb";
    sha256 = "a6115edc0518128a7855c954d4e7895e4ed79ee86e0080930ec272fd18c0a57d";
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

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
  version = "0.13.1";

  # Upstream currently publishes Linux binaries only for x86_64.
  src = fetchurl {
    url = "https://github.com/ruaylabs/hitsu/releases/download/v${version}/Hitsu_${version}_amd64.deb";
    sha256 = "564300ac1ffa8d18462107eebd1ad6d709b7d4da750951665a6fcf208b583d9b";
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

{ pkgs ? import <nixpkgs> { } }:

pkgs.mkShell {
  nativeBuildInputs = with pkgs; [
    pkg-config
    cargo
    rustc
    gcc
  ];

  buildInputs = with pkgs; [
    dbus
  ];

  shellHook = ''
    export PKG_CONFIG_PATH="${pkgs.dbus.dev}/lib/pkgconfig"
    echo "Bluetooth-Entwicklungsumgebung bereit!"
  '';
}

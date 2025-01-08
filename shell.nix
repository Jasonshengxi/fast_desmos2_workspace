{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
    # nativeBuildInputs is usually what you want -- tools you need to run
    buildInputs = with pkgs; [
        pkg-config
        xorg.libX11
        xorg.libXrandr
        xorg.libXinerama
        xorg.libXcursor
        xorg.libXi
        libglvnd
        cmake
    ];
}

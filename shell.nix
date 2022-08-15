{ pkgs ? import <nixpkgs> {} }:

with pkgs;

mkShell {
    buildInputs = [
      rustc
      cargo
      SDL2
      SDL2_image

      # Coding tools
      rls
      rustfmt
    ];
}


{ pkgs ? import <nixpkgs> {} }:

with pkgs;

mkShell {
    buildInputs = [
      rustc
      cargo
      SDL2

      # Coding tools
      rls
      rustfmt
    ];
}


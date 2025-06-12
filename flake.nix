{
  description = "Development environment for my Rust project";

  inputs = {
    nixpkgs.url = "nixpkgs/nixos-25.05";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };
      in 
      {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            gcc
            binutils
            rustup
            direnv
            xclip
            duf
            eza
            fd
            pkg-config
            postgresql

            openssl

            gtk4
            webkitgtk_4_1



            dbus
            webkitgtk
            glib
            nodejs_latest

            libsoup_3

          ];

        shellHook = ''
          export SHELL=${pkgs.fish}/bin/fish
          export PKG_CONFIG_PATH=${pkgs.webkitgtk_4_1.dev}/lib/pkgconfig
        '';

      };
  });
}

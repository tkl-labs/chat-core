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
          ];

        shellHook = ''
          export SHELL=${pkgs.fish}/bin/fish
        '';

      };
  });
}

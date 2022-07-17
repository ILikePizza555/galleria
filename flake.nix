{
  description = "A discord bot that turns your image channels into image galleries";

  inputs = {
    nixpkgs.url = github:NixOS/nixpkgs/nixos-22.05;
    fenix.url = github:nix-community/fenix;
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {self, nixpkgs, fenix, flake-utils, ...}:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        lib-fenix = fenix.packages.${system};
        rust = lib-fenix.combine [
          lib-fenix.stable.cargo
          lib-fenix.stable.rustc
          lib-fenix.stable.rustfmt
          lib-fenix.stable.rust-src
          lib-fenix.targets.wasm32-unknown-unknown.latest.rust-std
        ];
        commonBuildInputs = [
          pkgs.openssl
          pkgs.pkg-config
          rust
        ];
      in
        {
          defaultPackages = pkgs.stdenv.mkDerivation {
            pname = "galleria";
            version = "0.1.0";
            src = ./.;
            buildInputs = commonBuildInputs;
          };

          devShell = pkgs.mkShell {
            packages = [pkgs.postgresql_14] ++ commonBuildInputs;
          };
        }
    );
}

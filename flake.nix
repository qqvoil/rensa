{
  description = "Rust-based TUI garbage collection helper for NixOS";

  inputs = {
    nixpkgs.url = "nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
      in
      {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "rensa";
          version = "0.2.0";
          src = ./.;
          cargoLock = {
            lockFile = ./Cargo.lock;
          };
          meta = with pkgs.lib; {
            description = "Rust-based TUI garbage collection helper for NixOS";
            homepage = "https://github.com/qqvoil/rensa";
            license = licenses.mit;
            mainProgram = "rensa";
          };
        };

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            cargo
            rustc
            gcc
            rust-analyzer
            clippy
            rustfmt
            pkg-config
          ];
        };

        apps.default = flake-utils.lib.mkApp {
          drv = self.packages.${system}.default;
        };
      }
    );
}

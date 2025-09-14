{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
      };
    };
    obelisk = {
      url = "github:obeli-sk/obelisk/latest";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
        rust-overlay.follows = "rust-overlay";
      };
    };
  };
  outputs = { self, nixpkgs, flake-utils, rust-overlay, obelisk }:
    flake-utils.lib.eachDefaultSystem
      (system:
        let
          overlays = [ (import rust-overlay) ];
          pkgs = import nixpkgs {
            inherit system overlays;
          };
          rustToolchain = pkgs.pkgsBuildHost.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
          wit-bindgen-go-cli = pkgs.buildGoModule (rec {
            pname = "wit-bindgen-go-cli";
            version = "0.6.2"; # NB: Update version in dev-deps.sh
            src = pkgs.fetchFromGitHub {
              owner = "bytecodealliance";
              repo = "go-modules";
              rev = "v${version}";
              hash = "sha256-MM3jVhGgTbpC4QZX6HMhJnvWLcyhZZjT7DVqt712InY=";
            };
            modMode = "workspace";
            subPackages = [ "cmd/wit-bindgen-go" ];
            vendorHash = "sha256-HR2HE/urN5gs5sh5tHZw3ISoJGsPrVGoI9A24epoRZE=";
            proxyVendor = true;
          });
          commonDeps = with pkgs; [
            cargo-binstall
            cargo-edit
            cargo-expand
            cargo-generate
            cargo-nextest
            just
            nixpkgs-fmt
            pkg-config
            rustToolchain
            wasm-tools
            wasmtime.out
            # e2e tests
            openssl
            curlMinimal
            # javascript support
            nodejs_22
            wizer
            # Go
            go
            tinygo
            wit-bindgen-go-cli
          ];
          withObelisk = commonDeps ++ [ obelisk.packages.${system}.default ];
        in
        {
          devShells.noObelisk = pkgs.mkShell {
            nativeBuildInputs = commonDeps;
          };
          devShells.default = pkgs.mkShell {
            nativeBuildInputs = withObelisk;
          };
          devShells.cloudflared = pkgs.mkShell {
            nativeBuildInputs = withObelisk ++ [ pkgs.cloudflared ];
          };

        }
      );
}

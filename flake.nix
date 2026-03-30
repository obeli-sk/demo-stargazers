{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    nixpkgs-tinygo.url = "github:NixOS/nixpkgs/b40629efe5d6ec48dd1efba650c797ddbd39ace0";
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
  outputs = { self, nixpkgs, nixpkgs-tinygo, flake-utils, rust-overlay, obelisk }:
    flake-utils.lib.eachDefaultSystem
      (system:
        let
          overlays = [ (import rust-overlay) ];
          pkgs = import nixpkgs {
            inherit system overlays;
          };
          pkgsTinyGo = import nixpkgs-tinygo {
            inherit system;
          };
          rustToolchain = pkgs.pkgsBuildHost.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
          wit-bindgen-go-cli = pkgs.buildGoModule (rec {
            pname = "wit-bindgen-go-cli";
            version = "0.7.0"; # NB: Update version in dev-deps.sh
            src = pkgs.fetchFromGitHub {
              owner = "bytecodealliance";
              repo = "go-modules";
              rev = "v${version}";
              hash = "sha256-bzsB0EsDNk6x1xroIQqbUy7L97JbEJHo7wASnl35X+0=";
            };
            modMode = "workspace";
            subPackages = [ "cmd/wit-bindgen-go" ];
            vendorHash = "sha256-9BLzPxLc+HoVQuUtTwLj6QZvN7BLrX5Zy4s5eWTXvwA=";
            proxyVendor = true;
          });
          commonDeps = with pkgs; [
            cargo-binstall
            cargo-edit
            cargo-expand
            cargo-generate
            cargo-nextest
            cargo-deny
            just
            nixpkgs-fmt
            pkg-config
            rustToolchain
            wasm-tools
            wasmtime.out
            # e2e tests
            openssl
            curlMinimal
            python3
            # javascript support
            nodejs_22
            wizer
            # Go
            go_1_25
            pkgsTinyGo.tinygo
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
          devShells.sandbox = pkgs.mkShell {
            packages = commonDeps ++ ( with pkgs; [
              codex
              gemini-cli
              claude-code
              bubblewrap
              # tools
              git
              gh
              curl
              helix
              wget
              htop
              procps
              ripgrep
              which
              less
            ]);
            shellHook = ''
              CURRENT_DIR=$(pwd)

              # SSL/Network Fixes
              export SSL_CERT_FILE="${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt"
              export NIX_SSL_CERT_FILE=$SSL_CERT_FILE
              REAL_RESOLV=$(realpath /etc/resolv.conf)
              REAL_HOSTS=$(realpath /etc/hosts)
              REAL_GITCONFIG=$(realpath "$HOME/.gitconfig")

              # Construct mocked /run/current-system/sw/bin
              MOCKED_SYSTEM_BIN=$(mktemp -d)
              # Iterate through current PATH and symlink executables.
              # We use 'ln -s' without '-f' (force) so that the FIRST entry found
              # in the PATH (highest priority) wins, mimicking actual shell behavior.
              IFS=':' read -ra PATH_DIRS <<< "$PATH"
              for dir in "''${PATH_DIRS[@]}"; do
                if [ -d "$dir" ]; then
                   ln -s "$dir"/* "$MOCKED_SYSTEM_BIN/" 2>/dev/null || true
                fi
              done

              BWRAP_CMD=(
                ${pkgs.bubblewrap}/bin/bwrap
                --unshare-all
                --share-net
                --die-with-parent
                # --- Essential Binds ---
                --ro-bind /nix /nix
                --proc /proc
                --dev /dev
                --tmpfs /tmp
                # Tools need these to know "who" is running the process
                --ro-bind /etc/passwd /etc/passwd
                --ro-bind /etc/group /etc/group
                # --- Network ---
                --ro-bind "$REAL_RESOLV" /etc/resolv.conf
                --ro-bind "$REAL_HOSTS"  /etc/hosts
                # Git
                --ro-bind "$REAL_GITCONFIG" /tmp/.gitconfig
                # Claude
                --bind $HOME/.claude /tmp/.claude
                --bind $HOME/.claude.json /tmp/.claude.json
                # Cargo
                --bind $HOME/.cargo  /tmp/.cargo
                # --- Project Mount ---
                --dir /workspace
                --bind "$CURRENT_DIR" /workspace
                --chdir /workspace
                # --- Mocked System Bin ---
                # Create the directory structure in the sandbox
                --dir /run/current-system/sw/bin
                # Bind our constructed temp folder to it
                --ro-bind "$MOCKED_SYSTEM_BIN" /run/current-system/sw/bin
                --ro-bind "$MOCKED_SYSTEM_BIN" /usr/bin
                # --- Environment ---
                --setenv PS1 "[BWRAP] \w> "
                --setenv HOME /tmp
                --setenv TMPDIR /tmp
                --setenv TEMP /tmp
                --setenv CARGO_TARGET_DIR target-sandbox
              )
              exec "''${BWRAP_CMD[@]}" ${pkgs.bashInteractive}/bin/bash -l +m
            '';
          };
          devShells.cloudflared = pkgs.mkShell {
            nativeBuildInputs = withObelisk ++ [ pkgs.cloudflared ];
          };

        }
      );
}

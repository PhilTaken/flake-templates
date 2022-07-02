{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
    fenix.url = "github:nix-community/fenix";
  };

  outputs = { self, nixpkgs, flake-utils, naersk, fenix, ... }@inputs:
    flake-utils.lib.eachSystem [ "x86_64-linux" ] (system: let
      # ---------------------------------------

      pname = "template";
      rust = pkgs.fenix.fromToolchainFile {
        file = ./rust-toolchain.toml;
        sha256 = "sha256-qAAsuHw8IXejRJ5EdRXUavrSWkIYrp2s+Ozv9Zo/8zo=";
      };

      nativeBuildInputs = with pkgs; [ ];
      buildInputs = with pkgs; [ ];

      # ---------------------------------------

      pkgs = import nixpkgs {
        inherit system;
        overlays = [ fenix.overlay ];
      };

      naersk-lib = naersk.lib."${system}".override {
        cargo = rust;
        rustc = rust;
      };

      packWithTests = doCheck: naersk-lib.buildPackage {
        inherit pname doCheck nativeBuildInputs buildInputs;
        root = ./.;
        doDoc = true;
        doDocFail = true;

        cargoTestCommands = x:
          x ++ [
            # clippy
            ''cargo clippy --all --all-features --tests -- \
              -D clippy::pedantic \
              -D warnings \
              -A clippy::module-name-repetitions \
              -A clippy::too-many-lines \
              -A clippy::cast-possible-wrap \
              -A clippy::cast-possible-truncation \
              -A clippy::nonminimal_bool''
          ];
      };
    in rec {
      # `nix build`
      packages = {
        "${pname}" = packWithTests false;
        default = packages.${pname};
        doc = packages.${pname}.doc;
      };

      # `nix run`
      apps = {
        "${pname}" = flake-utils.lib.mkApp { drv = packages.${pname}; };
        default = apps."${pname}";
      };

      # `nix develop`
      devShells.default = pkgs.mkShell {
        inherit buildInputs;

        nativeBuildInputs = nativeBuildInputs ++ [
          rust
          pkgs.cargo-edit
        ];
      };

      # `nix flake check`
      checks = {
        buildPack = packWithTests true;
      };
    }
  );
}

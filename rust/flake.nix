{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
    fenix.url = "github:nix-community/fenix";
  };

  outputs = { self, nixpkgs, flake-utils, naersk, fenix, ... }@inputs:
  flake-utils.lib.eachSystem [ "x86_64-linux" ] (system: let
    pname = "template";
    overlay = _: prev: {
      ${pname} = prev.callPackage ./nix/default.nix { inherit naersk pname; };
    };
    pkgs = import nixpkgs {
      inherit system;
      overlays = [ fenix.overlay overlay ];
      config.allowUnfree = true;
    };
  in rec {
      # `nix build`
      packages = {
        ${pname} = pkgs.${pname};
        default = packages.${pname};
        doc = packages.${pname}.doc;
      };

      # `nix run`
      apps = {
        ${pname} = flake-utils.lib.mkApp { drv = packages.${pname}; };
        default = apps.${pname};
      };

      # overlay with package
      overlays.default = overlay;

      # `nix develop`
      devShells.default = pkgs.mkShell {
        inputsFrom = [ pkgs.${pname} ];

        nativeBuildInputs = [
          pkgs.cargo-edit
        ];
      };

      # `nix flake check`
      checks.default = pkgs.${pname};
    }
  );
}

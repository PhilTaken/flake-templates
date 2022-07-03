{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
    fenix.url = "github:nix-community/fenix";
  };

  outputs = { self, nixpkgs, flake-utils, naersk, fenix, ... }@inputs: let
    pname = "template";
    overlays.default = _: prev: {
      ${pname} = prev.callPackage ./nix/default.nix { inherit naersk pname; };
    };
  in {
    # overlay with package
    inherit overlays;
  } // flake-utils.lib.eachSystem [ "x86_64-linux" ] (system: let
    pname = "template";
    pkgs = import nixpkgs {
      inherit system;
      overlays = [ fenix.overlay overlays.default ];
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

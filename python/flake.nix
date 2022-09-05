{
  description = "Application packaged using poetry2nix";

  inputs.flake-utils.url = "github:numtide/flake-utils";
  inputs.nixpkgs.url = "github:NixOS/nixpkgs";
  inputs.poetry2nix.url = "github:nix-community/poetry2nix";

  outputs = { self, nixpkgs, flake-utils, poetry2nix }:
    {
      # Nixpkgs overlay providing the application
      overlay = nixpkgs.lib.composeManyExtensions [
        poetry2nix.overlay
        (final: prev: {
          # The application
          myapp = prev.poetry2nix.mkPoetryApplication {
            projectDir = ./.;
          };
        })
      ];
    } // (flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ self.overlay ];
        };

        appEnv = pkgs.poetry2nix.mkPoetryEnv {
          projectDir = ./.;
          editablePackageSources = {
            my-app = ./myapp;
          };
        };
      in
      {
        apps = {
          myapp = pkgs.myapp;
        };

        defaultApp = pkgs.myapp;

        devShells.default = appEnv.env.overrideAttrs (oldAttrs: {
          buildInputs = with pkgs; [
              poetry
          ];
        });
      }));
}

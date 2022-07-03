{ lib
, stdenv
, fenix
, naersk
, system

, pname
, doCheck ? false
}:

let
  rust = fenix.fromToolchainFile {
    file = ../rust-toolchain.toml;
    sha256 = "sha256-qAAsuHw8IXejRJ5EdRXUavrSWkIYrp2s+Ozv9Zo/8zo=";
  };
  naersk-lib = naersk.lib."${system}".override {
    cargo = rust;
    rustc = rust;
  };
in naersk-lib.buildPackage {
  inherit doCheck pname;
  root = ../.;

  doDoc = true;
  doDocFail = true;

  nativeBuildInputs = [ ];
  buildInputs = [ ];
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
}

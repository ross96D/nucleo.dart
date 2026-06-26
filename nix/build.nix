{
  pkgs ? import <nixpkgs> {},
}:

pkgs.rustPlatform.buildRustPackage {
  pname = "nucleo_dart";
  version = "0.1.0";
  src = ../rust;
  cargoLock = {
    lockFile = ../rust/Cargo.lock;
    outputHashes = {
      "nucleo-0.5.0" = "sha256-ztSgjBI8vhKvrWmpT5K1UoHQRnbbrbEtSnvRkFmhSNc=";
    };
  };
}

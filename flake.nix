{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable-small";
    rust-overlay.url = "github:oxalica/rust-overlay";
    futils.url = "github:numtide/flake-utils";
  };

  outputs = {
    self,
    nixpkgs,
    rust-overlay,
    futils,
  } @ inputs: let
    inherit (nixpkgs) lib;
    inherit (futils.lib) eachDefaultSystem defaultSystems;

    nixpkgsFor = lib.genAttrs defaultSystems (system:
      import nixpkgs {
        inherit system;
        overlays = [
          rust-overlay.overlays.default
        ];
      });
  in
    eachDefaultSystem (system: let
      pkgs = nixpkgsFor.${system};
    in {
      devShell = pkgs.mkShell {
        buildInputs = with pkgs; [
          (lib.hiPrio rust-bin.nightly.latest.rustfmt)
          (rust-bin.fromRustupToolchainFile ./rust-toolchain.toml)
          sccache

          protobuf

          cargo-msrv
          cargo-release
          cargo-workspaces
          git
          just
        ];

        RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
        RUST_BACKTRACE = 1;
        RUSTC_WRAPPER = "sccache";
      };
    });
}

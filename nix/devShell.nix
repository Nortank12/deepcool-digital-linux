pkgs:
let
  rustDevEnv = (
    pkgs.rust-bin.stable.latest.default.override {
      extensions = [ "rust-src" ];
    }
  );
in
pkgs.mkShell {
  packages =
    with pkgs;
    [
      pkg-config
      libudev-zero
    ]
    ++ [ rustDevEnv ];

  shellHook = ''
    [ -d ".devenv/profile" ] && exit 0
    mkdir -p .devenv/profile/{bin,lib/rustlib/src}

    ln -sfn ${rustDevEnv}/bin/* .devenv/profile/bin/
    ln -sfn ${rustDevEnv}/lib/rustlib/src/rust .devenv/profile/lib/rustlib/src/rust

    echo ""
    echo "Development environment initialized with local toolchain paths"
    echo "Toolchain: $PWD/.devenv/profile/bin"
    echo "Standard library: $PWD/.devenv/profile/lib/rustlib/src/rust/library"
    echo ""
  '';
}

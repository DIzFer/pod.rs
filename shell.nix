with import <nixpkgs> {};
stdenv.mkDerivation rec {
  name = "env";
  env = buildEnv { name = "pod.rs"; paths = buildInputs; };
  buildInputs = [
    pkgconfig
    openssl_1_1_0
    cargo
    rustc
    rustfmt
  ];
}

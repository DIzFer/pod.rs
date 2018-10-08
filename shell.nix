with import <nixpkgs> {};
stdenv.mkDerivation rec {
  name = "env";
  env = buildEnv { name = "pod.rs"; paths = buildInputs; };
  buildInputs = [
    pkgconfig
    openssl_1_0_2
    cargo
    rustc
    rustfmt
  ];
}

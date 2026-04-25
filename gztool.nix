{
  lib,
  stdenv,
  fetchFromGitHub,
  autoreconfHook,
  zlib-ng,
}:
let
  patch-file = ./patches/0001-Ensure-that-stdout-does-not-get-closed.patch;
in
stdenv.mkDerivation rec {
  pname = "gztool";
  version = "v1.8.0";

  src = fetchFromGitHub {
    owner = "circulosmeos";
    repo = "${pname}";
    rev = "refs/tags/${version}";
    sha256 = "sha256-/9wQi2sVCE3V2NelqinuvbRPBJrcSWRsaDA75CBD7rk=";
  };

  nativeBuildInputs = [
    autoreconfHook
  ];

  buildInputs = [
    (zlib-ng.override { withZlibCompat = true; })
  ];

  patches = [
    patch-file
  ];

  postInstall = ''
    mkdir $out/include
    cp $src/gztool.c $out/include/gztool.c
    patch $out/include/gztool.c ${patch-file}
  '';
}

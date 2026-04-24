{
  lib,
  stdenv,
  fetchFromGitHub,
  autoreconfHook,
  zlib-ng,
}:
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

  postInstall = ''
    mkdir $out/include
    cp $src/gztool.c $out/include/gztool.c
  '';
}

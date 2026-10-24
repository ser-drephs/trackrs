{
  description = "Simple time tracker using CLI";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs = { self, nixpkgs }:
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${system};
      # Updated by eng/update-flake.sh on release.
      version = "3.0.0";
      hash = "sha256-OnBJaX2HYf12e9uLWs0BwXPb142H0Mev8sQKbLD5oNA=";
    in
    {
      packages.${system} = rec {
        # The prebuilt binary of the GitHub release, linked against glibc and patched for nix.
        trackrs = pkgs.stdenv.mkDerivation {
          pname = "trackrs";
          inherit version;

          src = pkgs.fetchurl {
            url = "https://github.com/ser-drephs/trackrs/releases/download/v${version}/trackrs";
            inherit hash;
          };
          dontUnpack = true;

          nativeBuildInputs = [ pkgs.autoPatchelfHook ];
          buildInputs = [ pkgs.stdenv.cc.cc.lib ];

          installPhase = ''
            runHook preInstall
            install -Dm755 $src $out/bin/trackrs
            runHook postInstall
          '';

          meta = {
            description = "Simple time tracker using CLI";
            homepage = "https://github.com/ser-drephs/trackrs";
            license = pkgs.lib.licenses.mit;
            mainProgram = "trackrs";
            platforms = [ system ];
            sourceProvenance = [ pkgs.lib.sourceTypes.binaryNativeCode ];
          };
        };
        default = trackrs;
      };
    };
}

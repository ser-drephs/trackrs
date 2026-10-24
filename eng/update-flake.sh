#!/bin/bash
# Points flake.nix to the release binary: its version and the hash of release/linux-gnu/trackrs.

for i in "$@"; do
  case $i in
    -v=*|--version=*)
      VERSION="${i#*=}"
      shift # past argument=value
      ;;
    *)
      echo "Unknown option $i"
      exit 1
      ;;
  esac
done

BINARY=$PWD/release/linux-gnu/trackrs
if [ -z "$VERSION" ] || [ ! -f "$BINARY" ]; then
  echo "usage: --version=<version>, after building $BINARY"
  exit 1
fi

HASH="sha256-$(openssl dgst -sha256 -binary "$BINARY" | base64)"
echo "VERSION = ${VERSION}, HASH = ${HASH}"

sed -i \
  -e "s|^\( *version = \)\".*\";|\1\"$VERSION\";|" \
  -e "s|^\( *hash = \)\"sha256-.*\";|\1\"$HASH\";|" \
  $PWD/flake.nix

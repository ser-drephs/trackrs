#!/bin/bash

for i in "$@"; do
  case $i in
    -v=*|--version=*)
      VERSION="${i#*=}"
      shift # past argument=value
      ;;
    -*|--*)
      echo "Unknown option $i"
      exit 1
      ;;
    *)
      echo "Unknown option $i"
      exit 1
      ;;
  esac
done

echo "VERSION = ${VERSION}"

sed -i "s/^\(version *= *\).*/\1\"$VERSION\"/" $PWD/Cargo.toml

#!/usr/bin/env bash
# Read-only with respect to the system; no installation, service changes or sudo.
set -euo pipefail
cd "$(dirname "$0")/.."
if [[ -n "$(git status --porcelain --untracked-files=normal)" ]]; then
  echo 'Commit or stash project changes before packaging the reviewed HEAD.' >&2
  exit 1
fi
version=$(sed -n 's/^version = "\([^"]*\)"/\1/p' Cargo.toml | head -n1)
mkdir -p .package
git archive --format=tar --prefix="openpods-${version}/" HEAD > ".package/openpods-${version}.tar"
read -r checksum _ < <(sha256sum ".package/openpods-${version}.tar")
sed "s/GENERATED_BY_PACKAGE_SCRIPT/${checksum}/" PKGBUILD > .package/PKGBUILD
printf 'Prepared .package/PKGBUILD and source archive. Review, then: cd .package && makepkg -si\n'

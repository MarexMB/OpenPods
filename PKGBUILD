# Local, unpublished package. Run scripts/package.sh to prepare a checksummed
# archive and PKGBUILD under .package/, then review it and run makepkg -si there.
pkgname=openpods
pkgver=0.1.0
pkgrel=1
pkgdesc='Native local Bluetooth and AirPods monitoring for CachyOS'
arch=('x86_64' 'aarch64')
url='https://github.com/MarexMB/OpenPods'
license=('MIT')
depends=('bluez' 'gtk4' 'libadwaita' 'dbus' 'glibc' 'gcc-libs')
makedepends=('rust' 'pkgconf')
optdepends=('bluez-utils: Bluetooth diagnostics and pairing'
            'util-linux: rfkill diagnostics'
            'systemd: optional user notification service')
source=("openpods-${pkgver}.tar")
sha256sums=('GENERATED_BY_PACKAGE_SCRIPT')

prepare() {
  cd "openpods-${pkgver}"
  export CARGO_HOME="$srcdir/cargo-home"
  cargo fetch
}

build() {
  cd "openpods-${pkgver}"
  export CARGO_HOME="$srcdir/cargo-home"
  cargo build --release --offline
}

check() {
  cd "openpods-${pkgver}"
  export CARGO_HOME="$srcdir/cargo-home"
  cargo test --offline
}

package() {
  cd "openpods-${pkgver}"
  install -Dm755 target/release/openpods "$pkgdir/usr/bin/openpods"
  install -Dm644 LICENSE "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
  install -Dm644 data/io.github.MarexMB.OpenPods.desktop "$pkgdir/usr/share/applications/io.github.MarexMB.OpenPods.desktop"
  install -Dm644 data/io.github.MarexMB.OpenPods.svg "$pkgdir/usr/share/icons/hicolor/scalable/apps/io.github.MarexMB.OpenPods.svg"
  install -Dm644 data/openpods.service "$pkgdir/usr/lib/systemd/user/openpods.service"
  install -Dm644 README.md "$pkgdir/usr/share/doc/$pkgname/README.md"
  install -Dm644 docs/CACHYOS.md "$pkgdir/usr/share/doc/$pkgname/CACHYOS.md"
}

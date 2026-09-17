# Maintainer: Tu Nombre <tu@email.com>
pkgname=glypho-git
pkgver=0.2.0.r0.g5854904
pkgrel=1
pkgdesc="A markdown previewer focused on speed and simplicity"
arch=('x86_64' 'aarch64')
url="https://github.com/trafkin/glypho"
license=('unknown') # Considera añadir un archivo LICENSE al proyecto
depends=('gcc-libs' 'glibc')
makedepends=('rust' 'cargo' 'nodejs' 'npm' 'git')
provides=('glypho')
conflicts=('glypho')
source=("git+https://github.com/trafkin/glypho.git")
sha256sums=('SKIP')

pkgver() {
  cd "$srcdir/${pkgname%-git}" || return
  git describe --long --tags --always | sed 's/\([^-]*-g\)/r\1/;s/-/./g'
}

prepare() {
  cd "$srcdir/${pkgname%-git}/glypho-web" || return
  # Instalamos dependencias de node para la parte web
  npm install
}

build() {
  cd "$srcdir/${pkgname%-git}" || return

  # Construcción manual de la web para evitar condiciones de carrera en build.rs
  # Esto asegura que src/template.html esté listo para include_str!
  msg2 "Building web components..."
  cd glypho-web || return
  npm run build
  cp dist/index.html ../src/template.html
  cd ..

  # Compilación de Rust
  msg2 "Building Rust binary..."
  export CARGO_HOME="$srcdir/cargo-home"
  cargo build --release --locked
}

check() {
  cd "$srcdir/${pkgname%-git}" || return
}

package() {
  cd "$srcdir/${pkgname%-git}" || return
  install -Dm755 "target/release/glypho" "$pkgdir/usr/bin/glypho"
}

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
  cd "$srcdir/${pkgname%-git}"
  git describe --long --tags --always | sed 's/\([^-]*-g\)/r\1/;s/-/./g'
}

prepare() {
  cd "$srcdir/${pkgname%-git}/glypho-web"
  # Instalamos dependencias de node para la parte web
  npm install
}

build() {
  cd "$srcdir/${pkgname%-git}"

  # Construcción manual de la web para evitar condiciones de carrera en build.rs
  # Esto asegura que src/template.html esté listo para include_str!
  msg2 "Building web components..."
  cd glypho-web
  npm run build
  cp dist/index.html ../src/template.html
  cd ..

  # Compilación de Rust
  msg2 "Building Rust binary..."
  export CARGO_HOME="$srcdir/cargo-home"
  cargo build --release --locked
}

check() {
  cd "$srcdir/${pkgname%-git}"
  # Opcional: ejecutar tests si lo deseas
  # cargo test --release --locked
}

package() {
  cd "$srcdir/${pkgname%-git}"

  # Instalar el binario principal
  install -Dm755 "target/release/glypho" "$pkgdir/usr/bin/glypho"

  # Instalar el plugin de Neovim
  # Buscamos los archivos en nvim-plugin/glypho-nvim/
  install -Dm644 "nvim-plugin/glypho-nvim/lua/glypho.lua" \
    "$pkgdir/usr/share/nvim/runtime/lua/glypho.lua"
  
  # Si existe el archivo de plugin de vim, lo instalamos también
  if [ -f "nvim-plugin/glypho-nvim/plugin/glypho.vim" ]; then
    install -Dm644 "nvim-plugin/glypho-nvim/plugin/glypho.vim" \
      "$pkgdir/usr/share/nvim/runtime/plugin/glypho.vim"
  fi

  # Documentación y ejemplos
  install -Dm644 "README.md" "$pkgdir/usr/share/doc/$pkgname/README.md"
  cp -r examples "$pkgdir/usr/share/doc/$pkgname/examples"
}

project('mecalin', 'rust',
  version: '0.1.0',
  meson_version: '>= 0.59.0',
)

cargo = find_program('cargo')
rustc = find_program('rustc')

gtk4_dep = dependency('gtk4', version: '>= 4.10')
adwaita_dep = dependency('libadwaita-1', version: '>= 1.5')

cargo_options = ['--manifest-path', meson.project_source_root() / 'Cargo.toml']
cargo_env = environment()

if get_option('buildtype') == 'release'
  cargo_options += ['--release']
  rust_target = 'release'
else
  rust_target = 'debug'
endif

cargo_build = custom_target('cargo-build',
  build_by_default: true,
  build_always_stale: true,
  output: meson.project_name(),
  console: true,
  install: true,
  install_dir: get_option('bindir'),
  command: [
    cargo, 'build',
    cargo_options,
  ],
  env: cargo_env,
)

# Desktop file
desktop_file = i18n.merge_file(
  input: 'data' / 'org.gnome.mecalin.desktop.in',
  output: 'org.gnome.mecalin.desktop',
  type: 'desktop',
  po_dir: 'po',
  install: true,
  install_dir: get_option('datadir') / 'applications',
)

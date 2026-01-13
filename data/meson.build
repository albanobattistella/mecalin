# Desktop file
desktop_file = i18n.merge_file(
  input: 'io.github.nacho.mecalin.desktop.in',
  output: 'io.github.nacho.mecalin.desktop',
  type: 'desktop',
  po_dir: '../po',
  install: true,
  install_dir: get_option('datadir') / 'applications',
)

# GSchema
install_data('io.github.nacho.mecalin.gschema.xml',
  install_dir: get_option('datadir') / 'glib-2.0' / 'schemas'
)

subdir('icons')

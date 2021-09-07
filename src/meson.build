global_conf = configuration_data()
global_conf.set_quoted('APP_ID', application_id)
global_conf.set_quoted('PKGDATADIR', pkgdatadir)
global_conf.set_quoted('PROFILE', profile)
global_conf.set_quoted('VERSION', version + version_suffix)
global_conf.set_quoted('GETTEXT_PACKAGE', gettext_package)
global_conf.set_quoted('LOCALEDIR', localedir)
config = configure_file(
  input: 'config.rs.in',
  output: 'config.rs',
  configuration: global_conf
)
# Copy the config.rs output to the source directory.
run_command(
  'cp',
  meson.build_root() / 'src' / 'config.rs',
  meson.source_root() / 'src' / 'config.rs',
  check: true
)

rust_sources = files(
  'components/context_menu_bin.rs',
  'components/mod.rs',
  'session/content/view.rs',
  'session/note/metadata.rs',
  'session/note/mod.rs',
  'session/sidebar/mod.rs',
  'session/sidebar/note_row.rs',
  'session/mod.rs',
  'session/note_list.rs',
  'session/note_manager.rs',
  'application.rs',
  'config.rs',
  'date.rs',
  'error.rs',
  'main.rs',
  'window.rs',
)

sources = [rust_sources, cargo_sources]

custom_target(
  'cargo-build',
  build_by_default: true,
  input: sources,
  output: meson.project_name(),
  console: true,
  install: true,
  install_dir: bindir,
  depends: resources,
  command: [
    cargo_script,
    meson.build_root(),
    meson.source_root(),
    '@OUTPUT@',
    profile,
    meson.project_name(),
  ]
)

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
  'session/content/view/tag_list_view/mod.rs',
  'session/content/view/tag_list_view/row.rs',
  'session/content/view/mod.rs',
  'session/content/mod.rs',
  'session/note/id.rs',
  'session/note/metadata.rs',
  'session/note/mod.rs',
  'session/note/tag.rs',
  'session/note/tag_list.rs',
  'session/sidebar/view_switcher/item.rs',
  'session/sidebar/view_switcher/item_row.rs',
  'session/sidebar/view_switcher/item_list.rs',
  'session/sidebar/view_switcher/mod.rs',
  'session/sidebar/view_switcher/popover.rs',
  'session/sidebar/category.rs',
  'session/sidebar/category_row.rs',
  'session/sidebar/item_list.rs',
  'session/sidebar/mod.rs',
  'session/sidebar/note_row.rs',
  'session/sidebar/row.rs',
  'session/sidebar/selection.rs',
  'session/tag_dialog/mod.rs',
  'session/tag_dialog/row.rs',
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

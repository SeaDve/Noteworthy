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
  'core/mod.rs',
  'core/note_manager.rs',
  'core/note_repository.rs',
  'core/repository.rs',
  'core/repository_watcher.rs',
  'model/note/id.rs',
  'model/note/metadata.rs',
  'model/note/mod.rs',
  'model/date.rs',
  'model/mod.rs',
  'model/note_list.rs',
  'model/note_tag_list.rs',
  'model/tag.rs',
  'model/tag_list.rs',
  'session/content/view/tag_list_view/mod.rs',
  'session/content/view/tag_list_view/row.rs',
  'session/content/view/mod.rs',
  'session/content/mod.rs',
  'session/note_tag_dialog/mod.rs',
  'session/note_tag_dialog/row.rs',
  'session/sidebar/view_switcher/item.rs',
  'session/sidebar/view_switcher/item_list.rs',
  'session/sidebar/view_switcher/item_row.rs',
  'session/sidebar/view_switcher/mod.rs',
  'session/sidebar/view_switcher/popover.rs',
  'session/sidebar/mod.rs',
  'session/sidebar/note_row.rs',
  'session/sidebar/selection.rs',
  'session/sidebar/sync_button.rs',
  'session/tag_editor/mod.rs',
  'session/tag_editor/row.rs',
  'session/mod.rs',
  'application.rs',
  'config.rs',
  'main.rs',
  'setup.rs',
  'utils.rs',
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

test(
  'cargo-test',
  test_script,
  args: meson.build_root(),
  workdir: meson.source_root(),
  timeout: 3000
)

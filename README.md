<h1 align="center">
  Noteworthy
</h1>

<p align="center"><strong>Modern, Fast and Simple Notes App for the GNOME Desktop</strong></p>

<p align="center">
  <a href="https://github.com/SeaDve/Noteworthy/actions/workflows/ci.yml">
    <img src="https://github.com/SeaDve/Noteworthy/actions/workflows/ci.yml/badge.svg" alt="CI status"/>
  </a>
  <a href="https://repology.org/project/noteworthy/versions">
    <img src="https://repology.org/badge/tiny-repos/noteworthy.svg" alt="Packaging status">
  </a>
</p>

## v0.1.0 Milestone

- [x] Trash and pinning
- [x] Note creation and deletion
- [x] Note metadata
- [x] Powerful tag system
- [x] Filtering
- [x] Basic markdown
- [x] Batch notes selection and editing
- [ ] Attachments
- [ ] Syncing (Barely working)
- [ ] Git integration (Barely working)
- [ ] Setup page
- [ ] WYSIWG Editing
- [ ] Homepage (Includes reminders, recents, mini notepads etc.)


## Installation Instructions

Noteworthy is under heavy development. Thus, it is currently not recommended to 
be used for day-to-day tasks. However, it is possible to download the nightly
build artifact from the [Actions page](https://github.com/SeaDve/Noteworthy/actions/),
then install it locally by running `flatpak install noteworthy.flatpak`.


## Build Instructions

### GNOME Builder

GNOME Builder is the environment used for developing this application.
It can use Flatpak manifests to create a consistent building and running
environment cross-distro. Thus, it is highly recommended you use it.

1. Download [GNOME Builder](https://flathub.org/apps/details/org.gnome.Builder).
2. In Builder, click the "Clone Repository" button at the bottom, using 
`https://github.com/SeaDve/Noteworthy.git` as the URL.
3. Click the build button at the top once the project is loaded.

### Meson

#### Prerequisites

The following packages are required to build Kooha:

* meson
* ninja
* appstream-glib (for checks)
* cargo
* gstreamer
* gstreamer-plugins-base
* glib2
* gtk4
* gtksourceview5
* libadwaita

#### Build Instructions

```shell
meson . _build --prefix=/usr/local
ninja -C _build
ninja -C _build install
```

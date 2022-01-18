use gtk::{
    gio,
    glib::{self, clone},
    prelude::*,
    subclass::prelude::*,
};
use once_cell::unsync::OnceCell;
use serde::{Deserialize, Serialize};

use std::{
    cell::{Cell, RefCell},
    path::PathBuf,
};

use crate::{
    core::{NoteRepository, SyncState},
    model::{Note, NoteId, NoteList, TagList},
    spawn, utils,
};

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default)]
struct Data {
    tag_list: TagList,
}

mod imp {
    use super::*;
    use once_cell::sync::Lazy;

    #[derive(Debug, Default)]
    pub struct NoteManager {
        pub directory: OnceCell<gio::File>,
        pub repository: OnceCell<NoteRepository>,
        pub note_list: OnceCell<NoteList>,
        pub tag_list: RefCell<Option<TagList>>,
        pub is_syncing: Cell<bool>,
        pub is_offline_mode: Cell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for NoteManager {
        const NAME: &'static str = "NwtyNoteManager";
        type Type = super::NoteManager;
    }

    impl ObjectImpl for NoteManager {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecObject::new(
                        "directory",
                        "Directory",
                        "Directory where the notes are stored",
                        gio::File::static_type(),
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
                    ),
                    glib::ParamSpecObject::new(
                        "repository",
                        "Repository",
                        "Repository where the notes are stored",
                        NoteRepository::static_type(),
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
                    ),
                    glib::ParamSpecObject::new(
                        "note-list",
                        "Note List",
                        "List of notes",
                        NoteList::static_type(),
                        glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpecObject::new(
                        "tag-list",
                        "Tag List",
                        "List of tags",
                        TagList::static_type(),
                        glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpecBoolean::new(
                        "is-syncing",
                        "Is Syncing",
                        "Whether the session is currently syncing",
                        false,
                        glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpecBoolean::new(
                        "is-offline-mode",
                        "Is Offline Mode",
                        "Whether the repo syncs to a remote repo",
                        false,
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT,
                    ),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(
            &self,
            _obj: &Self::Type,
            _id: usize,
            value: &glib::Value,
            pspec: &glib::ParamSpec,
        ) {
            match pspec.name() {
                "directory" => {
                    let directory = value.get().unwrap();
                    self.directory.set(directory).unwrap();
                }
                "repository" => {
                    let repository = value.get().unwrap();
                    self.repository.set(repository).unwrap();
                }
                "note-list" => {
                    let note_list = value.get().unwrap();
                    self.note_list.set(note_list).unwrap();
                }
                "tag-list" => {
                    let tag_list = value.get().unwrap();
                    self.tag_list.replace(Some(tag_list));
                }
                "is-syncing" => {
                    let is_syncing = value.get().unwrap();
                    self.is_syncing.set(is_syncing);
                }
                "is-offline-mode" => {
                    let is_offline_mode = value.get().unwrap();
                    self.is_offline_mode.set(is_offline_mode);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "directory" => obj.directory().to_value(),
                "repository" => obj.repository().to_value(),
                "note-list" => obj.note_list().to_value(),
                "tag-list" => obj.tag_list().to_value(),
                "is-syncing" => self.is_syncing.get().to_value(),
                "is-offline-mode" => self.is_offline_mode.get().to_value(),
                _ => unimplemented!(),
            }
        }

        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            obj.setup_bindings();
            obj.setup_signals();
        }
    }
}

glib::wrapper! {
    pub struct NoteManager(ObjectSubclass<imp::NoteManager>);
}

impl NoteManager {
    // TODO add ways to convert offline mode to online mode
    pub async fn for_directory(directory: &gio::File, is_offline_mode: bool) -> Self {
        let repository = {
            let res = if is_offline_mode {
                NoteRepository::init(directory).await
            } else {
                NoteRepository::clone("git@github.com:SeaDve/test.git".into(), directory).await
            };

            if let Err(err) = res {
                log::warn!("Failed to clone or init repo: {:?}", err);
                log::info!("Opening existing instead...");
                NoteRepository::open(directory).await.unwrap()
            } else {
                res.unwrap()
            }
        };

        glib::Object::new(&[
            ("directory", directory),
            ("repository", &repository),
            ("is-offline-mode", &is_offline_mode),
        ])
        .expect("Failed to create NoteManager.")
    }

    pub fn directory(&self) -> gio::File {
        self.imp().directory.get().unwrap().clone()
    }

    pub fn repository(&self) -> NoteRepository {
        Clone::clone(self.imp().repository.get().unwrap())
    }

    pub fn note_list(&self) -> NoteList {
        self.imp()
            .note_list
            .get()
            .expect("Please call `load_notes` first")
            .clone()
    }

    pub fn tag_list(&self) -> TagList {
        self.imp()
            .tag_list
            .borrow()
            .clone()
            .expect("Please call `load_data_file` first")
    }

    pub fn is_offline_mode(&self) -> bool {
        self.property("is-offline-mode")
    }

    async fn load_notes(&self) -> anyhow::Result<()> {
        let note_list = NoteList::load_from_dir(&self.directory()).await?;

        self.set_property("note-list", note_list);

        Ok(())
    }

    async fn load_data_file(&self) -> anyhow::Result<()> {
        let data_file_path = self.data_file_path();
        let file = gio::File::for_path(&data_file_path);

        let data: Data = match file.load_contents_future().await {
            Ok((file_content, _)) => {
                log::info!(
                    "Data file found at `{}` is loaded successfully",
                    data_file_path.display()
                );
                serde_yaml::from_slice(&file_content).unwrap_or_default()
            }
            Err(err) => {
                log::warn!(
                    "Falling back to default data, Failed to load data file: {:?}",
                    err
                );
                Data::default()
            }
        };

        self.set_property("tag-list", data.tag_list);

        Ok(())
    }

    pub async fn save_all_notes(&self) -> anyhow::Result<()> {
        let unsaved_notes = self.note_list().take_unsaved_notes();

        if unsaved_notes.is_empty() {
            log::info!("No unsaved notes, skipping save...");
        }

        for note in &unsaved_notes {
            note.save().await?;
        }

        Ok(())
    }

    pub async fn save_data_file(&self) -> anyhow::Result<()> {
        let data = Data {
            tag_list: self.tag_list(),
        };
        let data_bytes = serde_yaml::to_vec(&data)?;

        // FIXME consider making backup on all replace_contents
        gio::File::for_path(self.data_file_path())
            .replace_contents_future(data_bytes, None, false, gio::FileCreateFlags::NONE)
            .await
            .map_err(|err| err.1)?;

        log::info!("Successfully saved data file");

        Ok(())
    }

    pub fn create_note(&self) {
        let base_path = self.directory().path().unwrap();
        let new_note_path = utils::generate_unique_path(base_path, "Note", Some("md"));
        let new_note = Note::new(&gio::File::for_path(new_note_path));

        log::info!("Created note `{}`", new_note);

        self.note_list().append(new_note);
    }

    pub async fn load(&self) -> anyhow::Result<()> {
        self.load_data_file().await?;
        self.load_notes().await?;

        Ok(())
    }

    // TODO Application::inhibit while syncing
    // TODO Better way to handle trying to sync multiple times (maybe refactor to use a thread pool)
    pub async fn sync(&self) -> anyhow::Result<()> {
        let repo = self.repository();

        if repo.sync_state() == SyncState::Pulling {
            log::info!("Currently pulling. Returning and skipping session sync...");
            return Ok(());
        }

        self.save_all_notes().await?;
        self.save_data_file().await?;

        let is_offline_mode = self.is_offline_mode();
        if is_offline_mode {
            repo.sync_offline().await?;
        } else {
            let changed_files = repo.sync().await?;
            self.handle_changed_files(&changed_files).await?;
        }

        log::info!("Session synced; is_offline_mode `{}`", is_offline_mode);

        Ok(())
    }

    async fn handle_changed_files(
        &self,
        changed_files: &[(PathBuf, git2::Delta)],
    ) -> anyhow::Result<()> {
        let note_list = self.note_list();
        let data_file_path = self.data_file_path();

        for (path, delta) in changed_files {
            if path == &data_file_path {
                // FIXME handle changed data file too, especially the tag list
                continue;
            }

            match delta {
                git2::Delta::Added => {
                    log::info!("Sync: Found added files `{}`; appending...", path.display());
                    let file = gio::File::for_path(&path);
                    let added_note = Note::load(&file).await?;
                    note_list.append(added_note);
                }
                git2::Delta::Deleted => {
                    log::info!(
                        "Sync: Found removed files `{}`; removing...",
                        path.display()
                    );
                    let note_id = NoteId::from_path(path);
                    note_list.remove(&note_id);
                }
                git2::Delta::Modified => {
                    log::info!(
                        "Sync: Found modified files `{}`; updating...",
                        path.display()
                    );
                    let note_id = NoteId::from_path(path);
                    let note = note_list.get(&note_id).unwrap();
                    note.update().await?;
                }
                other => {
                    log::warn!("Found other delta type: {:?}", other);
                }
            }
        }

        Ok(())
    }

    fn data_file_path(&self) -> PathBuf {
        let mut data_file_path = self.directory().path().unwrap();
        data_file_path.push("data.nwty");
        data_file_path
    }

    fn setup_bindings(&self) {
        self.repository()
            .bind_property("sync-state", self, "is-syncing")
            .transform_to(|_, value| {
                let sync_state: SyncState = value.get().unwrap();
                let is_syncing = sync_state != SyncState::Idle;

                Some(is_syncing.to_value())
            })
            .flags(glib::BindingFlags::SYNC_CREATE)
            .build();
    }

    fn setup_signals(&self) {
        if !self.is_offline_mode() {
            self.repository()
                .connect_remote_changed(clone!(@weak self as obj => move |_| {
                    log::info!("New remote changes! Syncing...");
                    spawn!(async move {
                        if let Err(err) = obj.sync().await {
                            log::error!("Failed to sync: {:?}", err);
                        }
                    });
                }));
        }
    }
}

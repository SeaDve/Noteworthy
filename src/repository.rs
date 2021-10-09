use gtk::{gio, glib, prelude::*, subclass::prelude::*};
use once_cell::unsync::OnceCell;

use std::{fs::File, io::Write, path::PathBuf, thread};

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct Repository {
        pub remote_url: OnceCell<String>,
        pub local_path: OnceCell<gio::File>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Repository {
        const NAME: &'static str = "NwtyRepository";
        type Type = super::Repository;
        type ParentType = glib::Object;
    }

    impl ObjectImpl for Repository {
        fn properties() -> &'static [glib::ParamSpec] {
            use once_cell::sync::Lazy;
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpec::new_string(
                        "remote-url",
                        "Remote Url",
                        "Remote URL of the repository",
                        None,
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
                    ),
                    glib::ParamSpec::new_object(
                        "local-path",
                        "Local Path",
                        "Where the repository is stored locally",
                        gio::File::static_type(),
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
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
                "remote-url" => {
                    let remote_url = value.get().unwrap();
                    self.remote_url.set(remote_url).unwrap();
                }
                "local-path" => {
                    let local_path = value.get().unwrap();
                    self.local_path.set(local_path).unwrap();
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "remote-url" => self.remote_url.get().to_value(),
                "local-path" => self.local_path.get().to_value(),
                _ => unimplemented!(),
            }
        }

        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
        }
    }
}

glib::wrapper! {
    pub struct Repository(ObjectSubclass<imp::Repository>);
}

impl Repository {
    pub fn new(remote_url: &String, local_path: &gio::File) -> Self {
        glib::Object::new::<Self>(&[("remote-url", remote_url), ("local-path", local_path)])
            .expect("Failed to create Repository.")
    }

    pub fn remote_url(&self) -> String {
        self.property("remote-url").unwrap().get().unwrap()
    }

    pub fn local_path(&self) -> gio::File {
        self.property("local-path").unwrap().get().unwrap()
    }

    pub async fn clone(&self, passphrase: Option<&str>) -> anyhow::Result<()> {
        let (sender, receiver) = futures::channel::oneshot::channel();

        // FIXME dont clone
        let passphrase = passphrase.map(std::string::ToString::to_string);
        let remote_url = self.remote_url();
        let local_path = self.local_path();

        thread::spawn(move || {
            let mut callbacks = git2::RemoteCallbacks::new();
            callbacks.credentials(|_url, username_from_url, _allowed_types| {
                let mut ssh_key_path = glib::home_dir();
                ssh_key_path.push(".ssh/id_ed25519");

                log::info!("Credential callback");

                git2::Cred::ssh_key(
                    username_from_url.unwrap(),
                    None,
                    &ssh_key_path,
                    passphrase.as_deref(),
                )
            });
            callbacks.transfer_progress(|progress| {
                dbg!(progress.total_objects());
                dbg!(progress.indexed_objects());
                dbg!(progress.received_objects());
                dbg!(progress.local_objects());
                dbg!(progress.total_deltas());
                dbg!(progress.indexed_deltas());
                dbg!(progress.received_bytes());
                true
            });

            log::info!("Preparing to clone");

            let mut fo = git2::FetchOptions::new();
            fo.remote_callbacks(callbacks);

            let mut builder = git2::build::RepoBuilder::new();
            builder.fetch_options(fo);

            let res = builder.clone(&remote_url, &local_path.path().unwrap());
            sender.send(res)
        });

        receiver.await.unwrap()?;

        Ok(())
    }

    pub async fn push(&self, remote_name: &str, passphrase: Option<&str>) -> anyhow::Result<()> {
        let (sender, receiver) = futures::channel::oneshot::channel();

        let remote_name = remote_name.to_string();
        let passphrase = passphrase.map(std::string::ToString::to_string);
        let local_path = self.local_path();

        thread::spawn(move || {
            let res = Repository::_push(&local_path, &remote_name, passphrase.as_deref());
            sender.send(res).unwrap();
        });

        receiver.await.unwrap()?;

        Ok(())
    }

    fn _push(
        local_path: &gio::File,
        remote_name: &str,
        passphrase: Option<&str>,
    ) -> anyhow::Result<()> {
        let repo = git2::Repository::open(&local_path.path().unwrap())?;
        let mut remote = repo.find_remote(remote_name)?;
        let ref_head = repo.head()?;
        let ref_head_name = ref_head
            .name()
            .ok_or_else(|| anyhow::anyhow!("Ref head name not found"))?;

        let head_type = ref_head.kind();

        if head_type != Some(git2::ReferenceType::Direct) {
            anyhow::bail!("Head is not a direct reference.")
        }

        let mut callbacks = git2::RemoteCallbacks::new();
        callbacks.credentials(|_url, username_from_url, _allowed_types| {
            let mut ssh_key_path = glib::home_dir();
            ssh_key_path.push(".ssh/id_ed25519");

            log::info!("Credential callback");

            git2::Cred::ssh_key(username_from_url.unwrap(), None, &ssh_key_path, passphrase)
        });
        callbacks.transfer_progress(|progress| {
            dbg!(progress.total_objects());
            dbg!(progress.indexed_objects());
            dbg!(progress.received_objects());
            dbg!(progress.local_objects());
            dbg!(progress.total_deltas());
            dbg!(progress.indexed_deltas());
            dbg!(progress.received_bytes());
            true
        });

        log::info!("Preparing to push local changes");

        let mut po = git2::PushOptions::new();
        po.remote_callbacks(callbacks);

        remote.push(&[ref_head_name], Some(&mut po))?;

        Ok(())
    }

    pub fn commit(
        &self,
        author_name: &str,
        author_email: &str,
        message: &str,
    ) -> anyhow::Result<()> {
        let repo = git2::Repository::open(self.local_path().path().unwrap())?;

        let mut index = repo.index()?;
        let tree_id = index.write_tree()?;
        let tree = repo.find_tree(tree_id)?;

        let signature = git2::Signature::now(author_name, author_email)?;

        match repo.refname_to_id("HEAD") {
            Ok(parent_id) => {
                let parent_commit = repo.find_commit(parent_id)?;
                repo.commit(
                    Some("HEAD"),
                    &signature,
                    &signature,
                    message,
                    &tree,
                    &[&parent_commit],
                )?;
            }
            Err(err) => {
                repo.commit(Some("HEAD"), &signature, &signature, message, &tree, &[])?;
                log::warn!("Failed to refname_to_id: {}", err);
            }
        };

        Ok(())
    }

    pub fn fetch(&self, remote_name: &str, passphrase: Option<&str>) -> anyhow::Result<()> {
        let repo = git2::Repository::open(self.local_path().path().unwrap())?;
        let mut remote = repo.find_remote(remote_name)?;

        let mut callbacks = git2::RemoteCallbacks::new();
        callbacks.credentials(|_url, username_from_url, _allowed_types| {
            let mut ssh_key_path = glib::home_dir();
            ssh_key_path.push(".ssh/id_ed25519");

            log::info!("Credential callback");

            git2::Cred::ssh_key(
                username_from_url.unwrap(),
                None,
                &ssh_key_path,
                passphrase.as_deref(),
            )
        });
        callbacks.transfer_progress(|progress| {
            dbg!(progress.total_objects());
            dbg!(progress.indexed_objects());
            dbg!(progress.received_objects());
            dbg!(progress.local_objects());
            dbg!(progress.total_deltas());
            dbg!(progress.indexed_deltas());
            dbg!(progress.received_bytes());
            true
        });

        log::info!("Preparing to clone");

        let mut fo = git2::FetchOptions::new();
        fo.remote_callbacks(callbacks);

        remote.fetch::<&str>(&[], Some(&mut fo), None)?;

        Ok(())
    }

    // From https://github.com/GitJournal/git_bindings/blob/master/gj_common/gitjournal.c
    pub fn merge(
        &self,
        source_branch: &str,
        author_name: &str,
        author_email: &str,
    ) -> anyhow::Result<()> {
        let repo = git2::Repository::open(self.local_path().path().unwrap())?;
        let origin_head_ref = repo.find_branch(source_branch, git2::BranchType::Remote)?;
        let origin_annotated_commit = repo.reference_to_annotated_commit(origin_head_ref.get())?;
        let (merge_analysis, _) = repo.merge_analysis(&[&origin_annotated_commit])?;

        dbg!(merge_analysis);

        if merge_analysis.contains(git2::MergeAnalysis::ANALYSIS_UP_TO_DATE) {
            log::info!("Merge analysis: Up to date");
            return Ok(());
        } else if merge_analysis.contains(git2::MergeAnalysis::ANALYSIS_UNBORN) {
            anyhow::bail!("Merge analysis: Unborn");
        } else if merge_analysis.contains(git2::MergeAnalysis::ANALYSIS_FASTFORWARD) {
            log::info!("Merge analysis: Fastforwarding...");
            let target_oid = origin_annotated_commit.id();
            Self::perform_fastforward(&repo, target_oid)?;
        } else if merge_analysis.contains(git2::MergeAnalysis::ANALYSIS_NORMAL) {
            log::info!("Merge analysis: Performing normal merge...");

            repo.merge(&[&origin_annotated_commit], None, None)?;
            let mut index = repo.index()?;
            let conflicts = index.conflicts()?;

            for conflict in conflicts.flatten() {
                let our = conflict.our.unwrap();
                let their = conflict.their.unwrap();

                let current_conflict_path = std::str::from_utf8(&their.path).unwrap();
                log::info!("Pull: Conflict on file {}", current_conflict_path);
                Repository::resolve_conflict(&repo, &our)?;
                log::info!("Resolved conflict on file {}", current_conflict_path);

                let path = std::str::from_utf8(&our.path).unwrap();
                let path = PathBuf::from(&path);

                let mut index = repo.index()?;
                index.remove_path(&path)?;
                index.add_all([our.path], git2::IndexAddOption::DEFAULT, None)?;
                index.write()?;
            }

            let tree_id = index.write_tree()?;
            let tree = repo.find_tree(tree_id)?;
            let signature = git2::Signature::now(author_name, author_email)?;
            let head_id = repo.refname_to_id("HEAD")?;
            let head_commit = repo.find_commit(head_id)?;
            let origin_head_commit = repo.find_commit(origin_annotated_commit.id())?;

            let parents = [&head_commit, &origin_head_commit];
            let message = "Custom merge commit";
            repo.commit(
                Some("HEAD"),
                &signature,
                &signature,
                message,
                &tree,
                &parents,
            )?;
        }

        Ok(())
    }

    fn perform_fastforward(repo: &git2::Repository, target_oid: git2::Oid) -> anyhow::Result<()> {
        let mut target_ref = repo.head()?;
        let target = repo.find_object(target_oid, Some(git2::ObjectType::Commit))?;

        repo.checkout_tree(&target, None)?;
        target_ref.set_target(target_oid, "")?;

        Ok(())
    }

    fn resolve_conflict(repo: &git2::Repository, our: &git2::IndexEntry) -> anyhow::Result<()> {
        let odb = repo.odb()?;
        let odb_object = odb.read(our.id)?;
        let file_data = odb_object.data();

        let file_path = &our.path;
        let mut file_full_path = PathBuf::from(repo.path());
        file_full_path.push(std::str::from_utf8(file_path).unwrap());

        let mut file = File::open(file_full_path)?;
        file.write_all(file_data)?;

        Ok(())
    }
}

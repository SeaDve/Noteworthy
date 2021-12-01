use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};

pub struct Repository {
    inner: git2::Repository,
    base_path: PathBuf,
}

impl std::fmt::Debug for Repository {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Repository")
            .field("base_path", &self.base_path)
            .finish()
    }
}

impl Repository {
    pub fn init(base_path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let mut init_options = git2::RepositoryInitOptions::new();
        init_options.no_reinit(true);

        let repo = git2::Repository::init_opts(base_path.as_ref(), &init_options)?;

        Ok(Self {
            inner: repo,
            base_path: base_path.as_ref().to_owned(),
        })
    }

    pub fn clone(base_path: impl AsRef<Path>, remote_url: &str) -> anyhow::Result<Self> {
        let mut callbacks = git2::RemoteCallbacks::new();
        callbacks.credentials(|_, username_from_url, _| Self::credentials_cb(username_from_url));
        callbacks.transfer_progress(Self::transfer_progress_cb);

        let mut fetch_options = git2::FetchOptions::new();
        fetch_options.remote_callbacks(callbacks);

        let mut repo_builder = git2::build::RepoBuilder::new();
        repo_builder.fetch_options(fetch_options);

        log::info!("Cloning from {} ...", remote_url);
        let repo = repo_builder.clone(remote_url, base_path.as_ref())?;

        Ok(Self {
            inner: repo,
            base_path: base_path.as_ref().to_owned(),
        })
    }

    pub fn open(base_path: &Path) -> anyhow::Result<Self> {
        log::info!("Opening repo from {}", base_path.display());
        let repo = git2::Repository::open(base_path)?;

        Ok(Self {
            inner: repo,
            base_path: base_path.to_owned(),
        })
    }

    pub fn base_path(&self) -> &Path {
        self.base_path.as_path()
    }

    pub fn remotes(&self) -> anyhow::Result<Vec<String>> {
        let repo = self.inner();
        let remotes = repo.remotes()?;

        // TODO find ways to not allocate a Vec
        Ok(remotes
            .iter()
            .flatten()
            .map(std::string::ToString::to_string)
            .collect())
    }

    pub fn diff_tree_to_tree(
        &self,
        old_tree: &git2::Tree,
        new_tree: &git2::Tree,
    ) -> anyhow::Result<Vec<(PathBuf, git2::Delta)>> {
        let repo = self.inner();

        let diff = repo.diff_tree_to_tree(Some(old_tree), Some(new_tree), None)?;

        let mut files = Vec::new();
        for diff_delta in diff.deltas() {
            let old_file_path = diff_delta.old_file().path().unwrap();
            let new_file_path = diff_delta.new_file().path().unwrap();
            log::info!(
                "Diff: Found file {} -> {}",
                old_file_path.display(),
                new_file_path.display()
            );

            let file_path = self.base_path().join(new_file_path);
            let status = diff_delta.status();

            files.push((file_path, status));
        }

        Ok(files)
    }

    pub fn is_file_changed_in_workdir(&self) -> anyhow::Result<bool> {
        let repo = self.inner();

        let mut diff_options = git2::DiffOptions::new();
        diff_options.include_untracked(true);

        let diff = repo.diff_index_to_workdir(None, Some(&mut diff_options))?;
        let diff_stats = diff.stats()?;
        Ok(diff_stats.files_changed() > 0)
    }

    pub fn is_same(&self, spec_a: &str, spec_b: &str) -> anyhow::Result<bool> {
        let repo = self.inner();

        let object_a_id = repo.revparse_single(spec_a)?.id();
        let object_b_id = repo.revparse_single(spec_b)?.id();

        log::info!("Revparse spec_a: {} <-> {}", spec_a, object_a_id);
        log::info!("Revparse spec_a: {} <-> {}", spec_b, object_b_id);

        Ok(object_a_id == object_b_id)
    }

    pub fn fetch(&self, remote_name: &str) -> anyhow::Result<()> {
        let repo = self.inner();

        let mut remote = repo.find_remote(remote_name)?;

        let mut callbacks = git2::RemoteCallbacks::new();
        callbacks.credentials(|_, username_from_url, _| Self::credentials_cb(username_from_url));
        callbacks.transfer_progress(Self::transfer_progress_cb);

        let mut fetch_options = git2::FetchOptions::new();
        fetch_options.remote_callbacks(callbacks);

        log::info!("Fetching from {} ...", remote_name);
        remote.fetch::<&str>(&[], Some(&mut fetch_options), None)?;

        Ok(())
    }

    pub fn add(&self, paths: &[impl AsRef<Path>]) -> anyhow::Result<()> {
        let repo = self.inner();

        let mut index = repo.index()?;

        index.add_all(
            paths.iter().map(|p| p.as_ref()),
            git2::IndexAddOption::DEFAULT,
            Some(&mut |path: &Path, _: &[u8]| {
                log::info!("Add match: {}", path.display());
                0
            }),
        )?;
        index.write()?;

        Ok(())
    }

    pub fn remove(&self, paths: &[impl AsRef<Path>]) -> anyhow::Result<()> {
        let repo = self.inner();

        let mut index = repo.index()?;

        index.remove_all(
            paths.iter().map(|p| p.as_ref()),
            Some(&mut |path: &Path, _: &[u8]| {
                let full_path = self.base_path().join(path);

                log::info!("Removing file: {}", full_path.display());
                if let Err(err) = fs::remove_file(&full_path) {
                    log::error!("File {} could not be deleted {}", full_path.display(), err);
                }

                0
            }),
        )?;

        Ok(())
    }

    // From https://github.com/GitJournal/git_bindings/blob/master/gj_common/gitjournal.c
    pub fn merge<'a>(
        &self,
        source_branch: &str,
        fetch_commit: Option<git2::AnnotatedCommit<'a>>,
        author_name: &str,
        author_email: &str,
    ) -> anyhow::Result<()> {
        let repo = self.inner();

        let annotated_commit = match fetch_commit {
            Some(commit) => commit,
            None => {
                let origin_head_ref = repo.find_branch(source_branch, git2::BranchType::Remote)?;
                repo.reference_to_annotated_commit(origin_head_ref.get())?
            }
        };

        let (merge_analysis, _) = repo.merge_analysis(&[&annotated_commit])?;
        log::info!("Merge analysis: {:?}", merge_analysis);

        if merge_analysis.contains(git2::MergeAnalysis::ANALYSIS_UP_TO_DATE) {
            log::info!("Merge analysis: Up to date");
        } else if merge_analysis.contains(git2::MergeAnalysis::ANALYSIS_UNBORN) {
            anyhow::bail!("Merge analysis: Unborn");
        } else if merge_analysis.contains(git2::MergeAnalysis::ANALYSIS_FASTFORWARD) {
            log::info!("Merge analysis: Fastforwarding...");
            let target_oid = annotated_commit.id();
            self.perform_fastforward(target_oid)?;
        } else if merge_analysis.contains(git2::MergeAnalysis::ANALYSIS_NORMAL) {
            log::info!("Merge analysis: Performing normal merge...");

            repo.merge(&[&annotated_commit], None, None)?;
            let mut index = repo.index()?;
            let conflicts = index.conflicts()?;

            for conflict in conflicts.flatten() {
                let our = conflict.our.unwrap();
                let their = conflict.their.unwrap();

                let current_conflict_path = std::str::from_utf8(&their.path).unwrap();
                log::info!("Pull: Conflict on file {}", current_conflict_path);
                self.resolve_conflict(&our)?;
                log::info!("Resolved conflict on file {}", current_conflict_path);

                let path = std::str::from_utf8(&our.path).unwrap();
                let path = Path::new(&path);

                let mut index = repo.index()?;
                index.remove_path(path)?;
                index.add_all([our.path], git2::IndexAddOption::DEFAULT, None)?;
                index.write()?;
            }

            let tree_id = index.write_tree()?;
            let tree = repo.find_tree(tree_id)?;
            let signature = git2::Signature::now(author_name, author_email)?;
            let head_id = repo.refname_to_id("HEAD")?;
            let head_commit = repo.find_commit(head_id)?;
            let origin_head_commit = repo.find_commit(annotated_commit.id())?;

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

    pub fn commit(
        &self,
        message: &str,
        author_name: &str,
        author_email: &str,
    ) -> anyhow::Result<()> {
        let repo = self.inner();

        let mut index = repo.index()?;
        let tree_id = index.write_tree()?;
        let tree = repo.find_tree(tree_id)?;

        let signature = git2::Signature::now(author_name, author_email)?;

        log::info!("Creating commit...");
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

    pub fn push(&self, remote_name: &str) -> anyhow::Result<()> {
        let repo = self.inner();

        let mut remote = repo.find_remote(remote_name)?;
        let ref_head = repo.head()?;
        let ref_head_name = ref_head
            .name()
            .ok_or_else(|| anyhow::anyhow!("Ref head name not found"))?;

        let head_type = ref_head.kind();

        anyhow::ensure!(
            head_type == Some(git2::ReferenceType::Direct),
            "Head is not a direct reference"
        );

        let mut callbacks = git2::RemoteCallbacks::new();
        callbacks.credentials(|_, username_from_url, _| Self::credentials_cb(username_from_url));
        callbacks.transfer_progress(Self::transfer_progress_cb);

        let mut push_options = git2::PushOptions::new();
        push_options.remote_callbacks(callbacks);

        log::info!("Pushing to {} ...", remote_name);
        remote.push(&[ref_head_name], Some(&mut push_options))?;

        Ok(())
    }

    pub fn pull(
        &self,
        remote_name: &str,
        author_name: &str,
        author_email: &str,
    ) -> anyhow::Result<Vec<(PathBuf, git2::Delta)>> {
        let repo = self.inner();

        self.fetch(remote_name)?;

        let head = repo.find_reference("HEAD")?;
        let old_tree = head.peel_to_tree()?;

        let fetch_head = repo.find_reference("FETCH_HEAD")?;
        let new_tree = fetch_head.peel_to_tree()?;

        let fetch_commit = repo.reference_to_annotated_commit(&fetch_head)?;

        // TODO better way to get this?
        let head = repo.head()?;
        let branch_name = head
            .name()
            .ok_or_else(|| anyhow::anyhow!("Ref head name not found"))?;
        let source_branch = format!("{}/{}", remote_name, branch_name);

        self.merge(
            &source_branch,
            Some(fetch_commit),
            author_name,
            author_email,
        )?;

        let changed_files = self.diff_tree_to_tree(&old_tree, &new_tree)?;
        Ok(changed_files)
    }

    fn perform_fastforward(&self, target_oid: git2::Oid) -> anyhow::Result<()> {
        let repo = self.inner();

        let mut target_ref = repo.head()?;
        let target = repo.find_object(target_oid, Some(git2::ObjectType::Commit))?;

        repo.checkout_tree(&target, None)?;
        target_ref.set_target(target_oid, "")?;

        Ok(())
    }

    fn resolve_conflict(&self, our: &git2::IndexEntry) -> anyhow::Result<()> {
        let repo = self.inner();

        let odb = repo.odb()?;
        let odb_object = odb.read(our.id)?;
        let file_data = odb_object.data();

        let file_path = &our.path;
        let file_full_path = self
            .base_path()
            .join(std::str::from_utf8(file_path).unwrap());

        let mut file = File::open(file_full_path)?;
        file.write_all(file_data)?;

        Ok(())
    }

    fn inner(&self) -> &git2::Repository {
        &self.inner
    }

    fn credentials_cb(username_from_url: Option<&str>) -> Result<git2::Cred, git2::Error> {
        log::info!(
            "Credential callback with username: {}",
            username_from_url.unwrap()
        );
        git2::Cred::ssh_key_from_agent(username_from_url.unwrap())
    }

    fn transfer_progress_cb(progress: git2::Progress) -> bool {
        if progress.received_objects() == progress.total_objects() {
            log::info!(
                "Resolving deltas {}/{}",
                progress.indexed_deltas(),
                progress.total_deltas()
            );
        } else if progress.total_objects() > 0 {
            log::info!(
                "Received {}/{} objects ({}) in {} bytes",
                progress.received_objects(),
                progress.total_objects(),
                progress.indexed_objects(),
                progress.received_bytes()
            );
        }
        true
    }
}

use gtk::glib;

use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};

pub fn clone(git_base_path: &Path, remote_url: &str) -> anyhow::Result<git2::Repository> {
    let mut callbacks = git2::RemoteCallbacks::new();
    callbacks.credentials(|_, username_from_url, _| credentials_cb(username_from_url));
    callbacks.transfer_progress(transfer_progress_cb);

    let mut fetch_options = git2::FetchOptions::new();
    fetch_options.remote_callbacks(callbacks);

    let mut repo_builder = git2::build::RepoBuilder::new();
    repo_builder.fetch_options(fetch_options);

    log::info!("Cloning from {} ...", remote_url);
    let repo = repo_builder.clone(remote_url, git_base_path)?;

    Ok(repo)
}

pub fn open(git_base_path: &Path) -> anyhow::Result<git2::Repository> {
    let repo = git2::Repository::open(git_base_path)?;

    Ok(repo)
}

pub fn fetch(repo: &git2::Repository, remote_name: &str) -> anyhow::Result<()> {
    let mut remote = repo.find_remote(remote_name)?;

    let mut callbacks = git2::RemoteCallbacks::new();
    callbacks.credentials(|_, username_from_url, _| credentials_cb(username_from_url));
    callbacks.transfer_progress(transfer_progress_cb);

    let mut fetch_options = git2::FetchOptions::new();
    fetch_options.remote_callbacks(callbacks);

    log::info!("Fetching from {} ...", remote_name);
    remote.fetch::<&str>(&[], Some(&mut fetch_options), None)?;

    Ok(())
}

pub fn add<P: AsRef<Path> + Clone + git2::IntoCString>(
    repo: &git2::Repository,
    paths: &[P],
) -> anyhow::Result<()> {
    let mut index = repo.index()?;

    index.add_all(
        paths,
        git2::IndexAddOption::DEFAULT,
        Some(&mut |path: &Path, _: &[u8]| {
            log::info!("Add match: {}", path.display());
            0
        }),
    )?;
    index.write()?;

    Ok(())
}

pub fn remove<P: AsRef<Path> + Clone + git2::IntoCString>(
    repo: &git2::Repository,
    paths: &[P],
) -> anyhow::Result<()> {
    let mut index = repo.index()?;

    index.remove_all(
        paths,
        Some(&mut |path: &Path, _: &[u8]| {
            let full_path = repo.path().join(path);

            log::info!("Removing file: {}", full_path.display());
            if let Err(err) = fs::remove_file(&full_path) {
                log::info!("File {} could not be delted {}", full_path.display(), err);
            }

            0
        }),
    )?;

    Ok(())
}

// From https://github.com/GitJournal/git_bindings/blob/master/gj_common/gitjournal.c
pub fn merge<'a>(
    repo: &'a git2::Repository,
    source_branch: &str,
    fetch_commit: Option<git2::AnnotatedCommit<'a>>,
    author_name: &str,
    author_email: &str,
) -> anyhow::Result<()> {
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
        perform_fastforward(repo, target_oid)?;
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
            resolve_conflict(repo, &our)?;
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
    repo: &git2::Repository,
    message: &str,
    author_name: &str,
    author_email: &str,
) -> anyhow::Result<()> {
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

pub fn push(repo: &git2::Repository, remote_name: &str) -> anyhow::Result<()> {
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
    callbacks.credentials(|_, username_from_url, _| credentials_cb(username_from_url));
    callbacks.transfer_progress(transfer_progress_cb);

    let mut push_options = git2::PushOptions::new();
    push_options.remote_callbacks(callbacks);

    log::info!("Pushing to {} ...", remote_name);
    remote.push(&[ref_head_name], Some(&mut push_options))?;

    Ok(())
}

pub fn pull(
    repo: &git2::Repository,
    remote_name: &str,
    author_name: &str,
    author_email: &str,
) -> anyhow::Result<()> {
    fetch(repo, remote_name)?;

    let fetch_head = repo.find_reference("FETCH_HEAD")?;
    let fetch_commit = repo.reference_to_annotated_commit(&fetch_head)?;

    // TODO better way to get this?
    let head = repo.head()?;
    let branch_name = head
        .name()
        .ok_or_else(|| anyhow::anyhow!("Ref head name not found"))?;
    let source_branch = format!("{}/{}", remote_name, branch_name);

    merge(
        repo,
        &source_branch,
        Some(fetch_commit),
        author_name,
        author_email,
    )?;

    Ok(())
}

fn credentials_cb(username_from_url: Option<&str>) -> Result<git2::Cred, git2::Error> {
    let mut ssh_key_path = glib::home_dir();
    ssh_key_path.push(".ssh/id_ed25519");

    log::info!("Credential callback");

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
    let file_full_path = repo.path().join(std::str::from_utf8(file_path).unwrap());

    let mut file = File::open(file_full_path)?;
    file.write_all(file_data)?;

    Ok(())
}

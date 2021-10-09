use gtk::glib;

use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

pub fn clone(git_base_path: &Path, remote_url: &str, passphrase: &str) -> anyhow::Result<()> {
    let mut callbacks = git2::RemoteCallbacks::new();
    callbacks.credentials(|_, username_from_url, _| credentials_cb(username_from_url, passphrase));
    callbacks.transfer_progress(transfer_progress_cb);

    let mut fetch_options = git2::FetchOptions::new();
    fetch_options.remote_callbacks(callbacks);

    let mut repo_builder = git2::build::RepoBuilder::new();
    repo_builder.fetch_options(fetch_options);

    log::info!("Cloning from {} ...", remote_url);
    repo_builder.clone(remote_url, git_base_path)?;

    Ok(())
}

pub fn fetch(git_base_path: &Path, remote_name: &str, passphrase: &str) -> anyhow::Result<()> {
    let repo = git2::Repository::open(git_base_path)?;
    let mut remote = repo.find_remote(remote_name)?;

    let mut callbacks = git2::RemoteCallbacks::new();
    callbacks.credentials(|_, username_from_url, _| credentials_cb(username_from_url, passphrase));
    callbacks.transfer_progress(transfer_progress_cb);

    let mut fetch_options = git2::FetchOptions::new();
    fetch_options.remote_callbacks(callbacks);

    log::info!("Fetching from {} ...", remote_name);
    remote.fetch::<&str>(&[], Some(&mut fetch_options), None)?;

    Ok(())
}

// From https://github.com/GitJournal/git_bindings/blob/master/gj_common/gitjournal.c
pub fn merge(
    git_base_path: &Path,
    source_branch: &str,
    author_name: &str,
    author_email: &str,
) -> anyhow::Result<()> {
    let repo = git2::Repository::open(git_base_path)?;

    let origin_head_ref = repo.find_branch(source_branch, git2::BranchType::Remote)?;
    let origin_annotated_commit = repo.reference_to_annotated_commit(origin_head_ref.get())?;

    let (merge_analysis, _) = repo.merge_analysis(&[&origin_annotated_commit])?;
    log::info!("Merge analysis: {:?}", merge_analysis);

    if merge_analysis.contains(git2::MergeAnalysis::ANALYSIS_UP_TO_DATE) {
        log::info!("Merge analysis: Up to date");
    } else if merge_analysis.contains(git2::MergeAnalysis::ANALYSIS_UNBORN) {
        anyhow::bail!("Merge analysis: Unborn");
    } else if merge_analysis.contains(git2::MergeAnalysis::ANALYSIS_FASTFORWARD) {
        log::info!("Merge analysis: Fastforwarding...");
        let target_oid = origin_annotated_commit.id();
        perform_fastforward(&repo, target_oid)?;
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
            resolve_conflict(&repo, &our)?;
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

pub fn commit(
    git_base_path: &Path,
    message: &str,
    author_name: &str,
    author_email: &str,
) -> anyhow::Result<()> {
    let repo = git2::Repository::open(git_base_path)?;

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

pub fn push(git_base_path: &Path, remote_name: &str, passphrase: &str) -> anyhow::Result<()> {
    let repo = git2::Repository::open(git_base_path)?;
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
    callbacks.credentials(|_, username_from_url, _| credentials_cb(username_from_url, passphrase));
    callbacks.transfer_progress(transfer_progress_cb);

    let mut push_options = git2::PushOptions::new();
    push_options.remote_callbacks(callbacks);

    log::info!("Pushing to {} ...", remote_name);
    remote.push(&[ref_head_name], Some(&mut push_options))?;

    Ok(())
}

fn credentials_cb(
    username_from_url: Option<&str>,
    passphrase: &str,
) -> Result<git2::Cred, git2::Error> {
    let mut ssh_key_path = glib::home_dir();
    ssh_key_path.push(".ssh/id_ed25519");

    log::info!("Credential callback");

    git2::Cred::ssh_key(
        username_from_url.unwrap(),
        None,
        &ssh_key_path,
        Some(passphrase),
    )
}

fn transfer_progress_cb(progress: git2::Progress) -> bool {
    dbg!(progress.total_objects());
    dbg!(progress.indexed_objects());
    dbg!(progress.received_objects());
    dbg!(progress.local_objects());
    dbg!(progress.total_deltas());
    dbg!(progress.indexed_deltas());
    dbg!(progress.received_bytes());
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
    let mut file_full_path = PathBuf::from(repo.path());
    file_full_path.push(std::str::from_utf8(file_path).unwrap());

    let mut file = File::open(file_full_path)?;
    file.write_all(file_data)?;

    Ok(())
}

use std::fs;

use axum::{body::Bytes, routing::post, Router};
use git2::{BranchType, Repository, Sort, TreeWalkMode, TreeWalkResult};
use itertools::Itertools;
use tar::Archive;

pub fn routes() -> Router {
    Router::new()
        .route("/archive_files", post(get_archive_file_nums))
        .route("/archive_files_size", post(get_archive_file_size))
        .route("/cookie", post(get_cookie))
}

pub async fn get_archive_file_nums(body: Bytes) -> String {
    Archive::new(body.as_ref())
        .entries()
        .unwrap()
        .count()
        .to_string()
}

pub async fn get_archive_file_size(body: Bytes) -> String {
    Archive::new(body.as_ref())
        .entries()
        .unwrap()
        .map(|entry| entry.unwrap().size())
        .sum::<u64>()
        .to_string()
}

// need to walk subtree (ie. subfolder)
fn find_cookie(commit: &git2::Commit, repo: &git2::Repository) -> bool {
    let mut found_it = false;
    let _ = commit
        .tree()
        .unwrap()
        .walk(TreeWalkMode::PreOrder, |_, entry| {
            if entry.name().unwrap() == "santa.txt"
                && std::str::from_utf8(entry.to_object(repo).unwrap().as_blob().unwrap().content())
                    .unwrap()
                    .contains("COOKIE")
            {
                found_it = true;
            }
            TreeWalkResult::Ok
        });
    found_it
}

pub async fn get_cookie(body: Bytes) -> String {
    // unpack and open archive
    Archive::new(body.as_ref()).unpack("tempfile").unwrap();
    let repo = Repository::open("tempfile").unwrap();

    // get branch ref
    let branch = repo.find_branch("christmas", BranchType::Local).unwrap();
    let branch_ref = branch.get();

    // traverse tree (including subtree)
    let mut revwalk = repo.revwalk().unwrap();
    let _ = revwalk.set_sorting(Sort::TOPOLOGICAL);
    revwalk.push_ref(branch_ref.name().unwrap()).unwrap();
    let commit = revwalk
        .map(|x| repo.find_commit(x.unwrap()).unwrap())
        .sorted_by(|a, b| b.time().cmp(&a.time()))
        .find(|commit| find_cookie(commit, &repo))
        .unwrap();

    // get commit info
    let committer = commit.committer().name().unwrap().to_string();
    let hash = commit.id().to_string();
    let result = format!("{} {}", committer, hash);

    // clean up
    let _ = fs::remove_dir_all("tempfile");

    result
}

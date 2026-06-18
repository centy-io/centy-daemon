#![allow(missing_docs)]
use std::path::{Path, PathBuf};
use std::process::Command;
use worktree_io::git::{
    bare_clone, branch_exists_local, branch_exists_remote, create_local_worktree, create_worktree,
    detect_default_branch, detect_local_default_branch, git_fetch, git_worktree_prune,
};

fn git(dir: &Path, args: &[&str]) {
    let status = Command::new("git")
        .args(["-C"])
        .arg(dir)
        .args(args)
        // Unset inherited git env vars so `-C dir` is honoured even inside
        // a git worktree hook, where GIT_DIR would otherwise override it.
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_INDEX_FILE")
        .status()
        .unwrap();
    assert!(status.success(), "git {args:?} failed");
}

fn make_test_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("wt-test-{}-{}", name, std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn setup_source_repo(base: &Path) -> PathBuf {
    let src = base.join("source");
    std::fs::create_dir_all(&src).unwrap();
    git(&src, &["init", "-b", "main"]);
    git(&src, &["config", "user.email", "test@test.com"]);
    git(&src, &["config", "user.name", "Test"]);
    std::fs::write(src.join("README.md"), "hello").unwrap();
    git(&src, &["add", "."]);
    git(&src, &["commit", "-m", "init"]);
    git(&src, &["branch", "issue-42"]);
    src
}

#[test]
fn test_bare_clone_and_git_fetch() {
    let base = make_test_dir("clone");
    let src = setup_source_repo(&base);
    let dest = base.join("bare.git");

    bare_clone(src.to_str().unwrap(), &dest).unwrap();
    assert!(dest.exists());

    // git_fetch should succeed on the bare clone
    git_fetch(&dest).unwrap();

    let _ = std::fs::remove_dir_all(&base);
}

#[test]
fn test_detect_default_branch() {
    let base = make_test_dir("branch");
    let src = setup_source_repo(&base);
    let dest = base.join("bare.git");

    bare_clone(src.to_str().unwrap(), &dest).unwrap();

    let branch = detect_default_branch(&dest).unwrap();
    assert_eq!(branch, "main");

    let _ = std::fs::remove_dir_all(&base);
}

#[test]
fn test_branch_exists_remote_true_and_false() {
    let base = make_test_dir("exists");
    let src = setup_source_repo(&base);
    let dest = base.join("bare.git");

    bare_clone(src.to_str().unwrap(), &dest).unwrap();

    assert!(branch_exists_remote(&dest, "main"));
    assert!(branch_exists_remote(&dest, "issue-42"));
    assert!(!branch_exists_remote(&dest, "nonexistent-branch-xyz"));

    let _ = std::fs::remove_dir_all(&base);
}

#[test]
fn test_create_worktree_new_branch() {
    let base = make_test_dir("worktree");
    let src = setup_source_repo(&base);
    let dest = base.join("bare.git");

    bare_clone(src.to_str().unwrap(), &dest).unwrap();

    let wt_path = base.join("wt-issue-99");
    create_worktree(&dest, &wt_path, "issue-99", "main", false).unwrap();
    assert!(wt_path.exists());

    let _ = std::fs::remove_dir_all(&base);
}

#[test]
fn test_create_worktree_existing_branch() {
    let base = make_test_dir("worktree2");
    let src = setup_source_repo(&base);
    let dest = base.join("bare.git");

    bare_clone(src.to_str().unwrap(), &dest).unwrap();

    let wt_path = base.join("wt-issue-42");
    create_worktree(&dest, &wt_path, "issue-42", "main", true).unwrap();
    assert!(wt_path.exists());

    let _ = std::fs::remove_dir_all(&base);
}

#[test]
fn test_detect_default_branch_remote_show_fallback() {
    // Delete HEAD symref → forces fallback to `git remote show origin`
    let base = make_test_dir("branch-remoteshow");
    let src = setup_source_repo(&base);
    let dest = base.join("bare.git");
    bare_clone(src.to_str().unwrap(), &dest).unwrap();
    // Delete the HEAD symref so symbolic-ref fails
    let _ = Command::new("git")
        .args(["-C"])
        .arg(&dest)
        .args(["symbolic-ref", "--delete", "refs/remotes/origin/HEAD"])
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_INDEX_FILE")
        .status();
    let branch = detect_default_branch(&dest).unwrap();
    assert_eq!(branch, "main");
    let _ = std::fs::remove_dir_all(&base);
}

#[test]
fn test_detect_default_branch_rev_parse_fallback() {
    // Bad origin URL → remote show fails → falls through to rev-parse candidates
    let base = make_test_dir("branch-revparse");
    let src = setup_source_repo(&base);
    let dest = base.join("bare.git");
    bare_clone(src.to_str().unwrap(), &dest).unwrap();
    let _ = Command::new("git")
        .args(["-C"])
        .arg(&dest)
        .args(["symbolic-ref", "--delete", "refs/remotes/origin/HEAD"])
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_INDEX_FILE")
        .status();
    git(
        &dest,
        &["remote", "set-url", "origin", "/nonexistent/bad/path"],
    );
    // refs/remotes/origin/main still exists from the clone
    let branch = detect_default_branch(&dest).unwrap();
    assert_eq!(branch, "main");
    let _ = std::fs::remove_dir_all(&base);
}

#[test]
fn test_detect_default_branch_bail() {
    // Empty bare repo with bad remote → all detection paths fail → bail
    let base = make_test_dir("branch-bail");
    std::fs::create_dir_all(&base).unwrap();
    let dest = base.join("bare.git");
    std::fs::create_dir_all(&dest).unwrap();
    git(&dest, &["init", "--bare"]);
    git(&dest, &["remote", "add", "origin", "/nonexistent/bad/path"]);
    // No remote refs, bad origin → bail!
    let result = detect_default_branch(&dest);
    assert!(result.is_err());
    let _ = std::fs::remove_dir_all(&base);
}

#[test]
fn test_detect_default_branch_wrong_symref_prefix() {
    // HEAD symref exits 0 but output doesn't start with refs/remotes/origin/
    // → falls through to remote show which succeeds
    let base = make_test_dir("branch-wrongpfx");
    let src = setup_source_repo(&base);
    let dest = base.join("bare.git");
    bare_clone(src.to_str().unwrap(), &dest).unwrap();
    // Set HEAD to something that won't match refs/remotes/origin/ prefix
    git(
        &dest,
        &[
            "symbolic-ref",
            "refs/remotes/origin/HEAD",
            "refs/heads/main",
        ],
    );
    let branch = detect_default_branch(&dest).unwrap();
    assert_eq!(branch, "main");
    let _ = std::fs::remove_dir_all(&base);
}

#[test]
fn test_bare_clone_bad_url() {
    let base = make_test_dir("clone-bad");
    let dest = base.join("bare.git");
    let result = bare_clone("/nonexistent/bad/path", &dest);
    assert!(result.is_err());
    let _ = std::fs::remove_dir_all(&base);
}

#[test]
fn test_git_fetch_bad_origin() {
    let base = make_test_dir("fetch-bad");
    let dest = base.join("bare.git");
    std::fs::create_dir_all(&dest).unwrap();
    git(&dest, &["init", "--bare"]);
    git(&dest, &["remote", "add", "origin", "/nonexistent/bad/path"]);
    let result = git_fetch(&dest);
    assert!(result.is_err());
    let _ = std::fs::remove_dir_all(&base);
}

#[test]
fn test_create_worktree_branch_in_use() {
    let base = make_test_dir("worktree-dup");
    let src = setup_source_repo(&base);
    let dest = base.join("bare.git");
    bare_clone(src.to_str().unwrap(), &dest).unwrap();
    let wt1 = base.join("wt-issue-42-a");
    create_worktree(&dest, &wt1, "issue-42", "main", true).unwrap();
    let wt2 = base.join("wt-issue-42-b");
    let result = create_worktree(&dest, &wt2, "issue-42", "main", true);
    assert!(result.is_err());
    let _ = std::fs::remove_dir_all(&base);
}

#[test]
fn test_branch_exists_local() {
    let base = make_test_dir("local-exists");
    let src = setup_source_repo(&base);
    assert!(branch_exists_local(&src, "main"));
    assert!(branch_exists_local(&src, "issue-42"));
    assert!(!branch_exists_local(&src, "nonexistent-xyz"));
    let _ = std::fs::remove_dir_all(&base);
}

#[test]
fn test_detect_local_default_branch() {
    let base = make_test_dir("local-branch");
    let src = setup_source_repo(&base);
    let branch = detect_local_default_branch(&src).unwrap();
    assert_eq!(branch, "main");
    let _ = std::fs::remove_dir_all(&base);
}

#[test]
fn test_detect_local_default_branch_detached_fallback() {
    // Detach HEAD so rev-parse returns "HEAD"; fallback finds refs/heads/main
    let base = make_test_dir("local-branch-det");
    let src = setup_source_repo(&base);
    git(&src, &["checkout", "--detach"]);
    let branch = detect_local_default_branch(&src).unwrap();
    assert_eq!(branch, "main");
    let _ = std::fs::remove_dir_all(&base);
}

#[test]
fn test_detect_local_default_branch_bail() {
    // Repo initialized with a non-standard branch; detached HEAD with no main/master/develop
    let base = make_test_dir("local-branch-bail");
    let src = base.join("source");
    std::fs::create_dir_all(&src).unwrap();
    git(&src, &["init", "-b", "feature"]);
    git(&src, &["config", "user.email", "test@test.com"]);
    git(&src, &["config", "user.name", "Test"]);
    std::fs::write(src.join("README.md"), "hello").unwrap();
    git(&src, &["add", "."]);
    git(&src, &["commit", "-m", "init"]);
    git(&src, &["checkout", "--detach"]);
    let result = detect_local_default_branch(&src);
    assert!(result.is_err());
    let _ = std::fs::remove_dir_all(&base);
}

#[test]
fn test_create_local_worktree_new_branch() {
    let base = make_test_dir("local-wt");
    let src = setup_source_repo(&base);
    let wt_path = base.join("wt-issue-5");
    create_local_worktree(&src, &wt_path, "issue-5", false).unwrap();
    assert!(wt_path.exists());
    let _ = std::fs::remove_dir_all(&base);
}

#[test]
fn test_create_local_worktree_existing_branch() {
    let base = make_test_dir("local-wt2");
    let src = setup_source_repo(&base); // has "issue-42" branch
    let wt_path = base.join("wt-issue-42");
    create_local_worktree(&src, &wt_path, "issue-42", true).unwrap();
    assert!(wt_path.exists());
    let _ = std::fs::remove_dir_all(&base);
}

#[test]
fn test_create_local_worktree_branch_in_use() {
    let base = make_test_dir("local-wt-dup");
    let src = setup_source_repo(&base);
    let wt1 = base.join("wt-issue-42-a");
    create_local_worktree(&src, &wt1, "issue-42", true).unwrap();
    let wt2 = base.join("wt-issue-42-b");
    let result = create_local_worktree(&src, &wt2, "issue-42", true);
    assert!(result.is_err());
    let _ = std::fs::remove_dir_all(&base);
}

#[test]
fn test_detect_local_default_branch_status_fails() {
    // Non-git directory: rev-parse exits non-zero → false branch of if status.success()
    let base = make_test_dir("local-branch-no-git");
    let result = detect_local_default_branch(&base);
    assert!(result.is_err());
    let _ = std::fs::remove_dir_all(&base);
}

#[test]
fn test_git_worktree_prune_bad_repo() {
    // Passing a non-git directory should cause `git worktree prune` to fail.
    let dir = make_test_dir("wt-prune-bad");
    let result = git_worktree_prune(&dir);
    assert!(result.is_err(), "expected error for non-git dir");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_git_worktree_prune() {
    let dir = make_test_dir("wt-prune");
    let src = setup_source_repo(&dir);
    let bare_repo = dir.join("bare.git");
    bare_clone(src.to_str().unwrap(), &bare_repo).unwrap();

    // Create a worktree, then delete it manually.
    let wt = dir.join("issue-99");
    create_worktree(&bare_repo, &wt, "issue-99", "main", false).unwrap();
    assert!(wt.exists());
    std::fs::remove_dir_all(&wt).unwrap();

    // Prune should succeed and remove the stale ref.
    git_worktree_prune(&bare_repo).unwrap();

    // After pruning, we can add the same path again without conflict.
    // The branch already exists locally so branch_exists = true.
    create_worktree(&bare_repo, &wt, "issue-99", "main", true).unwrap();
    assert!(wt.exists());

    let _ = std::fs::remove_dir_all(&dir);
}

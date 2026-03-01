use std::path::Path;

/// ワークツリー削除後に残った空の親ディレクトリを再帰的に削除する
///
/// `stop_dir` に到達したら停止する（そこ自体は削除しない）。
///
/// 例: `worktrees/feature/my-task` を削除したあと、
/// `feature/` が空なら削除し、`worktrees/` に達したら停止。
pub fn remove_empty_parents(dir: &Path, stop_dir: &Path) -> anyhow::Result<()> {
    let mut current = dir;

    loop {
        // stop_dir と同じか上位に達したら終了
        if current == stop_dir || !current.starts_with(stop_dir) {
            break;
        }

        // ディレクトリが存在しないか空でなければ終了
        if !current.exists() {
            // すでに削除済みなら親へ
        } else {
            let is_empty = current.read_dir()?.next().is_none();
            if !is_empty {
                break;
            }
            std::fs::remove_dir(current)?;
        }

        match current.parent() {
            Some(parent) => current = parent,
            None => break,
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn setup_dirs(base: &Path, rel_path: &str) -> PathBuf {
        let target = base.join(rel_path);
        fs::create_dir_all(&target).unwrap();
        target
    }

    #[test]
    fn removes_empty_parent_dirs() {
        let tmp = tempfile::tempdir().unwrap();
        let worktrees = tmp.path().join("worktrees");
        let leaf = setup_dirs(&worktrees, "feature/my-task");

        // leaf を削除した後、親ディレクトリを片付ける
        fs::remove_dir(&leaf).unwrap();
        remove_empty_parents(&leaf, &worktrees).unwrap();

        // feature/ は空なので削除される
        assert!(!worktrees.join("feature").exists());
        // worktrees/ 自体は残る
        assert!(worktrees.exists());
    }

    #[test]
    fn stops_when_parent_is_not_empty() {
        let tmp = tempfile::tempdir().unwrap();
        let worktrees = tmp.path().join("worktrees");
        let leaf1 = setup_dirs(&worktrees, "feature/task-1");
        let _leaf2 = setup_dirs(&worktrees, "feature/task-2");

        fs::remove_dir(&leaf1).unwrap();
        remove_empty_parents(&leaf1, &worktrees).unwrap();

        // feature/ に task-2 が残っているので削除されない
        assert!(worktrees.join("feature").exists());
    }

    #[test]
    fn does_not_remove_stop_dir_itself() {
        let tmp = tempfile::tempdir().unwrap();
        let worktrees = tmp.path().join("worktrees");
        let leaf = setup_dirs(&worktrees, "solo");

        fs::remove_dir(&leaf).unwrap();
        remove_empty_parents(&leaf, &worktrees).unwrap();

        // worktrees/ 自体は削除されない
        assert!(worktrees.exists());
    }
}

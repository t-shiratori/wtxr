use std::path::Path;

use anyhow::Context;

use super::types::{Config, CopyFile, HooksConfig};

/// グローバル設定とローカル設定を読み込んでマージした Config を返す
pub fn load_config(repo_root: &Path) -> anyhow::Result<Config> {
    let global_cfg = load_global_config()?;
    let local_cfg = load_local_config(repo_root)?;
    Ok(merge_config(global_cfg, local_cfg))
}

/// グローバル設定ファイルを読み込む (~/.config/wtxr/config.toml)
fn load_global_config() -> anyhow::Result<Option<Config>> {
    let Some(config_dir) = dirs::config_dir() else {
        return Ok(None);
    };
    let path = config_dir.join("wtxr").join("config.toml");
    load_config_file(&path)
}

/// ローカル設定ファイルを読み込む (<repo_root>/.wtxr/config.toml)
fn load_local_config(repo_root: &Path) -> anyhow::Result<Option<Config>> {
    let path = repo_root.join(".wtxr").join("config.toml");
    load_config_file(&path)
}

/// 設定ファイルを読み込む（存在しない場合は None を返す）
fn load_config_file(path: &Path) -> anyhow::Result<Option<Config>> {
    if !path.exists() {
        return Ok(None);
    }
    let content =
        std::fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    let config: Config =
        toml::from_str(&content).with_context(|| format!("failed to parse {}", path.display()))?;
    Ok(Some(config))
}

/// グローバル設定とローカル設定をマージする
///
/// マージ規則:
/// - 文字列フィールド: ローカルが空でなければローカル優先、空ならグローバル
/// - スライス（Patterns, Hooks）: 両方を結合して重複を排除
/// - CopyFiles: `from` をキーにして重複排除（ローカル優先）
fn merge_config(global: Option<Config>, local: Option<Config>) -> Config {
    match (global, local) {
        (None, None) => Config::default(),
        (Some(g), None) => g,
        (None, Some(l)) => l,
        (Some(g), Some(l)) => Config {
            worktree: super::types::WorktreeConfig {
                root_dir: merge_string(g.worktree.root_dir, l.worktree.root_dir),
                default_base_branch: merge_string(
                    g.worktree.default_base_branch,
                    l.worktree.default_base_branch,
                ),
            },
            copy: super::types::CopyConfig {
                patterns: merge_dedup(g.copy.patterns, l.copy.patterns),
                files: merge_copy_files(g.copy.files, l.copy.files),
            },
            hooks: HooksConfig {
                pre_create: merge_dedup(g.hooks.pre_create, l.hooks.pre_create),
                post_create: merge_dedup(g.hooks.post_create, l.hooks.post_create),
                post_copy: merge_dedup(g.hooks.post_copy, l.hooks.post_copy),
            },
        },
    }
}

/// ローカルが空でなければローカルを、空ならグローバルを返す
fn merge_string(global: String, local: String) -> String {
    if local.is_empty() {
        global
    } else {
        local
    }
}

/// 2つのスライスを結合して重複を排除する（順序は global → local）
fn merge_dedup(global: Vec<String>, local: Vec<String>) -> Vec<String> {
    let mut result = global;
    for item in local {
        if !result.contains(&item) {
            result.push(item);
        }
    }
    result
}

/// `from` をキーにして CopyFile をマージする（ローカル優先）
fn merge_copy_files(global: Vec<CopyFile>, local: Vec<CopyFile>) -> Vec<CopyFile> {
    let mut result = local.clone();
    for g in global {
        // ローカルに同じ from がなければ追加
        if !local.iter().any(|l| l.from == g.from) {
            result.push(g);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::types::{Config, CopyConfig, CopyFile, HooksConfig, WorktreeConfig};

    fn make_config(root_dir: &str, base_branch: &str, patterns: Vec<&str>) -> Config {
        Config {
            worktree: WorktreeConfig {
                root_dir: root_dir.to_string(),
                default_base_branch: base_branch.to_string(),
            },
            copy: CopyConfig {
                patterns: patterns.into_iter().map(String::from).collect(),
                files: vec![],
            },
            hooks: HooksConfig::default(),
        }
    }

    #[test]
    fn merge_global_only() {
        let g = make_config("global/worktrees", "main", vec!["*.env"]);
        let result = merge_config(Some(g), None);
        assert_eq!(result.worktree.root_dir, "global/worktrees");
        assert_eq!(result.copy.patterns, vec!["*.env"]);
    }

    #[test]
    fn merge_local_only() {
        let l = make_config("local/worktrees", "develop", vec!["*.yaml"]);
        let result = merge_config(None, Some(l));
        assert_eq!(result.worktree.root_dir, "local/worktrees");
        assert_eq!(result.worktree.default_base_branch, "develop");
    }

    #[test]
    fn local_string_overrides_global() {
        let g = make_config("global/worktrees", "main", vec![]);
        let l = make_config("local/worktrees", "develop", vec![]);
        let result = merge_config(Some(g), Some(l));
        assert_eq!(result.worktree.root_dir, "local/worktrees");
        assert_eq!(result.worktree.default_base_branch, "develop");
    }

    #[test]
    fn empty_local_string_falls_back_to_global() {
        let g = make_config("global/worktrees", "main", vec![]);
        let l = make_config("", "", vec![]);
        let result = merge_config(Some(g), Some(l));
        assert_eq!(result.worktree.root_dir, "global/worktrees");
        assert_eq!(result.worktree.default_base_branch, "main");
    }

    #[test]
    fn patterns_are_merged_deduped() {
        let g = make_config("", "", vec!["*.env", "shared.yaml"]);
        let l = make_config("", "", vec!["*.yaml", "shared.yaml"]);
        let result = merge_config(Some(g), Some(l));
        // global の順番 → local の新しいもの、重複除去
        assert_eq!(result.copy.patterns, vec!["*.env", "shared.yaml", "*.yaml"]);
    }

    #[test]
    fn copy_files_local_wins_on_duplicate_from() {
        let g = Config {
            copy: CopyConfig {
                files: vec![
                    CopyFile {
                        from: ".env.example".to_string(),
                        to: ".env.global".to_string(),
                    },
                    CopyFile {
                        from: "only_global".to_string(),
                        to: "only_global".to_string(),
                    },
                ],
                ..Default::default()
            },
            ..Default::default()
        };
        let l = Config {
            copy: CopyConfig {
                files: vec![CopyFile {
                    from: ".env.example".to_string(),
                    to: ".env.local".to_string(),
                }],
                ..Default::default()
            },
            ..Default::default()
        };
        let result = merge_config(Some(g), Some(l));
        assert_eq!(result.copy.files.len(), 2);
        let env_file = result
            .copy
            .files
            .iter()
            .find(|f| f.from == ".env.example")
            .unwrap();
        assert_eq!(env_file.to, ".env.local"); // ローカル優先
    }
}

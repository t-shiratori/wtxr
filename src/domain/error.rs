use std::path::PathBuf;
use thiserror::Error;

/// ワークツリー操作に関するドメインエラー
#[derive(Debug, Error)]
pub enum WorktreeError {
    #[error("worktree not found: {path}")]
    NotFound { path: PathBuf },

    #[error("worktree already exists: {path}")]
    AlreadyExists { path: PathBuf },

    #[error("git command failed: {message}")]
    GitCommandFailed { message: String },

    #[error("invalid worktree path: {path}")]
    InvalidPath { path: PathBuf },
}

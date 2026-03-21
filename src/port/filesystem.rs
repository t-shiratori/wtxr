use std::path::Path;

use crate::config::types::{CopyConfig, CopyFile};

/// ファイルシステム操作のインターフェース
pub trait FileSystem {
    /// glob パターンと明示的ファイル指定に基づいてファイルをコピーする
    ///
    /// - `src_root`: コピー元のルートディレクトリ（リポジトリルート）
    /// - `dst_root`: コピー先のルートディレクトリ（ワークツリーパス）
    /// - `cfg`: コピー設定（patterns と files）
    fn copy_files(
        &self,
        src_root: &Path,
        dst_root: &Path,
        cfg: &CopyConfig,
    ) -> anyhow::Result<Vec<CopyFile>>;
}

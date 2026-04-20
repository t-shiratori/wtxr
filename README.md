# wtxr

> このツールは [wtx](https://github.com/t-shiratori/wtx)（TypeScript 実装）を Rust で実装したらどうなるかを実験するために作成したものです。

Git worktree を簡単に管理できる CLI ツールです。worktree の作成・削除・一覧表示をシンプルなコマンドで行えます。

## 機能

- worktree の作成・削除・一覧表示
- インタラクティブな TUI による worktree の選択・削除
- worktree 作成時のファイル自動コピー（glob パターン / 明示的なファイルマッピング）
- フック対応（`pre_create` / `post_create` / `post_copy`）
- ローカル / グローバル設定の初期化
- `--dry-run` による実行前の操作プレビュー

## インストール

### Homebrew

```sh
brew install t-shiratori/tap/wtxr
```

### バイナリを直接ダウンロード

[GitHub Releases](https://github.com/t-shiratori/wtxr/releases) から Intel / ARM 向けバイナリをダウンロードできます。

### ソースからビルド

Rust 1.70 以上が必要です。

```sh
git clone https://github.com/t-shiratori/wtxr.git
cd wtxr
cargo install --path .
```

## コマンド

### バージョン確認

```sh
wtxr --version
```

### worktree の追加

```sh
wtxr add <branch>
```

| オプション | 説明 |
|---|---|
| `-b`, `--create-branch` | 新しいブランチを作成する |
| `--from <branch>` | ブランチの作成元を指定する |
| `--dry-run` | 実行せずに操作内容をプレビューする |

**例:**

```sh
# 既存ブランチの worktree を追加
wtxr add feature/foo

# 新規ブランチを作成して worktree を追加
wtxr add feature/bar -b

# main ベースで新規ブランチを作成
wtxr add feature/baz -b --from main
```

### worktree の一覧表示

```sh
wtxr list
```

パス・ブランチ名・コミットハッシュを表形式で出力します。

```
/path/to/repo                           main           abc1234
/path/to/repo/.wtxr/worktrees/feature   feature/foo    def5678
```

### worktree の削除

```sh
wtxr remove [worktree...]
```

引数を省略すると TUI でインタラクティブに選択できます。

| オプション | 説明 |
|---|---|
| `-b`, `--branch` | worktree と同時にブランチも削除する |
| `-f`, `--force` | 未コミットの変更があっても強制削除する |
| `--dry-run` | 実行せずに操作内容をプレビューする |

**例:**

```sh
# TUI で選択して削除
wtxr remove

# ブランチ名を指定して削除
wtxr remove feature/foo

# worktree とブランチをまとめて削除
wtxr remove feature/foo -b
```

### 設定ファイルの初期化

```sh
wtxr init
```

| オプション | 説明 |
|---|---|
| `--global` | グローバル設定（`~/.config/wtxr/config.toml`）を作成する |
| `-f`, `--force` | 既存の設定ファイルを上書きする |

## 設定

設定ファイルは以下のパスに配置されます。

| 種類 | パス |
|---|---|
| ローカル | `.wtxr/config.toml`（リポジトリルート） |
| グローバル | `~/.config/wtxr/config.toml` |

```toml
[worktree]
# worktree を配置するディレクトリ（デフォルト: .wtxr/worktrees）
root_dir = ".wtxr/worktrees"
# デフォルトのベースブランチ
default_base_branch = "main"

[copy]
# glob パターンでコピーするファイルを指定
patterns = ["*.env", "config/*.yaml"]

# 明示的なファイルマッピング（リネームも可能）
[[copy.files]]
from = ".env.example"
to = ".env"

[hooks]
# worktree 作成前に実行するコマンド
pre_create = ["echo pre_create"]
# worktree 作成後に実行するコマンド
post_create = ["npm install"]
# ファイルコピー後に実行するコマンド
post_copy = ["echo copied"]
```

## ライセンス

MIT

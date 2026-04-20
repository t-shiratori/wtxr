# wtxr

> This tool was created as an experiment to see what it would look like to reimplement [wtx](https://github.com/t-shiratori/wtx) (a TypeScript implementation) in Rust.

A CLI tool for managing Git worktrees easily. Create, remove, and list worktrees with simple commands.

## Features

- Create, remove, and list worktrees
- Interactive TUI for selecting and removing worktrees
- Automatic file copying on worktree creation (glob patterns / explicit file mappings)
- Hook support (`pre_create` / `post_create` / `post_copy`)
- Local / global config initialization
- `--dry-run` to preview operations before execution

## Installation

### Homebrew

```sh
brew install t-shiratori/tap/wtxr
```

### Download Binary

Download Intel / ARM binaries from [GitHub Releases](https://github.com/t-shiratori/wtxr/releases).

### Build from Source

Requires Rust 1.70 or later.

```sh
git clone https://github.com/t-shiratori/wtxr.git
cd wtxr
cargo install --path .
```

## Commands

### Version

```sh
wtxr --version
```

### Add a worktree

```sh
wtxr add <branch>
```

| Option | Description |
|---|---|
| `-b`, `--create-branch` | Create a new branch |
| `--from <branch>` | Specify the base branch to create from |
| `--dry-run` | Preview operations without executing |

**Examples:**

```sh
# Add a worktree for an existing branch
wtxr add feature/foo

# Create a new branch and add a worktree
wtxr add feature/bar -b

# Create a new branch based on main
wtxr add feature/baz -b --from main
```

### List worktrees

```sh
wtxr list
```

Outputs path, branch name, and commit hash in tabular format.

```
/path/to/repo                           main           abc1234
/path/to/repo/.wtxr/worktrees/feature   feature/foo    def5678
```

### Remove a worktree

```sh
wtxr remove [worktree...]
```

If no arguments are given, you can select interactively via TUI.

| Option | Description |
|---|---|
| `-b`, `--branch` | Also delete the branch along with the worktree |
| `-f`, `--force` | Force removal even if there are uncommitted changes |
| `--dry-run` | Preview operations without executing |

**Examples:**

```sh
# Select and remove via TUI
wtxr remove

# Remove by branch name
wtxr remove feature/foo

# Remove worktree and branch together
wtxr remove feature/foo -b
```

### Initialize config

```sh
wtxr init
```

| Option | Description |
|---|---|
| `--global` | Create global config (`~/.config/wtxr/config.toml`) |
| `-f`, `--force` | Overwrite existing config file |

## Configuration

Config files are located at the following paths:

| Type | Path |
|---|---|
| Local | `.wtxr/config.toml` (repository root) |
| Global | `~/.config/wtxr/config.toml` |

```toml
[worktree]
# Directory where worktrees are placed (default: .wtxr/worktrees)
root_dir = ".wtxr/worktrees"
# Default base branch
default_base_branch = "main"

[copy]
# Files to copy specified by glob patterns
patterns = ["*.env", "config/*.yaml"]

# Explicit file mappings (renaming is also supported)
[[copy.files]]
from = ".env.example"
to = ".env"

[hooks]
# Command to run before worktree creation
pre_create = ["echo pre_create"]
# Command to run after worktree creation
post_create = ["npm install"]
# Command to run after file copying
post_copy = ["echo copied"]
```

## License

MIT

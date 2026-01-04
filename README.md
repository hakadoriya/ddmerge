# ddmerge

Interactive directory diff and merge tool for the command line.

Compare two directories and interactively merge differences at the **hunk level**, similar to `git add -p`.

## Features

- **Hunk-level merging**: Choose left or right for each diff hunk, not just whole files
- **Bidirectional sync**: Updates both directories based on your choices
- **In-place updates**: No separate output directory needed
- **Immediate application**: Changes are applied as you make selections
- **Binary file detection**: Automatically detects and skips binary files
- **Flexible filtering**: Exclude files using regex patterns

## Installation

### Using pre-built binary

Download the latest binary for your platform from [GitHub Releases](https://github.com/hakadoriya/ddmerge/releases).

```bash
# Example for *nix
VERSION=v0.0.1
curl -LRSs -o /tmp/ddmerge.zip https://github.com/hakadoriya/ddmerge/releases/download/${VERSION}/ddmerge_$(uname -s)_$(uname -m).zip
unzip /tmp/ddmerge.zip ddmerge
sudo mv ddmerge /usr/local/bin/
```

Available binaries follow the naming convention: `ddmerge_<OS>_<arch>.zip`
- Linux: `ddmerge_Linux_x86_64.zip`, `ddmerge_Linux_arm64.zip`
- macOS: `ddmerge_Darwin_x86_64.zip`, `ddmerge_Darwin_arm64.zip`
- Windows: `ddmerge_Windows_x86_64.zip`, `ddmerge_Windows_arm64.zip`

### From source

```bash
git clone https://github.com/hakadoriya/ddmerge.git
cd ddmerge
cargo install --path .
```

## Usage

```bash
ddmerge [OPTIONS] <left-dir> <right-dir>
```

### Options

| Option | Description |
|--------|-------------|
| `--dry-run` | Show differences without applying changes |
| `--skip-binary` | Silently skip binary files (no warning messages) |
| `--exclude-regex-left <PATTERN>` | Exclude files matching regex in left directory |
| `--exclude-regex-right <PATTERN>` | Exclude files matching regex in right directory |

### Examples

```bash
# Compare and merge two directories
ddmerge ./project-v1 ./project-v2

# Preview changes without applying
ddmerge --dry-run ./left ./right

# Exclude backup and temp files
ddmerge --exclude-regex-left '\.bak$' --exclude-regex-right '\.tmp$' ./src ./dest
```

## Interactive Commands

### For modified files (hunk-level)

| Key | Action |
|-----|--------|
| `l` | Use left version (updates right file) |
| `r` | Use right version (updates left file) |
| `s` | Skip this hunk (keep both versions as-is) |
| `f` | Skip remaining hunks in this file |
| `q` | Quit |

### For files existing only on one side

| Key | Action |
|-----|--------|
| `c` | Copy to the other directory |
| `d` | Delete from source directory |
| `s` | Skip (leave as-is) |
| `q` | Quit |

## Example Session

```
$ ddmerge ./left-dir ./right-dir

Comparing directories...
Found 2 file(s) with differences.

File: config.yaml (2 hunk(s))

[1/2] Hunk in config.yaml
  @@ -1,3 +1,4 @@
   database:
  -  host: localhost
  +  host: production.db.example.com

  Choose: (l)eft (update right) / (r)ight (update left) / (s)kip / (q)uit > l
  ✓ Using left (will update right file)
  ✓ Applied.

[2/2] Hunk in config.yaml
  @@ -5,2 +5,3 @@
  +  port: 5432

  Choose: (l)eft (update right) / (r)ight (update left) / (s)kip / (q)uit > r
  ✓ Using right (will update left file)
  ✓ Applied.

Merge complete!
```

## Diff Types

| Type | Description | Options |
|------|-------------|---------|
| `LeftOnly` | File exists only in left directory | copy / delete / skip |
| `RightOnly` | File exists only in right directory | copy / delete / skip |
| `Modified` | File exists in both but content differs | hunk-level left / right / skip |
| `TypeMismatch` | Same name but different types (file vs directory) | left / right / skip |

## How It Works

1. **Directory scanning**: Recursively compares both directories
2. **Diff detection**: Identifies files that are added, removed, or modified
3. **Hunk extraction**: For modified files, extracts individual diff hunks
4. **Interactive selection**: Presents each difference for user decision
5. **Immediate application**: Applies changes to both directories as you decide

### Merge Behavior

- **`l` (left)**: Both files become identical to the left version
- **`r` (right)**: Both files become identical to the right version
- **`s` (skip)**: Each file keeps its original content (difference preserved)

## Building from Source

```bash
# Clone the repository
git clone https://github.com/hakadoriya/ddmerge.git
cd ddmerge

# Build
cargo build --release

# Run tests
cargo test

# Install locally
cargo install --path .
```

## License

Apache-2.0

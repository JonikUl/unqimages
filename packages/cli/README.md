# @unqimages/cli

CLI for finding duplicate images in a project.

## Requirements

- Node.js 18 or later.
- One of the supported platforms (the matching native binary is installed automatically).

## Supported platforms

| Platform    | Package                       | Minimum OS                  |
| ----------- | ----------------------------- | --------------------------- |
| Linux x64   | `@unqimages/cli-linux-x64`    | glibc 2.35+ (Ubuntu 22.04+) |
| macOS x64   | `@unqimages/cli-darwin-x64`   | macOS 13+                   |
| macOS ARM64 | `@unqimages/cli-darwin-arm64` | macOS 13+                   |
| Windows x64 | `@unqimages/cli-windows-x64`  | Windows 10+                 |

The main package declares the platform packages as `optionalDependencies`, so npm installs only the one that matches the host OS and CPU.

> **Note:** The Linux binary is built on Ubuntu 22.04 and links against glibc 2.35. It runs on Ubuntu 22.04 and newer, as well as other Linux distributions with a compatible glibc version.

## Installation

```bash
npm install -D @unqimages/cli
```

## Usage

```bash
npx unqimages
```

The CLI scans the directories configured in `unqimages.config.{js,mjs,json}` (or the `unqimages` field in `package.json`) and prints duplicate groups.

## CLI options

| Option                 | Description                                                   |
| ---------------------- | ------------------------------------------------------------- |
| `--output json\|table` | Output format. Default: `json`.                               |
| `--no-cache`           | Ignore the cache and recompute all hashes.                    |
| `--staged`             | Check staged images as new files. Useful in pre-commit hooks. |
| `--version`, `-V`      | Print the package version.                                    |

`--config` and `--cwd` are set automatically by the wrapper and cannot be overridden.

When passing staged file paths manually, put all flags before the paths:

```bash
unqimages --staged --output table file.png
```

Flags placed after the paths are treated as part of the path list.

## Configuration

Create `unqimages.config.{js,mjs,json}` in your project root:

```js
export default {
  includeDirs: ['src/assets', 'public'],
  excludeDirs: ['node_modules', 'dist'],
  extensions: ['png', 'jpg', 'jpeg', 'webp', 'gif', 'svg', 'ico'],
  failOnDuplicates: false,
  ignoreCache: false,
};
```

You can also use the `unqimages` field in `package.json`. File-based config overrides the `package.json` field.

### Configuration options

| Option                 | Type       | Default                                               | Description                                                       |
| ---------------------- | ---------- | ----------------------------------------------------- | ----------------------------------------------------------------- |
| `includeDirs`          | `string[]` | `['src/assets', 'public']`                            | Directories to scan.                                              |
| `excludeDirs`          | `string[]` | `[]`                                                  | Directories to skip.                                              |
| `extensions`           | `string[]` | `['png', 'jpg', 'jpeg', 'webp', 'gif', 'svg', 'ico']` | Image extensions to consider.                                     |
| `failOnDuplicates`     | `boolean`  | `false`                                               | Exit with code `1` when duplicates are found.                     |
| `ignoreCache`          | `boolean`  | `false`                                               | Ignore the cache for this run. Same as `--no-cache`.              |
| `cacheDir`             | `string`   | `node_modules/.cache/unqimages`                       | Directory for the cache file.                                     |
| `perceptual.enabled`   | `boolean`  | `false`                                               | Enable perceptual (visual) duplicate detection.                   |
| `perceptual.threshold` | `number`   | `10`                                                  | Hamming distance threshold for perceptual hashes. Range: `0..64`. |

Perceptual detection is disabled by default. Enable it to catch visually similar images, not just byte-identical files:

```js
export default {
  perceptual: {
    enabled: true,
    threshold: 10,
  },
};
```

## Cache

Scan results are cached in `node_modules/.cache/unqimages` by default. The cache is invalidated automatically when a file's size or modification time changes. Use `--no-cache` or `ignoreCache: true` to bypass it.

## Exit codes

| Code | Meaning                                                |
| ---- | ------------------------------------------------------ |
| `0`  | No duplicates found, or `failOnDuplicates` is `false`. |
| `1`  | Duplicates found and `failOnDuplicates` is `true`.     |
| `2`  | Runtime or configuration error.                        |

## Pre-commit hook with Husky and lint-staged

Add the optional peer dependencies:

```bash
npm install -D husky lint-staged
npx husky init
```

Configure `lint-staged` in `package.json`:

```json
{
  "lint-staged": {
    "*.{png,jpg,jpeg,webp,gif,svg,ico}": "unqimages --staged"
  }
}
```

Make sure your `.husky/pre-commit` hook runs `lint-staged`:

```sh
#!/bin/sh
. "$(dirname "$0")/_/husky.sh"
npx lint-staged
```

`unqimages --staged` receives the staged image paths from `lint-staged`, checks them as new files, and still compares them against the rest of the project so that duplicates against existing files are detected. If no staged image files are present, or if the command is run outside a git repository, it exits gracefully with code `0`.

You can also run the staged check manually:

```bash
npx unqimages --staged
```

This reads the staged file list directly from `git diff --cached`.

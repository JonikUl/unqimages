# @unqimages/cli

CLI for finding duplicate images in a project.

## Installation

```bash
npm install -D @unqimages/cli
```

## Usage

```bash
npx unqimages
```

The CLI scans the directories configured in `unqimages.config.js` (or the
`unqimages` field in `package.json`) and prints duplicate groups as JSON or a
formatted table.

## Configuration

Create `unqimages.config.js` in your project root:

```js
export default {
  includeDirs: ['src/assets', 'public'],
  excludeDirs: ['node_modules', 'dist'],
  extensions: ['png', 'jpg', 'jpeg', 'webp', 'gif', 'svg', 'ico'],
  failOnDuplicates: false,
  ignoreCache: false,
};
```

Set `failOnDuplicates: true` to exit with code `1` when duplicates are found.

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

> When passing staged file paths manually, put all flags before the paths:
> `unqimages --staged --output table file.png`. Flags placed after the paths
> are treated as part of the path list.

Make sure your `.husky/pre-commit` hook runs `lint-staged`:

```sh
#!/bin/sh
. "$(dirname "$0")/_/husky.sh"
npx lint-staged
```

`unqimages --staged` receives the staged image paths from `lint-staged`, checks
them as new files, and still compares them against the rest of the project so
that duplicates against existing files are detected. If no staged image files
are present, or if the command is run outside a git repository, it exits
gracefully with code `0`.

You can also run the staged check manually:

```bash
npx unqimages --staged
```

This reads the staged file list directly from `git diff --cached`.

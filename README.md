# unqimages

Find duplicate images in a project. Fast.

`unqimages` scans configured directories, hashes image contents, and reports duplicate groups. The heavy work is done by a Rust binary; a thin TypeScript wrapper provides the CLI interface and configuration loading.

## Features

- **Exact-duplicate detection** by file content hash.
- **Perceptual duplicate detection** for visually similar images (opt-in).
- **Configurable include/exclude directories and file extensions**.
- **JSON and table output** formats.
- **Cache** to avoid recomputing hashes across runs.
- **Pre-commit hook** support via Husky and lint-staged.
- **Cross-platform** native binaries for Linux, macOS, and Windows.

## Quick start

```bash
npm install -D @unqimages/cli
npx unqimages
```

Create `unqimages.config.js` in your project root:

```js
export default {
  includeDirs: ['src/assets', 'public'],
  excludeDirs: ['node_modules', 'dist'],
  extensions: ['png', 'jpg', 'jpeg', 'webp', 'gif', 'svg', 'ico'],
};
```

See [`packages/cli/README.md`](./packages/cli/README.md) for the full CLI documentation.

## Repository structure

```text
.
├── crates/unqimages-core   # Rust binary: hashing, discovery, cache, CLI parsing
├── packages/cli            # TypeScript wrapper and npm package
├── packages/config         # Shared configuration types
├── platform-packages/      # Per-OS npm packages with native binaries
├── scripts/                # Build, version sync, and publish pipeline scripts
└── .github/workflows/      # CI and publish automation
```

## Development

Requirements:

- Node.js 18+
- pnpm 11.5.1+
- Rust toolchain

Install dependencies and build:

```bash
pnpm install
pnpm run build
```

Run checks and tests:

```bash
pnpm test
pnpm run lint
pnpm typecheck
cargo test
cargo clippy -- -D warnings
```

## Contributing

Bug reports, feature requests, and pull requests are welcome. Release instructions for maintainers are in [`CONTRIBUTING.md`](./CONTRIBUTING.md).

## License

MIT

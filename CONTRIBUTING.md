# Contributing

Contributions are welcome. This document outlines how to report issues, submit changes, and keep the codebase healthy.

## Reporting issues

Before opening a new issue, please check whether a similar one already exists. When reporting a bug, include:

- The version of `@unqimages/cli` you are using.
- Your operating system and architecture.
- A minimal set of steps to reproduce the problem.
- The exact command you ran and the output you saw.

## Setting up the project

Requirements:

- Node.js 18 or later
- pnpm 11.5.1 or later
- Rust toolchain (stable)

Clone the repository and install dependencies:

```bash
pnpm install
```

Build the TypeScript wrapper and the Rust binary:

```bash
pnpm run build
```

## Running checks

All checks should pass before you open a pull request:

```bash
pnpm test
pnpm run lint
pnpm typecheck
cargo test
cargo clippy -- -D warnings
cargo fmt --check
pnpm format:check
```

## Making changes

- Keep changes focused on a single concern per pull request.
- Add or update tests for bug fixes and new features.
- Match the existing code style. Rust code is formatted with `cargo fmt`; TypeScript code is formatted with `oxfmt`.
- Write clear commit messages in English. Use the present tense and describe what the change does, for example:
  - `fix(cli): resolve binary path on Windows`
  - `feat(core): add perceptual hash threshold option`
  - `docs: update CLI README`

## Pull request process

1. Fork the repository and create a branch from `master`.
2. Make your changes and run the full check suite.
3. Open a pull request with a clear description of the problem and the solution.
4. Wait for CI to pass and address any review feedback.

## Releasing

Release instructions for maintainers are in [`RELEASING.md`](./RELEASING.md).

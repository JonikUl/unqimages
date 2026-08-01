# Releasing

This guide is for maintainers who publish `unqimages` to npm.

## Release model

`unqimages` is distributed as a scoped npm package (`@unqimages/cli`) using the **platform packages** pattern:

- `@unqimages/cli` — the main TypeScript wrapper.
- `@unqimages/cli-linux-x64`, `@unqimages/cli-darwin-x64`, `@unqimages/cli-darwin-arm64`, `@unqimages/cli-windows-x64` — platform-specific packages containing the Rust binary.

The main package lists the platform packages as `optionalDependencies`, so npm installs only the binary that matches the user's OS and CPU.

Publishing is automated by `.github/workflows/publish.yml`. The workflow builds native binaries on four runners, publishes the platform packages, waits for npm propagation, and then publishes the main package.

## One-time setup

1. Ensure the npm scope **`@unqimages`** exists and your account has publish access. If it does not exist, create an organization at [npmjs.com](https://www.npmjs.com/).
2. Add an `NPM_TOKEN` secret to the GitHub repository:
   - Go to **Settings → Secrets and variables → Actions → New repository secret**.
   - Use an npm access token with publish permissions.

## Publishing a release

### First release

The current base version is `0.1.0`. To trigger the first release:

```bash
git tag v0.1.0
git push origin v0.1.0
```

### Subsequent releases

Bump the version, propagate it through the workspace, and push a tag:

```bash
npm version patch   # or minor / major
pnpm run sync-versions
pnpm run add-platform-deps   # keeps optionalDependencies in @unqimages/cli in sync
pnpm run generate-manifests
git add -A
git commit -m "chore: release v$(node -p 'require('./package.json').version')"
git tag "v$(node -p 'require('./package.json').version')"
git push origin master --tags
```

The `publish.yml` workflow will then:

1. Build Rust release binaries for `linux-x64`, `darwin-x64`, `darwin-arm64`, and `windows-x64`.
2. Generate platform package manifests.
3. Copy binaries into the platform packages.
4. Publish the platform packages.
5. Wait for npm propagation.
6. Publish `@unqimages/cli`.

## Local smoke test

To verify the publish pipeline without uploading to the public npm registry:

```bash
pnpm run publish:local
```

This starts a local Verdaccio registry, publishes the host platform package and the main package, installs them in a temporary project, and checks `npx unqimages --version`.

## Verification and troubleshooting

After pushing a tag:

- Check the **Actions** tab in GitHub. The `Publish` workflow should be green.
- Confirm the packages exist on npm:
  - `@unqimages/cli@<version>`
  - `@unqimages/cli-linux-x64@<version>`
  - `@unqimages/cli-darwin-x64@<version>`
  - `@unqimages/cli-darwin-arm64@<version>`
  - `@unqimages/cli-windows-x64@<version>`

Common failures:

- Missing or invalid `NPM_TOKEN`.
- The `@unqimages` scope does not exist or the token's account lacks publish access.
- A platform package with the same version already exists.

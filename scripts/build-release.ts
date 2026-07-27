/**
 * build-release.ts — Build the native release Rust binary and copy it into the
 * matching platform package directory.
 *
 * Usage:
 *   pnpm run build:release
 */

import { pathToFileURL } from 'node:url';
import { chmodSync, copyFileSync, existsSync, mkdirSync } from 'fs';
import { execSync } from 'child_process';
import { join, resolve } from 'node:path';
import config from '../publish.config';
import { detectHostPlatform, PLATFORM_MAP } from './platforms.ts';

const ROOT = resolve(import.meta.dirname, '..');
const PLATFORM_DIR = join(ROOT, 'platform-packages');

function main(): void {
  if (config.binaries.length !== 1) {
    console.error(
      `build-release.ts currently supports exactly one binary, but ${config.binaries.length} were configured.`,
    );
    process.exit(1);
  }

  const platformKey = detectHostPlatform();
  const rustTarget = PLATFORM_MAP[platformKey].rustTarget;
  const binaryName = config.binaries[0].name;
  const binaryScope = config.binaries[0].scope;
  const ext = PLATFORM_MAP[platformKey].ext;

  console.log(`🔨 Building release binary for ${platformKey} (${rustTarget})...`);
  execSync(`cargo build --release --bin ${binaryName} --target ${rustTarget}`, {
    cwd: ROOT,
    stdio: 'inherit',
  });

  const src = join(ROOT, 'target', rustTarget, 'release', `${binaryName}${ext}`);
  const destDir = join(PLATFORM_DIR, `${binaryScope}-${platformKey}`);
  const dest = join(destDir, `${binaryName}${ext}`);

  if (!existsSync(destDir)) {
    mkdirSync(destDir, { recursive: true });
  }

  copyFileSync(src, dest);
  if (process.platform !== 'win32') {
    chmodSync(dest, 0o755);
  }

  console.log(`✅ Copied binary to ${dest}`);
}

const isMain = import.meta.url === pathToFileURL(resolve(process.argv[1] ?? '')).href;
if (isMain) {
  main();
}

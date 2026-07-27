/**
 * copy-platform-binaries.ts — Copy compiled Rust binaries from CI artifacts into
 * the platform package directories.
 *
 * Expects artifacts in the layout produced by the publish workflow:
 *   artifacts/binary-<platform>/<binary-name>[.exe]
 *
 * Usage:
 *   pnpm tsx scripts/copy-platform-binaries.ts
 *   pnpm tsx scripts/copy-platform-binaries.ts --source=./artifacts
 */

import { pathToFileURL } from 'node:url';
import { copyFileSync, existsSync, mkdirSync } from 'fs';
import { join, resolve } from 'node:path';
import config from '../publish.config';
import type { PublishConfig } from './types.ts';
import { PLATFORM_MAP } from './platforms.ts';

const ROOT = resolve(import.meta.dirname, '..');

export interface CopyBinariesResult {
  copied: string[];
}

export function copyBinaries(
  root: string,
  cfg: PublishConfig,
  artifactsDir: string,
): CopyBinariesResult {
  const platformDir = join(root, 'platform-packages');
  const copied: string[] = [];

  for (const binary of cfg.binaries) {
    for (const platform of cfg.platforms) {
      const mapping = PLATFORM_MAP[platform];
      if (!mapping) {
        throw new Error(`Unknown platform: ${platform}`);
      }

      const src = join(artifactsDir, `binary-${platform}`, `${binary.name}${mapping.ext}`);
      const destDir = join(platformDir, `${binary.scope}-${platform}`);
      const dest = join(destDir, `${binary.name}${mapping.ext}`);

      if (!existsSync(src)) {
        throw new Error(`Missing binary: ${src}`);
      }

      if (!existsSync(destDir)) {
        mkdirSync(destDir, { recursive: true });
      }

      copyFileSync(src, dest);
      copied.push(`${binary.scope}-${platform}`);
    }
  }

  return { copied };
}

function main(): void {
  const sourceOverride = process.argv.find((arg) => arg.startsWith('--source='))?.split('=')[1];
  const artifactsDir = join(ROOT, sourceOverride ?? 'artifacts');

  console.log(`📦 Copying binaries from ${artifactsDir}...`);
  const { copied } = copyBinaries(ROOT, config, artifactsDir);
  for (const platform of copied) {
    console.log(`  ✅ ${platform}`);
  }
  console.log(`\n📊 Copied ${copied.length} binary file(s)`);
}

const isMain = import.meta.url === pathToFileURL(resolve(process.argv[1] ?? '')).href;
if (isMain) {
  main();
}

/**
 * bump-dev-version.ts — Compute and apply a dev (pre-release) version.
 *
 * Usage:
 *   GITHUB_RUN_ID=12345678 pnpm tsx scripts/bump-dev-version.ts
 */

import { pathToFileURL } from 'node:url';
import { readFileSync, writeFileSync } from 'fs';
import { join, resolve } from 'node:path';

const ROOT = resolve(import.meta.dirname, '..');

export interface BumpResult {
  baseVersion: string;
  devVersion: string;
}

export function computeDevVersion(baseVersion: string, runId: string): string {
  const [major, minor, patch] = baseVersion.split('.').map(Number);
  return `${major}.${minor}.${patch + 1}-dev.${runId}`;
}

export function bumpDevVersion(root: string, runId: string): BumpResult {
  const rootPkgPath = join(root, 'package.json');
  const rootPkg = JSON.parse(readFileSync(rootPkgPath, 'utf8')) as {
    version: string;
  };
  const baseVersion = rootPkg.version;
  const devVersion = computeDevVersion(baseVersion, runId);

  rootPkg.version = devVersion;
  writeFileSync(rootPkgPath, JSON.stringify(rootPkg, null, 2) + '\n');

  return { baseVersion, devVersion };
}

function main(): void {
  const runId = process.env.GITHUB_RUN_ID;
  if (!runId) {
    console.error('❌ GITHUB_RUN_ID environment variable is required');
    console.error(
      '   Set it manually for local testing: GITHUB_RUN_ID=12345 pnpm tsx scripts/bump-dev-version.ts',
    );
    process.exit(1);
  }

  const { baseVersion, devVersion } = bumpDevVersion(ROOT, runId);
  console.log(`📦 Dev version: ${baseVersion} → ${devVersion}`);
  console.log('   Run sync-versions.ts to propagate to all packages');
}

const isMain = import.meta.url === pathToFileURL(resolve(process.argv[1] ?? '')).href;
if (isMain) {
  main();
}

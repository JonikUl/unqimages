/**
 * sync-versions.ts — Propagate the root package.json version to workspace
 * packages, platform packages, and the Cargo workspace manifest.
 *
 * Usage:
 *   pnpm tsx scripts/sync-versions.ts
 *   pnpm tsx scripts/sync-versions.ts --version 0.2.0-dev.12345678
 */

import { pathToFileURL } from 'node:url';
import { existsSync, readFileSync, writeFileSync } from 'fs';
import { globSync } from 'glob';
import { join, resolve } from 'node:path';
import { parse, stringify } from 'smol-toml';
import config from '../publish.config';
import type { PublishConfig } from './types.ts';

const ROOT = resolve(import.meta.dirname, '..');

const workspaceGlobs = ['packages/*/package.json', 'platform-packages/*/package.json'];

export interface SyncVersionsResult {
  version: string;
  updated: number;
}

export function syncVersions(
  root: string,
  config: PublishConfig,
  versionOverride?: string,
): SyncVersionsResult {
  const rootPkg = JSON.parse(readFileSync(join(root, 'package.json'), 'utf8')) as {
    version: string;
  };
  const version = versionOverride ?? rootPkg.version;

  let updated = 0;

  for (const pattern of workspaceGlobs) {
    for (const pkgPath of globSync(join(root, pattern))) {
      const pkg = JSON.parse(readFileSync(pkgPath, 'utf8')) as {
        version?: string;
        name?: string;
      };
      if (pkg.version !== version) {
        pkg.version = version;
        writeFileSync(pkgPath, JSON.stringify(pkg, null, 2) + '\n');
        updated++;
      }
    }
  }

  const cargoPath = join(root, config.cargoWorkspace);
  if (existsSync(cargoPath)) {
    const cargo = readFileSync(cargoPath, 'utf8');
    // Cargo pre-release semantics differ from npm, so strip npm pre-release suffix.
    const cargoVersion = version.replace(/-.*$/, '');
    const cargoDoc = parse(cargo) as { workspace?: { package?: { version?: string } } };
    if (cargoDoc.workspace?.package?.version !== undefined) {
      cargoDoc.workspace.package.version = cargoVersion;
      writeFileSync(cargoPath, stringify(cargoDoc));
      updated++;
    }
  }

  return { version, updated };
}

function parseVersionOverride(): string | undefined {
  return (
    process.argv.find((arg) => arg.startsWith('--version='))?.split('=')[1] ??
    (process.argv.includes('--version')
      ? process.argv[process.argv.indexOf('--version') + 1]
      : undefined)
  );
}

function main(): void {
  const versionOverride = parseVersionOverride();
  const { version, updated } = syncVersions(ROOT, config, versionOverride);
  console.log(`📦 Syncing version: ${version}`);
  console.log(`\n📊 Updated: ${updated} file(s)`);
}

const isMain = import.meta.url === pathToFileURL(resolve(process.argv[1] ?? '')).href;
if (isMain) {
  main();
}

/**
 * add-platform-deps.ts — Wire platform packages into the main wrapper package(s)
 * as optionalDependencies.
 *
 * Usage:
 *   pnpm tsx scripts/add-platform-deps.ts
 */

import { pathToFileURL } from 'node:url';
import { readFileSync, writeFileSync } from 'fs';
import { join, resolve } from 'node:path';
import config from '../publish.config';
import type { PublishConfig } from './types.ts';

const ROOT = resolve(import.meta.dirname, '..');

export function depsEqual(
  a: Record<string, string> | undefined,
  b: Record<string, string> | undefined,
): boolean {
  if (a === b) return true;
  if (!a || !b) return false;
  const aKeys = Object.keys(a);
  const bKeys = Object.keys(b);
  if (aKeys.length !== bKeys.length) return false;
  for (const key of aKeys) {
    if (a[key] !== b[key]) return false;
  }
  return true;
}

export interface AddPlatformDepsResult {
  updated: string[];
}

export function addPlatformDeps(root: string, cfg: PublishConfig): AddPlatformDepsResult {
  const updated: string[] = [];

  for (const mainPkg of cfg.mainPackages) {
    const pkgPath = join(root, mainPkg.path, 'package.json');
    const pkg = JSON.parse(readFileSync(pkgPath, 'utf8')) as {
      name?: string;
      optionalDependencies?: Record<string, string>;
    };

    const expected: Record<string, string> = {};
    for (const binary of cfg.binaries) {
      for (const platform of cfg.platforms) {
        const depName = `${cfg.scope}/${binary.scope}-${platform}`;
        // Platform packages are workspace members. workspace:* keeps pnpm's
        // lockfile in sync; prepare-publish injects the concrete version before publish.
        expected[depName] = 'workspace:*';
      }
    }

    if (!depsEqual(pkg.optionalDependencies, expected)) {
      pkg.optionalDependencies = expected;
      writeFileSync(pkgPath, JSON.stringify(pkg, null, 2) + '\n');
      updated.push(pkg.name ?? mainPkg.name);
    }
  }

  return { updated };
}

function main(): void {
  const rootPkg = JSON.parse(readFileSync(join(ROOT, 'package.json'), 'utf8')) as {
    version: string;
  };
  console.log(`📦 Adding platform optionalDependencies (version: ${rootPkg.version})`);
  const { updated } = addPlatformDeps(ROOT, config);
  for (const name of updated) {
    console.log(
      `  ✅ ${name}: added ${config.platforms.length * config.binaries.length} optionalDependencies`,
    );
  }
}

const isMain = import.meta.url === pathToFileURL(resolve(process.argv[1] ?? '')).href;
if (isMain) {
  main();
}

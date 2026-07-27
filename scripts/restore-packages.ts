/**
 * restore-packages.ts — Revert package.json files changed by prepare-publish.ts.
 *
 * Usage:
 *   pnpm tsx scripts/restore-packages.ts
 */

import { pathToFileURL } from 'node:url';
import { copyFileSync, unlinkSync } from 'fs';
import { globSync } from 'glob';
import { resolve } from 'node:path';

const ROOT = resolve(import.meta.dirname, '..');

const backupGlobs = ['packages/*/package.json.backup'];

export interface RestorePackagesResult {
  restored: number;
}

export function restorePackages(root: string): RestorePackagesResult {
  let restored = 0;

  for (const pattern of backupGlobs) {
    for (const backupPath of globSync(`${root}/${pattern}`)) {
      const originalPath = backupPath.replace('.backup', '');
      copyFileSync(backupPath, originalPath);
      unlinkSync(backupPath);
      restored++;
    }
  }

  return { restored };
}

function main(): void {
  const { restored } = restorePackages(ROOT);
  console.log(restored > 0 ? `\n📊 Restored ${restored} file(s)` : '  ⏭️  No backup files found');
}

const isMain = import.meta.url === pathToFileURL(resolve(process.argv[1] ?? '')).href;
if (isMain) {
  main();
}

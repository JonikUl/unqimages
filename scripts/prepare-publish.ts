/**
 * prepare-publish.ts — Replace workspace:* dependencies with concrete versions
 * before publishing. Creates .backup files so restore-packages.ts can revert.
 *
 * Usage:
 *   pnpm tsx scripts/prepare-publish.ts
 */

import { pathToFileURL } from 'node:url';
import { copyFileSync, existsSync, readFileSync, writeFileSync } from 'fs';
import { globSync } from 'glob';
import { join, resolve } from 'node:path';

const ROOT = resolve(import.meta.dirname, '..');

const publishableGlobs = ['packages/*/package.json'];

// Include optionalDependencies because platform packages are linked via
// workspace:* during development and must be rewritten before publish.
const DEP_TYPES = ['dependencies', 'peerDependencies', 'optionalDependencies'] as const;

export interface PreparePublishResult {
  replaced: number;
}

export function preparePublish(root: string, version: string): PreparePublishResult {
  let replaced = 0;

  for (const pattern of publishableGlobs) {
    for (const pkgPath of globSync(join(root, pattern))) {
      const pkg = JSON.parse(readFileSync(pkgPath, 'utf8')) as {
        name?: string;
        devDependencies?: Record<string, string>;
      } & Record<(typeof DEP_TYPES)[number], Record<string, string> | undefined>;
      let modified = false;

      for (const depType of DEP_TYPES) {
        const deps = pkg[depType];
        if (!deps) continue;

        for (const [name, ver] of Object.entries(deps)) {
          if (ver.startsWith('workspace:')) {
            deps[name] =
              ver === 'workspace:^'
                ? `^${version}`
                : ver === 'workspace:~'
                  ? `~${version}`
                  : version;
            modified = true;
            replaced++;
          }
        }
      }

      // Dev dependencies with workspace:* are not resolvable by consumers, so
      // drop them. They are restored by restore-packages.ts after publishing.
      if (pkg.devDependencies) {
        for (const [name, ver] of Object.entries(pkg.devDependencies)) {
          if (ver.startsWith('workspace:')) {
            delete pkg.devDependencies[name];
            modified = true;
            replaced++;
          }
        }
        if (Object.keys(pkg.devDependencies).length === 0) {
          delete pkg.devDependencies;
        }
      }

      if (modified) {
        const backupPath = `${pkgPath}.backup`;
        if (!existsSync(backupPath)) {
          copyFileSync(pkgPath, backupPath);
        }
        writeFileSync(pkgPath, JSON.stringify(pkg, null, 2) + '\n');
      }
    }
  }

  return { replaced };
}

function main(): void {
  const rootPkg = JSON.parse(readFileSync(join(ROOT, 'package.json'), 'utf8')) as {
    version: string;
  };
  console.log(`📦 Preparing packages for publish (version: ${rootPkg.version})`);
  const { replaced } = preparePublish(ROOT, rootPkg.version);
  console.log(`\n📊 Replaced ${replaced} workspace protocol reference(s)`);
}

const isMain = import.meta.url === pathToFileURL(resolve(process.argv[1] ?? '')).href;
if (isMain) {
  main();
}

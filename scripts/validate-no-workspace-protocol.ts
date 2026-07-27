/**
 * validate-no-workspace-protocol.ts — Final safety gate before publishing.
 *
 * Usage:
 *   pnpm tsx scripts/validate-no-workspace-protocol.ts
 */

import { pathToFileURL } from 'node:url';
import { readFileSync } from 'fs';
import { globSync } from 'glob';
import { join, resolve } from 'node:path';

const ROOT = resolve(import.meta.dirname, '..');

const publishableGlobs = ['packages/*/package.json'];
const DEP_TYPES = ['dependencies', 'devDependencies', 'peerDependencies'] as const;

function main(): void {
  console.log('🔍 Validating no workspace protocol references remain...');
  let violations = 0;

  for (const pattern of publishableGlobs) {
    for (const pkgPath of globSync(join(ROOT, pattern))) {
      const pkg = JSON.parse(readFileSync(pkgPath, 'utf8')) as {
        name?: string;
      } & Record<(typeof DEP_TYPES)[number], Record<string, string> | undefined>;

      for (const depType of DEP_TYPES) {
        const deps = pkg[depType];
        if (!deps) continue;

        for (const [name, ver] of Object.entries(deps)) {
          if (ver.startsWith('workspace:')) {
            console.error(`  ❌ ${pkg.name ?? pkgPath} → ${depType}.${name}: ${ver}`);
            violations++;
          }
        }
      }
    }
  }

  if (violations > 0) {
    console.error(`\n❌ Found ${violations} workspace protocol reference(s).`);
    process.exit(1);
  }

  console.log('✅ No workspace protocol references found.');
}

const isMain = import.meta.url === pathToFileURL(resolve(process.argv[1] ?? '')).href;
if (isMain) {
  main();
}

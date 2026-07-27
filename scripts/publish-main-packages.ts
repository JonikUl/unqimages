/**
 * publish-main-packages.ts — Publish the main wrapper package(s) after platform
 * packages have been published and propagated.
 *
 * Usage:
 *   pnpm tsx scripts/publish-main-packages.ts --tag dev --dry-run --allow-local
 */

import { pathToFileURL } from 'node:url';
import { join, resolve } from 'node:path';
import { execSync } from 'child_process';
import config from '../publish.config';
import { isAlreadyPublishedError } from './publish-utils.ts';

const ROOT = resolve(import.meta.dirname, '..');

function main(): void {
  const args = process.argv.slice(2);
  const dryRun = args.includes('--dry-run');
  const allowLocal = args.includes('--allow-local');
  const tagIdx = args.indexOf('--tag');
  const tag = tagIdx >= 0 ? args[tagIdx + 1] : undefined;
  const registryIdx = args.indexOf('--registry');
  const registry = registryIdx >= 0 ? args[registryIdx + 1] : process.env.NPM_CONFIG_REGISTRY;
  const userconfigIdx = args.indexOf('--userconfig');
  const userconfig = userconfigIdx >= 0 ? args[userconfigIdx + 1] : undefined;

  if (!process.env.CI && !process.env.GITHUB_ACTIONS && !allowLocal) {
    console.error('❌ This script must be run in a CI environment. Use --allow-local for testing.');
    process.exit(1);
  }

  console.log(`📦 Publishing main packages${dryRun ? ' (DRY RUN)' : ''}...`);

  let published = 0;
  let failed = 0;

  for (const mainPkg of config.mainPackages) {
    const pkgDir = join(ROOT, mainPkg.path);

    const tagArg = tag ? `--tag ${tag}` : '';
    const dryRunArg = dryRun ? '--dry-run' : '';
    const registryArg = registry ? `--registry ${registry}` : '';
    const userconfigArg = userconfig ? `--userconfig ${userconfig}` : '';
    const cmd =
      `npm publish ${tagArg} ${dryRunArg} ${registryArg} ${userconfigArg} --access public`.trim();

    console.log(`  📦 ${mainPkg.name}: ${cmd}`);

    try {
      execSync(cmd, { cwd: pkgDir, stdio: 'pipe' });
      console.log(`  ✅ ${mainPkg.name}: published`);
      published++;
    } catch (err: unknown) {
      if (isAlreadyPublishedError(err)) {
        console.log(`  ⏭️  ${mainPkg.name}: already published, skipping`);
        published++;
      } else {
        const message = err instanceof Error ? err.message : String(err);
        console.error(`  ❌ ${mainPkg.name}: ${message}`);
        failed++;
      }
    }
  }

  console.log(`\n📊 Published: ${published}, Failed: ${failed}`);
  if (failed > 0) process.exit(1);
}

const isMain = import.meta.url === pathToFileURL(resolve(process.argv[1] ?? '')).href;
if (isMain) {
  main();
}

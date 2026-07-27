/**
 * publish-local.ts — Smoke-test the full publish pipeline against a local
 * Verdaccio registry.
 *
 * Usage:
 *   pnpm run publish:local
 */

import { pathToFileURL } from 'node:url';
import { existsSync, mkdtempSync, mkdirSync, rmSync, writeFileSync } from 'fs';
import { execSync, spawn } from 'child_process';
import { createRequire } from 'module';
import { join, resolve } from 'node:path';
import { tmpdir } from 'os';
import { detectHostPlatform } from './platforms.ts';

const require = createRequire(import.meta.url);

const ROOT = resolve(import.meta.dirname, '..');
const REGISTRY = 'http://localhost:4873';
const VERDACCIO_CONFIG = join(ROOT, 'scripts', 'verdaccio.config.yaml');
const TMP_DIR = join(ROOT, 'scripts', 'tmp', 'verdaccio');

function run(cmd: string, cwd = ROOT, env?: Record<string, string>): void {
  console.log(`\n$ ${cmd}`);
  execSync(cmd, {
    cwd,
    stdio: 'inherit',
    env: { ...process.env, ...env },
  });
}

function runSilent(cmd: string, cwd = ROOT, env?: Record<string, string>): string {
  return execSync(cmd, {
    cwd,
    encoding: 'utf8',
    env: { ...process.env, ...env },
  }).trim();
}

function waitForRegistry(url: string, retries = 30): void {
  for (let i = 0; i < retries; i++) {
    try {
      runSilent(`npm ping --registry ${url}`);
      console.log('✅ Registry is ready');
      return;
    } catch {
      process.stdout.write('.');
      execSync('sleep 1');
    }
  }
  throw new Error('Timed out waiting for local registry');
}

function main(): void {
  if (!existsSync(TMP_DIR)) {
    mkdirSync(TMP_DIR, { recursive: true });
  }

  console.log('🚀 Starting local Verdaccio registry...');
  const verdaccio = spawn(
    require.resolve('verdaccio/bin/verdaccio'),
    ['-c', VERDACCIO_CONFIG, '-l', 'localhost:4873'],
    {
      cwd: ROOT,
      detached: true,
      stdio: 'ignore',
    },
  );

  let testDir = '';
  let npmrcPath = '';

  try {
    waitForRegistry(REGISTRY);

    // A minimal user config with a fake auth token lets npm publish to the
    // local registry without an interactive login.
    npmrcPath = join(TMP_DIR, 'npmrc');
    writeFileSync(npmrcPath, `registry=${REGISTRY}\n//localhost:4873/:_authToken=fake\n`);

    const hostPlatform = detectHostPlatform();

    run('pnpm run generate-manifests');
    run('pnpm run build:release');
    run('pnpm run add-platform-deps');
    run('pnpm run prepare-publish');

    run(
      `npx tsx scripts/publish-platform-packages.ts --allow-local --platform ${hostPlatform} --registry ${REGISTRY} --userconfig ${npmrcPath}`,
    );
    run(
      `npx tsx scripts/publish-main-packages.ts --allow-local --registry ${REGISTRY} --userconfig ${npmrcPath}`,
    );

    testDir = mkdtempSync(join(tmpdir(), 'unqimages-test-'));
    writeFileSync(
      join(testDir, '.npmrc'),
      `registry=${REGISTRY}\n//localhost:4873/:_authToken=fake\n`,
    );

    console.log('\n📦 Installing @unqimages/cli from local registry...');
    runSilent('npm install --legacy-peer-deps @unqimages/cli', testDir);

    console.log('\n🧪 Running npx unqimages --version');
    const version = runSilent('npx unqimages --version', testDir);
    console.log(`✅ Version output: ${version}`);
  } finally {
    console.log('\n🧹 Cleaning up...');
    try {
      run('pnpm run restore-packages');
    } catch {
      // Best-effort cleanup.
    }

    if (testDir) {
      try {
        rmSync(testDir, { recursive: true, force: true });
      } catch {
        // ignore
      }
    }

    if (npmrcPath) {
      try {
        rmSync(npmrcPath, { force: true });
      } catch {
        // ignore
      }
    }

    if (verdaccio.pid) {
      try {
        process.kill(-verdaccio.pid);
      } catch {
        try {
          verdaccio.kill();
        } catch {
          // ignore
        }
      }
    }

    try {
      rmSync(TMP_DIR, { recursive: true, force: true });
    } catch {
      // ignore
    }
  }

  console.log('\n✅ Local publish smoke test completed');
}

const isMain = import.meta.url === pathToFileURL(resolve(process.argv[1] ?? '')).href;
if (isMain) {
  main();
}

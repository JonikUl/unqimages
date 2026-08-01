import { execFileSync } from 'node:child_process';
import { createRequire } from 'node:module';
import { existsSync, mkdtempSync, rmdirSync, unlinkSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { tmpdir } from 'node:os';
import { ConfigError, loadConfig } from './config.js';
import { isObject } from './guards.js';
import type { ResolvedUnqimagesConfig } from './types.js';

const require = createRequire(import.meta.url);

const PLATFORM_PACKAGES: Record<string, string> = {
  'darwin-arm64': '@unqimages/cli-darwin-arm64',
  'darwin-x64': '@unqimages/cli-darwin-x64',
  'linux-x64': '@unqimages/cli-linux-x64',
  'win32-x64': '@unqimages/cli-windows-x64',
};

interface RustConfig {
  include_dirs: string[];
  exclude_dirs: string[];
  extensions: string[];
  fail_on_duplicates: boolean;
  ignore_cache: boolean;
  cache_dir?: string;
  perceptual?: { enabled: boolean; threshold: number };
}

interface ExecError {
  status: number | null;
  code?: string;
}

export async function run(): Promise<void> {
  const args = process.argv.slice(2);

  if (args.includes('--version') || args.includes('-V')) {
    const pkg: unknown = require('../package.json');
    if (!isObject(pkg) || typeof pkg.version !== 'string') {
      console.error('error: invalid package.json');
      process.exit(2);
    }
    console.log(pkg.version);
    process.exit(0);
  }

  const cwd = process.cwd();
  let tmpFile: string;
  let forwardedArgs: string[];

  try {
    const config = await loadConfig(cwd);
    forwardedArgs = collectForwardedArgs(args);
    tmpFile = writeTempConfig(buildRustConfig(config));
  } catch (e) {
    if (e instanceof ConfigError) {
      console.error(`error: ${e.message}`);
      process.exit(2);
    }
    throw e;
  }

  const binary = resolveBinary();

  try {
    execFileSync(binary, ['--config', tmpFile, '--cwd', cwd, ...forwardedArgs], {
      stdio: 'inherit',
    });
  } catch (error) {
    if (isExecError(error)) {
      if (error.status !== null) {
        process.exit(error.status);
      }
      if (error.code === 'ENOENT') {
        console.error(`error: binary not found: ${binary}`);
        process.exit(2);
      }
    }
    throw error;
  } finally {
    cleanupTempConfig(tmpFile);
  }
}

function buildRustConfig(config: ResolvedUnqimagesConfig): RustConfig {
  const rustConfig: RustConfig = {
    include_dirs: config.includeDirs,
    exclude_dirs: config.excludeDirs,
    extensions: config.extensions.map(normalizeExtension),
    fail_on_duplicates: config.failOnDuplicates,
    ignore_cache: config.ignoreCache,
  };

  if (config.cacheDir !== undefined) {
    rustConfig.cache_dir = config.cacheDir;
  }

  if (config.perceptual) {
    rustConfig.perceptual = config.perceptual;
  }

  return rustConfig;
}

function normalizeExtension(ext: string): string {
  const withoutDot = ext.replace(/^\./, '');
  if (withoutDot.length === 0) {
    throw new ConfigError(`extensions[] must not contain empty extensions: "${ext}"`);
  }
  return withoutDot.toLowerCase();
}

function writeTempConfig(rustConfig: RustConfig): string {
  const tmpDir = mkdtempSync(join(tmpdir(), 'unqimages-config-'));
  const tmpFile = join(tmpDir, 'config.json');
  writeFileSync(tmpFile, JSON.stringify(rustConfig), 'utf8');
  return tmpFile;
}

function cleanupTempConfig(tmpFile: string): void {
  try {
    unlinkSync(tmpFile);
    rmdirSync(dirname(tmpFile));
  } catch {
    // Best-effort cleanup.
  }
}

function resolveBinary(): string {
  const platformKey = `${process.platform}-${process.arch}`;
  const packageName = PLATFORM_PACKAGES[platformKey];

  if (!packageName) {
    const supported = Object.keys(PLATFORM_PACKAGES)
      .map((key) => `  - ${key}`)
      .join('\n');
    console.error(`Unsupported platform: ${platformKey}\n\nSupported platforms:\n${supported}`);
    process.exit(2);
  }

  try {
    const pkgJsonPath = require.resolve(`${packageName}/package.json`);
    const pkgDir = dirname(pkgJsonPath);
    const pkgMeta: unknown = require(pkgJsonPath);
    const binaryName = isPackageMeta(pkgMeta)
      ? (pkgMeta.main ?? 'unqimages-core')
      : 'unqimages-core';
    const binaryPath = join(pkgDir, binaryName);
    if (process.platform === 'win32' && !binaryPath.endsWith('.exe')) {
      return `${binaryPath}.exe`;
    }
    return binaryPath;
  } catch {
    const devPath = findLocalBinary();
    if (devPath) {
      return devPath;
    }

    console.error(
      `Failed to find package "${packageName}" for platform ${platformKey}.\n` +
        'This usually means the optional dependency was not installed.\n\n' +
        'Try reinstalling with:\n  npm install\n\n' +
        'If the problem persists, install the platform package directly:\n' +
        `  npm install ${packageName}`,
    );
    process.exit(2);
  }
}

function findLocalBinary(): string | null {
  try {
    const root = resolve(fileURLToPath(import.meta.url), '../../../..');
    const binaryName = process.platform === 'win32' ? 'unqimages-core.exe' : 'unqimages-core';
    const candidates = [
      join(root, 'target', 'release', binaryName),
      join(root, 'target', 'debug', binaryName),
    ];

    for (const candidate of candidates) {
      if (existsSync(candidate)) {
        return candidate;
      }
    }
  } catch {
    // ignore
  }

  return null;
}

function collectForwardedArgs(args: string[]): string[] {
  const filtered: string[] = [];

  for (let i = 0; i < args.length; i++) {
    const arg = args[i];

    if (arg === '--config' || arg === '--cwd') {
      throw new ConfigError(`${arg} is set automatically by unqimages and cannot be overridden`);
    }
    if (arg.startsWith('--config=') || arg.startsWith('--cwd=')) {
      throw new ConfigError(
        `${arg.split('=')[0]} is set automatically by unqimages and cannot be overridden`,
      );
    }

    filtered.push(arg);
  }

  return filtered;
}

function isExecError(error: unknown): error is ExecError {
  return (
    isObject(error) &&
    'status' in error &&
    (typeof error.status === 'number' || error.status === null)
  );
}

function isPackageMeta(value: unknown): value is { main?: string } {
  return isObject(value) && (value.main === undefined || typeof value.main === 'string');
}

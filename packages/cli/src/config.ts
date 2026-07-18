import { readFileSync, existsSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { pathToFileURL } from 'node:url';
import type { UnqimagesConfig } from '@unqimages/config';
import type { ResolvedUnqimagesConfig } from './types.js';
import { isObject } from './guards.js';

const DEFAULT_INCLUDE_DIRS = ['src/assets', 'public'];
const DEFAULT_EXTENSIONS = ['png', 'jpg', 'jpeg', 'gif', 'webp', 'svg', 'ico'];
const DEFAULT_PERCEPTUAL_THRESHOLD = 10;
const MAX_PERCEPTUAL_THRESHOLD = 64;

export class ConfigError extends Error {}

export async function loadConfig(cwd: string): Promise<ResolvedUnqimagesConfig> {
  const raw = await loadRawConfig(cwd);
  validate(raw);
  return resolveConfig(raw);
}

async function loadRawConfig(cwd: string): Promise<Record<string, unknown>> {
  const pkg = findNearestPackageJson(cwd);
  const pkgConfig = pkg?.unqimages;

  if (pkgConfig !== undefined && !isObject(pkgConfig)) {
    throw new ConfigError(`unqimages field in ${pkg?.path ?? 'package.json'} must be an object`);
  }

  const fileConfig = await findConfigFile(cwd);

  return {
    ...pkgConfig,
    ...fileConfig,
  };
}

function findNearestPackageJson(startDir: string): { path: string; unqimages: unknown } | null {
  let dir = resolve(startDir);

  while (true) {
    const candidate = join(dir, 'package.json');

    if (existsSync(candidate)) {
      try {
        const parsed: unknown = JSON.parse(readFileSync(candidate, 'utf8'));
        if (!isObject(parsed)) {
          throw new ConfigError(`invalid package.json: ${candidate}`);
        }
        return { path: candidate, unqimages: parsed.unqimages };
      } catch (e) {
        if (e instanceof ConfigError) throw e;
        throw new ConfigError(`failed to read package.json ${candidate}: ${errorMessage(e)}`);
      }
    }

    const parent = dirname(dir);
    if (parent === dir) break;
    dir = parent;
  }

  return null;
}

async function findConfigFile(startDir: string): Promise<Record<string, unknown> | null> {
  let dir = resolve(startDir);
  const extensions = ['js', 'mjs', 'json'];

  while (true) {
    for (const ext of extensions) {
      const candidate = join(dir, `unqimages.config.${ext}`);

      if (existsSync(candidate)) {
        return readConfigFile(candidate);
      }
    }

    const parent = dirname(dir);
    if (parent === dir) break;
    dir = parent;
  }

  return null;
}

async function readConfigFile(filePath: string): Promise<Record<string, unknown>> {
  if (filePath.endsWith('.json')) {
    try {
      const parsed: unknown = JSON.parse(readFileSync(filePath, 'utf8'));
      if (!isObject(parsed)) {
        throw new ConfigError(`config file must export an object: ${filePath}`);
      }
      return parsed;
    } catch (e) {
      if (e instanceof ConfigError) throw e;
      throw new ConfigError(`failed to parse JSON config ${filePath}: ${errorMessage(e)}`);
    }
  }

  try {
    const module: unknown = await import(pathToFileURL(filePath).href);
    const exported = getDefaultExport(module);

    if (!isObject(exported)) {
      throw new ConfigError(`config file must export an object: ${filePath}`);
    }

    return exported;
  } catch (e) {
    if (e instanceof ConfigError) throw e;
    throw new ConfigError(`failed to load config ${filePath}: ${errorMessage(e)}`);
  }
}

function getDefaultExport(module: unknown): unknown {
  if (!isObject(module)) {
    return undefined;
  }
  return module.default ?? module;
}

function resolveConfig(raw: UnqimagesConfig): ResolvedUnqimagesConfig {
  return {
    includeDirs: raw.includeDirs ?? DEFAULT_INCLUDE_DIRS,
    excludeDirs: raw.excludeDirs ?? [],
    extensions: raw.extensions ?? DEFAULT_EXTENSIONS,
    failOnDuplicates: raw.failOnDuplicates ?? false,
    ignoreCache: raw.ignoreCache ?? false,
    cacheDir: raw.cacheDir,
    perceptual: raw.perceptual
      ? {
          enabled: raw.perceptual.enabled ?? true,
          threshold: raw.perceptual.threshold ?? DEFAULT_PERCEPTUAL_THRESHOLD,
        }
      : null,
  };
}

function validate(raw: unknown): asserts raw is UnqimagesConfig {
  if (!isObject(raw)) {
    throw new ConfigError('config must be an object');
  }

  if (raw.includeDirs !== undefined) {
    assertStringArray(raw.includeDirs, 'includeDirs', true);
  }
  if (raw.excludeDirs !== undefined) {
    assertStringArray(raw.excludeDirs, 'excludeDirs', false);
  }
  if (raw.extensions !== undefined) {
    assertStringArray(raw.extensions, 'extensions', true);
  }
  if (raw.failOnDuplicates !== undefined) {
    assertBoolean(raw.failOnDuplicates, 'failOnDuplicates');
  }
  if (raw.ignoreCache !== undefined) {
    assertBoolean(raw.ignoreCache, 'ignoreCache');
  }
  if (raw.cacheDir !== undefined) {
    assertString(raw.cacheDir, 'cacheDir');
    if (raw.cacheDir.length === 0) {
      throw new ConfigError('cacheDir must not be empty');
    }
  }

  if (raw.perceptual !== undefined) {
    if (!isObject(raw.perceptual)) {
      throw new ConfigError('perceptual must be an object');
    }
    if (raw.perceptual.enabled !== undefined) {
      assertBoolean(raw.perceptual.enabled, 'perceptual.enabled');
    }
    if (raw.perceptual.threshold !== undefined) {
      assertNumber(raw.perceptual.threshold, 'perceptual.threshold');
      const threshold = raw.perceptual.threshold;
      if (!Number.isInteger(threshold) || threshold < 0 || threshold > MAX_PERCEPTUAL_THRESHOLD) {
        throw new ConfigError(
          `perceptual.threshold must be an integer between 0 and ${MAX_PERCEPTUAL_THRESHOLD}`
        );
      }
    }
  }
}

function assertStringArray(value: unknown, name: string, nonEmpty: boolean): asserts value is string[] {
  if (!Array.isArray(value)) {
    throw new ConfigError(`${name} must be an array of strings`);
  }
  for (const item of value) {
    assertString(item, `${name}[]`);
    if (nonEmpty && item.length === 0) {
      throw new ConfigError(`${name}[] must not contain empty strings`);
    }
  }
}

function assertString(value: unknown, name: string): asserts value is string {
  if (typeof value !== 'string') {
    throw new ConfigError(`${name} must be a string`);
  }
}

function assertBoolean(value: unknown, name: string): asserts value is boolean {
  if (typeof value !== 'boolean') {
    throw new ConfigError(`${name} must be a boolean`);
  }
}

function assertNumber(value: unknown, name: string): asserts value is number {
  if (typeof value !== 'number') {
    throw new ConfigError(`${name} must be a number`);
  }
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

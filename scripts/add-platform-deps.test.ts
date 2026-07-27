import { describe, expect, it } from 'vitest';
import { mkdtempSync, mkdirSync, rmSync, writeFileSync, readFileSync } from 'node:fs';
import { join } from 'node:path';
import { tmpdir } from 'node:os';
import { addPlatformDeps, depsEqual } from './add-platform-deps.ts';
import type { PublishConfig } from './types.ts';

const config: PublishConfig = {
  scope: '@unqimages',
  binaries: [{ name: 'unqimages-core', scope: 'cli', cargoPackage: 'unqimages-core' }],
  platforms: ['linux-x64', 'darwin-arm64'],
  mainPackages: [{ path: 'packages/cli', name: '@unqimages/cli' }],
  cargoWorkspace: 'Cargo.toml',
  repositoryUrl: 'https://github.com/JonikUl/unqimages',
};

describe('depsEqual', () => {
  it('returns true for identical records', () => {
    expect(depsEqual({ a: '1.0.0' }, { a: '1.0.0' })).toBe(true);
  });

  it('returns false when keys differ', () => {
    expect(depsEqual({ a: '1.0.0' }, { a: '1.0.0', b: '2.0.0' })).toBe(false);
  });

  it('returns false when values differ', () => {
    expect(depsEqual({ a: '1.0.0' }, { a: '2.0.0' })).toBe(false);
  });
});

describe('addPlatformDeps', () => {
  it('adds platform optionalDependencies to main packages', () => {
    const root = mkdtempSync(join(tmpdir(), 'unqimages-platform-deps-test-'));
    mkdirSync(join(root, 'packages', 'cli'), { recursive: true });
    writeFileSync(
      join(root, 'packages', 'cli', 'package.json'),
      JSON.stringify({ name: '@unqimages/cli', version: '0.2.0' }) + '\n',
    );

    try {
      const { updated } = addPlatformDeps(root, config, '0.2.0');
      expect(updated).toEqual(['@unqimages/cli']);

      const pkg = JSON.parse(readFileSync(join(root, 'packages', 'cli', 'package.json'), 'utf8'));
      expect(pkg.optionalDependencies).toEqual({
        '@unqimages/cli-linux-x64': '0.2.0',
        '@unqimages/cli-darwin-arm64': '0.2.0',
      });
    } finally {
      rmSync(root, { recursive: true, force: true });
    }
  });

  it('does not rewrite package.json when deps already match', () => {
    const root = mkdtempSync(join(tmpdir(), 'unqimages-platform-deps-up-to-date-test-'));
    mkdirSync(join(root, 'packages', 'cli'), { recursive: true });
    writeFileSync(
      join(root, 'packages', 'cli', 'package.json'),
      JSON.stringify({
        name: '@unqimages/cli',
        version: '0.2.0',
        optionalDependencies: {
          '@unqimages/cli-linux-x64': '0.2.0',
          '@unqimages/cli-darwin-arm64': '0.2.0',
        },
      }) + '\n',
    );

    try {
      const { updated } = addPlatformDeps(root, config, '0.2.0');
      expect(updated).toEqual([]);
    } finally {
      rmSync(root, { recursive: true, force: true });
    }
  });
});

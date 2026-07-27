import { describe, expect, it } from 'vitest';
import { mkdtempSync, mkdirSync, rmSync, writeFileSync, readFileSync, existsSync } from 'node:fs';
import { join } from 'node:path';
import { tmpdir } from 'node:os';
import { copyBinaries } from './copy-platform-binaries.ts';
import type { PublishConfig } from './types.ts';

const config: PublishConfig = {
  scope: '@unqimages',
  binaries: [{ name: 'unqimages-core', scope: 'cli', cargoPackage: 'unqimages-core' }],
  platforms: ['linux-x64', 'windows-x64'],
  mainPackages: [{ path: 'packages/cli', name: '@unqimages/cli' }],
  cargoWorkspace: 'Cargo.toml',
  repositoryUrl: 'https://github.com/JonikUl/unqimages',
};

describe('copyBinaries', () => {
  it('copies binaries for all platforms into platform-packages', () => {
    const root = mkdtempSync(join(tmpdir(), 'unqimages-copy-test-'));
    const artifactsDir = join(root, 'artifacts');

    mkdirSync(join(artifactsDir, 'binary-linux-x64'), { recursive: true });
    writeFileSync(join(artifactsDir, 'binary-linux-x64', 'unqimages-core'), 'linux binary');
    mkdirSync(join(artifactsDir, 'binary-windows-x64'), { recursive: true });
    writeFileSync(join(artifactsDir, 'binary-windows-x64', 'unqimages-core.exe'), 'windows binary');

    try {
      const { copied } = copyBinaries(root, config, artifactsDir);
      expect(copied).toEqual(['cli-linux-x64', 'cli-windows-x64']);

      const linuxDest = join(root, 'platform-packages', 'cli-linux-x64', 'unqimages-core');
      expect(existsSync(linuxDest)).toBe(true);
      expect(readFileSync(linuxDest, 'utf8')).toBe('linux binary');

      const windowsDest = join(root, 'platform-packages', 'cli-windows-x64', 'unqimages-core.exe');
      expect(existsSync(windowsDest)).toBe(true);
      expect(readFileSync(windowsDest, 'utf8')).toBe('windows binary');
    } finally {
      rmSync(root, { recursive: true, force: true });
    }
  });

  it('throws when a binary is missing', () => {
    const root = mkdtempSync(join(tmpdir(), 'unqimages-copy-missing-test-'));
    const artifactsDir = join(root, 'artifacts');

    try {
      expect(() => copyBinaries(root, config, artifactsDir)).toThrow('Missing binary');
    } finally {
      rmSync(root, { recursive: true, force: true });
    }
  });
});

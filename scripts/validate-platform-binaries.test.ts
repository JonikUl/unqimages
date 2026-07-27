import { describe, expect, it } from 'vitest';
import { mkdtempSync, mkdirSync, rmSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { tmpdir } from 'node:os';
import { validateBinaries } from './validate-platform-binaries.ts';
import type { PublishConfig } from './types.ts';

const config: PublishConfig = {
  scope: '@unqimages',
  binaries: [{ name: 'unqimages-core', scope: 'cli', cargoPackage: 'unqimages-core' }],
  platforms: ['linux-x64', 'windows-x64', 'darwin-arm64'],
  mainPackages: [{ path: 'packages/cli', name: '@unqimages/cli' }],
  cargoWorkspace: 'Cargo.toml',
  repositoryUrl: 'https://github.com/JonikUl/unqimages',
};

function createTempRoot(): string {
  return mkdtempSync(join(tmpdir(), 'unqimages-validate-test-'));
}

function writeBinary(root: string, platform: string, bytes: number[], ext = ''): void {
  const dir = join(root, 'platform-packages', `cli-${platform}`);
  mkdirSync(dir, { recursive: true });
  writeFileSync(join(dir, `unqimages-core${ext}`), Buffer.from(bytes));
}

const PLATFORM_EXTS: Record<string, string> = {
  'linux-x64': '',
  'windows-x64': '.exe',
  'darwin-arm64': '',
};

describe('validateBinaries', () => {
  it('passes for valid headers', () => {
    const root = createTempRoot();
    try {
      writeBinary(root, 'linux-x64', [0x7f, 0x45, 0x4c, 0x46], PLATFORM_EXTS['linux-x64']);
      writeBinary(root, 'windows-x64', [0x4d, 0x5a], PLATFORM_EXTS['windows-x64']);
      writeBinary(root, 'darwin-arm64', [0xcf, 0xfa, 0xed, 0xfe], PLATFORM_EXTS['darwin-arm64']);

      const { errors } = validateBinaries(root, config);
      expect(errors).toHaveLength(0);
    } finally {
      rmSync(root, { recursive: true, force: true });
    }
  });

  it('reports missing binaries', () => {
    const root = createTempRoot();
    try {
      const { errors } = validateBinaries(root, config);
      expect(errors).toContainEqual({
        platform: 'linux-x64',
        binaryFile: 'unqimages-core',
        reason: 'missing',
      });
    } finally {
      rmSync(root, { recursive: true, force: true });
    }
  });

  it('reports empty binaries', () => {
    const root = createTempRoot();
    try {
      writeBinary(root, 'linux-x64', [], PLATFORM_EXTS['linux-x64']);
      const { errors } = validateBinaries(root, config);
      expect(errors).toContainEqual({
        platform: 'linux-x64',
        binaryFile: 'unqimages-core',
        reason: 'empty',
      });
    } finally {
      rmSync(root, { recursive: true, force: true });
    }
  });

  it('reports invalid headers', () => {
    const root = createTempRoot();
    try {
      writeBinary(root, 'linux-x64', [0x00, 0x00, 0x00, 0x00], PLATFORM_EXTS['linux-x64']);
      const { errors } = validateBinaries(root, config);
      expect(errors).toContainEqual({
        platform: 'linux-x64',
        binaryFile: 'unqimages-core',
        reason: 'invalid-header',
      });
    } finally {
      rmSync(root, { recursive: true, force: true });
    }
  });
});

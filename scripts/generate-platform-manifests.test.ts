import { describe, expect, it } from 'vitest';
import { mkdtempSync, rmSync, writeFileSync, readFileSync } from 'node:fs';
import { join } from 'node:path';
import { tmpdir } from 'node:os';
import { generateManifests } from './generate-platform-manifests.ts';
import type { PublishConfig } from './types.ts';

function createTempRoot(version = '0.2.0'): string {
  const root = mkdtempSync(join(tmpdir(), 'unqimages-manifests-test-'));
  writeFileSync(
    join(root, 'package.json'),
    JSON.stringify({ name: '@unqimages/workspace', version, license: 'MIT' }) + '\n',
  );
  return root;
}

const config: PublishConfig = {
  scope: '@unqimages',
  binaries: [{ name: 'unqimages-core', scope: 'cli', cargoPackage: 'unqimages-core' }],
  platforms: ['linux-x64', 'windows-x64'],
  mainPackages: [{ path: 'packages/cli', name: '@unqimages/cli' }],
  cargoWorkspace: 'Cargo.toml',
  repositoryUrl: 'https://github.com/JonikUl/unqimages',
};

describe('generateManifests', () => {
  it('generates package.json and postinstall.js for each platform', () => {
    const root = createTempRoot();
    try {
      const generated = generateManifests(root, config, '0.2.0');
      expect(generated).toBe(2);

      const linuxManifest = JSON.parse(
        readFileSync(join(root, 'platform-packages', 'cli-linux-x64', 'package.json'), 'utf8'),
      );
      expect(linuxManifest).toMatchObject({
        name: '@unqimages/cli-linux-x64',
        version: '0.2.0',
        os: ['linux'],
        cpu: ['x64'],
        main: 'unqimages-core',
        files: ['unqimages-core', 'postinstall.js'],
      });

      const windowsManifest = JSON.parse(
        readFileSync(join(root, 'platform-packages', 'cli-windows-x64', 'package.json'), 'utf8'),
      );
      expect(windowsManifest).toMatchObject({
        name: '@unqimages/cli-windows-x64',
        version: '0.2.0',
        os: ['win32'],
        cpu: ['x64'],
        main: 'unqimages-core.exe',
        files: ['unqimages-core.exe', 'postinstall.js'],
      });

      const postinstall = readFileSync(
        join(root, 'platform-packages', 'cli-linux-x64', 'postinstall.js'),
        'utf8',
      );
      expect(postinstall).toContain('chmodSync');
    } finally {
      rmSync(root, { recursive: true, force: true });
    }
  });

  it('skips unknown platforms', () => {
    const root = createTempRoot();
    try {
      const generated = generateManifests(root, { ...config, platforms: ['freebsd-x64'] }, '0.2.0');
      expect(generated).toBe(0);
    } finally {
      rmSync(root, { recursive: true, force: true });
    }
  });
});

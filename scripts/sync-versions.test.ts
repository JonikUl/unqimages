import { describe, expect, it } from 'vitest';
import { mkdtempSync, mkdirSync, rmSync, writeFileSync, readFileSync } from 'node:fs';
import { join } from 'node:path';
import { tmpdir } from 'node:os';
import { syncVersions } from './sync-versions.ts';
import type { PublishConfig } from './types.ts';

function createTempWorkspace(): string {
  const root = mkdtempSync(join(tmpdir(), 'unqimages-sync-test-'));
  writeFileSync(
    join(root, 'package.json'),
    JSON.stringify({ name: '@unqimages/workspace', version: '0.2.0' }) + '\n',
  );
  mkdirSync(join(root, 'packages', 'cli'), { recursive: true });
  writeFileSync(
    join(root, 'packages', 'cli', 'package.json'),
    JSON.stringify({ name: '@unqimages/cli', version: '0.1.0' }) + '\n',
  );
  mkdirSync(join(root, 'platform-packages', 'cli-linux-x64'), { recursive: true });
  writeFileSync(
    join(root, 'platform-packages', 'cli-linux-x64', 'package.json'),
    JSON.stringify({ name: '@unqimages/cli-linux-x64', version: '0.1.0' }) + '\n',
  );
  writeFileSync(
    join(root, 'Cargo.toml'),
    `[workspace]\nmembers = ["crates/unqimages-core"]\n\n[workspace.package]\nversion = "0.1.0"\n`,
  );
  return root;
}

const config: PublishConfig = {
  scope: '@unqimages',
  binaries: [{ name: 'unqimages-core', scope: 'cli', cargoPackage: 'unqimages-core' }],
  platforms: ['linux-x64'],
  mainPackages: [{ path: 'packages/cli', name: '@unqimages/cli' }],
  cargoWorkspace: 'Cargo.toml',
  repositoryUrl: 'https://github.com/JonikUl/unqimages',
};

describe('syncVersions', () => {
  it('propagates root version to workspace packages and Cargo.toml', () => {
    const root = createTempWorkspace();
    try {
      const result = syncVersions(root, config);
      expect(result.version).toBe('0.2.0');
      expect(result.updated).toBe(3);

      const pkg = JSON.parse(readFileSync(join(root, 'packages', 'cli', 'package.json'), 'utf8'));
      expect(pkg.version).toBe('0.2.0');

      const platformPkg = JSON.parse(
        readFileSync(join(root, 'platform-packages', 'cli-linux-x64', 'package.json'), 'utf8'),
      );
      expect(platformPkg.version).toBe('0.2.0');

      const cargo = readFileSync(join(root, 'Cargo.toml'), 'utf8');
      expect(cargo).toContain('version = "0.2.0"');
    } finally {
      rmSync(root, { recursive: true, force: true });
    }
  });

  it('uses version override and strips npm pre-release from Cargo', () => {
    const root = createTempWorkspace();
    try {
      const result = syncVersions(root, config, '0.3.0-dev.123');
      expect(result.version).toBe('0.3.0-dev.123');

      const pkg = JSON.parse(readFileSync(join(root, 'packages', 'cli', 'package.json'), 'utf8'));
      expect(pkg.version).toBe('0.3.0-dev.123');

      const cargo = readFileSync(join(root, 'Cargo.toml'), 'utf8');
      expect(cargo).toContain('version = "0.3.0"');
      expect(cargo).not.toContain('0.3.0-dev.123');
    } finally {
      rmSync(root, { recursive: true, force: true });
    }
  });

  it('does not update files when versions already match', () => {
    const root = createTempWorkspace();
    writeFileSync(
      join(root, 'packages', 'cli', 'package.json'),
      JSON.stringify({ name: '@unqimages/cli', version: '0.2.0' }) + '\n',
    );
    writeFileSync(
      join(root, 'platform-packages', 'cli-linux-x64', 'package.json'),
      JSON.stringify({ name: '@unqimages/cli-linux-x64', version: '0.2.0' }) + '\n',
    );
    try {
      const result = syncVersions(root, config);
      expect(result.updated).toBe(1); // only Cargo.toml
    } finally {
      rmSync(root, { recursive: true, force: true });
    }
  });
});

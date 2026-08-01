import { describe, expect, it } from 'vitest';
import { mkdtempSync, mkdirSync, rmSync, writeFileSync, readFileSync, existsSync } from 'node:fs';
import { join } from 'node:path';
import { tmpdir } from 'node:os';
import { preparePublish } from './prepare-publish.ts';
import { restorePackages } from './restore-packages.ts';

describe('preparePublish + restorePackages', () => {
  it('replaces workspace:* dependencies and restores them', () => {
    const root = mkdtempSync(join(tmpdir(), 'unqimages-prepare-test-'));
    mkdirSync(join(root, 'packages', 'cli'), { recursive: true });
    const original = {
      name: '@unqimages/cli',
      version: '0.2.0',
      dependencies: {
        '@unqimages/config': 'workspace:*',
      },
      peerDependencies: {
        husky: 'workspace:^',
      },
      optionalDependencies: {
        '@unqimages/cli-linux-x64': 'workspace:*',
      },
      devDependencies: {
        '@unqimages/build-utils': 'workspace:*',
      },
    };
    writeFileSync(
      join(root, 'packages', 'cli', 'package.json'),
      JSON.stringify(original, null, 2) + '\n',
    );

    try {
      const { replaced } = preparePublish(root, '0.2.0');
      expect(replaced).toBe(4);

      const prepared = JSON.parse(
        readFileSync(join(root, 'packages', 'cli', 'package.json'), 'utf8'),
      );
      expect(prepared.dependencies['@unqimages/config']).toBe('0.2.0');
      expect(prepared.peerDependencies['husky']).toBe('^0.2.0');
      expect(prepared.optionalDependencies['@unqimages/cli-linux-x64']).toBe('0.2.0');
      expect(prepared.devDependencies).toBeUndefined();

      const { restored } = restorePackages(root);
      expect(restored).toBe(1);

      const restoredPkg = JSON.parse(
        readFileSync(join(root, 'packages', 'cli', 'package.json'), 'utf8'),
      );
      expect(restoredPkg).toEqual(original);
      expect(existsSync(join(root, 'packages', 'cli', 'package.json.backup'))).toBe(false);
    } finally {
      rmSync(root, { recursive: true, force: true });
    }
  });

  it('does not overwrite an existing backup', () => {
    const root = mkdtempSync(join(tmpdir(), 'unqimages-prepare-idempotent-test-'));
    mkdirSync(join(root, 'packages', 'cli'), { recursive: true });
    const original = {
      name: '@unqimages/cli',
      version: '0.2.0',
      dependencies: { '@unqimages/config': 'workspace:*' },
    };
    writeFileSync(
      join(root, 'packages', 'cli', 'package.json'),
      JSON.stringify(original, null, 2) + '\n',
    );
    // Pre-create backup to simulate a previous run
    writeFileSync(
      join(root, 'packages', 'cli', 'package.json.backup'),
      JSON.stringify(original, null, 2) + '\n',
    );

    try {
      preparePublish(root, '0.2.0');
      preparePublish(root, '0.2.0');

      const { restored } = restorePackages(root);
      expect(restored).toBe(1);

      const restoredPkg = JSON.parse(
        readFileSync(join(root, 'packages', 'cli', 'package.json'), 'utf8'),
      );
      expect(restoredPkg).toEqual(original);
    } finally {
      rmSync(root, { recursive: true, force: true });
    }
  });
});

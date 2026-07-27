import { describe, expect, it } from 'vitest';
import { mkdtempSync, rmSync, writeFileSync, readFileSync } from 'node:fs';
import { join } from 'node:path';
import { tmpdir } from 'node:os';
import { bumpDevVersion, computeDevVersion } from './bump-dev-version.ts';

describe('computeDevVersion', () => {
  it('increments patch and appends run id', () => {
    expect(computeDevVersion('0.2.15', '12345')).toBe('0.2.16-dev.12345');
  });
});

describe('bumpDevVersion', () => {
  it('writes the dev version to root package.json', () => {
    const root = mkdtempSync(join(tmpdir(), 'unqimages-bump-test-'));
    writeFileSync(
      join(root, 'package.json'),
      JSON.stringify({ name: '@unqimages/workspace', version: '0.2.15' }) + '\n',
    );

    try {
      const result = bumpDevVersion(root, '12345');
      expect(result).toEqual({ baseVersion: '0.2.15', devVersion: '0.2.16-dev.12345' });

      const pkg = JSON.parse(readFileSync(join(root, 'package.json'), 'utf8'));
      expect(pkg.version).toBe('0.2.16-dev.12345');
    } finally {
      rmSync(root, { recursive: true, force: true });
    }
  });
});

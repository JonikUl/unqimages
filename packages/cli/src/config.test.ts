import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { tmpdir } from 'node:os';
import { describe, expect, it } from 'vitest';
import { ConfigError, loadConfig } from './config.js';

describe('loadConfig', () => {
  function createTempDir(): string {
    return mkdtempSync(join(tmpdir(), 'unqimages-config-test-'));
  }

  function cleanup(dir: string): void {
    rmSync(dir, { recursive: true, force: true });
  }

  function writePackageJson(dir: string, unqimages: unknown): void {
    writeFileSync(join(dir, 'package.json'), JSON.stringify({ name: 'test', unqimages }));
  }

  it('applies defaults when no config exists', async () => {
    const dir = createTempDir();
    try {
      const config = await loadConfig(dir);
      expect(config.includeDirs).toEqual(['src/assets', 'public']);
      expect(config.extensions).toEqual(['png', 'jpg', 'jpeg', 'gif', 'webp', 'svg', 'ico']);
      expect(config.failOnDuplicates).toBe(false);
      expect(config.excludeDirs).toEqual([]);
      expect(config.perceptual).toBeNull();
    } finally {
      cleanup(dir);
    }
  });

  it('reads config from package.json unqimages field', async () => {
    const dir = createTempDir();
    try {
      writePackageJson(dir, { includeDirs: ['img'], extensions: ['png'] });
      const config = await loadConfig(dir);
      expect(config.includeDirs).toEqual(['img']);
      expect(config.extensions).toEqual(['png']);
    } finally {
      cleanup(dir);
    }
  });

  it('prefers config file over package.json field', async () => {
    const dir = createTempDir();
    try {
      writePackageJson(dir, { includeDirs: ['pkg'] });
      writeFileSync(
        join(dir, 'unqimages.config.js'),
        'export default { includeDirs: ["file"], extensions: ["jpg"] };\n'
      );
      const config = await loadConfig(dir);
      expect(config.includeDirs).toEqual(['file']);
      expect(config.extensions).toEqual(['jpg']);
    } finally {
      cleanup(dir);
    }
  });

  it('supports unqimages.config.json', async () => {
    const dir = createTempDir();
    try {
      writeFileSync(join(dir, 'unqimages.config.json'), JSON.stringify({ failOnDuplicates: true }));
      const config = await loadConfig(dir);
      expect(config.failOnDuplicates).toBe(true);
    } finally {
      cleanup(dir);
    }
  });

  it('supports unqimages.config.mjs default export', async () => {
    const dir = createTempDir();
    try {
      writeFileSync(
        join(dir, 'unqimages.config.mjs'),
        'export default { includeDirs: ["assets"] };\n'
      );
      const config = await loadConfig(dir);
      expect(config.includeDirs).toEqual(['assets']);
    } finally {
      cleanup(dir);
    }
  });

  it('resolves perceptual defaults when object is present', async () => {
    const dir = createTempDir();
    try {
      writeFileSync(join(dir, 'unqimages.config.json'), JSON.stringify({ perceptual: {} }));
      const config = await loadConfig(dir);
      expect(config.perceptual).toEqual({ enabled: true, threshold: 10 });
    } finally {
      cleanup(dir);
    }
  });

  it('rejects non-array includeDirs', async () => {
    const dir = createTempDir();
    try {
      writeFileSync(join(dir, 'unqimages.config.json'), JSON.stringify({ includeDirs: 'src' }));
      await expect(loadConfig(dir)).rejects.toThrow(ConfigError);
    } finally {
      cleanup(dir);
    }
  });

  it('rejects empty strings in includeDirs', async () => {
    const dir = createTempDir();
    try {
      writeFileSync(join(dir, 'unqimages.config.json'), JSON.stringify({ includeDirs: [''] }));
      await expect(loadConfig(dir)).rejects.toThrow(ConfigError);
    } finally {
      cleanup(dir);
    }
  });

  it('rejects perceptual threshold above 64', async () => {
    const dir = createTempDir();
    try {
      writeFileSync(
        join(dir, 'unqimages.config.json'),
        JSON.stringify({ perceptual: { threshold: 65 } })
      );
      await expect(loadConfig(dir)).rejects.toThrow(ConfigError);
    } finally {
      cleanup(dir);
    }
  });

  it('rejects non-integer perceptual threshold', async () => {
    const dir = createTempDir();
    try {
      writeFileSync(
        join(dir, 'unqimages.config.json'),
        JSON.stringify({ perceptual: { threshold: 10.5 } })
      );
      await expect(loadConfig(dir)).rejects.toThrow(ConfigError);
    } finally {
      cleanup(dir);
    }
  });

  it('rejects invalid unqimages field type in package.json', async () => {
    const dir = createTempDir();
    try {
      writePackageJson(dir, 'not-an-object');
      await expect(loadConfig(dir)).rejects.toThrow(ConfigError);
    } finally {
      cleanup(dir);
    }
  });

  it('finds nearest package.json in parent directory', async () => {
    const dir = createTempDir();
    try {
      writePackageJson(dir, { includeDirs: ['parent'] });
      const nested = join(dir, 'nested');
      mkdirSync(nested);
      const config = await loadConfig(nested);
      expect(config.includeDirs).toEqual(['parent']);
    } finally {
      cleanup(dir);
    }
  });
});

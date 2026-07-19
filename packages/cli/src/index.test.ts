import { spawnSync } from 'node:child_process';
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { tmpdir } from 'node:os';
import { describe, expect, it } from 'vitest';

const __dirname = dirname(fileURLToPath(import.meta.url));
const WRAPPER = resolve(__dirname, '../bin/unqimages.js');

interface RunResult {
  code: number;
  stdout: string;
  stderr: string;
}

function createTempDir(): string {
  return mkdtempSync(join(tmpdir(), 'unqimages-cli-test-'));
}

function cleanup(dir: string): void {
  rmSync(dir, { recursive: true, force: true });
}

function initGitRepo(dir: string): void {
  runGit(dir, ['init', '-q']);
  runGit(dir, ['config', 'user.email', 'test@example.com']);
  runGit(dir, ['config', 'user.name', 'Test']);
}

function runGit(dir: string, args: string[]): void {
  const result = spawnSync('git', args, { cwd: dir, encoding: 'utf8' });
  if (result.status !== 0) {
    throw new Error(`git ${args.join(' ')} failed: ${result.stderr}`);
  }
}

function writeImage(dir: string, segments: string[], content = 'dup'): void {
  const path = join(dir, ...segments);
  mkdirSync(dirname(path), { recursive: true });
  writeFileSync(path, content);
}

function runWrapper(cwd: string, args: string[]): RunResult {
  const result = spawnSync('node', [WRAPPER, ...args], {
    cwd,
    encoding: 'utf8',
  });
  return {
    code: result.status ?? -1,
    stdout: result.stdout ?? '',
    stderr: result.stderr ?? '',
  };
}

describe('unqimages --staged', () => {
  it('blocks the commit when a staged file duplicates an existing one', () => {
    const dir = createTempDir();
    try {
      initGitRepo(dir);
      writeImage(dir, ['public', 'existing.png']);
      runGit(dir, ['add', '.']);
      runGit(dir, ['commit', '-q', '-m', 'initial']);

      writeImage(dir, ['public', 'staged.png']);
      runGit(dir, ['add', 'public/staged.png']);
      writeFileSync(
        join(dir, 'unqimages.config.json'),
        JSON.stringify({ failOnDuplicates: true })
      );

      const { code, stdout } = runWrapper(dir, ['--staged']);
      expect(code).toBe(1);
      expect(stdout).toContain('existing.png');
      expect(stdout).toContain('staged.png');
    } finally {
      cleanup(dir);
    }
  });

  it('exits cleanly when staged files are unique', () => {
    const dir = createTempDir();
    try {
      initGitRepo(dir);
      writeImage(dir, ['public', 'existing.png']);
      runGit(dir, ['add', '.']);
      runGit(dir, ['commit', '-q', '-m', 'initial']);

      writeImage(dir, ['public', 'staged.png'], 'unique');
      runGit(dir, ['add', 'public/staged.png']);
      writeFileSync(
        join(dir, 'unqimages.config.json'),
        JSON.stringify({ failOnDuplicates: true })
      );

      const { code, stdout } = runWrapper(dir, ['--staged']);
      expect(code).toBe(0);
      expect(stdout).toContain('"duplicates": []');
    } finally {
      cleanup(dir);
    }
  });

  it('exits gracefully when there are no staged files', () => {
    const dir = createTempDir();
    try {
      initGitRepo(dir);
      writeImage(dir, ['public', 'existing.png']);
      runGit(dir, ['add', '.']);
      runGit(dir, ['commit', '-q', '-m', 'initial']);

      const { code, stderr } = runWrapper(dir, ['--staged']);
      expect(code).toBe(0);
      expect(stderr).toContain('no staged image files');
    } finally {
      cleanup(dir);
    }
  });

  it('exits gracefully outside a git repository', () => {
    const dir = createTempDir();
    try {
      writeFileSync(
        join(dir, 'unqimages.config.json'),
        JSON.stringify({ failOnDuplicates: true })
      );

      const { code, stderr } = runWrapper(dir, ['--staged']);
      expect(code).toBe(0);
      expect(stderr).toContain('no git repository');
    } finally {
      cleanup(dir);
    }
  });
});

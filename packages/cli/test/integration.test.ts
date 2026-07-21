import { execFileSync, spawnSync } from "node:child_process";
import { cpSync, existsSync, mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { tmpdir } from "node:os";
import { beforeAll, describe, expect, it } from "vitest";

const __dirname = dirname(fileURLToPath(import.meta.url));
const PROJECT_ROOT = resolve(__dirname, "../../..");
const WRAPPER = resolve(__dirname, "../bin/unqimages.js");
const FIXTURES = resolve(PROJECT_ROOT, "test/fixtures");

interface RunResult {
  code: number;
  stdout: string;
  stderr: string;
}

interface CliOutput {
  duplicates: Array<{
    hash: string;
    kind: "exact" | "perceptual";
    entries: Array<{ path: string }>;
  }>;
  scanned: number;
  elapsed_ms: number;
  used_cache: boolean;
}

function createTempDir(): string {
  return mkdtempSync(join(tmpdir(), "unqimages-integration-"));
}

function cleanup(dir: string): void {
  rmSync(dir, { recursive: true, force: true });
}

function copyFixtures(target: string, ...names: string[]): void {
  for (const name of names) {
    const source = join(FIXTURES, name);
    if (!existsSync(source)) {
      throw new Error(`fixture not found: ${source}`);
    }
    cpSync(source, join(target, name), { recursive: true, force: true });
  }
}

function writeConfig(dir: string, config: unknown): void {
  writeFileSync(join(dir, "unqimages.config.json"), JSON.stringify(config), "utf8");
}

function ensureBinary(): string {
  const binary = join(PROJECT_ROOT, "target", "release", "unqimages-core");
  if (existsSync(binary)) {
    return binary;
  }

  const debugBinary = join(PROJECT_ROOT, "target", "debug", "unqimages-core");
  if (existsSync(debugBinary)) {
    return debugBinary;
  }

  execFileSync("cargo", ["build", "--release", "--bin", "unqimages-core"], {
    cwd: PROJECT_ROOT,
    stdio: "inherit",
  });
  return binary;
}

function ensureWrapperBuild(): void {
  const distIndex = resolve(__dirname, "../dist/index.js");
  if (existsSync(distIndex)) {
    return;
  }

  execFileSync("pnpm", ["run", "build"], {
    cwd: resolve(__dirname, ".."),
    stdio: "inherit",
  });
}

function runWrapper(cwd: string, args: string[] = []): RunResult {
  const result = spawnSync("node", [WRAPPER, ...args], {
    cwd,
    encoding: "utf8",
  });
  return {
    code: result.status ?? -1,
    stdout: result.stdout ?? "",
    stderr: result.stderr ?? "",
  };
}

function parseJsonOutput(result: RunResult): CliOutput {
  const firstBrace = result.stdout.indexOf("{");
  if (firstBrace === -1) {
    throw new Error(`no JSON found in stdout: ${result.stdout}`);
  }

  let depth = 0;
  let end = -1;
  for (let i = firstBrace; i < result.stdout.length; i++) {
    const char = result.stdout[i];
    if (char === "{") {
      depth++;
    } else if (char === "}") {
      depth--;
    }
    if (depth === 0) {
      end = i + 1;
      break;
    }
  }

  if (end === -1) {
    throw new Error(`unbalanced JSON in stdout: ${result.stdout}`);
  }

  return JSON.parse(result.stdout.slice(firstBrace, end)) as CliOutput;
}

function initGitRepo(dir: string): void {
  runGit(dir, ["init", "-q"]);
  runGit(dir, ["config", "user.email", "test@example.com"]);
  runGit(dir, ["config", "user.name", "Test"]);
}

function runGit(dir: string, args: string[]): void {
  const result = spawnSync("git", args, { cwd: dir, encoding: "utf8" });
  if (result.status !== 0) {
    throw new Error(`git ${args.join(" ")} failed: ${result.stderr}`);
  }
}

beforeAll(() => {
  ensureBinary();
  ensureWrapperBuild();
});

describe("unqimages integration", () => {
  it("finds no duplicates when all images are unique", () => {
    const dir = createTempDir();
    try {
      copyFixtures(dir, "exact/unique.png");
      writeConfig(dir, { includeDirs: ["exact"] });

      const result = runWrapper(dir);
      const output = parseJsonOutput(result);

      expect(result.code).toBe(0);
      expect(output.scanned).toBe(1);
      expect(output.duplicates).toHaveLength(0);
      expect(output.used_cache).toBe(false);
    } finally {
      cleanup(dir);
    }
  });

  it("finds exact duplicates in JSON output", () => {
    const dir = createTempDir();
    try {
      copyFixtures(dir, "exact/dupe-a.png", "exact/dupe-b.png");
      writeConfig(dir, { includeDirs: ["exact"] });

      const result = runWrapper(dir);
      const output = parseJsonOutput(result);

      expect(result.code).toBe(0);
      expect(output.scanned).toBe(2);
      expect(output.duplicates).toHaveLength(1);
      expect(output.duplicates[0].kind).toBe("exact");

      const paths = output.duplicates[0].entries.map((e) => e.path);
      expect(paths).toContain("exact/dupe-a.png");
      expect(paths).toContain("exact/dupe-b.png");
    } finally {
      cleanup(dir);
    }
  });

  it("returns exit code 1 when failOnDuplicates is enabled", () => {
    const dir = createTempDir();
    try {
      copyFixtures(dir, "exact/dupe-a.png", "exact/dupe-b.png");
      writeConfig(dir, { includeDirs: ["exact"], failOnDuplicates: true });

      const result = runWrapper(dir);

      expect(result.code).toBe(1);
    } finally {
      cleanup(dir);
    }
  });

  it("finds perceptual duplicates across formats", () => {
    const dir = createTempDir();
    try {
      copyFixtures(dir, "perceptual");
      writeConfig(dir, {
        includeDirs: ["perceptual"],
        perceptual: { enabled: true, threshold: 10 },
      });

      const result = runWrapper(dir);
      const output = parseJsonOutput(result);

      expect(result.code).toBe(0);
      expect(output.scanned).toBe(3);
      expect(output.duplicates).toHaveLength(1);
      expect(output.duplicates[0].kind).toBe("perceptual");

      const paths = output.duplicates[0].entries.map((e) => e.path);
      expect(paths).toContain("perceptual/original.png");
      expect(paths).toContain("perceptual/compressed.jpg");
      expect(paths).not.toContain("perceptual/different.png");
    } finally {
      cleanup(dir);
    }
  });

  it("is idempotent across two runs and uses cache on the second run", () => {
    const dir = createTempDir();
    try {
      copyFixtures(dir, "exact/dupe-a.png", "exact/dupe-b.png");
      writeConfig(dir, { includeDirs: ["exact"] });

      const first = runWrapper(dir);
      const firstOutput = parseJsonOutput(first);
      expect(first.code).toBe(0);
      expect(firstOutput.used_cache).toBe(false);

      const second = runWrapper(dir);
      const secondOutput = parseJsonOutput(second);
      expect(second.code).toBe(0);
      expect(secondOutput.used_cache).toBe(true);

      expect(secondOutput.scanned).toBe(firstOutput.scanned);
      expect(secondOutput.duplicates).toEqual(firstOutput.duplicates);
    } finally {
      cleanup(dir);
    }
  });

  it("pre-commit hook blocks a staged duplicate of an existing file", () => {
    const dir = createTempDir();
    try {
      copyFixtures(dir, "exact/unique.png");
      writeConfig(dir, { includeDirs: ["exact"], failOnDuplicates: true });

      initGitRepo(dir);
      runGit(dir, ["add", "exact/unique.png"]);
      runGit(dir, ["commit", "-q", "-m", "initial"]);

      cpSync(join(FIXTURES, "exact/unique.png"), join(dir, "exact/staged.png"), { force: true });
      runGit(dir, ["add", "exact/staged.png"]);

      const result = runWrapper(dir, ["--staged"]);
      const output = parseJsonOutput(result);

      expect(result.code).toBe(1);
      const paths = output.duplicates.flatMap((g) => g.entries.map((e) => e.path));
      expect(paths).toContain("exact/unique.png");
      expect(paths).toContain("exact/staged.png");
    } finally {
      cleanup(dir);
    }
  });

  it("exits with code 2 for invalid config", () => {
    const dir = createTempDir();
    try {
      writeFileSync(
        join(dir, "unqimages.config.json"),
        '{"perceptual": {"threshold": 65}}',
        "utf8",
      );

      const result = runWrapper(dir);

      expect(result.code).toBe(2);
      expect(result.stderr).toMatch(/threshold|config/i);
    } finally {
      cleanup(dir);
    }
  });

  it("ignores unsupported files and non-matching extensions", () => {
    const dir = createTempDir();
    try {
      copyFixtures(dir, "exact/dupe-a.png", "unsupported");
      writeConfig(dir, { includeDirs: ["exact", "unsupported"], extensions: ["png"] });

      const result = runWrapper(dir);
      const output = parseJsonOutput(result);

      expect(result.code).toBe(0);
      expect(output.scanned).toBe(1);
      expect(output.duplicates).toHaveLength(0);
    } finally {
      cleanup(dir);
    }
  });

  it("finds perceptual duplicates across png and webp formats", () => {
    const dir = createTempDir();
    try {
      copyFixtures(dir, "exact/dupe-a.png", "mixed/dupe.webp");
      writeConfig(dir, {
        includeDirs: ["exact", "mixed"],
        perceptual: { enabled: true, threshold: 10 },
      });

      const result = runWrapper(dir);
      const output = parseJsonOutput(result);

      expect(result.code).toBe(0);
      expect(output.scanned).toBe(2);
      expect(output.duplicates).toHaveLength(1);
      expect(output.duplicates[0].kind).toBe("perceptual");

      const paths = output.duplicates[0].entries.map((e) => e.path);
      expect(paths).toContain("exact/dupe-a.png");
      expect(paths).toContain("mixed/dupe.webp");
    } finally {
      cleanup(dir);
    }
  });

  it("respects excludeDirs", () => {
    const dir = createTempDir();
    try {
      copyFixtures(dir, "exact/dupe-a.png", "exact/dupe-b.png");
      mkdirSync(join(dir, "exact/ignored"), { recursive: true });
      cpSync(join(FIXTURES, "exact/dupe-b.png"), join(dir, "exact/ignored/dupe-b.png"), {
        force: true,
      });
      writeConfig(dir, { includeDirs: ["exact"], excludeDirs: ["exact/ignored"] });

      const result = runWrapper(dir);
      const output = parseJsonOutput(result);

      expect(result.code).toBe(0);
      expect(output.scanned).toBe(2);
      expect(output.duplicates).toHaveLength(1);
    } finally {
      cleanup(dir);
    }
  });

  it("honors ignoreCache flag", () => {
    const dir = createTempDir();
    try {
      copyFixtures(dir, "exact/dupe-a.png", "exact/dupe-b.png");
      writeConfig(dir, { includeDirs: ["exact"] });

      const first = runWrapper(dir);
      expect(first.code).toBe(0);

      const second = runWrapper(dir, ["--no-cache"]);
      const secondOutput = parseJsonOutput(second);

      expect(second.code).toBe(0);
      expect(secondOutput.used_cache).toBe(false);
    } finally {
      cleanup(dir);
    }
  });

  it("supports table output format", () => {
    const dir = createTempDir();
    try {
      copyFixtures(dir, "exact/dupe-a.png", "exact/dupe-b.png");
      writeConfig(dir, { includeDirs: ["exact"] });

      const result = runWrapper(dir, ["--output", "table"]);

      expect(result.code).toBe(0);
      expect(result.stdout).toContain("Exact:");
      expect(result.stdout).toContain("exact/dupe-a.png");
      expect(result.stdout).toContain("exact/dupe-b.png");
    } finally {
      cleanup(dir);
    }
  });

  it("prints version with --version", () => {
    const dir = createTempDir();
    try {
      const result = runWrapper(dir, ["--version"]);

      expect(result.code).toBe(0);
      expect(result.stdout.trim()).toMatch(/^\d+\.\d+\.\d+/);
    } finally {
      cleanup(dir);
    }
  });
});

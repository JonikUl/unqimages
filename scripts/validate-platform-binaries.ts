/**
 * validate-platform-binaries.ts — Pre-publish safety check for platform binaries.
 *
 * Usage:
 *   pnpm tsx scripts/validate-platform-binaries.ts
 */

import { pathToFileURL } from 'node:url';
import { closeSync, existsSync, openSync, readSync, statSync } from 'fs';
import { join, resolve } from 'node:path';
import config from '../publish.config';
import type { PublishConfig } from './types.ts';
import { getOsFromPlatform, PLATFORM_MAP } from './platforms.ts';

const ROOT = resolve(import.meta.dirname, '..');

const HEADERS: Record<string, { bytes: number[]; name: string }[]> = {
  darwin: [
    { bytes: [0xcf, 0xfa, 0xed, 0xfe], name: 'Mach-O 64-bit (LE)' },
    { bytes: [0xfe, 0xed, 0xfa, 0xcf], name: 'Mach-O 64-bit (BE)' },
  ],
  linux: [{ bytes: [0x7f, 0x45, 0x4c, 0x46], name: 'ELF' }],
  windows: [{ bytes: [0x4d, 0x5a], name: 'PE/MZ' }],
};

const MAX_HEADER_BYTES = 4;

export interface BinaryValidationError {
  platform: string;
  binaryFile: string;
  reason: 'missing' | 'empty' | 'invalid-header';
}

export interface ValidateBinariesResult {
  errors: BinaryValidationError[];
}

function validateHeader(filePath: string, os: string): boolean {
  const expectedHeaders = HEADERS[os];
  if (!expectedHeaders) return true;

  const fd = openSync(filePath, 'r');
  try {
    const buffer = Buffer.alloc(MAX_HEADER_BYTES);
    const bytesRead = readSync(fd, buffer, 0, MAX_HEADER_BYTES, 0);
    return expectedHeaders.some((header) =>
      header.bytes.every((byte, i) => i < bytesRead && buffer[i] === byte),
    );
  } finally {
    closeSync(fd);
  }
}

export function validateBinaries(root: string, cfg: PublishConfig): ValidateBinariesResult {
  const platformDir = join(root, 'platform-packages');
  const errors: BinaryValidationError[] = [];

  for (const binary of cfg.binaries) {
    for (const platform of cfg.platforms) {
      const mapping = PLATFORM_MAP[platform];
      if (!mapping) {
        errors.push({ platform, binaryFile: binary.name, reason: 'invalid-header' });
        continue;
      }

      const os = getOsFromPlatform(platform);
      const binaryFile = `${binary.name}${mapping.ext}`;
      const binaryPath = join(platformDir, `${binary.scope}-${platform}`, binaryFile);

      if (!existsSync(binaryPath)) {
        errors.push({ platform, binaryFile, reason: 'missing' });
        continue;
      }

      const stat = statSync(binaryPath);
      if (stat.size === 0) {
        errors.push({ platform, binaryFile, reason: 'empty' });
        continue;
      }

      if (!validateHeader(binaryPath, os)) {
        errors.push({ platform, binaryFile, reason: 'invalid-header' });
      }
    }
  }

  return { errors };
}

function main(): void {
  console.log('🔍 Validating platform binaries...');

  const { errors } = validateBinaries(ROOT, config);

  for (const binary of config.binaries) {
    for (const platform of config.platforms) {
      const mapping = PLATFORM_MAP[platform];
      if (!mapping) continue;

      const binaryFile = `${binary.name}${mapping.ext}`;
      const binaryPath = join(ROOT, 'platform-packages', `${binary.scope}-${platform}`, binaryFile);

      if (errors.some((e) => e.platform === platform && e.binaryFile === binaryFile)) {
        continue;
      }

      const stat = statSync(binaryPath);
      const sizeMB = (stat.size / 1024 / 1024).toFixed(1);
      console.log(`  ✅ ${binary.scope}-${platform}: ${binaryFile} (${sizeMB} MB)`);
    }
  }

  if (errors.length > 0) {
    for (const err of errors) {
      const binaryPath = join(
        ROOT,
        'platform-packages',
        `${config.binaries.find((b) => err.binaryFile.startsWith(b.name))?.scope ?? 'unknown'}-${err.platform}`,
        err.binaryFile,
      );
      const label =
        err.reason === 'missing'
          ? 'Missing'
          : err.reason === 'empty'
            ? 'Empty file'
            : 'Invalid binary header';
      console.error(`  ❌ ${label}: ${binaryPath}`);
    }
    console.error(`\n❌ ${errors.length} validation error(s)`);
    process.exit(1);
  }

  console.log('\n✅ All binaries validated');
}

const isMain = import.meta.url === pathToFileURL(resolve(process.argv[1] ?? '')).href;
if (isMain) {
  main();
}

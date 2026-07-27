/**
 * Shared platform metadata used by the publish pipeline.
 *
 * Centralising this data avoids drift between manifest generation, binary
 * copying, validation and local builds.
 */

export interface PlatformMetadata {
  /** Value for the `os` field in package.json. */
  os: string;
  /** Value for the `cpu` field in package.json. */
  cpu: string;
  /** Binary file extension (empty for Unix, `.exe` for Windows). */
  ext: string;
  /** Rust target triple used by cargo. */
  rustTarget: string;
}

export const PLATFORM_MAP: Record<string, PlatformMetadata> = {
  'darwin-arm64': {
    os: 'darwin',
    cpu: 'arm64',
    ext: '',
    rustTarget: 'aarch64-apple-darwin',
  },
  'darwin-x64': {
    os: 'darwin',
    cpu: 'x64',
    ext: '',
    rustTarget: 'x86_64-apple-darwin',
  },
  'linux-x64': {
    os: 'linux',
    cpu: 'x64',
    ext: '',
    rustTarget: 'x86_64-unknown-linux-gnu',
  },
  'windows-x64': {
    os: 'win32',
    cpu: 'x64',
    ext: '.exe',
    rustTarget: 'x86_64-pc-windows-msvc',
  },
};

const NODE_PLATFORM_TO_PLATFORM_KEY: Record<string, string> = {
  'darwin-arm64': 'darwin-arm64',
  'darwin-x64': 'darwin-x64',
  'linux-x64': 'linux-x64',
  'win32-x64': 'windows-x64',
};

export function getOsFromPlatform(platform: string): string {
  if (platform.startsWith('darwin')) return 'darwin';
  if (platform.startsWith('linux')) return 'linux';
  if (platform.startsWith('windows')) return 'windows';
  return 'unknown';
}

export function detectHostPlatform(): string {
  const nodeKey = `${process.platform}-${process.arch}`;
  const platformKey = NODE_PLATFORM_TO_PLATFORM_KEY[nodeKey];

  if (!platformKey) {
    throw new Error(
      `Unsupported build platform: ${nodeKey}. ` +
        'Use the CI workflow to build binaries for other platforms.',
    );
  }

  return platformKey;
}

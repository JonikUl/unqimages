import { describe, expect, it } from 'vitest';
import { detectHostPlatform, getOsFromPlatform, PLATFORM_MAP } from './platforms.ts';

describe('PLATFORM_MAP', () => {
  it('contains the four supported platforms', () => {
    expect(Object.keys(PLATFORM_MAP).sort()).toEqual([
      'darwin-arm64',
      'darwin-x64',
      'linux-x64',
      'windows-x64',
    ]);
  });

  it('maps windows-x64 to win32 os and exe extension', () => {
    expect(PLATFORM_MAP['windows-x64']).toEqual({
      os: 'win32',
      cpu: 'x64',
      ext: '.exe',
      rustTarget: 'x86_64-pc-windows-msvc',
    });
  });

  it('maps darwin-arm64 to darwin os with no extension', () => {
    expect(PLATFORM_MAP['darwin-arm64']).toEqual({
      os: 'darwin',
      cpu: 'arm64',
      ext: '',
      rustTarget: 'aarch64-apple-darwin',
    });
  });
});

describe('getOsFromPlatform', () => {
  it('returns darwin, linux, or windows prefixes', () => {
    expect(getOsFromPlatform('darwin-x64')).toBe('darwin');
    expect(getOsFromPlatform('linux-x64')).toBe('linux');
    expect(getOsFromPlatform('windows-x64')).toBe('windows');
  });

  it('returns unknown for unrecognized platforms', () => {
    expect(getOsFromPlatform('freebsd-arm64')).toBe('unknown');
  });
});

describe('detectHostPlatform', () => {
  it('returns a supported platform for the current process', () => {
    const platform = detectHostPlatform();
    expect(Object.keys(PLATFORM_MAP)).toContain(platform);
  });
});

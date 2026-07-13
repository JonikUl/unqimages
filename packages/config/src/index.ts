export interface UnqimagesConfig {
  include?: string[];
  exclude?: string[];
  extensions?: string[];
  perceptual?: boolean;
  cacheDir?: string;
  failOnDuplicates?: boolean;
}

export interface ImageEntry {
  path: string;
  size: number;
  exactHash: string;
  perceptualHash?: string;
}

export interface DuplicateGroup {
  hash: string;
  kind: 'exact' | 'perceptual';
  files: string[];
}

export interface CacheEntry {
  path: string;
  modified: number;
  exactHash: string;
  perceptualHash?: string;
}

export interface WorkspaceConfig {
  name: string;
  version: string;
  packages: string[];
}

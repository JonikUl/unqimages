import type { UnqimagesConfig as BaseUnqimagesConfig } from '@unqimages/config';

export type { BaseUnqimagesConfig as UnqimagesConfig };

/**
 * Fully-resolved configuration after defaults have been applied and validation
 * has passed. This is the shape the rest of the CLI wrapper works with.
 *
 * `cacheDir` is intentionally left undefined when the user does not set it,
 * so the Rust binary owns the default cache directory.
 */
export interface ResolvedUnqimagesConfig {
  includeDirs: string[];
  excludeDirs: string[];
  extensions: string[];
  failOnDuplicates: boolean;
  ignoreCache: boolean;
  cacheDir?: string;
  perceptual: { enabled: boolean; threshold: number } | null;
}

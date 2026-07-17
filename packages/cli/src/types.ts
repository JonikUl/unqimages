import type { UnqimagesConfig as BaseUnqimagesConfig } from '@unqimages/config';

export type { BaseUnqimagesConfig as UnqimagesConfig };

/**
 * Fully-resolved configuration after defaults have been applied and validation
 * has passed. This is the shape the rest of the CLI wrapper works with.
 */
export interface ResolvedUnqimagesConfig {
  includeDirs: string[];
  excludeDirs: string[];
  extensions: string[];
  failOnDuplicates: boolean;
  perceptual: { enabled: boolean; threshold: number } | null;
}

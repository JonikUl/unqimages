export interface UnqimagesConfig {
  includeDirs?: string[];
  excludeDirs?: string[];
  extensions?: string[];
  failOnDuplicates?: boolean;
  perceptual?: {
    enabled?: boolean;
    threshold?: number;
  };
}

export interface WorkspaceConfig {
  name: string;
  version: string;
  packages: string[];
}

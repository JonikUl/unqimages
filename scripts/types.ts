/**
 * Type definitions for the npm + Cargo publish configuration.
 */

export interface BinaryConfig {
  /** Binary filename inside the platform package. */
  name: string;
  /** npm scope suffix (e.g. 'cli' → @scope/cli-darwin-arm64). */
  scope: string;
  /** Cargo package name used for building. */
  cargoPackage: string;
}

export interface MainPackage {
  /** Path relative to the repository root (e.g. 'packages/cli'). */
  path: string;
  /** Full npm package name (e.g. '@scope/cli'). */
  name: string;
}

export interface PublishConfig {
  /** npm scope (e.g. '@unqimages'). */
  scope: string;
  /** Rust binaries to distribute as platform packages. */
  binaries: BinaryConfig[];
  /** Target platforms (must match PLATFORM_MAP in the scripts). */
  platforms: string[];
  /** Main wrapper packages that depend on the platform packages. */
  mainPackages: MainPackage[];
  /** Path to the Cargo workspace manifest. */
  cargoWorkspace: string;
  /** Repository URL used in package.json metadata. */
  repositoryUrl: string;
}

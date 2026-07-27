import type { PublishConfig } from './scripts/types.ts';

export default {
  scope: '@unqimages',
  binaries: [
    {
      name: 'unqimages-core',
      scope: 'cli',
      cargoPackage: 'unqimages-core',
    },
  ],
  platforms: ['darwin-arm64', 'darwin-x64', 'linux-x64', 'windows-x64'],
  mainPackages: [{ path: 'packages/cli', name: '@unqimages/cli' }],
  cargoWorkspace: 'Cargo.toml',
  repositoryUrl: 'https://github.com/JonikUl/unqimages',
} satisfies PublishConfig;

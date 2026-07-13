import oxlint from 'eslint-plugin-oxlint';
import tseslint from 'typescript-eslint';
import globals from 'globals';

export default [
  { ignores: ['node_modules/**', 'dist/**', 'target/**', '.agents/**', '.kimi-code/**', '.claude/**'] },
  ...tseslint.configs.recommended,
  {
    files: ['**/*.{ts,mts,cts}'],
    languageOptions: {
      globals: { ...globals.node },
      parserOptions: { project: null },
    },
  },
  ...oxlint.configs['flat/recommended'],
];

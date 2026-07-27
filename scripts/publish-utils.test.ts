import { describe, expect, it } from 'vitest';
import { isAlreadyPublishedError } from './publish-utils.ts';

describe('isAlreadyPublishedError', () => {
  it('returns true for E403 in stderr', () => {
    const err = Object.assign(new Error('Command failed: npm publish'), {
      stderr: Buffer.from('npm ERR! code E403'),
    });
    expect(isAlreadyPublishedError(err)).toBe(true);
  });

  it('returns true for "version already exists" in stderr', () => {
    const err = Object.assign(new Error('Command failed: npm publish'), {
      stderr: Buffer.from('You cannot publish over the previously published version'),
    });
    expect(isAlreadyPublishedError(err)).toBe(true);
  });

  it('returns false for unrelated errors', () => {
    const err = Object.assign(new Error('Command failed: npm publish'), {
      stderr: Buffer.from('npm ERR! network timeout'),
    });
    expect(isAlreadyPublishedError(err)).toBe(false);
  });

  it('returns false for non-Error values', () => {
    expect(isAlreadyPublishedError('oops')).toBe(false);
  });
});

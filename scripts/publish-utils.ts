/**
 * Shared helpers for npm publish scripts.
 */

export function isAlreadyPublishedError(err: unknown): boolean {
  const output = extractErrorOutput(err).toLowerCase();
  return (
    output.includes('e403') ||
    output.includes('cannot publish over the previously published') ||
    output.includes('version already exists') ||
    output.includes('already exists')
  );
}

function extractErrorOutput(err: unknown): string {
  if (err instanceof Error) {
    if ('stderr' in err && Buffer.isBuffer(err.stderr)) {
      return `${err.message}\n${err.stderr.toString()}`;
    }
    return err.message;
  }
  return String(err);
}

/** @internal */
export function randstr(prefix?: string): string {
  return Math.random()
    .toString(36)
    .replace('0.', prefix || '');
}

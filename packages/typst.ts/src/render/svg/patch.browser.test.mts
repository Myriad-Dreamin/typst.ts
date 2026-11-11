import { describe, expect, it } from 'vitest';
import { patchRoot } from './patch.mjs';

const svg = (imageId?: string) => {
  const imageDefs = imageId
    ? `<defs class="image"><image id="${imageId}" href="data:image/webp;base64,${imageId}"/></defs>`
    : '';
  const imageUse = imageId ? `<use class="typst-image" href="#${imageId}"/>` : '';
  return new DOMParser().parseFromString(
    `<svg xmlns="http://www.w3.org/2000/svg"><defs class="glyph"/><defs class="clip-path"/>${imageDefs}<style/><g data-tid="page-${imageId ?? 'empty'}">${imageUse}</g></svg>`,
    'image/svg+xml',
  ).documentElement as unknown as SVGElement;
};

describe('patchRoot', () => {
  it('installs changed image definitions once', () => {
    const current = svg();

    patchRoot(current, svg('image-a'));
    patchRoot(current, svg('image-b'));
    patchRoot(current, svg('image-b'));

    const imageDefs = current.querySelector('defs.image')!;
    expect(Array.from(imageDefs.children, child => child.id)).toEqual(['image-a', 'image-b']);
    expect(current.querySelector('use.typst-image')?.getAttribute('href')).toBe('#image-b');
    expect(current.querySelector('#image-b')).not.toBeNull();
  });
});

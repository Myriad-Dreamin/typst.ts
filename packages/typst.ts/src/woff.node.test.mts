import { readFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import { describe, expect, it } from 'vitest';

import { isWoff, isWoff2, woffToSfnt } from './woff.mjs';
import { sfntChecksum, tagString } from './woff.all.test.mjs';

const here = path.dirname(fileURLToPath(import.meta.url));
const fixtureDir = path.join(here, '..', 'tests', 'fixtures', 'fonts');
const woffBytes = new Uint8Array(
  readFileSync(path.join(fixtureDir, 'LibertinusSerif-Regular-subset.woff')),
);
const woff2Bytes = new Uint8Array(
  readFileSync(path.join(fixtureDir, 'LibertinusSerif-Regular-subset.woff2')),
);
const sfntReference = new Uint8Array(
  readFileSync(
    path.join(here, '..', '..', '..', 'assets', 'data', 'LibertinusSerif-Regular-subset.otf'),
  ),
);

interface SfntDirEntry {
  tag: number;
  checksum: number;
  offset: number;
  length: number;
}

function readSfntDirectory(bytes: Uint8Array): SfntDirEntry[] {
  const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  const numTables = view.getUint16(4);
  const entries: SfntDirEntry[] = [];
  for (let i = 0; i < numTables; i++) {
    const at = 12 + i * 16;
    entries.push({
      tag: view.getUint32(at),
      checksum: view.getUint32(at + 4),
      offset: view.getUint32(at + 8),
      length: view.getUint32(at + 12),
    });
  }
  return entries;
}

function readTable(bytes: Uint8Array, tag: number): Uint8Array {
  const entry = readSfntDirectory(bytes).find(e => e.tag === tag);
  if (!entry) throw new Error(`table ${tagString(tag)} not found`);
  return bytes.slice(entry.offset, entry.offset + entry.length);
}

/** The decoded head table with the fields a WOFF transcoder rewrites zeroed. */
function normalizedHead(head: Uint8Array): Uint8Array {
  const out = head.slice();
  // checkSumAdjustment (8..12) is flavor-dependent; fontTools, which produced
  // the fixture, also refreshes created/modified (12..28) and the glyph
  // bounding box (36..44) when transcoding, so those fields legitimately
  // differ from the older OTF copy while the payload is the same.
  out.fill(0, 8, 12);
  out.fill(0, 28, 44);
  return out;
}

describe('woffToSfnt on a real font fixture', () => {
  it('detects the fixture containers', () => {
    expect(isWoff(woffBytes)).toBe(true);
    expect(isWoff2(woffBytes)).toBe(false);
    expect(isWoff2(woff2Bytes)).toBe(true);
    expect(isWoff(woff2Bytes)).toBe(false);
    // the original SFNT is neither container
    expect(isWoff(sfntReference)).toBe(false);
    expect(isWoff2(sfntReference)).toBe(false);
  });

  it('decodes the real WOFF fixture back into a well-formed SFNT', async () => {
    const decoded = await woffToSfnt(woffBytes);

    // a well-formed header, same flavor and table count as the reference font
    const view = new DataView(decoded.buffer, decoded.byteOffset, decoded.byteLength);
    const refView = new DataView(
      sfntReference.buffer,
      sfntReference.byteOffset,
      sfntReference.byteLength,
    );
    expect(decoded.slice(0, 12)).toEqual(sfntReference.slice(0, 12));

    let totalLength = 12 + view.getUint16(4) * 16;
    let prevTag = -1;
    for (const entry of readSfntDirectory(decoded)) {
      // ascending tags, 4-byte-aligned in-range offsets, tight layout
      expect(entry.tag).toBeGreaterThan(prevTag);
      prevTag = entry.tag;
      expect(entry.offset % 4).toBe(0);
      expect(entry.offset + entry.length).toBeLessThanOrEqual(decoded.length);
      expect(entry.offset).toBe(totalLength);
      totalLength += (entry.length + 3) & ~3;
    }
    expect(decoded.length).toBe(totalLength);

    // every table of the reference font survives the WOFF round trip
    for (const entry of readSfntDirectory(sfntReference)) {
      const reference = sfntReference.slice(entry.offset, entry.offset + entry.length);
      const roundTripped = readTable(decoded, entry.tag);
      if (entry.tag === 0x68656164) {
        expect(normalizedHead(roundTripped)).toEqual(normalizedHead(reference));
      } else {
        expect(roundTripped).toEqual(reference);
      }
    }

    // the WOFF directory's own checksums corroborate the decoded payloads:
    // they were written by an independent implementation (fontTools)
    const woffView = new DataView(woffBytes.buffer, woffBytes.byteOffset, woffBytes.byteLength);
    expect(woffView.getUint32(4)).toBe(refView.getUint32(0)); // flavor carried over
    for (let i = 0; i < woffView.getUint16(12); i++) {
      const at = 44 + i * 20;
      const payload = readTable(decoded, woffView.getUint32(at));
      const checksummed =
        woffView.getUint32(at) === 0x68656164 ? normalizedHeadChecksummed(payload) : payload;
      expect(sfntChecksum(checksummed)).toBe(woffView.getUint32(at + 16));
    }
  });
});

/** Zero only the checkSumAdjustment field, as checksums in font files define it. */
function normalizedHeadChecksummed(head: Uint8Array): Uint8Array {
  const out = head.slice();
  out.fill(0, 8, 12);
  return out;
}

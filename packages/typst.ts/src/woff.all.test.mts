import { describe, expect, it } from 'vitest';
import {
  WOFF2_SIGNATURE,
  WOFF_SIGNATURE,
  isWoff,
  isWoff2,
  woffToSfnt,
} from './woff.mjs';

// ---------------------------------------------------------------------------
// Minimal in-test SFNT / WOFF encoders.
//
// The fixtures here are synthetic on purpose: the encoder is independent from
// the decoder under test, so a byte-for-byte round trip is a meaningful check.
// ---------------------------------------------------------------------------

interface SfntTable {
  tag: string;
  data: Uint8Array;
}

function align4(value: number): number {
  return (value + 3) & ~3;
}

function u32tag(tag: string): number {
  return (
    (tag.charCodeAt(0) << 24) |
    (tag.charCodeAt(1) << 16) |
    (tag.charCodeAt(2) << 8) |
    tag.charCodeAt(3)
  );
}

export function tagString(value: number): string {
  return String.fromCharCode(value >>> 24, (value >>> 16) & 0xff, (value >>> 8) & 0xff, value & 0xff);
}

/** Checksum over the 4-byte-padded table data, as the SFNT spec defines it. */
export function sfntChecksum(data: Uint8Array): number {
  const padded = new Uint8Array(align4(data.length));
  padded.set(data);
  const view = new DataView(padded.buffer);
  let sum = 0;
  for (let at = 0; at < padded.length; at += 4) {
    sum = (sum + view.getUint32(at)) >>> 0;
  }
  return sum;
}

/** Serialize the given tables into a well-formed SFNT (`0x00010000` flavor). */
export function buildSfnt(tables: SfntTable[]): Uint8Array {
  const sorted = [...tables].sort((a, b) => (a.tag < b.tag ? -1 : 1));
  const numTables = sorted.length;
  const headerSize = 12 + numTables * 16;

  let offset = headerSize;
  const entries = sorted.map(table => {
    const entry = {
      table,
      checksum: sfntChecksum(table.data),
      offset,
      length: table.data.length,
    };
    offset += align4(table.data.length);
    return entry;
  });

  const sfnt = new Uint8Array(offset);
  const view = new DataView(sfnt.buffer);
  view.setUint32(0, 0x00010000);
  view.setUint16(4, numTables);
  const entrySelector = Math.floor(Math.log2(numTables));
  const searchRange = 16 * 2 ** entrySelector;
  view.setUint16(6, searchRange);
  view.setUint16(8, entrySelector);
  view.setUint16(10, numTables * 16 - searchRange);

  entries.forEach((entry, index) => {
    const at = 12 + index * 16;
    view.setUint32(at, u32tag(entry.table.tag));
    view.setUint32(at + 4, entry.checksum);
    view.setUint32(at + 8, entry.offset);
    view.setUint32(at + 12, entry.length);
    sfnt.set(entry.table.data, entry.offset);
  });
  return sfnt;
}

async function deflate(data: Uint8Array): Promise<Uint8Array> {
  const stream = new Blob([data.slice()]).stream().pipeThrough(new CompressionStream('deflate'));
  return new Uint8Array(await new Response(stream).arrayBuffer());
}

interface WoffDirEntry {
  tag: number;
  offset: number;
  compLength: number;
  origLength: number;
  origChecksum: number;
}

/**
 * Pack an SFNT into a WOFF (version 1) container. Tables are zlib-compressed
 * unless compression does not pay off, which is exactly the WOFF rule, so a
 * fixture like this exercises both the compressed and the stored code paths.
 */
export async function buildWoff(sfnt: Uint8Array): Promise<Uint8Array> {
  const view = new DataView(sfnt.buffer, sfnt.byteOffset, sfnt.byteLength);
  const numTables = view.getUint16(4);

  const directory: { entry: WoffDirEntry; payload: Uint8Array }[] = [];
  for (let i = 0; i < numTables; i++) {
    const at = 12 + i * 16;
    const data = sfnt.slice(view.getUint32(at + 8), view.getUint32(at + 8) + view.getUint32(at + 12));
    const compressed = await deflate(data);
    const payload = compressed.length < data.length ? compressed : data;
    directory.push({
      entry: {
        tag: view.getUint32(at),
        offset: 0, // patched below
        compLength: payload.length,
        origLength: data.length,
        origChecksum: view.getUint32(at + 4),
      },
      payload,
    });
  }
  directory.sort((a, b) => a.entry.tag - b.entry.tag);

  const headerSize = 44 + numTables * 20;
  let offset = headerSize;
  for (const { entry } of directory) {
    entry.offset = offset;
    offset += align4(entry.compLength);
  }

  const woff = new Uint8Array(offset);
  const out = new DataView(woff.buffer);
  out.setUint32(0, WOFF_SIGNATURE);
  out.setUint32(4, view.getUint32(0)); // flavor
  out.setUint32(8, woff.length);
  out.setUint16(12, numTables);
  out.setUint16(14, 0); // reserved
  out.setUint32(16, sfnt.length);
  out.setUint16(20, 1); // majorVersion
  out.setUint16(22, 0); // minorVersion
  // metaOffset / metaLength / metaOrigLength / privOffset / privLength stay 0
  directory.forEach(({ entry, payload }, index) => {
    const at = 44 + index * 20;
    out.setUint32(at, entry.tag);
    out.setUint32(at + 4, entry.offset);
    out.setUint32(at + 8, entry.compLength);
    out.setUint32(at + 12, entry.origLength);
    out.setUint32(at + 16, entry.origChecksum);
    woff.set(payload, entry.offset);
  });
  return woff;
}

/** A tiny deterministic pseudo-random table that does not compress well. */
function noisyTable(length: number): Uint8Array {
  const data = new Uint8Array(length);
  let state = 0x12345678;
  for (let i = 0; i < length; i++) {
    state = (state * 1103515245 + 12345) >>> 0;
    data[i] = (state >>> 16) & 0xff;
  }
  return data;
}

describe('woffToSfnt', () => {
  it('round-trips a synthetic SFNT through WOFF byte-for-byte', async () => {
    const sfnt = buildSfnt([
      // incompressible: stored uncompressed in the WOFF
      { tag: 'AAAA', data: noisyTable(64) },
      // repetitive: compresses well below the original size
      { tag: 'BBBB', data: new Uint8Array(2048).fill(0x5a) },
      // tiny: no room for zlib overhead, stored uncompressed
      { tag: 'CCCC', data: new Uint8Array([1, 2, 3, 4, 5]) },
    ]);

    const woff = await buildWoff(sfnt);
    expect(isWoff(woff)).toBe(true);

    // prove both storage paths are present in this fixture
    const wv = new DataView(woff.buffer);
    const numTables = wv.getUint16(12);
    let compressed = 0;
    let stored = 0;
    for (let i = 0; i < numTables; i++) {
      const at = 44 + i * 20;
      if (wv.getUint32(at + 8) === wv.getUint32(at + 12)) stored++;
      else compressed++;
    }
    expect(compressed).toBeGreaterThan(0);
    expect(stored).toBeGreaterThan(0);

    const roundTripped = await woffToSfnt(woff);
    expect(roundTripped).toEqual(sfnt);
  });

  it('passes non-WOFF bytes through unchanged', async () => {
    const sfnt = buildSfnt([{ tag: 'AAAA', data: noisyTable(16) }]);
    const garbage = new Uint8Array([0x00, 0x11, 0x22, 0x33, 0xde, 0xad]);
    const empty = new Uint8Array(0);

    expect(await woffToSfnt(sfnt)).toBe(sfnt);
    expect(await woffToSfnt(garbage)).toBe(garbage);
    expect(await woffToSfnt(empty)).toBe(empty);
    expect(isWoff(sfnt)).toBe(false);
    expect(isWoff2(sfnt)).toBe(false);
    expect(isWoff(empty)).toBe(false);
    expect(isWoff2(empty)).toBe(false);
  });

  it('rejects WOFF2 without a decoder and routes to a decoder when given', async () => {
    const woff2Like = new Uint8Array(8);
    new DataView(woff2Like.buffer).setUint32(0, WOFF2_SIGNATURE);
    expect(isWoff2(woff2Like)).toBe(true);
    expect(isWoff(woff2Like)).toBe(false);

    await expect(woffToSfnt(woff2Like)).rejects.toThrow(/decodeWoff2/);

    const decoded = new Uint8Array([0x00, 0x01, 0x00, 0x00]);
    const seen: Uint8Array[] = [];
    const decodeWoff2 = (bytes: Uint8Array) => {
      seen.push(bytes);
      return decoded;
    };
    expect(await woffToSfnt(woff2Like, { decodeWoff2 })).toBe(decoded);
    expect(seen).toEqual([woff2Like]);

    // an async decoder is awaited as well
    expect(await woffToSfnt(woff2Like, { decodeWoff2: async bytes => bytes.slice(0, 4) })).toEqual(
      new Uint8Array([0x77, 0x4f, 0x46, 0x32]),
    );
  });

  it('rejects truncated WOFF payloads instead of reading out of range', async () => {
    const emptyWoff = new Uint8Array(4);
    new DataView(emptyWoff.buffer).setUint32(0, WOFF_SIGNATURE);
    await expect(woffToSfnt(emptyWoff)).rejects.toThrow(/truncated/);

    const sfnt = buildSfnt([{ tag: 'AAAA', data: noisyTable(16) }]);
    const woff = await buildWoff(sfnt);
    await expect(woffToSfnt(woff.slice(0, 30))).rejects.toThrow(/truncated/);
    await expect(woffToSfnt(woff.slice(0, woff.length - 4))).rejects.toThrow();
  });
});

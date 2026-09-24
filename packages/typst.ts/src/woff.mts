/**
 * WOFF helpers.
 *
 * The compiler consumes SFNT font data (`ttf` / `otf`). WOFF and WOFF2 are
 * distribution containers, not a font format the compiler core should learn:
 * decode them here, then hand the resulting bytes to the usual font entries
 * (`addRawFont`, `loadFontSync`, the font loader, the font options, ...).
 *
 * WOFF (version 1) is zlib-compressed tables, so it needs nothing but the
 * platform `DecompressionStream`. WOFF2 is brotli plus a table transform, which
 * the platform does not expose — supply `decodeWoff2` for it.
 */

/** WOFF (version 1) container signature, `'wOFF'`. */
export const WOFF_SIGNATURE = 0x774f4646;
/** WOFF2 container signature, `'wOF2'`. */
export const WOFF2_SIGNATURE = 0x774f4632;

export interface WoffDecodeOptions {
  /**
   * Decoder turning a WOFF2 payload into SFNT bytes. WOFF2 needs brotli and a
   * table transform, so bring a decoder (a wasm build of the woff2 library, a
   * node addon, ...); its result is returned as is.
   */
  decodeWoff2?: (bytes: Uint8Array) => Uint8Array | Promise<Uint8Array>;
}

/** True when the bytes start with the WOFF (version 1) signature. */
export function isWoff(bytes: Uint8Array): boolean {
  return bytes.byteLength >= 4 && readU32(bytes, 0) === WOFF_SIGNATURE;
}

/** True when the bytes start with the WOFF2 signature. */
export function isWoff2(bytes: Uint8Array): boolean {
  return bytes.byteLength >= 4 && readU32(bytes, 0) === WOFF2_SIGNATURE;
}

/**
 * Decode a WOFF or WOFF2 payload into SFNT bytes the compiler accepts.
 *
 * Bytes that are already SFNT, or any other container, are returned unchanged,
 * so this is safe to put in front of a font loading path.
 */
export async function woffToSfnt(
  bytes: Uint8Array,
  options?: WoffDecodeOptions,
): Promise<Uint8Array> {
  if (isWoff2(bytes)) {
    const decodeWoff2 = options?.decodeWoff2;
    if (!decodeWoff2) {
      throw new Error(
        'woffToSfnt: WOFF2 needs a decoder — pass `decodeWoff2`, brotli and the WOFF2 table transform are not built in',
      );
    }
    return await decodeWoff2(bytes);
  }
  if (!isWoff(bytes)) {
    return bytes;
  }
  return await decodeWoff(bytes);
}

interface WoffTable {
  tag: number;
  offset: number;
  compLength: number;
  origLength: number;
  origChecksum: number;
}

async function decodeWoff(bytes: Uint8Array): Promise<Uint8Array> {
  if (bytes.byteLength < 44) {
    throw new Error('woffToSfnt: truncated WOFF header');
  }
  const flavor = readU32(bytes, 4);
  const numTables = readU16(bytes, 12);
  if (bytes.byteLength < 44 + numTables * 20) {
    throw new Error('woffToSfnt: truncated WOFF table directory');
  }

  const directory: { table: WoffTable; bytes: Uint8Array }[] = [];
  for (let i = 0; i < numTables; i++) {
    const at = 44 + i * 20;
    const table: WoffTable = {
      tag: readU32(bytes, at),
      offset: readU32(bytes, at + 4),
      compLength: readU32(bytes, at + 8),
      origLength: readU32(bytes, at + 12),
      origChecksum: readU32(bytes, at + 16),
    };
    if (table.offset + table.compLength > bytes.byteLength) {
      throw new Error('woffToSfnt: table data out of range');
    }
    const payload = bytes.subarray(table.offset, table.offset + table.compLength);
    const content =
      table.compLength === table.origLength ? payload.slice() : await inflateZlib(payload);
    if (content.byteLength !== table.origLength) {
      throw new Error(
        `woffToSfnt: table 0x${table.tag.toString(16)} decoded to ${content.byteLength} bytes, expected ${table.origLength}`,
      );
    }
    directory.push({ table, bytes: content });
  }

  // The WOFF directory is already in ascending tag order, which is the order the
  // SFNT table directory wants, so the entries line up one to one.
  const headerSize = 12 + numTables * 16;
  let totalLength = headerSize;
  for (const entry of directory) {
    totalLength += align4(entry.table.origLength);
  }

  const sfnt = new Uint8Array(totalLength);
  const view = new DataView(sfnt.buffer);
  view.setUint32(0, flavor);
  view.setUint16(4, numTables);
  const entrySelector = numTables > 0 ? Math.floor(Math.log2(numTables)) : 0;
  const searchRange = numTables > 0 ? 16 * 2 ** entrySelector : 0;
  view.setUint16(6, searchRange);
  view.setUint16(8, entrySelector);
  view.setUint16(10, numTables * 16 - searchRange);

  let offset = headerSize;
  let index = 0;
  for (const entry of directory) {
    const at = 12 + index * 16;
    view.setUint32(at, entry.table.tag);
    view.setUint32(at + 4, entry.table.origChecksum);
    view.setUint32(at + 8, offset);
    view.setUint32(at + 12, entry.table.origLength);
    sfnt.set(entry.bytes, offset);
    offset += align4(entry.table.origLength);
    index++;
  }
  return sfnt;
}

async function inflateZlib(payload: Uint8Array): Promise<Uint8Array> {
  const stream = new Blob([payload])
    .stream()
    .pipeThrough(new DecompressionStream('deflate'));
  return new Uint8Array(await new Response(stream).arrayBuffer());
}

function align4(value: number): number {
  return (value + 3) & ~3;
}

function viewOf(bytes: Uint8Array): DataView {
  return new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
}

function readU32(bytes: Uint8Array, at: number): number {
  return viewOf(bytes).getUint32(at);
}

function readU16(bytes: Uint8Array, at: number): number {
  return viewOf(bytes).getUint16(at);
}

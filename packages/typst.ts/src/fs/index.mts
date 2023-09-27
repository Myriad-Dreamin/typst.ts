import { FsAccessModel } from '../internal.types.mjs';

export { FetchAccessModel } from './fetch.mjs';
export type { FetchAccessOptions } from './fetch.mjs';

export { MemoryAccessModel } from './memory.mjs';

export interface WritableAccessModel extends FsAccessModel {
  insertFile(path: string, data: Uint8Array, mtime: Date): void;
  removeFile(path: string): void;
}

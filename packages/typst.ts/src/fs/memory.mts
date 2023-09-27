import { FsAccessModel } from '../internal.types.mjs';
import { WritableAccessModel } from './index.mjs';

export class MemoryAccessModel implements FsAccessModel, WritableAccessModel {
  mTimes: Map<string, Date | undefined> = new Map();
  mData: Map<string, Uint8Array | undefined> = new Map();
  constructor() {}

  reset() {
    this.mTimes.clear();
    this.mData.clear();
  }

  insertFile(path: string, data: Uint8Array, mtime: Date) {
    this.mTimes.set(path, mtime);
    this.mData.set(path, data);
  }

  removeFile(path: string) {
    this.mTimes.delete(path);
    this.mData.delete(path);
  }

  getMTime(path: string): Date | undefined {
    if (!path.startsWith('/@memory/')) {
      return undefined;
    }

    if (this.mTimes.has(path)) {
      return this.mTimes.get(path);
    }
    return undefined;
  }

  isFile(): boolean | undefined {
    return true;
  }

  getRealPath(path: string): string | undefined {
    return path;
  }

  readAll(path: string): Uint8Array | undefined {
    if (!path.startsWith('/@memory/')) {
      return undefined;
    }

    if (this.mData.has(path)) {
      return this.mData.get(path);
    }

    return undefined;
  }
}

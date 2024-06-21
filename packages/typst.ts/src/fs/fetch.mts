import { FsAccessModel } from '../internal.types.mjs';
import { WritableAccessModel } from './index.mjs';

export interface FetchAccessOptions {
  polyfillHeadRequest?: boolean;
  fullyCached?: boolean;
}

export class FetchAccessModel implements FsAccessModel, WritableAccessModel {
  fullyCached: boolean;
  mTimes: Map<string, Date | undefined> = new Map();
  mRealPaths: Map<string, string | undefined> = new Map();
  mData: Map<string, Uint8Array | undefined> = new Map();
  constructor(
    private root: string,
    options?: FetchAccessOptions,
  ) {
    if (root.endsWith('/')) {
      this.root = this.root.slice(0, this.root.length - 1);
    }
    if (options?.polyfillHeadRequest) {
      void 0;
    }
    this.fullyCached = !!options?.fullyCached;
  }

  reset() {
    this.mTimes.clear();
    this.mRealPaths.clear();
    this.mData.clear();
  }

  resolvePath(path: string): string {
    return this.root + path;
  }

  insertFile(path: string, data: Uint8Array, mtime: Date) {
    this.mTimes.set(path, mtime);
    this.mData.set(path, data);
  }

  removeFile(path: string) {
    this.mTimes.delete(path);
    this.mData.delete(path);
  }

  async getPreloadScript(): Promise<string> {
    const snapshot: string[] = [];

    snapshot.push('((async () => {');
    snapshot.push(
      `const snapshot = {  root: '', mTimes: new Map(),  mRealPaths: new Map(),  mData: [],};`,
    );
    // runFetch
    snapshot.push(`const runFetch = async (path) => {`);
    snapshot.push(`  const res = await fetch(snapshot.root + path);`);
    snapshot.push(`  const buffer = await res.arrayBuffer();`);
    snapshot.push(`  return [path, new Uint8Array(buffer)];`);
    snapshot.push(`};`);
    snapshot.push(`snapshot.root = ${JSON.stringify(this.root)};`);
    snapshot.push(
      `snapshot.mTimes = new Map([${[...this.mTimes.entries()]
        .map(([k, v]) => `[${JSON.stringify(k)}, ${v?.getTime() || 'undefined'}]`)
        .join(', ')}]);`,
    );
    snapshot.push(
      `snapshot.mRealPaths = new Map([${[...this.mRealPaths.entries()]
        .map(([k, v]) => `[${JSON.stringify(k)}, ${JSON.stringify(v)}]`)
        .join(', ')}]);`,
    );

    const dataEntries = await Promise.all(
      [...this.mData.entries()].map(async ([k, v]) => {
        k = JSON.stringify(k);
        return v ? `runFetch(${k})` : `Promise.resolve([${k}, undefined])`;
      }),
    );
    snapshot.push(`snapshot.mData = await Promise.all([${dataEntries.join(', ')}]);`);

    snapshot.push(`return snapshot;`);
    snapshot.push('})())');
    return snapshot.join('\n');
  }

  getLastModified(path: string) {
    const request = new XMLHttpRequest();
    request.open('HEAD', path, false);
    request.send(null);
    if (request.status === 200) {
      return request.getResponseHeader('Last-Modified');
    }
    return null;
  }

  getMTimeInternal(path: string): Date | undefined {
    const lastModified = this.getLastModified(this.resolvePath(path));
    if (lastModified) {
      return new Date(lastModified);
    }

    return undefined;
  }

  getMTime(path: string): Date | undefined {
    // todo: no hack
    if (path.startsWith('/@memory/')) {
      if (this.mTimes.has(path)) {
        return this.mTimes.get(path);
      }

      return undefined;
    }

    if (!this.fullyCached) {
      return this.getMTimeInternal(path);
    }

    if (this.mTimes.has(path)) {
      return this.mTimes.get(path);
    }
    const mTime = this.getMTimeInternal(path);
    this.mTimes.set(path, mTime);
    return mTime;
  }

  // todo: isFile
  isFile(): boolean | undefined {
    // path: string
    // const request = this.getLastModified(this.resolvePath(path));
    // if (request.status === 200) {
    //   console.log(request, request.getAllResponseHeaders());
    // }
    return true;
  }

  // todo: getRealPath
  getRealPath(path: string): string | undefined {
    return path;
  }

  readAllInternal(path: string): Uint8Array | undefined {
    const request = new XMLHttpRequest();
    request.overrideMimeType('text/plain; charset=x-user-defined');
    request.open('GET', this.resolvePath(path), false);
    request.send(null);

    if (
      request.status === 200 &&
      (request.response instanceof String || typeof request.response === 'string')
    ) {
      return Uint8Array.from(request.response, (c: string) => c.charCodeAt(0));
    }
    return undefined;
  }

  readAll(path: string): Uint8Array | undefined {
    if (path.startsWith('/@memory/')) {
      if (this.mData.has(path)) {
        return this.mData.get(path);
      }

      return undefined;
    }

    if (!this.fullyCached) {
      return this.readAllInternal(path);
    }

    if (this.mData.has(path)) {
      return this.mData.get(path);
    }

    const data = this.readAllInternal(path);
    this.mData.set(path, data);
    return data;
  }
}

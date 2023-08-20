import { FsAccessModel } from '../internal.types';

export interface FetchAccessOptions {
  polyfillHeadRequest?: boolean;
  fullyCached?: boolean;
}

interface FetchSnapshot {
  root: string;
  mTimes: Map<string, number | undefined>;
  mRealPaths: Map<string, string | undefined>;
  mData: [string, string | Uint8Array][];
}

/// https://stackoverflow.com/questions/21797299/convert-base64-string-to-arraybuffer
const bufferToBase64 = async (data: Uint8Array) => {
  // Use a FileReader to generate a base64 data URI
  const base64url = await new Promise<string | null>((r, reject) => {
    const reader = new FileReader();
    reader.onload = () => {
      const result = reader.result;
      if (typeof result === 'string' || result === null) {
        r(result);
      }
      reject(new Error('Unexpected result type'));
    };
    reader.readAsDataURL(new Blob([data], { type: 'application/octet-binary' }));
  });

  return base64url || '';
};

export class FetchAccessModel implements FsAccessModel {
  fullyCached: boolean;
  mTimes: Map<string, Date | undefined> = new Map();
  mRealPaths: Map<string, string | undefined> = new Map();
  mData: Map<string, Uint8Array | undefined> = new Map();
  constructor(private root: string, options?: FetchAccessOptions) {
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

  async loadSnapshot(snapshot: FetchSnapshot): Promise<void> {
    async function base64UrlToBuffer(base64Url: string) {
      const res = await fetch(base64Url);
      const buffer = await res.arrayBuffer();
      return new Uint8Array(buffer);
    }

    this.root = snapshot.root;
    snapshot.mTimes.forEach((v, k) => this.mTimes.set(k, v ? new Date(v) : undefined));
    this.mRealPaths = new Map(snapshot.mRealPaths);
    await Promise.all(
      snapshot.mData.map(async ([k, v]) => {
        if (typeof v == 'string' && v.startsWith('data:')) {
          this.mData.set(k, await base64UrlToBuffer(v));
        } else if (v instanceof Uint8Array) {
          this.mData.set(k, v);
        } else {
          this.mData.set(k, undefined);
        }
      }),
    );
  }

  async exportSnapshot(): Promise<string> {
    const snapshot: string[] = [];

    snapshot.push('((() => {');
    snapshot.push(
      `const snapshot = {  root: '', mTimes: new Map(),  mRealPaths: new Map(),  mData: [],};`,
    );
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
      [...this.mData.entries()].map(async ([k, v]) =>
        v
          ? `[${JSON.stringify(k)}, "${await bufferToBase64(v)}"]`
          : `[${JSON.stringify(k)}, undefined}]`,
      ),
    );
    snapshot.push(`snapshot.mData = [${dataEntries.join(', ')}];`);

    snapshot.push(`return snapshot;`);
    snapshot.push('})())');
    return snapshot.join('\n');
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

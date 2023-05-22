import { FsAccessModel } from 'src/internal.types';

export interface FetchAccessOptions {
  polyfillHeadRequest?: boolean;
  fullyCached?: boolean;
}

export class FetchAccessModel implements FsAccessModel {
  fullyCached: boolean;
  mTimes: Map<string, Date | undefined> = new Map();
  mRealPaths: Map<string, string | undefined> = new Map();
  mData: Map<string, Uint8Array | undefined> = new Map();
  constructor(private root: string, options?: FetchAccessOptions) {
    if (!root.endsWith('/')) {
      this.root += '/';
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

import { PackageRegistry, PackageResolveContext, PackageSpec } from '../internal.types.mjs';
import { WritableAccessModel } from './index.mjs';

export class FetchPackageRegistry implements PackageRegistry {
  cache: Map<string, () => string | undefined> = new Map();
  constructor(
    /** 
     * @internal 
     * Access model for internal use only
     */
    protected am: WritableAccessModel
  ) {}

  resolvePath(path: PackageSpec): string {
    return `https://packages.typst.org/preview/${path.name}-${path.version}.tar.gz`;
  }

  pullPackageData(path: PackageSpec): Uint8Array | undefined {
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

  resolve(spec: PackageSpec, context: PackageResolveContext): string | undefined {
    if (spec.namespace !== 'preview') {
      return undefined;
    }

    /// Check cache
    const path = this.resolvePath(spec);
    if (this.cache.has(path)) {
      return this.cache.get(path)!();
    }

    /// Fetch data
    const data = this.pullPackageData(spec);
    if (!data) {
      return undefined;
    }

    /// Extract package bundle to the underlying access model `this.am`
    const previewDir = `/@memory/fetch/packages/preview/${spec.namespace}/${spec.name}/${spec.version}`;
    const entries: [string, Uint8Array, Date][] = [];
    context.untar(data, (path: string, data: Uint8Array, mtime: number) => {
      entries.push([previewDir + '/' + path, data, new Date(mtime)]);
    });
    const cacheClosure = () => {
      for (const [path, data, mtime] of entries) {
        this.am.insertFile(path, data, mtime);
      }

      /// Return the resolved directory to the package
      /// It is then used to access the package data by the access model `this.am`
      return previewDir;
    };
    this.cache.set(path, cacheClosure);

    /// Trigger write out
    return cacheClosure();
  }
}

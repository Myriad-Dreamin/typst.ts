import type { PackageSpec } from '../internal.types.mjs';
import { FetchPackageRegistry } from './package.mjs';
import type { WritableAccessModel } from './index.mjs';

// const escapeImport = new Function('m', 'return import(m)');
// const { default: request } = await escapeImport('sync-request-curl');

export class NodeFetchPackageRegistry extends FetchPackageRegistry {
  constructor(
    am: WritableAccessModel,
    private request: any,
  ) {
    super(am);
  }

  pullPackageData(path: PackageSpec): Uint8Array | undefined {
    const response = this.request('GET', this.resolvePath(path));

    if (response.statusCode === 200) {
      return response.getBody(undefined);
    }
    return undefined;
  }
}

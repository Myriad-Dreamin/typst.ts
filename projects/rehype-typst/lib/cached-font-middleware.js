import { preloadFontAssets } from '@myriaddreamin/typst.ts/dist/esm/options.init.mjs';
import * as fs from 'fs';
import * as path from 'path';
import { HttpsProxyAgent } from 'https-proxy-agent';

export async function cachedFontInitOptions() {
  const { existsSync, mkdirSync, readFileSync, writeFileSync } = fs;
  const fetcher = (await import('node-fetch')).default;
  const dataDir =
    process.env.APPDATA ||
    (process.platform == 'darwin'
      ? process.env.HOME + '/Library/Preferences'
      : process.env.HOME + '/.local/share');

  const cacheDir = path.join(dataDir, 'typst/fonts');

  return {
    beforeBuild: [
      preloadFontAssets({
        assets: ['text', 'cjk', 'emoji'],
        fetcher: async (url, init) => {
          const cachePath = path.join(cacheDir, url.toString().replace(/[^a-zA-Z0-9]/g, '_'));
          if (existsSync(cachePath)) {
            const font_res = {
              arrayBuffer: async () => {
                return readFileSync(cachePath).buffer;
              },
            };

            return font_res;
          }

          console.log('loading remote font:', url);
          const proxyOption = process.env.HTTPS_PROXY
            ? { agent: new HttpsProxyAgent(process.env.HTTPS_PROXY) }
            : {};

          const font_res = await fetcher(url, {
            ...proxyOption,
            ...((init) || {}),
          });
          const buffer = await font_res.arrayBuffer();
          mkdirSync(path.dirname(cachePath), { recursive: true });
          writeFileSync(cachePath, Buffer.from(buffer));
          font_res.arrayBuffer = async () => {
            return buffer;
          };
          return font_res;
        },
      }),
    ],
  };
}

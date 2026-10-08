import { describe, expect, it, vi } from 'vitest';
import { CompileFormatEnum, IncrementalServer, createTypstCompiler } from './compiler.mjs';
import { TypstSnippet } from './contrib/snippet.mjs';

describe('compiler snapshot cleanup', () => {
  it.each([false, true])('frees the snapshot when export throws: %s', async fail => {
    const world = {
      free: vi.fn(),
      get_artifact: vi.fn(() => {
        if (fail) throw new Error('export failed');
        return { result: '<!DOCTYPE html><html></html>' };
      }),
    };
    const compiler = new createTypstCompiler._impl();
    compiler.compiler = { snapshot: () => world } as any;
    const result = compiler.compile({ mainFilePath: '/main.typ', format: CompileFormatEnum.html });
    if (fail) await expect(result).rejects.toThrow('export failed');
    else expect((await result).result).toContain('<!DOCTYPE html>');
    expect(world.free).toHaveBeenCalledOnce();
  });

  it.each([false, true])('frees the snapshot when a callback throws: %s', async fail => {
    const world = { free: vi.fn() };
    const compiler = new createTypstCompiler._impl();
    compiler.compiler = { snapshot: () => world } as any;
    const result = compiler.runWithWorld({ mainFilePath: '/main.typ' }, async () => {
      await Promise.resolve();
      expect(world.free).not.toHaveBeenCalled();
      if (fail) throw new Error('callback failed');
      return 'done';
    });
    if (fail) await expect(result).rejects.toThrow('callback failed');
    else expect(await result).toBe('done');
    expect(world.free).toHaveBeenCalledOnce();
  });

  it.each([false, true])(
    'frees the snapshot when incremental compilation throws: %s',
    async fail => {
      const resultBytes = new Uint8Array([1, 2, 3]);
      const server = {} as any;
      const world = {
        free: vi.fn(),
        incr_compile: vi.fn(() => {
          if (fail) throw new Error('incremental failed');
          return { result: resultBytes };
        }),
      };
      const compiler = new createTypstCompiler._impl();
      compiler.compiler = { snapshot: () => world } as any;
      const result = compiler.compile({
        mainFilePath: '/main.typ',
        incrementalServer: new IncrementalServer(server),
      });
      if (fail) await expect(result).rejects.toThrow('incremental failed');
      else expect((await result).result).toBe(resultBytes);
      expect(world.incr_compile).toHaveBeenCalledWith(server, 3);
      expect(world.free).toHaveBeenCalledOnce();
    },
  );

  it('removes inline HTML source when resetting the compiler fails', async () => {
    const compiler = new createTypstCompiler._impl();
    compiler.addSource = vi.fn();
    compiler.reset = vi.fn().mockRejectedValue(new Error('reset failed'));
    compiler.unmapShadow = vi.fn();
    const snippet = new TypstSnippet({ compiler });
    await expect(snippet.html({ mainContent: 'Hello' })).rejects.toThrow('reset failed');
    const path = vi.mocked(compiler.addSource).mock.calls[0][0];
    expect(compiler.unmapShadow).toHaveBeenCalledWith(path);
  });
});

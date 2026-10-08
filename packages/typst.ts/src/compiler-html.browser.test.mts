import { beforeEach, describe, expect, expectTypeOf, it, vi } from 'vitest';
import compilerUrl from '../../compiler/pkg/typst_ts_web_compiler_bg.wasm?url';
import regularFont from '../../../assets/data/LibertinusSerif-Regular-subset.otf?inline';
import { CompileFormatEnum, createTypstCompiler, type TypstCompiler } from './compiler.mjs';
import { TypstSnippet } from './contrib/snippet.mjs';
import { MemoryAccessModel } from './fs/memory.mjs';
import { disableDefaultFontAssets, loadFonts, withAccessModel } from './options.init.mjs';

// Run against each build: the normal bundle, then build:html with VITE_TYPST_HTML=1.
const htmlEnabled = import.meta.env.VITE_TYPST_HTML === '1';
const mathNamespace = 'http://www.w3.org/1998/Math/MathML';

async function createCompiler() {
  const compiler = createTypstCompiler();
  await compiler.init({
    getModule: () => compilerUrl,
    beforeBuild: [
      disableDefaultFontAssets(),
      loadFonts([regularFont]),
      withAccessModel(new MemoryAccessModel()),
    ],
  });
  return compiler;
}

function parseHtml(html: string | undefined) {
  expectTypeOf(html).toEqualTypeOf<string | undefined>();
  expect(html).toMatch(/^<!DOCTYPE html>/);
  return new DOMParser().parseFromString(html!, 'text/html');
}

describe.skipIf(!htmlEnabled)('native HTML export', () => {
  let compiler: TypstCompiler;
  let snippet: TypstSnippet;

  beforeEach(async () => {
    compiler = await createCompiler();
    snippet = new TypstSnippet({ compiler });
  });

  it('exports semantic HTML with the existing font loader and no renderer', async () => {
    const html = await snippet.html({ mainContent: '= Hello\nA *bold* café & tea.' });
    const doc = parseHtml(html);
    expect(doc.querySelector('h2')?.textContent).toBe('Hello');
    expect(doc.querySelector('strong')?.textContent).toBe('bold');
    expect(doc.body.textContent).toContain('café & tea.');
    expect(doc.querySelector('svg, canvas, math')).toBeNull();
  });

  it.each([
    ['fraction', '$ (a + b) / c $', 'mfrac', 2],
    ['square root', '$ sqrt(x) $', 'msqrt', 1],
    ['indexed root', '$ root(3, x) $', 'mroot', 2],
    ['subscript', '$ x_1 $', 'msub', 2],
    ['superscript', '$ x^2 $', 'msup', 2],
    ['subscript and superscript', '$ x_1^2 $', 'msubsup', 3],
    ['matrix', '$ mat(1, 2; 3, 4) $', 'mtable', 2],
  ])('retains native MathML for %s', async (_, mainContent, tag, childCount) => {
    const doc = parseHtml(await snippet.html({ mainContent }));
    const math = doc.querySelector('math')!;
    expect(math).not.toBeNull();
    expect(math.namespaceURI).toBe(mathNamespace);
    expect(math.getAttribute('display')).toBe('block');
    const structure = math.getElementsByTagNameNS(mathNamespace, tag)[0];
    expect(structure).toBeDefined();
    expect(structure.children).toHaveLength(childCount);
    for (const element of [math, ...math.querySelectorAll('*')]) {
      expect(element.namespaceURI).toBe(mathNamespace);
    }
    if (tag === 'mtable') {
      expect(structure.querySelectorAll('mtr')).toHaveLength(2);
      expect(structure.querySelectorAll('mtd')).toHaveLength(4);
    }
    expect(doc.querySelector('svg, canvas, img')).toBeNull();

    // XMLSerializer supplies xmlns for standalone MathML copied out of HTML.
    const xml = new XMLSerializer().serializeToString(math);
    const standalone = new DOMParser().parseFromString(xml, 'application/xml');
    expect(standalone.querySelector('parsererror')).toBeNull();
    expect(standalone.documentElement.namespaceURI).toBe(mathNamespace);
  });

  it('preserves Unicode text and inline equations', async () => {
    const doc = parseHtml(await snippet.html({ mainContent: '你好, café: $α + β ≤ ∞$.' }));
    expect(doc.body.textContent).toContain('你好, café:');
    const math = doc.querySelector('math')!;
    expect(math.textContent).toContain('≤');
    expect(math.textContent).toContain('∞');
    expect(math.getAttribute('display')).not.toBe('block');
    expect([...math.querySelectorAll('mi')].map(node => node.textContent)).toEqual(['𝛼', '𝛽', '∞']);
  });

  it('uses the HTML target and keeps warnings on successful export', async () => {
    compiler.addSource('/main.typ', '#set page(width: 100pt)\n#html.elem("em")[HTML only]');
    const result = await compiler.compile({
      mainFilePath: '/main.typ',
      format: CompileFormatEnum.html,
      diagnostics: 'full',
    });
    expectTypeOf(result.result).toEqualTypeOf<string | undefined>();
    expect(parseHtml(result.result).querySelector('em')?.textContent).toBe('HTML only');
    expect(result.hasError).toBe(false);
    expect(result.diagnostics?.some(d => d.severity === 'warning')).toBe(true);
  });

  it.each(['full', 'unix', 'none'] as const)(
    'reports invalid source with %s diagnostics',
    async diagnostics => {
      compiler.addSource('/bad.typ', '#let x =');
      const pending = compiler.compile({
        mainFilePath: '/bad.typ',
        format: CompileFormatEnum.html,
        diagnostics,
      });
      if (diagnostics === 'none') {
        await expect(pending).rejects.toBeDefined();
      } else {
        const result = await pending;
        expect(result.result).toBeUndefined();
        expect(result.hasError).toBe(true);
        expect(result.diagnostics?.length).toBeGreaterThan(0);
        if (diagnostics === 'unix') {
          expect(result.diagnostics?.join('\n')).toContain('bad.typ:');
        } else {
          expect(result.diagnostics).toContainEqual(
            expect.objectContaining({
              path: '/bad.typ',
              severity: 'error',
              range: expect.any(String),
            }),
          );
        }
      }
    },
  );

  it.each(['full', 'unix', 'none'] as const)(
    'reports serialization errors with %s diagnostics',
    async diagnostics => {
      compiler.addSource('/bad.typ', '#html.elem("br")[invalid child]');
      await compiler.runWithWorld({ mainFilePath: '/bad.typ' }, async world => {
        // This document compiles: the error belongs to the serializer.
        expect((await world.compileHtml()).hasError).toBe(false);
      });
      const pending = compiler.compile({
        mainFilePath: '/bad.typ',
        format: CompileFormatEnum.html,
        diagnostics,
      });
      if (diagnostics === 'none') {
        await expect(pending).rejects.toBeDefined();
      } else {
        const result = await pending;
        expect(result.hasError).toBe(true);
        expect(result.result).toBeUndefined();
        expect(JSON.stringify(result.diagnostics)).toContain(
          'void elements must not have children',
        );
      }
    },
  );

  it('resolves files, imports, roots, inputs, and the shared main path', async () => {
    await snippet.addSource('/project/main.typ', '#include "part.typ"\n#sys.inputs.name');
    await snippet.addSource('/project/part.typ', '= Included');
    const doc = parseHtml(
      await snippet.html({
        mainFilePath: '/project/main.typ',
        root: '/project',
        inputs: { name: 'From inputs' },
      }),
    );
    expect(doc.querySelector('h2')?.textContent).toBe('Included');
    expect(doc.body.textContent).toContain('From inputs');
    snippet.setMainFilePath('/project/part.typ');
    expect(parseHtml(await snippet.html()).querySelector('h2')?.textContent).toBe('Included');
  });

  it('removes temporary source after success and errors', async () => {
    const remove = vi.spyOn(compiler, 'unmapShadow');
    await snippet.html({ mainContent: 'First' });
    await expect(snippet.html({ mainContent: '#let x =' })).rejects.toBeDefined();
    expect(remove).toHaveBeenCalledTimes(2);
    for (const [path] of remove.mock.calls) {
      const result = await compiler.compile({ mainFilePath: path, format: CompileFormatEnum.html });
      expect(result.hasError).toBe(true);
      expect(result.result).toBeUndefined();
    }
    expect(parseHtml(await snippet.html({ mainContent: 'Recovered' })).body.textContent).toContain(
      'Recovered',
    );
  });

  it('switches HTML, PDF, vector, and dummy exports on the same compiler and snapshot', async () => {
    compiler.addSource(
      '/main.typ',
      '#context {\nif target() == "html" [HTML] else [Paged]\n[ #metadata(target()) <export-target> ]\n}',
    );
    const options = { mainFilePath: '/main.typ' };
    const before = await compiler.compile(options);
    expectTypeOf(before.result).toEqualTypeOf<Uint8Array | undefined>();
    expect(before.result?.byteLength).toBeGreaterThan(0);
    for (let i = 0; i < 2; i++) {
      const html = await compiler.compile({ ...options, format: CompileFormatEnum.html });
      expect(parseHtml(html.result).body.textContent).toContain('HTML');
      const pdf = await compiler.compile({ ...options, format: CompileFormatEnum.pdf });
      expect(new TextDecoder().decode(pdf.result?.slice(0, 5))).toBe('%PDF-');
      expect((await compiler.compile(options)).result).toBeInstanceOf(Uint8Array);
    }
    await compiler.runWithWorld(options, async world => {
      expect((await world.compileHtml()).hasError).toBe(false);
      expect(parseHtml((await world.html()).result).body.textContent).toContain('HTML');
      expect(new TextDecoder().decode((await world.pdf()).result?.slice(0, 5))).toBe('%PDF-');
      expect((await world.vector()).result).toBeInstanceOf(Uint8Array);
      expect(await world.query({ selector: '<export-target>', field: 'value' })).toEqual(['paged']);
      expect(parseHtml((await world.html()).result).body.textContent).toContain('HTML');
      expect(await world.query({ selector: '<export-target>', field: 'value' })).toEqual(['paged']);
    });
    expect(
      (await compiler.compile({ ...options, format: CompileFormatEnum._dummy })).result,
    ).toHaveLength(0);
    expect(CompileFormatEnum._dummy).toBe(2);
  });

  it('supports the raw WASM string format and numeric snapshot format', async () => {
    const raw = (compiler as InstanceType<typeof createTypstCompiler._impl>).compiler;
    compiler.addSource('/main.typ', 'Raw HTML');
    const html = raw.compile('/main.typ', undefined, 'html', 0);
    expect(parseHtml(html).body.textContent).toContain('Raw HTML');
    const world = raw.snapshot(undefined, '/main.typ', undefined);
    try {
      expect(world.get_artifact(3, 0)).toBe(html);
    } finally {
      world.free();
    }
  });
});

describe.skipIf(htmlEnabled)('compiler without HTML export', () => {
  it('rejects HTML clearly and still exports PDF and vector', async () => {
    const compiler = await createCompiler();
    const snippet = new TypstSnippet({ compiler });
    const remove = vi.spyOn(compiler, 'unmapShadow');
    await expect(snippet.html({ mainContent: '#let x =' })).rejects.toContain(
      '`html` Cargo feature',
    );
    expect(remove).toHaveBeenCalledOnce();
    compiler.addSource('/main.typ', 'Still works');
    await expect(
      compiler.compile({ mainFilePath: '/main.typ', format: CompileFormatEnum.html }),
    ).rejects.toContain('`html` Cargo feature');
    await compiler.runWithWorld({ mainFilePath: '/main.typ' }, async world => {
      await expect(world.html()).rejects.toContain('`html` Cargo feature');
      // Preserve the pre-existing compilation-only helper in the default build.
      expect((await world.compileHtml()).hasError).toBe(false);
      expect((await world.vector()).result).toBeInstanceOf(Uint8Array);
      expect(new TextDecoder().decode((await world.pdf()).result?.slice(0, 5))).toBe('%PDF-');
    });
  });
});

/**
 * @typedef {import('hast').ElementContent} ElementContent
 * @typedef {import('hast').Root} Root
 * @typedef {import('vfile').VFile} VFile
 */

import { fromHtmlIsomorphic } from 'hast-util-from-html-isomorphic';
import { toText } from 'hast-util-to-text';
import { NodeCompiler } from '@myriaddreamin/typst-ts-node-compiler';
import { SKIP, visitParents } from 'unist-util-visit-parents';
/** @type {Readonly<Options>} */
const emptyOptions = {};
/** @type {ReadonlyArray<unknown>} */
const emptyClasses = [];

/**
 * Render elements with a `language-math` (or `math-display`, `math-inline`)
 * class with KaTeX.
 *
 * @param {Readonly<Options> | null | undefined} [options]
 *   Configuration (optional).
 * @returns
 *   Transform.
 */
export default function rehypeTypst(options) {
  const settings = options || emptyOptions;

  /**
   * Transform.
   *
   * @param {Root} tree
   *   Tree.
   * @param {VFile} file
   *   File.
   * @returns {undefined}
   *   Nothing.
   */
  return async function (tree, file) {
    const matches = [];
    visitParents(tree, 'element', (...args) => {
      matches.push(args);
      return tree;
    });
    const visitor = async function (element, parents) {
      const classes = Array.isArray(element.properties.className)
        ? element.properties.className
        : emptyClasses;
      // This class can be generated from markdown with ` ```math `.
      const languageMath = classes.includes('language-math');
      // This class is used by `remark-math` for flow math (block, `$$\nmath\n$$`).
      const mathDisplay = classes.includes('math-display');
      // This class is used by `remark-math` for text math (inline, `$math$`).
      const mathInline = classes.includes('math-inline');
      let displayMode = mathDisplay;

      // Any class is fine.
      if (!languageMath && !mathDisplay && !mathInline) {
        return;
      }

      let parent = parents[parents.length - 1];
      let scope = element;

      // If this was generated with ` ```math `, replace the `<pre>` and use
      // display.
      if (
        element.tagName === 'code' &&
        languageMath &&
        parent &&
        parent.type === 'element' &&
        parent.tagName === 'pre'
      ) {
        scope = parent;
        parent = parents[parents.length - 2];
        displayMode = true;
      }

      /* c8 ignore next -- verbose to test. */
      if (!parent) return;

      const value = toText(scope, { whitespace: 'pre' });

      /** @type {Array<ElementContent> | string | undefined} */
      let result;

      try {
        result = await renderToSVGString(value, displayMode);
      } catch (error) {
        const cause = /** @type {Error} */ (error);
        file.message('Could not render math with typst', {
          ancestors: [...parents, element],
          cause,
          place: element.position,
          source: 'rehype-typst',
        });

        result = [
          {
            type: 'element',
            tagName: 'span',
            properties: {
              className: ['typst-error'],
              style: 'color:' + (settings.errorColor || '#cc0000'),
              title: String(error),
            },
            children: [{ type: 'text', value }],
          },
        ];
      }

      if ('svg' in result) {
        const root = fromHtmlIsomorphic(result.svg, { fragment: true });
        const defaultEm = 11;
        const height = parseFloat(root.children[0].properties['dataHeight']);
        const width = parseFloat(root.children[0].properties['dataWidth']);
        const shift = height - result.baselinePosition;
        const shiftEm = shift / defaultEm;
        root.children[0].properties.style = `vertical-align: -${shiftEm}em;`;
        root.children[0].properties.height = `${height / defaultEm}em`;
        root.children[0].properties.width = `${width / defaultEm}em`;
        if (!root.children[0].classNames) root.children[0].classNames = [];
        if (displayMode) {
          root.children[0].properties.style += '; display: block; margin: 0 auto;';
          root.children[0].classNames;
        } else {
          root.children[0].classNames.push('typst-inline');
        }
        result = /** @type {Array<ElementContent>} */ (root.children);
      }

      const index = parent.children.indexOf(scope);
      parent.children.splice(index, 1, ...result);
      return SKIP;
    };
    const promises = matches.map(async args => {
      await visitor(...args);
    });
    await Promise.all(promises);
  };
}

/**
 * @type {NodeCompiler}
 */
let compilerIns;

async function renderToSVGString(code, displayMode) {
  const $typst = (compilerIns ||= NodeCompiler.create());
  const res = renderToSVGString_($typst, code, displayMode);
  $typst.evictCache(10);
  return res;
}

/**
 *
 * @param {NodeCompiler} $typst
 * @returns
 */
async function renderToSVGString_($typst, code, displayMode) {
  const inlineMathTemplate = `
#set page(height: auto, width: auto, margin: 0pt)

#let s = state("t", (:))

#let pin(t) = context {
  let width = measure(line(length: here().position().y)).width
  s.update(it => it.insert(t, width) + it)
}

#show math.equation: it => {
  box(it, inset: (top: 0.5em, bottom: 0.5em))
}

$pin("l1")${code}$

#context [
  #metadata(s.final().at("l1")) <label>
]
`;
  const displayMathTemplate = `
#set page(height: auto, width: auto, margin: 0pt)

$ ${code} $
`;
  const mainFileContent = displayMode ? displayMathTemplate : inlineMathTemplate;
  const docRes = $typst.compile({ mainFileContent });
  if (!docRes.result) {
    const diags = $typst.fetchDiagnostics(docRes.takeDiagnostics());
    console.error(diags);
    return {};
  }
  const doc = docRes.result;

  const svg = $typst.svg(doc);
  const res = {
    svg,
  };
  if (!displayMode) {
    const query = $typst.query(doc, { selector: '<label>' });
    // parse baselinePosition from query ignore last 2 chars
    res.baselinePosition = parseFloat(query[0].value.slice(0, -2));
  }

  return res;
}

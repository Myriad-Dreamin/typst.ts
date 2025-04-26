import { TypstParserBuilder } from '@myriaddreamin/typst-ts-parser';
import type { TypstParser } from '@myriaddreamin/typst-ts-parser';
import {
  SemanticTokensProviderStylingConstants,
  toMultilineTokens2,
  TypstSemaTokenHljsStyling,
  TypstSemaTokenStyling,
} from './semantic_tokens.mjs';

let parser: TypstParser;
const styling = new TypstSemaTokenHljsStyling(false);

/**
 * The initilization function that must be called before using the {@link hljsTypst} function.
 */
export async function initHljs() {
  const p = await new TypstParserBuilder().build();
  parser = p;
  TypstSemaTokenStyling._legend = p.get_semantic_token_legend();
  console.log('typst parser module loaded for hljs', parser);
  return parser;
}

/**
 * Options for the `hljsTypst` function.
 * @param handleCodeBlocks - Whether to handle code blocks.
 *   Defaults to true.
 *   If set to false, code blocks will be rendered as plain code blocks.
 *   If set to true, a default handler will be used.
 *   If set to a function, the function will be used as the handler.
 *
 *   When the `hljsTypst` has a code block handler, the code block will be called with the code block content and the emitter.
 *
 *   If the handler return false, the code block will be still rendered as plain code blocks.
 *
 * @param codeBlockDefaultLanguage - The default language for code blocks.
 *   Defaults to undefined.
 */
export interface TypstHljsOptions {
  handleCodeBlocks?: boolean | ((code: string, emitter: any) => /*handled*/ boolean);
  codeBlockDefaultLanguage?: string;
}

/**
 * A function that constructs a language definition for hljs
 * @param options options for the hljsTypst function.
 * @returns a language definition for hljs.
 * See {@link TypstHljsOptions} for more details.
 *
 * @example
 *
 * Default usage:
 * ```ts
 * hljs.registerLanguage('typst', window.hljsTypst());
 * ```
 *
 * @example
 *
 * Don't handle code blocks:
 * ```ts
 * hljs.registerLanguage('typst', window.hljsTypst({
 *  handleCodeBlocks: false,
 * }));
 *
 * @example
 *
 * Handle code blocks with a custom function:
 * ```ts
 * hljs.registerLanguage('typst', window.hljsTypst({
 *   handleCodeBlocks: (code, emitter) => {
 *     return false;
 *   });
 * }));
 * ```
 *
 * @example
 *
 * Set the default language for code blocks:
 * ```ts
 * hljs.registerLanguage('typst', window.hljsTypst({
 *  codeBlockDefaultLanguage: 'rust',
 * }));
 * ```
 */
export function hljsTypst(options?: TypstHljsOptions) {
  return (hljs: any) => {
    const handleSubLanguage = (
      code: string,
      emitter: any,
      langTag: string,
      useDefault: boolean,
    ) => {
      code = code.slice(3);
      if (!useDefault) {
        code = code.slice(langTag.length);
      }
      code = code.slice(0, code.length - 3);
      const result = hljs.highlight(code, {
        language: langTag,
        ignoreIllegals: true,
      });
      if (result.errorRaised) {
        return false;
      }
      emitter.startScope('code');
      emitter.addText('```');
      if (!useDefault) {
        emitter.addText(langTag);
      }
      console.log('handleNestedCodeBlocks', langTag, code, useDefault, result);
      emitter.__addSublanguage(result._emitter, langTag);
      emitter.addText('```');
      emitter.endScope('code');
      return true;
    };

    let defaultHandleCodeBlocks =
      options?.handleCodeBlocks === false
        ? undefined
        : (code: string, emitter: any) => {
            if (!code.startsWith('``') || !code.endsWith('```')) {
              return false;
            }
            const useDefault = options?.codeBlockDefaultLanguage;
            let index = code.indexOf('\n');
            if (index === -1) {
              index = code.indexOf(' ');
            }
            if (index !== -1) {
              const langTag = code.slice(3, index).trim();
              if (!langTag && useDefault) {
                return handleSubLanguage(code, emitter, useDefault, true);
              }
              if (langTag && hljs.getLanguage(langTag)) {
                return handleSubLanguage(code, emitter, langTag, false);
              }
            } else if (useDefault) {
              return handleSubLanguage(code, emitter, useDefault, true);
            }
            return false;
          };
    let handleCodeBlocks =
      typeof options?.handleCodeBlocks === 'function'
        ? options?.handleCodeBlocks
        : defaultHandleCodeBlocks;
    return {
      case_insensitive: false,
      keywords: ['let', 'set', 'show', 'import'],
      contains: [],
      __emitTokens: function (code: string, emitter: any) {
        // todo: '\r'
        const semaTokens = parser.get_semantic_tokens_by_string(code, 'utf-8');
        const styledTokens = toMultilineTokens2({ data: semaTokens }, styling, 'typst');

        const lines = code.split('\n');

        let globalLastLine = 1;
        let globalLastColumn = 0;
        function emitFeat(content: string, feat: number) {
          if (feat === SemanticTokensProviderStylingConstants.NO_STYLING) {
            emitter.addText(content);
          } else {
            const scope = styling.getScope(feat);
            if (scope === 'code' && handleCodeBlocks) {
              if (handleCodeBlocks(content, emitter)) {
                return;
              }
            }
            if (Array.isArray(scope)) {
              for (const s of scope) {
                emitter.startScope(s);
              }
              emitter.addText(content);
              for (const s of scope) {
                emitter.endScope(s);
              }
              return;
            } else {
              emitter.startScope(scope);
              emitter.addText(content);
              emitter.endScope(scope);
            }
          }
        }
        function advanceLine(deltaLine: number, feat: number) {
          for (let i = 0; i < deltaLine; i++) {
            let content = lines[globalLastLine + i - 1];
            if (i === 0) {
              content = content.substring(globalLastColumn);
              globalLastColumn = 0;
            }
            emitFeat(content, feat);
            if (globalLastLine + i !== lines.length) {
              emitFeat('\n', SemanticTokensProviderStylingConstants.NO_STYLING);
            }
          }
          globalLastLine += deltaLine;
        }
        function advanceRange(startCharacter: number, endCharacter: number, feat: number) {
          let line = lines[globalLastLine - 1];
          if (startCharacter !== globalLastColumn) {
            let content = line.substring(globalLastColumn, startCharacter);
            emitFeat(content, SemanticTokensProviderStylingConstants.NO_STYLING);
          }
          let content = line.substring(startCharacter, endCharacter);
          if (endCharacter <= line.length) {
            globalLastColumn = endCharacter;
          } else {
            endCharacter -= line.length;
            while (endCharacter > 0) {
              content += '\n';
              globalLastLine++;
              globalLastColumn = 0;
              endCharacter--;

              if (endCharacter) {
                let newContent = lines[globalLastLine - 1].substring(0, endCharacter);
                content += newContent;
                endCharacter -= newContent.length;
                globalLastColumn = newContent.length;
              }
            }
          }
          emitFeat(content, feat);
        }

        for (const [areaLine, tokens] of styledTokens) {
          advanceLine(areaLine - globalLastLine, 0);
          for (let i = 0; i < tokens.length; i += 4) {
            const deltaLine = tokens[i];
            advanceLine(
              deltaLine + areaLine - globalLastLine,
              SemanticTokensProviderStylingConstants.NO_STYLING,
            );
            advanceRange(tokens[i + 1], tokens[i + 2], tokens[i + 3]);
          }
        }
        advanceLine(lines.length - globalLastLine, 0);
      },
    };
  };
}

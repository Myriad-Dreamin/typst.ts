export * from './snippet.mjs';
export * from '../main.mjs';
import { $typst, TypstSnippet } from './snippet.mjs';

(window as any).$typst = $typst;
(window as any).TypstSnippet = TypstSnippet;

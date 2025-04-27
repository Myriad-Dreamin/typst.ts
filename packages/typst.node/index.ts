/**
 * The well-known PDF standards.
 */
export enum PdfStandard {
  /**
   * PDF 1.7.
   */
  V_1_7 = '1.7',
  /**
   * PDF/A-2b.
   */
  A_2b = 'a-2b',
  /**
   * PDF/A-3b.
   */
  A_3b = 'a-3b',
}

export * from './index-napi.js';

export type ProjectWatchItem =
  | string
  | {
      main: string;
      workspace?: string;
    };
export type ProjectWatchItems = ProjectWatchItem | ProjectWatchItem[];

/**
 * The well-known PDF standards.
 */
export enum PdfStandard {
  /**
   * PDF 1.4.
   */
  V_1_4 = '1.4',
  /**
   * PDF 1.5.
   */
  V_1_5 = '1.5',
  /**
   * PDF 1.6.
   */
  V_1_6 = '1.6',
  /**
   * PDF 1.7.
   */
  V_1_7 = '1.7',
  /**
   * PDF 2.0.
   */
  V_2_0 = '2.0',
  /**
   * PDF/A-1b.
   */
  A_1b = 'a-1b',
  /**
   * PDF/A-1a.
   */
  A_1a = 'a-1a',
  /**
   * PDF/A-2b.
   */
  A_2b = 'a-2b',
  /**
   * PDF/A-2u.
   */
  A_2u = 'a-2u',
  /**
   * PDF/A-2a.
   */
  A_2a = 'a-2a',
  /**
   * PDF/A-3b.
   */
  A_3b = 'a-3b',
  /**
   * PDF/A-3u.
   */
  A_3u = 'a-3u',
  /**
   * PDF/A-3a.
   */
  A_3a = 'a-3a',
  /**
   * PDF/A-4.
   */
  A_4 = 'a-4',
  /**
   * PDF/A-4f.
   */
  A_4f = 'a-4f',
  /**
   * PDF/A-4e.
   */
  A_4e = 'a-4e',
  /**
   * PDF/UA-1.
   */
  UA_1 = 'ua-1',

}

export * from './index-napi.js';

export type ProjectWatchItem =
  | string
  | {
    main: string;
    workspace?: string;
  };
export type ProjectWatchItems = ProjectWatchItem | ProjectWatchItem[];

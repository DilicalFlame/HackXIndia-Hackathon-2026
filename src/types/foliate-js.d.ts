declare module 'foliate-js/epubcfi.js';
declare module 'foliate-js/overlayer.js' {
  export class Overlayer {
    static highlight: unknown;
    static underline: unknown;
    static squiggly: unknown;
    static strikethrough: unknown;
    static outline: unknown;
    static bubble: unknown;
    [key: string]: unknown;
    add(key: string, range: Range, style: unknown, options?: unknown): void;
    remove(key: string): void;
  }
}
declare module 'foliate-js/view.js';

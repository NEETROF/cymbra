/** Declared for the tests: `tool/` is plain ESM, outside the TypeScript project. */
export interface Rect {
  x: number;
  y: number;
  width: number;
  height: number;
}
export declare function pngHeader(bytes: Uint8Array): { width: number; height: number; colourType: number };
export declare function assertStorePng(name: string, bytes: Uint8Array, size: { width: number; height: number }): void;
export declare function cropAround(options: {
  box: Rect;
  aspect: number;
  padding?: number;
  minWidth?: number;
  snapTop?: number;
  bounds: { width: number; height: number };
}): Rect;
export declare function columnCrop(options: {
  column: Rect;
  focus: Rect;
  width: number;
  aspect: number;
  padding?: number;
  bounds: { width: number; height: number };
}): Rect;
export declare function padded(
  box: Rect,
  padding: number,
  bounds: { width: number; height: number },
  options?: { bottom?: number },
): Rect;
export declare function union(...boxes: Rect[]): Rect;
export declare function scaled(rect: Rect, factor: number): Rect;
export declare function imageArgs(options: {
  input: string;
  output: string;
  format: "webp" | "png" | "jpeg";
  crop?: Rect;
  width?: number;
  height?: number;
  quality?: number;
}): string[];
export declare function videoArgs(options: {
  input: string;
  output: string;
  start?: number;
  duration?: number;
  width?: number;
  height?: number;
  crf?: number;
}): string[];
export declare function hasFfmpeg(): boolean;
export declare function ffmpeg(args: string[]): void;

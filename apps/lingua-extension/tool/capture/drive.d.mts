/** Declared for the tests: `tool/` is plain ESM, outside the TypeScript project. */
export interface CdpNode {
  nodeId?: number;
  nodeType: number;
  localName?: string;
  nodeValue?: string;
  attributes?: string[];
  children?: CdpNode[];
  shadowRoots?: CdpNode[];
  contentDocument?: CdpNode;
}
export declare const HOSTS: { card: string; pill: string; panel: string };
export declare function nodeText(node: CdpNode, skip?: string): string;
export declare function attributesOf(node: CdpNode): Record<string, string>;
export declare function walk(node: CdpNode): Generator<CdpNode>;
export declare function findIn(
  documentNode: CdpNode,
  host: string,
  matches: { tag?: string; text?: string; className?: string; attribute?: [string, string] },
): CdpNode | null;
export declare function quadCentre(quad: number[]): { x: number; y: number };
export declare function waitFor<T>(probe: () => T | Promise<T>, timeout?: number, every?: number): Promise<T>;

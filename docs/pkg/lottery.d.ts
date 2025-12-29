/* tslint:disable */
/* eslint-disable */

export class LotteryEngine {
  free(): void;
  [Symbol.dispose](): void;
  constructor(json_data: string);
  /**
   * 5개 세트의 로또 번호 생성
   */
  generateNumbersSets(): any;
  /**
   * 특정 번호를 포함한 5개 세트 생성
   */
  generateNumbersSetsWithRequired(required: Uint8Array): any;
  /**
   * 빈도 기반 번호 추천 (낮은 빈도순)
   */
  getNumberFrequency(): any;
  /**
   * 새 회차 추가
   */
  addNewDrawing(round: number, numbers: Uint8Array, bonus: number): void;
  /**
   * 현재 저장된 회차 범위 조회
   */
  getRoundRange(): any;
  /**
   * 현재 데이터를 JSON으로 내보내기
   */
  exportToJson(): string;
}

export function main(): void;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_lotteryengine_free: (a: number, b: number) => void;
  readonly lotteryengine_new: (a: number, b: number) => [number, number, number];
  readonly lotteryengine_generateNumbersSets: (a: number) => any;
  readonly lotteryengine_generateNumbersSetsWithRequired: (a: number, b: number, c: number) => [number, number, number];
  readonly lotteryengine_getNumberFrequency: (a: number) => any;
  readonly lotteryengine_addNewDrawing: (a: number, b: number, c: number, d: number, e: number) => [number, number];
  readonly lotteryengine_getRoundRange: (a: number) => any;
  readonly lotteryengine_exportToJson: (a: number) => [number, number];
  readonly main: () => void;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_exn_store: (a: number) => void;
  readonly __externref_table_alloc: () => number;
  readonly __wbindgen_externrefs: WebAssembly.Table;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly __externref_table_dealloc: (a: number) => void;
  readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
*
* @returns {InitOutput}
*/
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
*
* @returns {Promise<InitOutput>}
*/
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;

/* tslint:disable */
/* eslint-disable */
/**
*/
export function run(): void;
/**
*/
export function style_otl31hq3_dep(): void;
/**
*/
export function style_9kadaf7q_dep(): void;
/**
*/
export function style_nt0o8v7s_dep(): void;
/**
*/
export function style_dv5i2trl_dep(): void;
/**
*/
export function style_odi9mfip_dep(): void;
/**
*/
export function style_m9f0qa6h_dep(): void;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly run: () => void;
  readonly style_otl31hq3_dep: () => void;
  readonly style_9kadaf7q_dep: () => void;
  readonly style_nt0o8v7s_dep: () => void;
  readonly style_dv5i2trl_dep: () => void;
  readonly style_odi9mfip_dep: () => void;
  readonly style_m9f0qa6h_dep: () => void;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_export_2: WebAssembly.Table;
  readonly _dyn_core__ops__function__Fn__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h6e6e08e318a33c41: (a: number, b: number, c: number) => void;
  readonly _dyn_core__ops__function__FnMut_____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h6dd36d1bf6efeb78: (a: number, b: number) => void;
  readonly _dyn_core__ops__function__Fn_____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h065b0fca1eda8af0: (a: number, b: number) => void;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly __wbindgen_exn_store: (a: number) => void;
  readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;
/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {SyncInitInput} module
*
* @returns {InitOutput}
*/
export function initSync(module: SyncInitInput): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {InitInput | Promise<InitInput>} module_or_path
*
* @returns {Promise<InitOutput>}
*/
export default function __wbg_init (module_or_path?: InitInput | Promise<InitInput>): Promise<InitOutput>;

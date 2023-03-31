/* tslint:disable */
/* eslint-disable */
/**
*/
export enum FinishType {
  Draw1 = 0,
  Draw2 = 1,
  Draw3 = 2,
  Draw4 = 3,
  Draw5 = 4,
  BlackWin = 5,
  WhiteWin = 6,
}
/**
*/
export enum Color {
  Black = 0,
  White = 1,
}
export type BoardPos = number;

export interface QuietMove {
    from: BoardPos;
    to: BoardPos;
    king_move: boolean;
}

/**
*/
export class BestPos {
  free(): void;
}
/**
*/
export class Game {
  free(): void;
/**
* @param {number} size
*/
  constructor(size: number);
/**
* @param {number} depth
*/
  set_depth(depth: number): void;
/**
* @param {number} mcts_lim
*/
  set_mcts_lim(mcts_lim: number): void;
/**
* @param {Piece} piece
*/
  insert_piece(piece: Piece): void;
/**
* @param {number} pos
* @returns {boolean}
*/
  remove_piece(pos: number): boolean;
/**
* @param {BestPos} pos
*/
  make_move_by_pos_item(pos: BestPos): void;
/**
* @param {MoveItem} move_item
*/
  make_move_by_move_item(move_item: MoveItem): void;
/**
* @param {number} max_depth
* @param {number} best_white
* @param {number} best_black
* @param {number} depth
* @returns {BestPos}
*/
  best_move(max_depth: number, best_white: number, best_black: number, depth: number): BestPos;
/**
* @param {boolean} apply
* @returns {any}
*/
  get_or_apply_best_move(apply: boolean): any;
/**
* @param {BestPos} pos
*/
  make_best_move(pos: BestPos): void;
/**
* @returns {any}
*/
  find_and_make_best_move_ts_n(): any;
/**
* @param {boolean} apply
* @returns {any}
*/
  find_mcts_and_make_best_move_ts_n(apply: boolean): any;
/**
* @returns {any}
*/
  get_board_list_ts_n(): any;
/**
* @param {number} i
* @returns {any}
*/
  move_by_tree_index_ts_n(i: number): any;
/**
* @param {number} i
* @returns {any}
*/
  move_by_index_ts_n(i: number): any;
/**
* @returns {BestPos}
*/
  get_best_move_rust(): BestPos;
/**
* @returns {string}
*/
  state_(): string;
/**
* @param {number} pack_index
* @returns {number}
*/
  to_board(pack_index: number): number;
/**
* @param {number} board_index
* @returns {number}
*/
  to_pack(board_index: number): number;
/**
* @returns {any}
*/
  get_move_list_for_front(): any;
/**
* @param {any} pos_chain
* @returns {any}
*/
  make_move_for_front(pos_chain: any): any;
/**
*/
  moveColor: number;
/**
*/
  readonly position: any;
}
/**
*/
export class MoveItem {
  free(): void;
}
/**
*/
export class MoveList {
  free(): void;
}
/**
*/
export class Piece {
  free(): void;
/**
* @param {number} pos
* @param {number} color
* @param {boolean} is_king
* @returns {Piece}
*/
  static new(pos: number, color: number, is_king: boolean): Piece;
/**
* @param {any} js
* @returns {Piece | undefined}
*/
  static new_fom_js(js: any): Piece | undefined;
/**
*/
  color: number;
/**
*/
  is_king: boolean;
/**
*/
  it: any;
/**
*/
  pos: number;
/**
*/
  stricken: boolean;
}
/**
*/
export class PositionAndMove {
  free(): void;
}
/**
*/
export class PositionEnvironment {
  free(): void;
/**
* @param {number} size
*/
  constructor(size: number);
/**
* @returns {any}
*/
  js(): any;
/**
* @param {Piece} piece
* @param {number} pos
* @returns {boolean}
*/
  is_king_move_for(piece: Piece, pos: number): boolean;
/**
*/
  static game(): void;
/**
* @returns {any}
*/
  static test(): any;
/**
*/
  size: number;
}
/**
*/
export class StraightStrike {
  free(): void;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_moveitem_free: (a: number) => void;
  readonly __wbg_movelist_free: (a: number) => void;
  readonly __wbg_straightstrike_free: (a: number) => void;
  readonly __wbg_bestpos_free: (a: number) => void;
  readonly __wbg_game_free: (a: number) => void;
  readonly game_new: (a: number) => number;
  readonly game_set_depth: (a: number, b: number) => void;
  readonly game_set_mcts_lim: (a: number, b: number) => void;
  readonly game_insert_piece: (a: number, b: number) => void;
  readonly game_remove_piece: (a: number, b: number) => number;
  readonly game_position: (a: number) => number;
  readonly game_make_move_by_move_item: (a: number, b: number) => void;
  readonly game_best_move: (a: number, b: number, c: number, d: number, e: number) => number;
  readonly game_get_or_apply_best_move: (a: number, b: number) => number;
  readonly game_make_best_move: (a: number, b: number) => void;
  readonly game_find_and_make_best_move_ts_n: (a: number) => number;
  readonly game_find_mcts_and_make_best_move_ts_n: (a: number, b: number) => number;
  readonly game_get_board_list_ts_n: (a: number) => number;
  readonly game_move_by_tree_index_ts_n: (a: number, b: number) => number;
  readonly game_move_by_index_ts_n: (a: number, b: number) => number;
  readonly game_get_best_move_rust: (a: number) => number;
  readonly game_state_: (a: number, b: number) => void;
  readonly game_to_board: (a: number, b: number) => number;
  readonly game_to_pack: (a: number, b: number) => number;
  readonly game_get_move_list_for_front: (a: number) => number;
  readonly game_get_color: (a: number) => number;
  readonly game_set_color: (a: number, b: number) => void;
  readonly game_make_move_for_front: (a: number, b: number, c: number) => void;
  readonly game_make_move_by_pos_item: (a: number, b: number) => void;
  readonly __wbg_positionenvironment_free: (a: number) => void;
  readonly __wbg_get_positionenvironment_size: (a: number) => number;
  readonly __wbg_set_positionenvironment_size: (a: number, b: number) => void;
  readonly positionenvironment_new: (a: number) => number;
  readonly positionenvironment_js: (a: number) => number;
  readonly positionenvironment_is_king_move_for: (a: number, b: number, c: number) => number;
  readonly positionenvironment_game: () => void;
  readonly positionenvironment_test: () => number;
  readonly piece_new: (a: number, b: number, c: number) => number;
  readonly piece_new_fom_js: (a: number) => number;
  readonly piece_it: (a: number) => number;
  readonly piece_set_it: (a: number, b: number) => void;
  readonly __wbg_piece_free: (a: number) => void;
  readonly __wbg_get_piece_pos: (a: number) => number;
  readonly __wbg_set_piece_pos: (a: number, b: number) => void;
  readonly __wbg_get_piece_color: (a: number) => number;
  readonly __wbg_set_piece_color: (a: number, b: number) => void;
  readonly __wbg_get_piece_is_king: (a: number) => number;
  readonly __wbg_set_piece_is_king: (a: number, b: number) => void;
  readonly __wbg_get_piece_stricken: (a: number) => number;
  readonly __wbg_set_piece_stricken: (a: number, b: number) => void;
  readonly __wbg_positionandmove_free: (a: number) => void;
  readonly __wbindgen_malloc: (a: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number) => number;
  readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
  readonly __wbindgen_free: (a: number, b: number) => void;
  readonly __wbindgen_exn_store: (a: number) => void;
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
export default function init (module_or_path?: InitInput | Promise<InitInput>): Promise<InitOutput>;

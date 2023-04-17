/* tslint:disable */
/* eslint-disable */
/**
*/
export enum Method {
  Deep = 0,
  MCTS = 1,
  Mix = 2,
}
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
    kingMove: boolean;
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
* @param {number} method
*/
  set_method(method: number): void;
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
* @returns {number | undefined}
*/
  make_move_by_move_item(move_item: MoveItem): number | undefined;
/**
* @param {number} max_depth
* @param {number} best_white
* @param {number} best_black
* @param {number} depth
* @param {boolean} state_only
* @returns {BestPos}
*/
  best_move(max_depth: number, best_white: number, best_black: number, depth: number, state_only: boolean): BestPos;
/**
* @param {boolean} apply
* @returns {BestPos}
*/
  mix_method(apply: boolean): BestPos;
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
* @returns {MCTSRes | undefined}
*/
  check_tree_for_finish(): MCTSRes | undefined;
/**
*/
  preparing_tree(): void;
/**
* @param {boolean} apply
* @returns {MCTSRes}
*/
  find_mcts_and_make_best_move(apply: boolean): MCTSRes;
/**
* @returns {any}
*/
  get_board_list_ts_n(): any;
/**
*/
  mov_back(): void;
/**
* @param {number} i
* @returns {any}
*/
  move_by_tree_index_ts_n(i: number): any;
/**
* @param {number} i
* @returns {number | undefined}
*/
  move_by_tree_index(i: number): number | undefined;
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
export class MCTSRes {
  free(): void;
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

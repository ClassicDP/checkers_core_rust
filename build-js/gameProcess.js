"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || function (mod) {
    if (mod && mod.__esModule) return mod;
    var result = {};
    if (mod != null) for (var k in mod) if (k !== "default" && Object.prototype.hasOwnProperty.call(mod, k)) __createBinding(result, mod, k);
    __setModuleDefault(result, mod);
    return result;
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.GameProcess = void 0;
const wasm = __importStar(require("../build-wasm/checkers_core"));
const checkers_core_1 = require("../build-wasm/checkers_core");
class GameProcess {
    static color(color) {
        if (!color)
            return undefined;
        return color == "White" ? checkers_core_1.Color.White : checkers_core_1.Color.Black;
    }
    constructor(size, color) {
        this.strikeChainInd = 0;
        this.moveChainPack = [];
        this.game = new wasm.Game(size);
        if (color !== undefined)
            this.moveColor = color;
    }
    isQuiteMoveList() {
        return this.moveList?.list.length && this.moveList.list[0].mov;
    }
    get moveColor() {
        return this.game.moveColor;
    }
    set moveColor(color) {
        this.game.moveColor = color;
    }
    invertMoveColor() {
        this.moveColor = this.moveColor === checkers_core_1.Color.Black ? checkers_core_1.Color.White : checkers_core_1.Color.Black;
    }
    insertPiece(pos, color, isKing) {
        this.game.insert_piece(wasm.Piece.new(this.game.to_pack(pos), color, isKing));
    }
    removePiece(pos) {
        return this.game.remove_piece(this.game.to_pack(pos));
    }
    get_best_move() {
        return this.game.get_best_move_rust();
    }
    make_best_move(pos) {
        this.game.make_best_move(pos);
    }
    getOrApplyBestMove(apply) {
        let best = this.game.get_or_apply_best_move(apply);
        if (best.pos?.mov) {
            let x = best.pos.mov;
            if (x.mov)
                x.mov = {
                    from: this.game.to_board(x.mov.from),
                    to: this.game.to_board(x.mov.to),
                    king_move: x.mov.king_move
                };
        }
        if (best.pos?.mov?.strike) {
            best.pos.mov.strike = {
                vec: best.pos.mov.strike.vec.map(it => ({
                    king_move: it.king_move,
                    from: this.game.to_board(it.from),
                    to: this.game.to_board(it.to),
                    take: this.game.to_board(it.take),
                    v: it.v
                })),
                king_move: best.pos.mov.strike.king_move
            };
        }
        return best;
    }
    get position() {
        let pos = this.game.position;
        let newPos = { cells: [], state: pos.state, next_move: pos.next_move, move_list: null };
        for (let piece of pos.cells) {
            if (piece)
                newPos.cells[this.game.to_board(piece.pos)] = {
                    pos: this.game.to_board(piece.pos),
                    color: piece.color,
                    is_king: piece.is_king,
                    stricken: piece.stricken
                };
        }
        return newPos;
    }
    frontClick(pos) {
        let getMoveChainElements = (moveList, i) => {
            if (moveList?.list.length) {
                let moveKey = moveList.list[0].strike ? 'strike' : 'mov';
                let res = [];
                for (let move of moveList.list) {
                    if (moveKey == 'strike') {
                        let candidate = move[moveKey].vec[i];
                        if (candidate)
                            res.push({
                                from: this.game.to_board(candidate.from),
                                to: this.game.to_board(candidate.to),
                                kingMove: candidate.king_move,
                                take: this.game.to_board(candidate.take)
                            });
                    }
                    else {
                        res.push({
                            from: this.game.to_board(move[moveKey].from), to: this.game.to_board(move[moveKey].to),
                            kingMove: move[moveKey].king_move
                        });
                    }
                }
                return res;
            }
            return [];
        };
        if (this.isQuiteMoveList()) {
            if (!this.moveList.list.filter(x => x.mov?.to == this.game.to_pack(pos)).length) {
                this.moveList = undefined;
            }
        }
        if (!this.moveList) {
            let color = this.game.position.cells[this.game.to_pack(pos)]?.color;
            if (color == undefined ||
                this.moveColor !== color)
                return { confirmed: undefined };
            this.moveList = this.game.get_move_list_for_front();
            if (this.isQuiteMoveList()) {
                this.moveList.list = this.moveList.list.filter(x => x.mov?.from == this.game.to_pack(pos));
            }
            else {
                if (this.strikeChainInd == 0) {
                    this.moveList.list = this.moveList.list.filter(x => x.strike?.vec[0].from == this.game.to_pack(pos));
                }
            }
        }
        let moveItems = getMoveChainElements(this.moveList, this.strikeChainInd);
        if (!moveItems.length) {
            if (this.strikeChainInd) {
                this.strikeChainInd = 0;
                return { done: true, confirmed: undefined };
            }
            return { confirmed: undefined };
        }
        let moveItems_ = moveItems.filter(x => x.to == pos);
        if (moveItems_.length) {
            let isStrike = moveItems_[0].take !== undefined;
            if (isStrike) {
                this.moveList.list = this.moveList.list.filter(x => x.strike.vec[this.strikeChainInd]?.to == this.game.to_pack(pos));
                let confirmed = this.moveList.list[0].strike.vec[this.strikeChainInd++];
                confirmed = {
                    from: this.game.to_board(confirmed.from),
                    to: this.game.to_board(confirmed.to),
                    take: this.game.to_board(confirmed.take),
                    king_move: confirmed.king_move,
                    v: confirmed.v
                };
                let done = this.moveList.list.length == 1 &&
                    this.moveList.list[0].strike.vec.length == this.strikeChainInd;
                if (done) {
                    this.moveList = undefined;
                    this.strikeChainInd = 0;
                }
                return {
                    done: done,
                    list: done ? undefined : getMoveChainElements(this.moveList, this.strikeChainInd),
                    confirmed
                };
            }
            else {
                let confirmed = moveItems_[0];
                this.moveList = undefined;
                return { done: true, list: undefined, confirmed };
            }
        }
        // if user solve to change move piece
        if (!this.strikeChainInd) {
            let moveItems_ = moveItems.filter(x => x.from == pos);
            if (moveItems_.length)
                return { list: moveItems_, confirmed: undefined };
        }
        return { confirmed: undefined };
    }
    getMoveList(color) {
        if (color !== undefined)
            this.game.moveColor = color;
        let list = this.game.get_move_list_for_front();
        list.list.map(x => {
            if (x.mov)
                x.mov = {
                    from: this.game.to_board(x.mov.from),
                    to: this.game.to_board(x.mov.to),
                    king_move: x.mov.king_move
                };
            if (x.strike)
                x.strike.vec = x.strike.vec.map(x => ({
                    king_move: x.king_move,
                    from: this.game.to_board(x.from),
                    to: this.game.to_board(x.to),
                    take: this.game.to_board(x.take)
                }));
        });
        return list;
    }
    applyFrontClick(pos) {
        let variants = this.frontClick(pos);
        if (variants.confirmed) {
            if (!this.moveChainPack.length) {
                this.moveChainPack.push(variants.confirmed.from, variants.confirmed.to);
            }
            else {
                this.moveChainPack.push(variants.confirmed.to);
            }
        }
        if (variants.done) {
            this.game.make_move_for_front(this.moveChainPack.map(x => this.game.to_pack(x)));
            this.moveChainPack = [];
        }
        return variants;
    }
}
exports.GameProcess = GameProcess;
//# sourceMappingURL=gameProcess.js.map
let wasm;

const heap = new Array(128).fill(undefined);

heap.push(undefined, null, true, false);

function getObject(idx) { return heap[idx]; }

let heap_next = heap.length;

function dropObject(idx) {
    if (idx < 132) return;
    heap[idx] = heap_next;
    heap_next = idx;
}

function takeObject(idx) {
    const ret = getObject(idx);
    dropObject(idx);
    return ret;
}

const cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });

cachedTextDecoder.decode();

let cachedUint8Memory0 = null;

function getUint8Memory0() {
    if (cachedUint8Memory0 === null || cachedUint8Memory0.byteLength === 0) {
        cachedUint8Memory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8Memory0;
}

function getStringFromWasm0(ptr, len) {
    return cachedTextDecoder.decode(getUint8Memory0().subarray(ptr, ptr + len));
}

function addHeapObject(obj) {
    if (heap_next === heap.length) heap.push(heap.length + 1);
    const idx = heap_next;
    heap_next = heap[idx];

    heap[idx] = obj;
    return idx;
}

function isLikeNone(x) {
    return x === undefined || x === null;
}

let cachedFloat64Memory0 = null;

function getFloat64Memory0() {
    if (cachedFloat64Memory0 === null || cachedFloat64Memory0.byteLength === 0) {
        cachedFloat64Memory0 = new Float64Array(wasm.memory.buffer);
    }
    return cachedFloat64Memory0;
}

let cachedInt32Memory0 = null;

function getInt32Memory0() {
    if (cachedInt32Memory0 === null || cachedInt32Memory0.byteLength === 0) {
        cachedInt32Memory0 = new Int32Array(wasm.memory.buffer);
    }
    return cachedInt32Memory0;
}

let WASM_VECTOR_LEN = 0;

const cachedTextEncoder = new TextEncoder('utf-8');

const encodeString = (typeof cachedTextEncoder.encodeInto === 'function'
    ? function (arg, view) {
    return cachedTextEncoder.encodeInto(arg, view);
}
    : function (arg, view) {
    const buf = cachedTextEncoder.encode(arg);
    view.set(buf);
    return {
        read: arg.length,
        written: buf.length
    };
});

function passStringToWasm0(arg, malloc, realloc) {

    if (realloc === undefined) {
        const buf = cachedTextEncoder.encode(arg);
        const ptr = malloc(buf.length);
        getUint8Memory0().subarray(ptr, ptr + buf.length).set(buf);
        WASM_VECTOR_LEN = buf.length;
        return ptr;
    }

    let len = arg.length;
    let ptr = malloc(len);

    const mem = getUint8Memory0();

    let offset = 0;

    for (; offset < len; offset++) {
        const code = arg.charCodeAt(offset);
        if (code > 0x7F) break;
        mem[ptr + offset] = code;
    }

    if (offset !== len) {
        if (offset !== 0) {
            arg = arg.slice(offset);
        }
        ptr = realloc(ptr, len, len = offset + arg.length * 3);
        const view = getUint8Memory0().subarray(ptr + offset, ptr + len);
        const ret = encodeString(arg, view);

        offset += ret.written;
    }

    WASM_VECTOR_LEN = offset;
    return ptr;
}

let cachedBigInt64Memory0 = null;

function getBigInt64Memory0() {
    if (cachedBigInt64Memory0 === null || cachedBigInt64Memory0.byteLength === 0) {
        cachedBigInt64Memory0 = new BigInt64Array(wasm.memory.buffer);
    }
    return cachedBigInt64Memory0;
}

function debugString(val) {
    // primitive types
    const type = typeof val;
    if (type == 'number' || type == 'boolean' || val == null) {
        return  `${val}`;
    }
    if (type == 'string') {
        return `"${val}"`;
    }
    if (type == 'symbol') {
        const description = val.description;
        if (description == null) {
            return 'Symbol';
        } else {
            return `Symbol(${description})`;
        }
    }
    if (type == 'function') {
        const name = val.name;
        if (typeof name == 'string' && name.length > 0) {
            return `Function(${name})`;
        } else {
            return 'Function';
        }
    }
    // objects
    if (Array.isArray(val)) {
        const length = val.length;
        let debug = '[';
        if (length > 0) {
            debug += debugString(val[0]);
        }
        for(let i = 1; i < length; i++) {
            debug += ', ' + debugString(val[i]);
        }
        debug += ']';
        return debug;
    }
    // Test for built-in
    const builtInMatches = /\[object ([^\]]+)\]/.exec(toString.call(val));
    let className;
    if (builtInMatches.length > 1) {
        className = builtInMatches[1];
    } else {
        // Failed to match the standard '[object ClassName]'
        return toString.call(val);
    }
    if (className == 'Object') {
        // we're a user defined class or Object
        // JSON.stringify avoids problems with cycles, and is generally much
        // easier than looping through ownProperties of `val`.
        try {
            return 'Object(' + JSON.stringify(val) + ')';
        } catch (_) {
            return 'Object';
        }
    }
    // errors
    if (val instanceof Error) {
        return `${val.name}: ${val.message}\n${val.stack}`;
    }
    // TODO we could test for more things here, like `Set`s and `Map`s.
    return className;
}

function _assertClass(instance, klass) {
    if (!(instance instanceof klass)) {
        throw new Error(`expected instance of ${klass.name}`);
    }
    return instance.ptr;
}

let stack_pointer = 128;

function addBorrowedObject(obj) {
    if (stack_pointer == 1) throw new Error('out of js stack');
    heap[--stack_pointer] = obj;
    return stack_pointer;
}

function handleError(f, args) {
    try {
        return f.apply(this, args);
    } catch (e) {
        wasm.__wbindgen_exn_store(addHeapObject(e));
    }
}

function getArrayU8FromWasm0(ptr, len) {
    return getUint8Memory0().subarray(ptr / 1, ptr / 1 + len);
}
/**
*/
export const Method = Object.freeze({ Deep:0,"0":"Deep",MCTS:1,"1":"MCTS",Mix:2,"2":"Mix", });
/**
*/
export const FinishType = Object.freeze({ Draw1:0,"0":"Draw1",Draw2:1,"1":"Draw2",Draw3:2,"2":"Draw3",Draw4:3,"3":"Draw4",Draw5:4,"4":"Draw5",BlackWin:5,"5":"BlackWin",WhiteWin:6,"6":"WhiteWin", });
/**
*/
export const Color = Object.freeze({ Black:0,"0":"Black",White:1,"1":"White", });
/**
*/
export class BestPos {

    static __wrap(ptr) {
        const obj = Object.create(BestPos.prototype);
        obj.ptr = ptr;

        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.ptr;
        this.ptr = 0;

        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_bestpos_free(ptr);
    }
}
/**
*/
export class Game {

    static __wrap(ptr) {
        const obj = Object.create(Game.prototype);
        obj.ptr = ptr;

        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.ptr;
        this.ptr = 0;

        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_game_free(ptr);
    }
    /**
    * @param {number} size
    */
    constructor(size) {
        const ret = wasm.game_new(size);
        return Game.__wrap(ret);
    }
    /**
    * @param {number} depth
    */
    set_depth(depth) {
        wasm.game_set_depth(this.ptr, depth);
    }
    /**
    * @param {number} mcts_lim
    */
    set_mcts_lim(mcts_lim) {
        wasm.game_set_mcts_lim(this.ptr, mcts_lim);
    }
    /**
    * @param {number} method
    */
    set_method(method) {
        wasm.game_set_method(this.ptr, method);
    }
    /**
    * @param {Piece} piece
    */
    insert_piece(piece) {
        _assertClass(piece, Piece);
        var ptr0 = piece.__destroy_into_raw();
        wasm.game_insert_piece(this.ptr, ptr0);
    }
    /**
    * @param {number} pos
    * @returns {boolean}
    */
    remove_piece(pos) {
        const ret = wasm.game_remove_piece(this.ptr, pos);
        return ret !== 0;
    }
    /**
    * @returns {any}
    */
    get position() {
        const ret = wasm.game_position(this.ptr);
        return takeObject(ret);
    }
    /**
    * @param {BestPos} pos
    */
    make_move_by_pos_item(pos) {
        _assertClass(pos, BestPos);
        wasm.game_make_best_move(this.ptr, pos.ptr);
    }
    /**
    * @param {MoveItem} move_item
    * @returns {number | undefined}
    */
    make_move_by_move_item(move_item) {
        _assertClass(move_item, MoveItem);
        const ret = wasm.game_make_move_by_move_item(this.ptr, move_item.ptr);
        return ret === 7 ? undefined : ret;
    }
    /**
    * @param {number} max_depth
    * @param {number} best_white
    * @param {number} best_black
    * @param {number} depth
    * @param {boolean} state_only
    * @returns {BestPos}
    */
    best_move(max_depth, best_white, best_black, depth, state_only) {
        const ret = wasm.game_best_move(this.ptr, max_depth, best_white, best_black, depth, state_only);
        return BestPos.__wrap(ret);
    }
    /**
    * @param {boolean} apply
    * @returns {BestPos}
    */
    mix_method(apply) {
        const ret = wasm.game_mix_method(this.ptr, apply);
        return BestPos.__wrap(ret);
    }
    /**
    * @param {boolean} apply
    * @returns {any}
    */
    get_or_apply_best_move(apply) {
        const ret = wasm.game_get_or_apply_best_move(this.ptr, apply);
        return takeObject(ret);
    }
    /**
    * @param {BestPos} pos
    */
    make_best_move(pos) {
        _assertClass(pos, BestPos);
        wasm.game_make_best_move(this.ptr, pos.ptr);
    }
    /**
    * @returns {any}
    */
    find_and_make_best_move_ts_n() {
        const ret = wasm.game_find_and_make_best_move_ts_n(this.ptr);
        return takeObject(ret);
    }
    /**
    * @param {boolean} apply
    * @returns {any}
    */
    find_mcts_and_make_best_move_ts_n(apply) {
        const ret = wasm.game_find_mcts_and_make_best_move_ts_n(this.ptr, apply);
        return takeObject(ret);
    }
    /**
    */
    init_tree() {
        wasm.game_init_tree(this.ptr);
    }
    /**
    */
    resort_cache() {
        wasm.game_resort_cache(this.ptr);
    }
    /**
    * @returns {MCTSRes | undefined}
    */
    check_tree_for_finish() {
        const ret = wasm.game_check_tree_for_finish(this.ptr);
        return ret === 0 ? undefined : MCTSRes.__wrap(ret);
    }
    /**
    */
    preparing_tree() {
        wasm.game_preparing_tree(this.ptr);
    }
    /**
    * @param {boolean} apply
    * @returns {MCTSRes}
    */
    find_mcts_and_make_best_move(apply) {
        const ret = wasm.game_find_mcts_and_make_best_move(this.ptr, apply);
        return MCTSRes.__wrap(ret);
    }
    /**
    * @returns {any}
    */
    get_board_list_ts_n() {
        const ret = wasm.game_get_board_list_ts_n(this.ptr);
        return takeObject(ret);
    }
    /**
    */
    mov_back() {
        wasm.game_mov_back(this.ptr);
    }
    /**
    * @param {number} i
    * @returns {any}
    */
    move_by_tree_index_ts_n(i) {
        const ret = wasm.game_move_by_tree_index_ts_n(this.ptr, i);
        return takeObject(ret);
    }
    /**
    * @param {number} i
    * @returns {number | undefined}
    */
    move_by_tree_index(i) {
        const ret = wasm.game_move_by_tree_index(this.ptr, i);
        return ret === 7 ? undefined : ret;
    }
    /**
    * @param {number} i
    * @returns {any}
    */
    move_by_index_ts_n(i) {
        const ret = wasm.game_move_by_index_ts_n(this.ptr, i);
        return takeObject(ret);
    }
    /**
    * @returns {BestPos}
    */
    get_best_move_rust() {
        const ret = wasm.game_get_best_move_rust(this.ptr);
        return BestPos.__wrap(ret);
    }
    /**
    * @returns {string}
    */
    state_() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.game_state_(retptr, this.ptr);
            var r0 = getInt32Memory0()[retptr / 4 + 0];
            var r1 = getInt32Memory0()[retptr / 4 + 1];
            return getStringFromWasm0(r0, r1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_free(r0, r1);
        }
    }
    /**
    * @param {number} pack_index
    * @returns {number}
    */
    to_board(pack_index) {
        const ret = wasm.game_to_board(this.ptr, pack_index);
        return ret >>> 0;
    }
    /**
    * @param {number} board_index
    * @returns {number}
    */
    to_pack(board_index) {
        const ret = wasm.game_to_pack(this.ptr, board_index);
        return ret >>> 0;
    }
    /**
    * @returns {any}
    */
    get_move_list_for_front() {
        const ret = wasm.game_get_move_list_for_front(this.ptr);
        return takeObject(ret);
    }
    /**
    * @returns {any}
    */
    get moveColor() {
        const ret = wasm.game_get_color(this.ptr);
        return takeObject(ret);
    }
    /**
    * @param {number} color
    */
    set moveColor(color) {
        wasm.game_set_color(this.ptr, color);
    }
    /**
    * @param {any} pos_chain
    * @returns {any}
    */
    make_move_for_front(pos_chain) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.game_make_move_for_front(retptr, this.ptr, addBorrowedObject(pos_chain));
            var r0 = getInt32Memory0()[retptr / 4 + 0];
            var r1 = getInt32Memory0()[retptr / 4 + 1];
            var r2 = getInt32Memory0()[retptr / 4 + 2];
            if (r2) {
                throw takeObject(r1);
            }
            return takeObject(r0);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            heap[stack_pointer++] = undefined;
        }
    }
}
/**
*/
export class MCTSRes {

    static __wrap(ptr) {
        const obj = Object.create(MCTSRes.prototype);
        obj.ptr = ptr;

        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.ptr;
        this.ptr = 0;

        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_mctsres_free(ptr);
    }
}
/**
*/
export class MoveItem {

    __destroy_into_raw() {
        const ptr = this.ptr;
        this.ptr = 0;

        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_moveitem_free(ptr);
    }
}
/**
*/
export class MoveList {

    __destroy_into_raw() {
        const ptr = this.ptr;
        this.ptr = 0;

        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_movelist_free(ptr);
    }
}
/**
*/
export class Piece {

    static __wrap(ptr) {
        const obj = Object.create(Piece.prototype);
        obj.ptr = ptr;

        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.ptr;
        this.ptr = 0;

        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_piece_free(ptr);
    }
    /**
    * @param {number} pos
    * @param {number} color
    * @param {boolean} is_king
    * @returns {Piece}
    */
    static new(pos, color, is_king) {
        const ret = wasm.piece_new(pos, color, is_king);
        return Piece.__wrap(ret);
    }
    /**
    * @param {any} js
    * @returns {Piece | undefined}
    */
    static new_fom_js(js) {
        const ret = wasm.piece_new_fom_js(addHeapObject(js));
        return ret === 0 ? undefined : Piece.__wrap(ret);
    }
    /**
    * @returns {any}
    */
    get it() {
        const ptr = this.__destroy_into_raw();
        const ret = wasm.piece_it(ptr);
        return takeObject(ret);
    }
    /**
    * @param {any} js
    */
    set it(js) {
        wasm.piece_set_it(this.ptr, addHeapObject(js));
    }
    /**
    * @returns {number}
    */
    get pos() {
        const ret = wasm.__wbg_get_piece_pos(this.ptr);
        return ret >>> 0;
    }
    /**
    * @param {number} arg0
    */
    set pos(arg0) {
        wasm.__wbg_set_piece_pos(this.ptr, arg0);
    }
    /**
    * @returns {number}
    */
    get color() {
        const ret = wasm.__wbg_get_piece_color(this.ptr);
        return ret >>> 0;
    }
    /**
    * @param {number} arg0
    */
    set color(arg0) {
        wasm.__wbg_set_piece_color(this.ptr, arg0);
    }
    /**
    * @returns {boolean}
    */
    get is_king() {
        const ret = wasm.__wbg_get_piece_is_king(this.ptr);
        return ret !== 0;
    }
    /**
    * @param {boolean} arg0
    */
    set is_king(arg0) {
        wasm.__wbg_set_piece_is_king(this.ptr, arg0);
    }
    /**
    * @returns {boolean}
    */
    get stricken() {
        const ret = wasm.__wbg_get_piece_stricken(this.ptr);
        return ret !== 0;
    }
    /**
    * @param {boolean} arg0
    */
    set stricken(arg0) {
        wasm.__wbg_set_piece_stricken(this.ptr, arg0);
    }
}
/**
*/
export class PositionAndMove {

    __destroy_into_raw() {
        const ptr = this.ptr;
        this.ptr = 0;

        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_positionandmove_free(ptr);
    }
}
/**
*/
export class PositionEnvironment {

    static __wrap(ptr) {
        const obj = Object.create(PositionEnvironment.prototype);
        obj.ptr = ptr;

        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.ptr;
        this.ptr = 0;

        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_positionenvironment_free(ptr);
    }
    /**
    * @returns {number}
    */
    get size() {
        const ret = wasm.__wbg_get_positionenvironment_size(this.ptr);
        return ret;
    }
    /**
    * @param {number} arg0
    */
    set size(arg0) {
        wasm.__wbg_set_positionenvironment_size(this.ptr, arg0);
    }
    /**
    * @param {number} size
    */
    constructor(size) {
        const ret = wasm.positionenvironment_new(size);
        return PositionEnvironment.__wrap(ret);
    }
    /**
    * @returns {any}
    */
    js() {
        const ret = wasm.positionenvironment_js(this.ptr);
        return takeObject(ret);
    }
    /**
    * @param {Piece} piece
    * @param {number} pos
    * @returns {boolean}
    */
    is_king_move_for(piece, pos) {
        _assertClass(piece, Piece);
        const ret = wasm.positionenvironment_is_king_move_for(this.ptr, piece.ptr, pos);
        return ret !== 0;
    }
    /**
    */
    static game() {
        wasm.positionenvironment_game();
    }
    /**
    * @returns {any}
    */
    static test() {
        const ret = wasm.positionenvironment_test();
        return takeObject(ret);
    }
}
/**
*/
export class StraightStrike {

    __destroy_into_raw() {
        const ptr = this.ptr;
        this.ptr = 0;

        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_straightstrike_free(ptr);
    }
}

async function load(module, imports) {
    if (typeof Response === 'function' && module instanceof Response) {
        if (typeof WebAssembly.instantiateStreaming === 'function') {
            try {
                return await WebAssembly.instantiateStreaming(module, imports);

            } catch (e) {
                if (module.headers.get('Content-Type') != 'application/wasm') {
                    console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);

                } else {
                    throw e;
                }
            }
        }

        const bytes = await module.arrayBuffer();
        return await WebAssembly.instantiate(bytes, imports);

    } else {
        const instance = await WebAssembly.instantiate(module, imports);

        if (instance instanceof WebAssembly.Instance) {
            return { instance, module };

        } else {
            return instance;
        }
    }
}

function getImports() {
    const imports = {};
    imports.wbg = {};
    imports.wbg.__wbindgen_object_drop_ref = function(arg0) {
        takeObject(arg0);
    };
    imports.wbg.__wbindgen_string_new = function(arg0, arg1) {
        const ret = getStringFromWasm0(arg0, arg1);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_number_get = function(arg0, arg1) {
        const obj = getObject(arg1);
        const ret = typeof(obj) === 'number' ? obj : undefined;
        getFloat64Memory0()[arg0 / 8 + 1] = isLikeNone(ret) ? 0 : ret;
        getInt32Memory0()[arg0 / 4 + 0] = !isLikeNone(ret);
    };
    imports.wbg.__wbindgen_error_new = function(arg0, arg1) {
        const ret = new Error(getStringFromWasm0(arg0, arg1));
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_string_get = function(arg0, arg1) {
        const obj = getObject(arg1);
        const ret = typeof(obj) === 'string' ? obj : undefined;
        var ptr0 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        getInt32Memory0()[arg0 / 4 + 1] = len0;
        getInt32Memory0()[arg0 / 4 + 0] = ptr0;
    };
    imports.wbg.__wbindgen_is_string = function(arg0) {
        const ret = typeof(getObject(arg0)) === 'string';
        return ret;
    };
    imports.wbg.__wbindgen_is_object = function(arg0) {
        const val = getObject(arg0);
        const ret = typeof(val) === 'object' && val !== null;
        return ret;
    };
    imports.wbg.__wbindgen_is_undefined = function(arg0) {
        const ret = getObject(arg0) === undefined;
        return ret;
    };
    imports.wbg.__wbindgen_in = function(arg0, arg1) {
        const ret = getObject(arg0) in getObject(arg1);
        return ret;
    };
    imports.wbg.__wbindgen_is_bigint = function(arg0) {
        const ret = typeof(getObject(arg0)) === 'bigint';
        return ret;
    };
    imports.wbg.__wbindgen_bigint_from_u64 = function(arg0) {
        const ret = BigInt.asUintN(64, arg0);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_jsval_eq = function(arg0, arg1) {
        const ret = getObject(arg0) === getObject(arg1);
        return ret;
    };
    imports.wbg.__wbindgen_boolean_get = function(arg0) {
        const v = getObject(arg0);
        const ret = typeof(v) === 'boolean' ? (v ? 1 : 0) : 2;
        return ret;
    };
    imports.wbg.__wbg_log_7529978016e706d9 = function(arg0, arg1) {
        console.log(getStringFromWasm0(arg0, arg1));
    };
    imports.wbg.__wbindgen_number_new = function(arg0) {
        const ret = arg0;
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_object_clone_ref = function(arg0) {
        const ret = getObject(arg0);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_jsval_loose_eq = function(arg0, arg1) {
        const ret = getObject(arg0) == getObject(arg1);
        return ret;
    };
    imports.wbg.__wbg_String_91fba7ded13ba54c = function(arg0, arg1) {
        const ret = String(getObject(arg1));
        const ptr0 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        getInt32Memory0()[arg0 / 4 + 1] = len0;
        getInt32Memory0()[arg0 / 4 + 0] = ptr0;
    };
    imports.wbg.__wbg_getwithrefkey_15c62c2b8546208d = function(arg0, arg1) {
        const ret = getObject(arg0)[getObject(arg1)];
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_set_20cbc34131e76824 = function(arg0, arg1, arg2) {
        getObject(arg0)[takeObject(arg1)] = takeObject(arg2);
    };
    imports.wbg.__wbg_randomFillSync_6894564c2c334c42 = function() { return handleError(function (arg0, arg1, arg2) {
        getObject(arg0).randomFillSync(getArrayU8FromWasm0(arg1, arg2));
    }, arguments) };
    imports.wbg.__wbg_getRandomValues_805f1c3d65988a5a = function() { return handleError(function (arg0, arg1) {
        getObject(arg0).getRandomValues(getObject(arg1));
    }, arguments) };
    imports.wbg.__wbg_crypto_e1d53a1d73fb10b8 = function(arg0) {
        const ret = getObject(arg0).crypto;
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_process_038c26bf42b093f8 = function(arg0) {
        const ret = getObject(arg0).process;
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_versions_ab37218d2f0b24a8 = function(arg0) {
        const ret = getObject(arg0).versions;
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_node_080f4b19d15bc1fe = function(arg0) {
        const ret = getObject(arg0).node;
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_msCrypto_6e7d3e1f92610cbb = function(arg0) {
        const ret = getObject(arg0).msCrypto;
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_require_78a3dcfbdba9cbce = function() { return handleError(function () {
        const ret = module.require;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbindgen_is_function = function(arg0) {
        const ret = typeof(getObject(arg0)) === 'function';
        return ret;
    };
    imports.wbg.__wbg_get_27fe3dac1c4d0224 = function(arg0, arg1) {
        const ret = getObject(arg0)[arg1 >>> 0];
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_length_e498fbc24f9c1d4f = function(arg0) {
        const ret = getObject(arg0).length;
        return ret;
    };
    imports.wbg.__wbg_new_b525de17f44a8943 = function() {
        const ret = new Array();
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_newnoargs_2b8b6bd7753c76ba = function(arg0, arg1) {
        const ret = new Function(getStringFromWasm0(arg0, arg1));
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_next_b7d530c04fd8b217 = function(arg0) {
        const ret = getObject(arg0).next;
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_next_88560ec06a094dea = function() { return handleError(function (arg0) {
        const ret = getObject(arg0).next();
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_done_1ebec03bbd919843 = function(arg0) {
        const ret = getObject(arg0).done;
        return ret;
    };
    imports.wbg.__wbg_value_6ac8da5cc5b3efda = function(arg0) {
        const ret = getObject(arg0).value;
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_iterator_55f114446221aa5a = function() {
        const ret = Symbol.iterator;
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_get_baf4855f9a986186 = function() { return handleError(function (arg0, arg1) {
        const ret = Reflect.get(getObject(arg0), getObject(arg1));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_call_95d1ea488d03e4e8 = function() { return handleError(function (arg0, arg1) {
        const ret = getObject(arg0).call(getObject(arg1));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_new_f9876326328f45ed = function() {
        const ret = new Object();
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_self_e7c1f827057f6584 = function() { return handleError(function () {
        const ret = self.self;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_window_a09ec664e14b1b81 = function() { return handleError(function () {
        const ret = window.window;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_globalThis_87cbb8506fecf3a9 = function() { return handleError(function () {
        const ret = globalThis.globalThis;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_global_c85a9259e621f3db = function() { return handleError(function () {
        const ret = global.global;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_set_17224bc548dd1d7b = function(arg0, arg1, arg2) {
        getObject(arg0)[arg1 >>> 0] = takeObject(arg2);
    };
    imports.wbg.__wbg_instanceof_ArrayBuffer_a69f02ee4c4f5065 = function(arg0) {
        let result;
        try {
            result = getObject(arg0) instanceof ArrayBuffer;
        } catch {
            result = false;
        }
        const ret = result;
        return ret;
    };
    imports.wbg.__wbg_call_9495de66fdbe016b = function() { return handleError(function (arg0, arg1, arg2) {
        const ret = getObject(arg0).call(getObject(arg1), getObject(arg2));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_isSafeInteger_8c4789029e885159 = function(arg0) {
        const ret = Number.isSafeInteger(getObject(arg0));
        return ret;
    };
    imports.wbg.__wbg_entries_4e1315b774245952 = function(arg0) {
        const ret = Object.entries(getObject(arg0));
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_buffer_cf65c07de34b9a08 = function(arg0) {
        const ret = getObject(arg0).buffer;
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_new_537b7341ce90bb31 = function(arg0) {
        const ret = new Uint8Array(getObject(arg0));
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_set_17499e8aa4003ebd = function(arg0, arg1, arg2) {
        getObject(arg0).set(getObject(arg1), arg2 >>> 0);
    };
    imports.wbg.__wbg_length_27a2afe8ab42b09f = function(arg0) {
        const ret = getObject(arg0).length;
        return ret;
    };
    imports.wbg.__wbg_instanceof_Uint8Array_01cebe79ca606cca = function(arg0) {
        let result;
        try {
            result = getObject(arg0) instanceof Uint8Array;
        } catch {
            result = false;
        }
        const ret = result;
        return ret;
    };
    imports.wbg.__wbg_newwithlength_b56c882b57805732 = function(arg0) {
        const ret = new Uint8Array(arg0 >>> 0);
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_subarray_7526649b91a252a6 = function(arg0, arg1, arg2) {
        const ret = getObject(arg0).subarray(arg1 >>> 0, arg2 >>> 0);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_bigint_get_as_i64 = function(arg0, arg1) {
        const v = getObject(arg1);
        const ret = typeof(v) === 'bigint' ? v : undefined;
        getBigInt64Memory0()[arg0 / 8 + 1] = isLikeNone(ret) ? BigInt(0) : ret;
        getInt32Memory0()[arg0 / 4 + 0] = !isLikeNone(ret);
    };
    imports.wbg.__wbindgen_debug_string = function(arg0, arg1) {
        const ret = debugString(getObject(arg1));
        const ptr0 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        getInt32Memory0()[arg0 / 4 + 1] = len0;
        getInt32Memory0()[arg0 / 4 + 0] = ptr0;
    };
    imports.wbg.__wbindgen_throw = function(arg0, arg1) {
        throw new Error(getStringFromWasm0(arg0, arg1));
    };
    imports.wbg.__wbindgen_memory = function() {
        const ret = wasm.memory;
        return addHeapObject(ret);
    };

    return imports;
}

function initMemory(imports, maybe_memory) {

}

function finalizeInit(instance, module) {
    wasm = instance.exports;
    init.__wbindgen_wasm_module = module;
    cachedBigInt64Memory0 = null;
    cachedFloat64Memory0 = null;
    cachedInt32Memory0 = null;
    cachedUint8Memory0 = null;


    return wasm;
}

function initSync(module) {
    const imports = getImports();

    initMemory(imports);

    if (!(module instanceof WebAssembly.Module)) {
        module = new WebAssembly.Module(module);
    }

    const instance = new WebAssembly.Instance(module, imports);

    return finalizeInit(instance, module);
}

async function init(input) {
    if (typeof input === 'undefined') {
        input = new URL('checkers_core_bg.wasm', import.meta.url);
    }
    const imports = getImports();

    if (typeof input === 'string' || (typeof Request === 'function' && input instanceof Request) || (typeof URL === 'function' && input instanceof URL)) {
        input = fetch(input);
    }

    initMemory(imports);

    const { instance, module } = await load(await input, imports);

    return finalizeInit(instance, module);
}

export { initSync }
export default init;

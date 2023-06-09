import {Color} from "../../build-wasm/checkers_core"
import {GameProcess} from "../gameProcess";

type ListOrFinish = Array<number>[] | 'BlackWin' | 'WhiteWin' | `Draw${number}`
let listOrFinish: ListOrFinish


do {
    const selectPosition = (listOrFinish: ListOrFinish) => {
        if (listOrFinish.length) {
            gameProcess.game.move_by_tree_index_ts_n(Math.random() * listOrFinish.length >> 0)
        }
    }
    let gameProcess = new GameProcess(8)
    gameProcess.game.set_mcts_lim(10000) // <-- mcts limit
    let whitePosList = [0, 2, 4, 6, 9, 11, 13, 15, 16, 18, 20, 22]
    let blackPosList = whitePosList.map(x => 63 - x)
    whitePosList.forEach(x => gameProcess.insertPiece(x, Color.White, false))
    blackPosList.forEach(x => gameProcess.insertPiece(x, Color.Black, false))
    gameProcess.moveColor = Color.White;
    let movesCount = 0
    let neuralMakeFirstMove = Math.random() < 0.5
    neuralMakeFirstMove = false
    if (neuralMakeFirstMove) {
        listOrFinish = gameProcess.game.find_mcts_and_make_best_move_ts_n(false)
        if (listOrFinish instanceof Array) {
            selectPosition(listOrFinish)
            movesCount++
        }
        console.log("neural play White")
    } else {
        console.log("neural play Black")
    }
    do {
        listOrFinish = gameProcess.game.find_mcts_and_make_best_move_ts_n(true)
        movesCount++ // todo result may have +1 count mistake in case Deep algorithm lost
        if (listOrFinish instanceof Array) {
            selectPosition(listOrFinish)
            movesCount++
        }
    } while (listOrFinish instanceof Array)
    console.log(listOrFinish, movesCount)
} while (1)



import type { PositionAndMove } from "./PositionAndMove";
export interface BestPos {
    pos: PositionAndMove | null;
    pos_list: Array<PositionAndMove>;
    deep_eval: number;
}

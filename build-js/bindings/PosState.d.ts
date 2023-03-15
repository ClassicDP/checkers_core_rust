import type { PieceCount } from "./PieceCount";
export interface PosState {
    black: PieceCount;
    white: PieceCount;
    kings_start_at: number | null;
    kings_only_move_start_at: number | null;
    triangle_start_at: number | null;
    power_equal_start_at: number | null;
    main_road_start_at: number | null;
    repeats: number | null;
}

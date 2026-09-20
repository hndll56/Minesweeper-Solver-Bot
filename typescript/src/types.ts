export interface CellData {
  state: 'unknown' | 'revealed' | 'flagged';
  number?: number;
}

export interface BoardState {
  width: number;
  height: number;
  totalMines: number;
  cells: CellData[];
  gameStatus: 'playing' | 'won' | 'lost' | 'idle';
}

export interface SolveAction {
  type: 'click' | 'flag';
  index: number;
}

export interface SolveResponse {
  request_id: string;
  actions: SolveAction[];
  is_guess: boolean;
  confidence?: number;
}

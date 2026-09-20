import { Page } from 'playwright';
import { BoardState, CellData } from '../types';
import { GameType } from './game_detector';

// Google Minesweeper cell class patterns
const GOOGLE_NUMBER_CLASSES: Record<string, number> = {
  'minesweeper-cell-revealed-1': 1,
  'minesweeper-cell-revealed-2': 2,
  'minesweeper-cell-revealed-3': 3,
  'minesweeper-cell-revealed-4': 4,
  'minesweeper-cell-revealed-5': 5,
  'minesweeper-cell-revealed-6': 6,
  'minesweeper-cell-revealed-7': 7,
  'minesweeper-cell-revealed-8': 8,
};

// minesweeperonline.com cell id/class patterns
// cells are td elements with id like "1", class like "blank", "open1".."open8", "bombflagged"
function parseMSOCell(className: string): CellData {
  if (className.includes('bombflagged') || className.includes('flag')) return { state: 'flagged' };
  if (className === 'blank' || className === '') return { state: 'unknown' };
  const m = className.match(/open(\d)/);
  if (m) return { state: 'revealed', number: parseInt(m[1], 10) };
  if (className.includes('open')) return { state: 'revealed', number: 0 };
  return { state: 'unknown' };
}

export async function readBoard(page: Page, gameType: GameType): Promise<BoardState> {
  if (gameType === 'google') return readGoogleBoard(page);
  if (gameType === 'minesweeperonline') return readMSOBoard(page);
  return readGoogleBoard(page); // best-effort fallback
}

async function readGoogleBoard(page: Page): Promise<BoardState> {
  return page.evaluate(() => {
    // Google Minesweeper uses a canvas + accessible elements, or DOM cells
    // Try aria-based reading first
    const gameEl = document.querySelector('[jsname="tbSPXd"]') ||
                   document.querySelector('.minesweeper-game') ||
                   document.querySelector('[data-difficulty]');

    // Read status
    let gameStatus: BoardState['gameStatus'] = 'playing';
    const statusEl = document.querySelector('[data-game-status]');
    if (statusEl) {
      const s = statusEl.getAttribute('data-game-status') || '';
      if (s === 'win') gameStatus = 'won';
      else if (s === 'lose') gameStatus = 'lost';
      else if (s === 'idle' || s === 'ready') gameStatus = 'idle';
    }

    // Read cells — Google wraps them in td or div with aria-label
    const cells = Array.from(document.querySelectorAll('[aria-label]')).filter(el => {
      const label = el.getAttribute('aria-label') || '';
      return label.includes('mine') || label.match(/\d/) || label.includes('blank') || label.includes('flag');
    }) as HTMLElement[];

    if (cells.length === 0) {
      // Try table-based layout
      const tds = Array.from(document.querySelectorAll('td[class]')) as HTMLElement[];
      if (tds.length > 0) {
        const cellData = tds.map(td => {
          const cls = td.className || '';
          if (cls.includes('flag')) return { state: 'flagged' as const };
          const m = cls.match(/num_(\d)/);
          if (m) return { state: 'revealed' as const, number: parseInt(m[1], 10) };
          if (cls.includes('open') || cls.includes('blank0')) return { state: 'revealed' as const, number: 0 };
          return { state: 'unknown' as const };
        });
        // Infer dimensions from table
        const table = document.querySelector('table');
        const rows = table ? table.querySelectorAll('tr').length : Math.round(Math.sqrt(tds.length));
        const cols = tds.length / rows;
        return { width: cols, height: rows, totalMines: 0, cells: cellData, gameStatus };
      }
      return { width: 0, height: 0, totalMines: 0, cells: [], gameStatus: 'idle' };
    }

    const cellData: CellData[] = cells.map(el => {
      const label = (el.getAttribute('aria-label') || '').toLowerCase();
      if (label.includes('flag') || label.includes('marked')) return { state: 'flagged' as const };
      const numMatch = label.match(/(\d+)/);
      if (numMatch && !label.includes('mine')) return { state: 'revealed' as const, number: parseInt(numMatch[1], 10) };
      if (label.includes('mine') && label.includes('revealed')) return { state: 'revealed' as const, number: -1 };
      if (label.includes('revealed') || label.includes('blank') || label.includes('empty')) return { state: 'revealed' as const, number: 0 };
      return { state: 'unknown' as const };
    });

    // Infer grid size from parent container
    const parent = cells[0]?.parentElement;
    const allSiblings = parent ? Array.from(parent.children).length : cells.length;
    const gridEl = document.querySelector('[data-rows]');
    const width = gridEl ? parseInt(gridEl.getAttribute('data-cols') || '9', 10) : 9;
    const height = gridEl ? parseInt(gridEl.getAttribute('data-rows') || '9', 10) : Math.ceil(cells.length / width);
    const totalMines = parseInt(document.querySelector('[data-mines]')?.getAttribute('data-mines') || '0', 10);

    return { width, height, totalMines, cells: cellData, gameStatus };

    // ponytail: grid size inference is heuristic; inject data attrs from game DOM when available
  }) as Promise<BoardState>;
}

async function readMSOBoard(page: Page): Promise<BoardState> {
  return page.evaluate(() => {
    // minesweeperonline.com uses div elements with id="row_col" (e.g. "1_1")
    const allDivs = Array.from(document.querySelectorAll('#game div[id]')) as HTMLElement[];
    const cells = allDivs
      .filter(d => /^\d+_\d+$/.test(d.id))
      .sort((a, b) => {
        const [ar, ac] = a.id.split('_').map(Number);
        const [br, bc] = b.id.split('_').map(Number);
        return ar !== br ? ar - br : ac - bc;
      });

    if (cells.length === 0) return { width: 0, height: 0, totalMines: 0, cells: [], gameStatus: 'idle' as const };

    // Derive width/height from max row/col in ids
    let maxRow = 0, maxCol = 0;
    for (const d of cells) {
      const [r, c] = d.id.split('_').map(Number);
      if (r > maxRow) maxRow = r;
      if (c > maxCol) maxCol = c;
    }
    const width = maxCol;
    const height = maxRow;

    // Mine count from digit display divs
    const digit = (id: string) => {
      const el = document.querySelector(`#${id}`) as HTMLElement | null;
      if (!el) return 0;
      const m = el.className.match(/time(\d)/);
      return m ? parseInt(m[1], 10) : 0;
    };
    const totalMines = digit('mines_hundreds') * 100 + digit('mines_tens') * 10 + digit('mines_ones')
      || (width === 9 ? 10 : width === 16 ? 40 : 99);

    const cellData = cells.map(td => {
      const cls = td.className || '';
      if (cls.includes('bombflagged') || cls.includes('flag')) return { state: 'flagged' as const };
      if (cls.includes('bombrevealed') || cls.includes('bombdeath')) return { state: 'revealed' as const, number: 0 };
      const m = cls.match(/open(\d)/);
      if (m) return { state: 'revealed' as const, number: parseInt(m[1], 10) };
      if (cls.includes('open')) return { state: 'revealed' as const, number: 0 };
      return { state: 'unknown' as const };
    });

    // Check win/lose from face button
    const face = document.querySelector('#face') as HTMLElement | null;
    let gameStatus: BoardState['gameStatus'] = 'playing';
    if (face) {
      const faceClass = face.className || '';
      if (faceClass.includes('win') || faceClass.includes('facewin')) gameStatus = 'won';
      else if (faceClass.includes('lose') || faceClass.includes('dead') || faceClass.includes('facedead')) gameStatus = 'lost';
    }

    return { width, height, totalMines, cells: cellData, gameStatus };
  }) as Promise<BoardState>;
}

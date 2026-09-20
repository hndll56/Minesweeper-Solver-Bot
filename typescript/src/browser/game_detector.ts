import { Page } from 'playwright';

export type GameType = 'google' | 'minesweeperonline' | 'unknown';

export async function detectGame(page: Page): Promise<GameType> {
  const url = page.url();
  if (url.includes('google.com')) return 'google';
  if (url.includes('minesweeperonline.com')) return 'minesweeperonline';
  return 'unknown';
}

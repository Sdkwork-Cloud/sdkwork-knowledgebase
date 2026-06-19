export type MediaPlayMode = 'sequential' | 'loop-all' | 'loop-one' | 'shuffle';

export function buildShuffleOrder(length: number, seed = 0): number[] {
  const order = Array.from({ length }, (_, index) => index);
  let state = seed || 1;
  for (let i = order.length - 1; i > 0; i -= 1) {
    state = (state * 1664525 + 1013904223) >>> 0;
    const j = state % (i + 1);
    [order[i], order[j]] = [order[j], order[i]];
  }
  return order;
}

export function findShuffleCursor(order: number[], activeIndex: number): number {
  const cursor = order.indexOf(activeIndex);
  return cursor >= 0 ? cursor : 0;
}

export function getNextPlaylistIndex(
  activeIndex: number,
  itemCount: number,
  mode: MediaPlayMode,
  shuffleOrder: number[],
  shuffleCursor: number
): { index: number; shuffleCursor: number } | null {
  if (itemCount <= 0) return null;

  if (itemCount === 1) {
    return mode === 'sequential' ? null : { index: 0, shuffleCursor: 0 };
  }

  if (mode === 'loop-one') {
    return { index: activeIndex, shuffleCursor };
  }

  if (mode === 'shuffle') {
    const nextCursor = shuffleCursor + 1;
    if (nextCursor < shuffleOrder.length) {
      return { index: shuffleOrder[nextCursor], shuffleCursor: nextCursor };
    }
    const wrapped = buildShuffleOrder(itemCount, Date.now());
    return { index: wrapped[0], shuffleCursor: 0 };
  }

  if (activeIndex < itemCount - 1) {
    return { index: activeIndex + 1, shuffleCursor };
  }

  if (mode === 'loop-all') {
    return { index: 0, shuffleCursor: 0 };
  }

  return null;
}

export function getPreviousPlaylistIndex(
  activeIndex: number,
  itemCount: number,
  mode: MediaPlayMode,
  shuffleOrder: number[],
  shuffleCursor: number
): { index: number; shuffleCursor: number } | null {
  if (itemCount <= 0) return null;
  if (itemCount === 1) return { index: 0, shuffleCursor: 0 };

  if (mode === 'shuffle') {
    const prevCursor = shuffleCursor - 1;
    if (prevCursor >= 0) {
      return { index: shuffleOrder[prevCursor], shuffleCursor: prevCursor };
    }
    return { index: shuffleOrder[shuffleOrder.length - 1], shuffleCursor: shuffleOrder.length - 1 };
  }

  if (activeIndex > 0) {
    return { index: activeIndex - 1, shuffleCursor };
  }

  if (mode === 'loop-all' || mode === 'loop-one') {
    return { index: itemCount - 1, shuffleCursor };
  }

  return null;
}

export function cyclePlayMode(mode: MediaPlayMode): MediaPlayMode {
  switch (mode) {
    case 'sequential':
      return 'loop-all';
    case 'loop-all':
      return 'loop-one';
    case 'loop-one':
      return 'sequential';
    case 'shuffle':
      return 'shuffle';
    default:
      return 'sequential';
  }
}

export function playModeLabel(mode: MediaPlayMode): string {
  switch (mode) {
    case 'sequential':
      return '顺序播放';
    case 'loop-all':
      return '列表循环';
    case 'loop-one':
      return '单曲循环';
    case 'shuffle':
      return '随机播放';
  }
}

export function effectivePlayMode(baseMode: MediaPlayMode, shuffleEnabled: boolean): MediaPlayMode {
  return shuffleEnabled ? 'shuffle' : baseMode === 'shuffle' ? 'sequential' : baseMode;
}

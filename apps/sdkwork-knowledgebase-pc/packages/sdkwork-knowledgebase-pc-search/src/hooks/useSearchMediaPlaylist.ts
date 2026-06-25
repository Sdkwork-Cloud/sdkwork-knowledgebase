import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import type { SearchMediaItem } from '../types';
import {
  buildShuffleOrder,
  effectivePlayMode,
  findShuffleCursor,
  getNextPlaylistIndex,
  getPreviousPlaylistIndex,
  type MediaPlayMode
} from '../utils/mediaPlaylist';
import type { MediaPlaylistControls } from '../components/media-viewers/types';

export interface UseSearchMediaPlaylistOptions {
  items: SearchMediaItem[];
  activeIndex: number;
  enabled: boolean;
  onIndexChange: (index: number) => void;
}

export interface UseSearchMediaPlaylistResult {
  isPlaylistOpen: boolean;
  isPlaying: boolean;
  playMode: MediaPlayMode;
  shuffleEnabled: boolean;
  playlistControls: MediaPlaylistControls | undefined;
  setIsPlaylistOpen: React.Dispatch<React.SetStateAction<boolean>>;
  setPlayMode: React.Dispatch<React.SetStateAction<MediaPlayMode>>;
  setShuffleEnabled: React.Dispatch<React.SetStateAction<boolean>>;
  handlePreviousTrack: () => void;
  handleNextTrack: () => void;
  selectPlaylistTrack: (index: number) => void;
}

export function useSearchMediaPlaylist({
  items,
  activeIndex,
  enabled,
  onIndexChange
}: UseSearchMediaPlaylistOptions): UseSearchMediaPlaylistResult {
  const safeIndex = Math.min(Math.max(activeIndex, 0), Math.max(items.length - 1, 0));
  const continuePlaybackRef = useRef(false);
  const [isPlaylistOpen, setIsPlaylistOpen] = useState(true);
  const [playMode, setPlayMode] = useState<MediaPlayMode>('sequential');
  const [shuffleEnabled, setShuffleEnabled] = useState(false);
  const [autoPlay, setAutoPlay] = useState(false);
  const [shuffleOrder, setShuffleOrder] = useState<number[]>(() => buildShuffleOrder(items.length));
  const [shuffleCursor, setShuffleCursor] = useState(0);
  const [isPlaying, setIsPlaying] = useState(false);
  const effectiveMode = effectivePlayMode(playMode, shuffleEnabled);

  useEffect(() => {
    if (!enabled) return;
    const order = buildShuffleOrder(items.length, items[0]?.id?.length ?? 0);
    setShuffleOrder(order);
    setShuffleCursor(findShuffleCursor(order, safeIndex));
  }, [enabled, items]);

  useEffect(() => {
    if (!enabled) return;
    setShuffleCursor(findShuffleCursor(shuffleOrder, safeIndex));
  }, [enabled, safeIndex, shuffleOrder]);

  useEffect(() => {
    if (!enabled) return;
    setAutoPlay(false);
  }, [enabled, items[safeIndex]?.id]);

  const goToTrack = useCallback(
    (index: number, shouldAutoPlay = true) => {
      if (shouldAutoPlay && continuePlaybackRef.current) {
        setAutoPlay(true);
      }
      onIndexChange(index);
    },
    [onIndexChange]
  );

  const handleTrackEnded = useCallback(() => {
    if (!enabled || effectiveMode === 'loop-one') return;

    let order = shuffleOrder;
    let cursor = shuffleCursor;

    if (effectiveMode === 'shuffle' && cursor >= order.length - 1) {
      order = buildShuffleOrder(items.length, Date.now());
      setShuffleOrder(order);
      cursor = -1;
    }

    const next = getNextPlaylistIndex(safeIndex, items.length, effectiveMode, order, cursor);
    if (!next) return;

    setShuffleCursor(next.shuffleCursor);
    continuePlaybackRef.current = true;
    setAutoPlay(true);
    onIndexChange(next.index);
  }, [effectiveMode, enabled, items.length, onIndexChange, safeIndex, shuffleCursor, shuffleOrder]);

  const handlePreviousTrack = useCallback(() => {
    if (!enabled) return;
    const prev = getPreviousPlaylistIndex(safeIndex, items.length, effectiveMode, shuffleOrder, shuffleCursor);
    if (!prev) return;
    setShuffleCursor(prev.shuffleCursor);
    goToTrack(prev.index, continuePlaybackRef.current);
  }, [effectiveMode, enabled, goToTrack, items.length, safeIndex, shuffleCursor, shuffleOrder]);

  const handleNextTrack = useCallback(() => {
    if (!enabled) return;
    let order = shuffleOrder;
    let cursor = shuffleCursor;
    if (effectiveMode === 'shuffle' && cursor >= order.length - 1) {
      order = buildShuffleOrder(items.length, Date.now());
      setShuffleOrder(order);
      cursor = -1;
    }
    const next = getNextPlaylistIndex(safeIndex, items.length, effectiveMode, order, cursor);
    if (!next) return;
    setShuffleCursor(next.shuffleCursor);
    goToTrack(next.index, continuePlaybackRef.current);
  }, [effectiveMode, enabled, goToTrack, items.length, safeIndex, shuffleCursor, shuffleOrder]);

  const selectPlaylistTrack = useCallback(
    (index: number) => {
      continuePlaybackRef.current = true;
      setAutoPlay(true);
      onIndexChange(index);
    },
    [onIndexChange]
  );

  const playlistControls = useMemo((): MediaPlaylistControls | undefined => {
    if (!enabled || items.length <= 1) return undefined;
    return {
      items,
      activeIndex: safeIndex,
      playMode: effectiveMode,
      shuffleEnabled,
      canGoPrevious: Boolean(getPreviousPlaylistIndex(safeIndex, items.length, effectiveMode, shuffleOrder, shuffleCursor)),
      canGoNext: Boolean(getNextPlaylistIndex(safeIndex, items.length, effectiveMode, shuffleOrder, shuffleCursor)),
      isPlaylistOpen,
      autoPlay,
      onSelectTrack: (index: number) => goToTrack(index, continuePlaybackRef.current),
      onPreviousTrack: handlePreviousTrack,
      onNextTrack: handleNextTrack,
      onPlayModeChange: (mode: MediaPlayMode) => {
        setPlayMode(mode);
        if (mode !== 'shuffle') setShuffleEnabled(false);
      },
      onShuffleToggle: () => setShuffleEnabled((value) => !value),
      onPlaylistToggle: () => setIsPlaylistOpen((value) => !value),
      onTrackEnded: handleTrackEnded,
      onAutoPlayConsumed: () => setAutoPlay(false),
      onPlaybackActiveChange: (playing: boolean) => {
        setIsPlaying(playing);
        continuePlaybackRef.current = playing;
        if (playing) setAutoPlay(false);
      }
    };
  }, [
    autoPlay,
    effectiveMode,
    enabled,
    goToTrack,
    handleNextTrack,
    handlePreviousTrack,
    handleTrackEnded,
    isPlaylistOpen,
    items,
    safeIndex,
    shuffleCursor,
    shuffleEnabled,
    shuffleOrder
  ]);

  return {
    isPlaylistOpen,
    isPlaying,
    playMode,
    shuffleEnabled,
    playlistControls,
    setIsPlaylistOpen,
    setPlayMode,
    setShuffleEnabled,
    handlePreviousTrack,
    handleNextTrack,
    selectPlaylistTrack
  };
}

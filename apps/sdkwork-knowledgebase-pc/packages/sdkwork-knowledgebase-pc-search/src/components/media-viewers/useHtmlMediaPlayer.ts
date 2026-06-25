import React, { useCallback, useEffect, useRef, useState } from 'react';

function formatTime(seconds: number): string {
  if (!Number.isFinite(seconds) || seconds < 0) return '0:00';
  const mins = Math.floor(seconds / 60);
  const secs = Math.floor(seconds % 60);
  return `${mins}:${secs.toString().padStart(2, '0')}`;
}

function readBufferedPercent(audio: HTMLAudioElement): number {
  if (!audio.duration || audio.buffered.length === 0) return 0;
  const end = audio.buffered.end(audio.buffered.length - 1);
  return Math.min(100, (end / audio.duration) * 100);
}

export interface UseHtmlMediaPlayerResult {
  audioRef: React.RefObject<HTMLAudioElement | null>;
  isPlaying: boolean;
  hasEnded: boolean;
  isBuffering: boolean;
  currentTime: number;
  duration: number;
  progress: number;
  buffered: number;
  togglePlay: () => void;
  seek: (ratio: number) => void;
  canPlay: boolean;
}

export function useHtmlMediaPlayer(
  src?: string,
  resetKey?: string,
  options?: {
    loop?: boolean;
    autoPlay?: boolean;
    onEnded?: () => void;
    onPlayingChange?: (isPlaying: boolean) => void;
  }
): UseHtmlMediaPlayerResult {
  const audioRef = useRef<HTMLAudioElement>(null);
  const [isPlaying, setIsPlaying] = useState(false);
  const [hasEnded, setHasEnded] = useState(false);
  const [isBuffering, setIsBuffering] = useState(false);
  const [currentTime, setCurrentTime] = useState(0);
  const [duration, setDuration] = useState(0);
  const [buffered, setBuffered] = useState(0);
  const [canPlay, setCanPlay] = useState(Boolean(src));

  useEffect(() => {
    const audio = audioRef.current;
    if (!audio) return;

    setIsPlaying(false);
    setHasEnded(false);
    setIsBuffering(false);
    setCurrentTime(0);
    setDuration(0);
    setBuffered(0);
    setCanPlay(Boolean(src));

    if (!src) return;

    audio.pause();
    audio.currentTime = 0;
    audio.load();
  }, [src, resetKey, options?.loop, options?.onEnded, options?.onPlayingChange]);

  useEffect(() => {
    const audio = audioRef.current;
    if (!audio || !options?.autoPlay || !src || !canPlay) return;
    void audio.play();
  }, [src, resetKey, options?.autoPlay, canPlay]);

  useEffect(() => {
    const audio = audioRef.current;
    if (!audio) return;

    const onPlay = () => {
      setIsPlaying(true);
      setHasEnded(false);
      setIsBuffering(false);
      options?.onPlayingChange?.(true);
    };
    const onPause = () => {
      setIsPlaying(false);
      options?.onPlayingChange?.(false);
    };
    const syncTime = () => {
      setCurrentTime(audio.currentTime);
      setBuffered(readBufferedPercent(audio));
      if (audio.duration && audio.currentTime < audio.duration - 0.35) {
        setHasEnded(false);
      }
    };
    const onTimeUpdate = () => syncTime();
    const onLoaded = () => {
      setDuration(audio.duration || 0);
      setBuffered(readBufferedPercent(audio));
      setCanPlay(true);
      setIsBuffering(false);
    };
    const onEnded = () => {
      if (options?.loop) {
        audio.currentTime = 0;
        void audio.play();
        return;
      }
      setIsPlaying(false);
      setHasEnded(true);
      setIsBuffering(false);
      options?.onPlayingChange?.(false);
      options?.onEnded?.();
    };
    const onError = () => {
      setCanPlay(false);
      setIsBuffering(false);
    };
    const onWaiting = () => setIsBuffering(true);
    const onPlaying = () => setIsBuffering(false);
    const onProgress = () => setBuffered(readBufferedPercent(audio));

    audio.addEventListener('play', onPlay);
    audio.addEventListener('pause', onPause);
    audio.addEventListener('timeupdate', onTimeUpdate);
    audio.addEventListener('loadedmetadata', onLoaded);
    audio.addEventListener('durationchange', onLoaded);
    audio.addEventListener('ended', onEnded);
    audio.addEventListener('error', onError);
    audio.addEventListener('waiting', onWaiting);
    audio.addEventListener('playing', onPlaying);
    audio.addEventListener('progress', onProgress);

    return () => {
      audio.removeEventListener('play', onPlay);
      audio.removeEventListener('pause', onPause);
      audio.removeEventListener('timeupdate', onTimeUpdate);
      audio.removeEventListener('loadedmetadata', onLoaded);
      audio.removeEventListener('durationchange', onLoaded);
      audio.removeEventListener('ended', onEnded);
      audio.removeEventListener('error', onError);
      audio.removeEventListener('waiting', onWaiting);
      audio.removeEventListener('playing', onPlaying);
      audio.removeEventListener('progress', onProgress);
    };
  }, [src, resetKey, options?.loop, options?.onEnded, options?.onPlayingChange]);

  useEffect(() => {
    if (!options?.autoPlay || !canPlay) return;
    const audio = audioRef.current;
    if (!audio || !src) return;
    void audio.play();
  }, [options?.autoPlay, canPlay, src, resetKey]);

  useEffect(() => {
    const audio = audioRef.current;
    if (!audio || !isPlaying || !src) return;

    let frame = 0;
    const tick = () => {
      setCurrentTime(audio.currentTime);
      frame = requestAnimationFrame(tick);
    };
    frame = requestAnimationFrame(tick);
    return () => cancelAnimationFrame(frame);
  }, [isPlaying, src, resetKey]);

  const togglePlay = useCallback(() => {
    const audio = audioRef.current;
    if (!audio || !src) return;
    if (audio.paused || audio.ended) {
      if (audio.ended || hasEnded) {
        audio.currentTime = 0;
        setHasEnded(false);
      }
      void audio.play();
      return;
    }
    audio.pause();
  }, [hasEnded, src]);

  const seek = useCallback(
    (ratio: number) => {
      const audio = audioRef.current;
      if (!audio || !duration) return;
      const next = Math.max(0, Math.min(duration, duration * ratio));
      audio.currentTime = next;
      setCurrentTime(next);
      if (next < duration - 0.35) {
        setHasEnded(false);
      }
    },
    [duration]
  );

  const progress = duration > 0 ? (currentTime / duration) * 100 : 0;

  return {
    audioRef,
    isPlaying,
    hasEnded,
    isBuffering,
    currentTime,
    duration,
    progress,
    buffered,
    togglePlay,
    seek,
    canPlay
  };
}

export { formatTime };

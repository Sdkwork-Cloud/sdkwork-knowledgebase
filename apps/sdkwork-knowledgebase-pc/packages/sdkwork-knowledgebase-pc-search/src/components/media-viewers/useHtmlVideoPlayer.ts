import React, { useCallback, useEffect, useRef, useState } from 'react';
import { formatTime } from './useHtmlMediaPlayer';

function readBufferedPercent(video: HTMLVideoElement): number {
  if (!video.duration || video.buffered.length === 0) return 0;
  const end = video.buffered.end(video.buffered.length - 1);
  return Math.min(100, (end / video.duration) * 100);
}

export interface UseHtmlVideoPlayerResult {
  videoRef: React.RefObject<HTMLVideoElement | null>;
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
  isMuted: boolean;
  toggleMute: () => void;
  toggleFullscreen: () => void;
  togglePictureInPicture: () => void;
}

export function useHtmlVideoPlayer(
  src?: string,
  resetKey?: string,
  options?: {
    loop?: boolean;
    autoPlay?: boolean;
    onEnded?: () => void;
    onPlayingChange?: (isPlaying: boolean) => void;
  }
): UseHtmlVideoPlayerResult {
  const videoRef = useRef<HTMLVideoElement>(null);
  const optionsRef = useRef(options);
  optionsRef.current = options;
  const [isPlaying, setIsPlaying] = useState(false);
  const [hasEnded, setHasEnded] = useState(false);
  const [isBuffering, setIsBuffering] = useState(false);
  const [currentTime, setCurrentTime] = useState(0);
  const [duration, setDuration] = useState(0);
  const [buffered, setBuffered] = useState(0);
  const [canPlay, setCanPlay] = useState(Boolean(src));
  const [isMuted, setIsMuted] = useState(false);

  useEffect(() => {
    const video = videoRef.current;
    if (!video) return;

    setIsPlaying(false);
    setHasEnded(false);
    setIsBuffering(false);
    setCurrentTime(0);
    setDuration(0);
    setBuffered(0);
    setCanPlay(Boolean(src));

    if (!src) return;

    video.pause();
    video.currentTime = 0;
    video.load();
  }, [src, resetKey]);

  useEffect(() => {
    const video = videoRef.current;
    if (!video) return;

    const onPlay = () => {
      setIsPlaying(true);
      setHasEnded(false);
      setIsBuffering(false);
      optionsRef.current?.onPlayingChange?.(true);
    };
    const onPause = () => {
      setIsPlaying(false);
      optionsRef.current?.onPlayingChange?.(false);
    };
    const onTimeUpdate = () => {
      setCurrentTime(video.currentTime);
      setBuffered(readBufferedPercent(video));
      if (video.duration && video.currentTime < video.duration - 0.35) {
        setHasEnded(false);
      }
    };
    const onLoaded = () => {
      setDuration(video.duration || 0);
      setBuffered(readBufferedPercent(video));
      setCanPlay(true);
      setIsBuffering(false);
    };
    const onEnded = () => {
      if (optionsRef.current?.loop) {
        video.currentTime = 0;
        void video.play();
        return;
      }
      setIsPlaying(false);
      setHasEnded(true);
      setIsBuffering(false);
      optionsRef.current?.onPlayingChange?.(false);
      optionsRef.current?.onEnded?.();
    };
    const onError = () => {
      setCanPlay(false);
      setIsBuffering(false);
    };
    const onWaiting = () => setIsBuffering(true);
    const onPlaying = () => setIsBuffering(false);
    const onProgress = () => setBuffered(readBufferedPercent(video));
    const onVolumeChange = () => setIsMuted(video.muted || video.volume === 0);

    video.addEventListener('play', onPlay);
    video.addEventListener('pause', onPause);
    video.addEventListener('timeupdate', onTimeUpdate);
    video.addEventListener('loadedmetadata', onLoaded);
    video.addEventListener('durationchange', onLoaded);
    video.addEventListener('ended', onEnded);
    video.addEventListener('error', onError);
    video.addEventListener('waiting', onWaiting);
    video.addEventListener('playing', onPlaying);
    video.addEventListener('progress', onProgress);
    video.addEventListener('volumechange', onVolumeChange);

    return () => {
      video.removeEventListener('play', onPlay);
      video.removeEventListener('pause', onPause);
      video.removeEventListener('timeupdate', onTimeUpdate);
      video.removeEventListener('loadedmetadata', onLoaded);
      video.removeEventListener('durationchange', onLoaded);
      video.removeEventListener('ended', onEnded);
      video.removeEventListener('error', onError);
      video.removeEventListener('waiting', onWaiting);
      video.removeEventListener('playing', onPlaying);
      video.removeEventListener('progress', onProgress);
      video.removeEventListener('volumechange', onVolumeChange);
    };
  }, [src, resetKey]);

  useEffect(() => {
    if (!options?.autoPlay || !canPlay) return;
    const video = videoRef.current;
    if (!video || !src) return;
    void video.play();
  }, [options?.autoPlay, canPlay, src, resetKey]);

  useEffect(() => {
    const video = videoRef.current;
    if (!video) return;
    video.loop = Boolean(options?.loop);
  }, [options?.loop, src, resetKey]);

  const togglePlay = useCallback(() => {
    const video = videoRef.current;
    if (!video || !src) return;
    if (video.paused || video.ended) {
      if (video.ended || hasEnded) {
        video.currentTime = 0;
        setHasEnded(false);
      }
      void video.play();
      return;
    }
    video.pause();
  }, [hasEnded, src]);

  const seek = useCallback(
    (ratio: number) => {
      const video = videoRef.current;
      if (!video || !duration) return;
      const next = Math.max(0, Math.min(duration, duration * ratio));
      video.currentTime = next;
      setCurrentTime(next);
      if (next < duration - 0.35) {
        setHasEnded(false);
      }
    },
    [duration]
  );

  const toggleMute = useCallback(() => {
    const video = videoRef.current;
    if (!video) return;
    video.muted = !video.muted;
    setIsMuted(video.muted);
  }, []);

  const toggleFullscreen = useCallback(() => {
    const video = videoRef.current;
    if (!video) return;
    if (document.fullscreenElement) {
      void document.exitFullscreen();
      return;
    }
    void video.requestFullscreen?.();
  }, []);

  const togglePictureInPicture = useCallback(async () => {
    const video = videoRef.current;
    if (!video) return;
    try {
      if (document.pictureInPictureElement) {
        await document.exitPictureInPicture();
        return;
      }
      await video.requestPictureInPicture?.();
    } catch {
      // PiP may be blocked by browser policy.
    }
  }, []);

  const progress = duration > 0 ? (currentTime / duration) * 100 : 0;

  return {
    videoRef,
    isPlaying,
    hasEnded,
    isBuffering,
    currentTime,
    duration,
    progress,
    buffered,
    togglePlay,
    seek,
    canPlay,
    isMuted,
    toggleMute,
    toggleFullscreen,
    togglePictureInPicture
  };
}

export { formatTime };

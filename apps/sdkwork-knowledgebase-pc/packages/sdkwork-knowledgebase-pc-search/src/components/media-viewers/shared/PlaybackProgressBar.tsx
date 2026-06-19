import React, { useCallback, useRef } from 'react';

export interface PlaybackProgressBarProps {
  progress: number;
  buffered?: number;
  onSeek: (ratio: number) => void;
  disabled?: boolean;
  variant?: 'default' | 'on-dark';
}

function clampRatio(value: number): number {
  return Math.max(0, Math.min(1, value));
}

export function PlaybackProgressBar({
  progress,
  buffered = 0,
  onSeek,
  disabled,
  variant = 'default'
}: PlaybackProgressBarProps) {
  const trackRef = useRef<HTMLButtonElement>(null);

  const seekFromPointer = useCallback(
    (clientX: number) => {
      const track = trackRef.current;
      if (!track || disabled) return;
      const rect = track.getBoundingClientRect();
      const ratio = clampRatio((clientX - rect.left) / rect.width);
      onSeek(ratio);
    },
    [disabled, onSeek]
  );

  const startDrag = useCallback(
    (event: React.MouseEvent<HTMLButtonElement>) => {
      if (disabled) return;
      event.preventDefault();
      seekFromPointer(event.clientX);

      const onMove = (moveEvent: MouseEvent) => seekFromPointer(moveEvent.clientX);
      const onUp = () => {
        window.removeEventListener('mousemove', onMove);
        window.removeEventListener('mouseup', onUp);
      };

      window.addEventListener('mousemove', onMove);
      window.addEventListener('mouseup', onUp);
    },
    [disabled, seekFromPointer]
  );

  return (
    <button
      ref={trackRef}
      type="button"
      className={`search-playback-progress ${variant === 'on-dark' ? 'search-playback-progress--on-dark' : ''}`}
      disabled={disabled}
      onMouseDown={startDrag}
      aria-label="播放进度"
    >
      <span className="search-playback-progress__track">
        {buffered > 0 && (
          <span className="search-playback-progress__buffer" style={{ width: `${buffered}%` }} />
        )}
        <span className="search-playback-progress__fill" style={{ width: `${progress}%` }} />
        <span className="search-playback-progress__thumb" style={{ left: `${progress}%` }} />
      </span>
    </button>
  );
}

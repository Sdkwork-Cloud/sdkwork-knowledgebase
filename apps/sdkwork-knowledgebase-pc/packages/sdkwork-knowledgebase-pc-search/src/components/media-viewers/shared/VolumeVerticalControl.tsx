import React, { useCallback, useRef } from 'react';
import { Volume2, VolumeX } from 'lucide-react';

export interface VolumeVerticalControlProps {
  volume: number;
  isMuted: boolean;
  onVolumeChange: (volume: number) => void;
  onMuteToggle: () => void;
  variant?: 'default' | 'on-dark';
}

function clampVolume(value: number): number {
  return Math.max(0, Math.min(1, value));
}

export function VolumeVerticalControl({
  volume,
  isMuted,
  onVolumeChange,
  onMuteToggle,
  variant = 'default'
}: VolumeVerticalControlProps) {
  const trackRef = useRef<HTMLDivElement>(null);
  const displayVolume = isMuted ? 0 : volume;
  const percent = Math.round(displayVolume * 100);

  const setVolumeFromPointer = useCallback(
    (clientY: number) => {
      const track = trackRef.current;
      if (!track) return;
      const rect = track.getBoundingClientRect();
      const ratio = clampVolume(1 - (clientY - rect.top) / rect.height);
      onVolumeChange(ratio);
    },
    [onVolumeChange]
  );

  const startDrag = useCallback(
    (event: React.MouseEvent<HTMLDivElement>) => {
      event.preventDefault();
      setVolumeFromPointer(event.clientY);

      const onMove = (moveEvent: MouseEvent) => setVolumeFromPointer(moveEvent.clientY);
      const onUp = () => {
        window.removeEventListener('mousemove', onMove);
        window.removeEventListener('mouseup', onUp);
      };

      window.addEventListener('mousemove', onMove);
      window.addEventListener('mouseup', onUp);
    },
    [setVolumeFromPointer]
  );

  return (
    <div
      className={`search-volume-control ${variant === 'on-dark' ? 'search-volume-control--on-dark' : ''}`}
    >
      <div className="search-volume-control__popover" role="group" aria-label="音量调节">
        <span className="search-volume-control__level">{percent}%</span>
        <div
          ref={trackRef}
          className="search-volume-control__track"
          onMouseDown={startDrag}
          role="slider"
          aria-label="音量"
          aria-valuemin={0}
          aria-valuemax={100}
          aria-valuenow={percent}
          tabIndex={0}
          onKeyDown={(event) => {
            if (event.key === 'ArrowUp' || event.key === 'ArrowRight') {
              event.preventDefault();
              onVolumeChange(clampVolume(displayVolume + 0.05));
            }
            if (event.key === 'ArrowDown' || event.key === 'ArrowLeft') {
              event.preventDefault();
              onVolumeChange(clampVolume(displayVolume - 0.05));
            }
          }}
        >
          <span className="search-volume-control__track-bg" aria-hidden />
          <span className="search-volume-control__fill" style={{ height: `${percent}%` }} aria-hidden />
          <span className="search-volume-control__thumb" style={{ bottom: `${percent}%` }} aria-hidden />
        </div>
      </div>
      <button
        type="button"
        className="search-volume-control__btn"
        onClick={onMuteToggle}
        aria-label={isMuted ? '取消静音' : '静音'}
      >
        {isMuted || volume === 0 ? <VolumeX className="w-4 h-4" /> : <Volume2 className="w-4 h-4" />}
      </button>
    </div>
  );
}

import React, { useEffect, useMemo, useState } from 'react';
import {
  ExternalLink,
  Fullscreen,
  ListMusic,
  Loader2,
  Pause,
  PictureInPicture2,
  Play,
  SkipBack,
  SkipForward,
  Video
} from 'lucide-react';
import {
  getMediaOrientation,
  orientationClass,
  resolveVideoDimensions
} from '../../utils/mediaAspect';
import { resolveVideoPlayback } from '../../utils/searchMediaViewerBridge';
import { openExternalUrl } from './openExternalUrl';
import { PlaybackProgressBar } from './shared/PlaybackProgressBar';
import { VolumeVerticalControl } from './shared/VolumeVerticalControl';
import type { SearchMediaViewerContentProps } from './types';
import { formatTime, useHtmlVideoPlayer } from './useHtmlVideoPlayer';

function buildStageAspectStyle(
  width: number,
  height: number,
  options?: { mini?: boolean }
): React.CSSProperties {
  if (options?.mini) {
    return {
      aspectRatio: `${width} / ${height}`,
      width: '100%',
      maxHeight: getMediaOrientation(width, height) === 'portrait' ? '240px' : '180px'
    };
  }

  const orientation = getMediaOrientation(width, height);
  const maxHeight =
    orientation === 'portrait' || orientation === 'ultratall'
      ? 'min(72vh, 640px)'
      : orientation === 'ultrawide'
        ? 'min(52vh, 420px)'
        : orientation === 'square'
          ? 'min(68vh, 560px)'
          : 'min(62vh, 560px)';

  return {
    aspectRatio: `${width} / ${height}`,
    width: orientation === 'portrait' || orientation === 'ultratall' ? 'auto' : '100%',
    maxWidth: '100%',
    maxHeight,
    margin: '0 auto'
  };
}

export function SearchVideoPlayerContent({
  item,
  layoutMode = 'expanded',
  onOpenWebLink,
  playlist
}: SearchMediaViewerContentProps) {
  const playback = resolveVideoPlayback(item);
  const externalUrl = item.url ?? item.previewUrl;
  const isDirect = playback.mode === 'direct' && Boolean(playback.url);
  const isMini = layoutMode === 'minimized';
  const [volume, setVolume] = useState(0.85);
  const [isMuted, setIsMuted] = useState(false);

  const frameDims = useMemo(() => resolveVideoDimensions(item), [item]);
  const videoOrientation = useMemo(
    () => getMediaOrientation(frameDims.width, frameDims.height),
    [frameDims.height, frameDims.width]
  );
  const stageAspectStyle = useMemo(
    () => buildStageAspectStyle(frameDims.width, frameDims.height, { mini: isMini }),
    [frameDims.height, frameDims.width, isMini]
  );
  const screenFillClass = isMini ? '' : 'search-video-player__screen--fill';
  const playerOrientationClass = orientationClass('search-video-player', videoOrientation);

  const {
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
    toggleFullscreen,
    togglePictureInPicture
  } = useHtmlVideoPlayer(isDirect ? playback.url : undefined, item.id, {
    loop: playlist?.playMode === 'loop-one',
    autoPlay: playlist?.autoPlay,
    onEnded: playlist?.onTrackEnded,
    onPlayingChange: (playing) => {
      playlist?.onPlaybackActiveChange?.(playing);
      if (playing) playlist?.onAutoPlayConsumed?.();
    }
  });

  useEffect(() => {
    const video = videoRef.current;
    if (!video) return;
    video.volume = isMuted ? 0 : volume;
    video.muted = isMuted;
  }, [videoRef, isMuted, volume, item.id]);

  if (playback.mode === 'embed' && playback.url) {
    return (
      <div
        className={`search-video-player search-video-player--embed ${playerOrientationClass} ${isMini ? 'search-video-player--mini' : ''} ${playlist ? 'search-video-player--with-playlist' : ''}`}
      >
        <div className="search-video-player__stage search-video-player__stage--adaptive">
          <div
            className={`search-video-player__screen search-video-player__screen--embed search-video-player__screen--adaptive ${screenFillClass}`.trim()}
            style={isMini ? stageAspectStyle : undefined}
          >
            <iframe
              key={item.id}
              src={playback.url}
              title={item.title}
              className="search-video-player__iframe"
              allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share"
              allowFullScreen
            />
          </div>
        </div>
      </div>
    );
  }

  if (isDirect) {
    const isActivelyPlaying = isPlaying && !hasEnded;
    const screenStateClass = isActivelyPlaying
      ? 'search-video-player__screen--playing'
      : hasEnded
        ? 'search-video-player__screen--ended'
        : 'search-video-player__screen--paused';

    return (
      <div
        className={`search-video-player search-video-player--native ${playerOrientationClass} ${isMini ? 'search-video-player--mini' : ''} ${playlist ? 'search-video-player--with-playlist' : ''}`}
      >
        <div className="search-video-player__stage search-video-player__stage--native">
          <div
            className={`search-video-player__screen search-video-player__screen--native search-video-player__screen--adaptive ${screenStateClass} ${screenFillClass}`.trim()}
            style={isMini ? stageAspectStyle : undefined}
          >
            <video
              ref={videoRef}
              key={item.id}
              src={playback.url}
              poster={playback.poster}
              playsInline
              preload="metadata"
              className="search-video-player__video"
              onClick={togglePlay}
            />
            <div className="search-video-player__shade" aria-hidden />
            {isBuffering && isActivelyPlaying && (
              <div className="search-video-player__buffer-indicator" aria-hidden>
                <Loader2 className="w-8 h-8 animate-spin" />
              </div>
            )}
            <div className={`search-video-player__overlay ${isMini ? 'search-video-player__overlay--mini' : ''}`}>
              {!isMini && !isActivelyPlaying && (
                <div className="search-video-player__center-action">
                  <button
                    type="button"
                    className="search-video-player__center-play"
                    onClick={togglePlay}
                    aria-label={hasEnded ? '重播' : '播放'}
                  >
                    {isBuffering && !hasEnded ? (
                      <Loader2 className="w-8 h-8 animate-spin" />
                    ) : (
                      <Play className="w-8 h-8 fill-current ml-0.5" />
                    )}
                  </button>
                  {hasEnded && <span className="search-video-player__replay-label">重播</span>}
                </div>
              )}
              <div className="search-video-player__controls-bar">
                <div className={`search-video-player__controls-bottom ${isMini ? 'search-video-player__controls-bottom--mini' : ''}`}>
                  {playlist && !isMini && (
                    <button
                      type="button"
                      className="search-video-player__control-btn search-video-player__control-btn--track"
                      onClick={playlist.onPreviousTrack}
                      disabled={!playlist.canGoPrevious}
                      aria-label="上一个视频"
                    >
                      <SkipBack className="w-4 h-4" />
                    </button>
                  )}
                  <button type="button" className="search-video-player__control-btn" onClick={togglePlay} disabled={!canPlay}>
                    {isActivelyPlaying ? <Pause className="w-4 h-4" /> : <Play className="w-4 h-4 fill-current" />}
                  </button>
                  {playlist && !isMini && (
                    <button
                      type="button"
                      className="search-video-player__control-btn search-video-player__control-btn--track"
                      onClick={playlist.onNextTrack}
                      disabled={!playlist.canGoNext}
                      aria-label="下一个视频"
                    >
                      <SkipForward className="w-4 h-4" />
                    </button>
                  )}
                  {!isMini && <span className="search-video-player__time">{formatTime(currentTime)}</span>}
                  <PlaybackProgressBar
                    progress={progress}
                    buffered={buffered}
                    onSeek={seek}
                    disabled={!canPlay}
                    variant="on-dark"
                  />
                  {!isMini && <span className="search-video-player__time">{formatTime(duration)}</span>}
                  {!isMini && (
                    <VolumeVerticalControl
                      volume={volume}
                      isMuted={isMuted}
                      onVolumeChange={(next) => {
                        setIsMuted(false);
                        setVolume(next);
                      }}
                      onMuteToggle={() => setIsMuted((value) => !value)}
                      variant="on-dark"
                    />
                  )}
                  {!isMini && (
                    <>
                      <button type="button" className="search-video-player__control-btn" onClick={togglePictureInPicture} aria-label="画中画">
                        <PictureInPicture2 className="w-4 h-4" />
                      </button>
                      <button type="button" className="search-video-player__control-btn" onClick={toggleFullscreen} aria-label="视频全屏">
                        <Fullscreen className="w-4 h-4" />
                      </button>
                      {playlist && (
                        <button
                          type="button"
                          className={`search-video-player__control-btn ${playlist.isPlaylistOpen ? 'search-video-player__control-btn--active' : ''}`}
                          onClick={playlist.onPlaylistToggle}
                          aria-label={playlist.isPlaylistOpen ? '收起视频列表' : '展开视频列表'}
                        >
                          <ListMusic className="w-4 h-4" />
                        </button>
                      )}
                    </>
                  )}
                  {isMini && <span className="search-video-player__time search-video-player__time--mini">{formatTime(currentTime)}</span>}
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div
      className={`search-video-player search-video-player--fallback ${playerOrientationClass} ${isMini ? 'search-video-player--mini' : ''} ${playlist ? 'search-video-player--with-playlist' : ''}`}
    >
      <div className="search-video-player__stage search-video-player__stage--adaptive">
        <div
          className={`search-video-player__screen search-video-player__screen--fallback search-video-player__screen--adaptive ${screenFillClass}`.trim()}
          style={isMini ? stageAspectStyle : undefined}
        >
          {playback.poster ? (
            <img src={playback.poster} alt={item.title} className="search-video-player__poster" />
          ) : (
            <div className="search-video-player__empty">
              <Video className="w-12 h-12 opacity-35" />
              <p>暂无视频预览</p>
            </div>
          )}
          {externalUrl && (
            <button
              type="button"
              className="search-video-player__center-play"
              onClick={() => openExternalUrl(externalUrl, item.title, onOpenWebLink)}
              aria-label="在浏览器中播放"
            >
              <Play className="w-8 h-8 fill-current ml-0.5" />
            </button>
          )}
        </div>
      </div>
      {isMini && externalUrl && (
        <button
          type="button"
          className="search-media-viewer-link-btn"
          onClick={() => openExternalUrl(externalUrl, item.title, onOpenWebLink)}
        >
          <ExternalLink className="w-3.5 h-3.5" />
          在浏览器中播放
        </button>
      )}
    </div>
  );
}

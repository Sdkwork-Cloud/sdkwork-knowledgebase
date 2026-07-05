import type { DocumentMeta } from '@sdkwork/sdkwork-knowledgebase-pc-knowledgebase/services/document';
import { shouldUseKnowledgebaseDemoFallback } from 'sdkwork-knowledgebase-pc-core';
import type { SearchMediaItem, SearchRelatedMedia } from '../types';
import {
  buildDemoKbMeetingTranscript,
  buildDemoKbMusicLyrics,
  buildDemoMusicLyrics,
  buildDemoPodcastTranscript,
  buildMeetingMinutesTranscript
} from './demoMediaTimedText';

const MEDIA_DOC_TYPES = new Set(['image', 'video', 'audio', 'music']);

const DEMO_AUDIO_URL = 'https://www.soundhelix.com/examples/mp3/SoundHelix-Song-1.mp3';
const DEMO_AUDIO_URL_2 = 'https://www.soundhelix.com/examples/mp3/SoundHelix-Song-2.mp3';
const DEMO_MUSIC_URL = 'https://www.soundhelix.com/examples/mp3/SoundHelix-Song-3.mp3';
const DEMO_MUSIC_URL_2 = 'https://www.soundhelix.com/examples/mp3/SoundHelix-Song-4.mp3';
const DEMO_VIDEO_MP4 = 'https://commondatastorage.googleapis.com/gtv-videos-bucket/sample/BigBuckBunny.mp4';

const EMPTY_MEDIA: SearchRelatedMedia = {
  images: [],
  videos: [],
  audio: [],
  music: [],
  products: []
};

const IMAGE_SIZE_PRESETS = [
  { width: 1920, height: 1080, shape: '横图 16:9' },
  { width: 1080, height: 1920, shape: '竖图 9:16' },
  { width: 1200, height: 1200, shape: '方图 1:1' },
  { width: 2560, height: 800, shape: '超宽图 3.2:1' },
  { width: 1600, height: 1200, shape: '横图 4:3' },
  { width: 900, height: 1350, shape: '竖图 2:3' },
  { width: 600, height: 2000, shape: '超长竖图' },
  { width: 3200, height: 900, shape: '全景横图' }
] as const;

const VIDEO_SIZE_PRESETS = [
  { width: 1920, height: 1080, shape: '横屏 16:9' },
  { width: 1080, height: 1920, shape: '竖屏 9:16' },
  { width: 1080, height: 1080, shape: '方屏 1:1' },
  { width: 2560, height: 800, shape: '超宽 21:9' },
  { width: 720, height: 1280, shape: '竖屏 9:16' },
  { width: 1440, height: 1080, shape: '横屏 4:3' }
] as const;

const COVER_SIZE_PRESETS = [
  { width: 320, height: 320, shape: '方版封面' },
  { width: 360, height: 640, shape: '竖版封面 9:16' },
  { width: 480, height: 360, shape: '横版封面 4:3' },
  { width: 300, height: 500, shape: '竖版封面 3:5' },
  { width: 640, height: 360, shape: '横版封面 16:9' },
  { width: 400, height: 400, shape: '方形封面' }
] as const;

function hashSeed(text: string): number {
  let hash = 0;
  for (let i = 0; i < text.length; i++) {
    hash = (hash << 5) - hash + text.charCodeAt(i);
    hash |= 0;
  }
  return Math.abs(hash);
}

function pickVideoPreset(seedText: string, index = 0) {
  const offset = hashSeed(seedText) + index;
  return VIDEO_SIZE_PRESETS[offset % VIDEO_SIZE_PRESETS.length];
}

function pickCoverPreset(seedText: string, index = 0) {
  const offset = hashSeed(seedText) + index;
  return COVER_SIZE_PRESETS[offset % COVER_SIZE_PRESETS.length];
}

function buildCoverThumb(seed: string | number, width: number, height: number) {
  const encoded = encodeURIComponent(String(seed));
  const thumbW = Math.max(Math.round(width / 2), 96);
  const thumbH = Math.max(Math.round(height / 2), 96);
  return `https://picsum.photos/seed/${encoded}/${thumbW}/${thumbH}`;
}

function buildVideoThumb(seed: string | number, width: number, height: number) {
  return buildCoverThumb(`v-${seed}`, width, height);
}

function buildKbMediaItems(docs: DocumentMeta[], allowDemo: boolean): SearchRelatedMedia {
  const images: SearchMediaItem[] = [];
  const videos: SearchMediaItem[] = [];
  const audio: SearchMediaItem[] = [];
  const music: SearchMediaItem[] = [];

  for (const doc of docs) {
    if (!MEDIA_DOC_TYPES.has(doc.type)) continue;

    const base = {
      id: `kb-media-${doc.id}`,
      title: doc.title,
      source: 'kb' as const,
      kbId: doc.kbId,
      docId: doc.id,
      docType: doc.type,
      snippet: `来自知识库 · ${doc.author || '未知作者'}`
    };

    if (doc.type === 'image') {
      const previewUrl = doc.url ?? (allowDemo
        ? buildCoverThumb(`kb-img-${doc.id}`, IMAGE_SIZE_PRESETS[0].width, IMAGE_SIZE_PRESETS[0].height)
        : undefined);
      if (!previewUrl) continue;
      const preset = IMAGE_SIZE_PRESETS[hashSeed(doc.id) % IMAGE_SIZE_PRESETS.length];
      images.push({
        ...base,
        category: 'image',
        thumbnailUrl: previewUrl,
        previewUrl,
        imageWidth: preset.width,
        imageHeight: preset.height,
        snippet: `${preset.shape} · 来自知识库 · ${doc.author || '未知作者'}`
      });
    } else if (doc.type === 'video') {
      const previewUrl = doc.url ?? (allowDemo ? DEMO_VIDEO_MP4 : undefined);
      if (!previewUrl) continue;
      const preset = pickVideoPreset(doc.id);
      videos.push({
        ...base,
        category: 'video',
        thumbnailUrl: buildVideoThumb(doc.id, preset.width, preset.height),
        previewUrl,
        videoWidth: preset.width,
        videoHeight: preset.height,
        duration: allowDemo ? '03:24' : undefined,
        snippet: `${preset.shape} · 来自知识库 · ${doc.author || '未知作者'}`
      });
    } else if (doc.type === 'audio') {
      const previewUrl = doc.url ?? (allowDemo ? DEMO_AUDIO_URL : undefined);
      if (!previewUrl) continue;
      const preset = pickCoverPreset(doc.id);
      audio.push({
        ...base,
        category: 'audio',
        audioKind: 'recording',
        thumbnailUrl: buildCoverThumb(`a-${doc.id}`, preset.width, preset.height),
        coverWidth: preset.width,
        coverHeight: preset.height,
        previewUrl,
        duration: allowDemo ? '12:08' : undefined,
        transcript: allowDemo ? buildDemoKbMeetingTranscript(doc.title) : undefined,
        snippet: `${preset.shape} · 会议纪要 · 语音转写`
      });
    } else if (doc.type === 'music') {
      const previewUrl = doc.url ?? (allowDemo ? DEMO_MUSIC_URL : undefined);
      if (!previewUrl) continue;
      music.push({
        ...base,
        category: 'music',
        artist: doc.author || '未知艺术家',
        thumbnailUrl: doc.url ?? (allowDemo
          ? `https://picsum.photos/seed/m-${encodeURIComponent(doc.id)}/480/480`
          : buildCoverThumb(`m-${doc.id}`, 320, 320)),
        previewUrl,
        duration: allowDemo ? '04:12' : undefined,
        lyrics: allowDemo ? buildDemoKbMusicLyrics(doc.title) : undefined,
        snippet: allowDemo ? '含同步歌词' : '来自知识库'
      });
    }
  }

  return { ...EMPTY_MEDIA, images, videos, audio, music };
}

function buildWebMediaItems(query: string, lower: string): SearchRelatedMedia {
  const seed = hashSeed(query);
  const topic = query.length > 18 ? `${query.slice(0, 18)}…` : query;

  const images: SearchMediaItem[] = Array.from({ length: 6 }, (_, i) => {
    const preset = IMAGE_SIZE_PRESETS[i % IMAGE_SIZE_PRESETS.length];
    return {
      id: `web-img-${seed}-${i}`,
      category: 'image',
      title: `${topic} 相关图示 ${i + 1}`,
      source: 'web',
      url: `https://unsplash.com/s/photos/${encodeURIComponent(query)}`,
      thumbnailUrl: buildCoverThumb(`img-${seed}-${i}`, preset.width, preset.height),
      previewUrl: buildCoverThumb(`img-${seed}-${i}`, preset.width, preset.height),
      snippet: `${preset.shape} · 网络检索 · 与「${query}」相关的高清参考图`,
      imageWidth: preset.width,
      imageHeight: preset.height
    };
  });

  const videos: SearchMediaItem[] = VIDEO_SIZE_PRESETS.map((preset, index) => {
    const playback =
      index === 0
        ? { previewUrl: 'https://www.youtube.com/watch?v=dQw4w9WgXcQ', snippet: 'YouTube · 横屏讲解' }
        : index === 1
          ? { previewUrl: 'https://www.bilibili.com/video/BV1GJ411x7h7', snippet: 'Bilibili · 竖屏短视频' }
          : index === 2
            ? { previewUrl: DEMO_VIDEO_MP4, snippet: 'MP4 · 方屏样片' }
            : index === 3
              ? { previewUrl: DEMO_VIDEO_MP4, snippet: 'MP4 · 超宽 cinematic' }
              : index === 4
                ? { previewUrl: DEMO_VIDEO_MP4, snippet: 'MP4 · 手机竖屏录制' }
                : { previewUrl: DEMO_VIDEO_MP4, snippet: 'MP4 · 经典 4:3' };

    return {
      id: `web-vid-${seed}-${index}`,
      category: 'video',
      title:
        index === 0
          ? `${topic} — 深度解读与实操演示`
          : index === 1
            ? `${topic} · 竖屏快剪`
            : index === 2
              ? `${topic} · 方屏直播回放`
              : index === 3
                ? `${topic} · 超宽银幕预告`
                : index === 4
                  ? `${topic} · 手机竖屏 Vlog`
                  : `${topic} · 4:3 经典画幅`,
      source: 'web',
      url: 'https://www.youtube.com/results?search_query=' + encodeURIComponent(query),
      thumbnailUrl: buildVideoThumb(`${seed}-${index}`, preset.width, preset.height),
      previewUrl: playback.previewUrl,
      videoWidth: preset.width,
      videoHeight: preset.height,
      duration: ['08:42', '00:58', '05:16', '02:24', '01:12', '15:20'][index],
      snippet: `${preset.shape} · ${playback.snippet}`
    };
  });

  const audio: SearchMediaItem[] = COVER_SIZE_PRESETS.slice(0, 5).map((preset, index) => {
    const isRecording = index > 0;
    const meetingVariant = index - 1;
    return {
      id: `web-aud-${seed}-${index}`,
      category: 'audio',
      title:
        index === 0
          ? `播客 · 聊聊${topic}`
          : index === 1
            ? `${topic} · 产品评审会议纪要（语音转写）`
            : index === 2
              ? `${topic} · 周会同步录音`
              : index === 3
                ? `${topic} · 需求澄清会议纪要`
                : `${topic} · 访谈录音（自动转写）`,
      source: 'web',
      audioKind: isRecording ? 'recording' : 'podcast',
      url:
        index === 0
          ? 'https://podcasts.apple.com/search?term=' + encodeURIComponent(query)
          : undefined,
      thumbnailUrl: buildCoverThumb(`a${seed}-${index}`, preset.width, preset.height),
      coverWidth: preset.width,
      coverHeight: preset.height,
      previewUrl: index % 2 === 0 ? DEMO_AUDIO_URL : DEMO_AUDIO_URL_2,
      duration: ['32:10', '18:45', '24:06', '09:32', '45:18'][index],
      transcript: isRecording
        ? buildMeetingMinutesTranscript(topic, meetingVariant)
        : buildDemoPodcastTranscript(),
      snippet: `${preset.shape} · ${isRecording ? '会议纪要 · 语音转写' : '播客 · 含字幕'}`
    };
  });

  const music: SearchMediaItem[] = [
    {
      id: `web-mus-${seed}-0`,
      category: 'music',
      title: `${topic} · 氛围旋律`,
      artist: 'SoundHelix',
      source: 'web',
      thumbnailUrl: `https://picsum.photos/seed/m${seed}/480/480`,
      previewUrl: DEMO_MUSIC_URL,
      duration: '06:12',
      lyrics: buildDemoMusicLyrics(seed),
      snippet: '网络检索 · 轻音乐 · 含歌词'
    },
    {
      id: `web-mus-${seed}-1`,
      category: 'music',
      title: `Focus on ${topic}`,
      artist: 'Demo Ensemble',
      source: 'web',
      thumbnailUrl: `https://picsum.photos/seed/m${seed + 1}/480/480`,
      previewUrl: DEMO_MUSIC_URL_2,
      duration: '04:38',
      lyrics: buildDemoMusicLyrics(seed + 1),
      snippet: '网络检索 · 专注音乐 · 含歌词'
    },
    {
      id: `web-mus-${seed}-2`,
      category: 'music',
      title: `${topic} · 中文单曲`,
      artist: 'Demo Studio',
      source: 'web',
      thumbnailUrl: `https://picsum.photos/seed/m${seed + 2}/480/480`,
      previewUrl: DEMO_MUSIC_URL,
      duration: '03:28',
      lyrics: buildDemoMusicLyrics(seed + 2),
      snippet: '网络检索 · 中文歌词 · 同步滚动'
    }
  ];

  const productKeywords =
    lower.includes('买') ||
    lower.includes('商品') ||
    lower.includes('价格') ||
    lower.includes('shop') ||
    lower.includes('product');

  const productBase = (index: number, overrides: Partial<SearchMediaItem>): SearchMediaItem => ({
    id: `web-prod-${seed}-${index}`,
    category: 'product',
    title: `${topic} 精选商品 ${index + 1}`,
    source: 'web',
    thumbnailUrl: `https://picsum.photos/seed/p${seed + index}/640/640`,
    galleryUrls: [
      `https://picsum.photos/seed/p${seed + index}a/640/640`,
      `https://picsum.photos/seed/p${seed + index}b/640/640`,
      `https://picsum.photos/seed/p${seed + index}c/640/640`
    ],
    price: '¥399',
    originalPrice: '¥599',
    merchant: '官方旗舰',
    rating: 4.7,
    reviewCount: 1200 + seed + index * 37,
    tags: ['热销', '包邮', '7天无理由'],
    highlights: ['核心功能完整', '适合入门到进阶', '支持多平台同步使用'],
    shippingNote: '包邮 · 预计 1-3 天送达',
    description: `围绕「${query}」检索到的相关商品，聚合多平台销量与口碑数据。适合作为采购参考，具体规格以原平台详情页为准。`,
    specs: [
      { label: '品牌', value: '官方授权' },
      { label: '型号', value: `KB-${seed + index}` },
      { label: '适用场景', value: topic },
      { label: '保修', value: '全国联保 1 年' }
    ],
    url: 'https://www.amazon.com/s?k=' + encodeURIComponent(query),
    snippet: '综合电商检索',
    ...overrides
  });

  const products: SearchMediaItem[] = productKeywords
    ? [
        productBase(0, {
          title: `${topic} 专业版套装`,
          price: '¥1,299',
          originalPrice: '¥1,699',
          merchant: 'Amazon 精选',
          rating: 4.6,
          reviewCount: 2840,
          tags: ['专业版', '包邮', '企业采购'],
          highlights: ['全功能授权', '含 1 年技术支持', '支持团队多人协作']
        }),
        productBase(1, {
          title: `${topic} 入门学习套件`,
          price: '¥399',
          originalPrice: '¥529',
          merchant: '京东自营',
          rating: 4.8,
          reviewCount: 5621,
          tags: ['入门', '当日达', '官方旗舰'],
          shippingNote: '京东配送 · 当日达'
        })
      ]
    : [
        productBase(0, {
          title: `与「${topic}」相关的精选工具/书籍`,
          price: '¥268起',
          originalPrice: '¥368',
          merchant: '多平台聚合',
          rating: 4.5,
          reviewCount: 986
        }),
        productBase(1, {
          title: `${topic} 配件与周边`,
          price: '¥89起',
          originalPrice: '¥129',
          merchant: '淘宝热销店',
          rating: 4.7,
          reviewCount: 3412
        }),
        productBase(2, {
          title: `企业级 ${topic} 解决方案`,
          price: '询价',
          originalPrice: undefined,
          merchant: '1688 企业购',
          rating: 4.4,
          reviewCount: 128,
          tags: ['B2B', '批发', '可开票'],
          highlights: ['支持批量采购', '提供部署方案', '专属客户经理']
        })
      ];

  return { ...EMPTY_MEDIA, images, videos, audio, music, products };
}

function mergeMedia(a: SearchRelatedMedia, b: SearchRelatedMedia): SearchRelatedMedia {
  return {
    images: [...a.images, ...b.images].slice(0, 8),
    videos: [...a.videos, ...b.videos].slice(0, 6),
    audio: [...a.audio, ...b.audio].slice(0, 8),
    music: [...a.music, ...b.music].slice(0, 6),
    products: [...a.products, ...b.products].slice(0, 6)
  };
}

function countKbMedia(media: SearchRelatedMedia): number {
  return media.images.length + media.videos.length + media.audio.length + media.music.length;
}

export function buildRelatedMedia(
  query: string,
  docs: DocumentMeta[],
  webSearchEnabled: boolean
): SearchRelatedMedia {
  const lower = query.toLowerCase();
  const allowDemo = shouldUseKnowledgebaseDemoFallback();
  const kbMedia = buildKbMediaItems(docs, allowDemo);

  if (!webSearchEnabled && countKbMedia(kbMedia) === 0) {
    return EMPTY_MEDIA;
  }

  if (!webSearchEnabled || !allowDemo) {
    return kbMedia;
  }

  const webMedia = buildWebMediaItems(query, lower);
  return mergeMedia(kbMedia, webMedia);
}

import type { MediaTimedLine } from '../types';

const DEMO_MUSIC_LYRICS: MediaTimedLine[] = [
  { startTime: 0, text: '♪ 前奏 · 渐入' },
  { startTime: 14, text: 'Light begins to fill the room tonight' },
  { startTime: 28, text: 'Every shadow learns to fade away' },
  { startTime: 42, text: 'We are moving with the rhythm of the tide' },
  { startTime: 56, text: 'Hold the moment, let it breathe' },
  { startTime: 70, text: '星光落在你的眼眸' },
  { startTime: 84, text: 'And all the noise becomes a quiet sea' },
  { startTime: 98, text: 'Sing it softly, sing it true' },
  { startTime: 112, text: '我们在旋律里慢慢靠近' },
  { startTime: 126, text: 'Every heartbeat finds its melody' },
  { startTime: 140, text: 'Turn the page, another verse begins' },
  { startTime: 154, text: '让时间停在这一段副歌' },
  { startTime: 168, text: 'We are rising with the morning sun' },
  { startTime: 182, text: 'Carry on until the day is done' },
  { startTime: 196, text: '♪ 间奏' },
  { startTime: 224, text: 'When the night returns we start again' },
  { startTime: 248, text: 'No beginning and no end' },
  { startTime: 272, text: 'Just this song between the stars and sand' },
  { startTime: 296, text: '最后一遍 · 和声渐弱' },
  { startTime: 320, text: '♪ 尾奏 · 渐远' }
];

const DEMO_MUSIC_LYRICS_ALT: MediaTimedLine[] = [
  { startTime: 0, text: '♪ Intro' },
  { startTime: 10, text: 'Focus on the path ahead of you' },
  { startTime: 24, text: 'Step by step the world grows clear' },
  { startTime: 38, text: 'Keep your mind on what you want to do' },
  { startTime: 52, text: 'Deep breath in, let the doubt disappear' },
  { startTime: 66, text: '专注此刻 · 心流渐起' },
  { startTime: 80, text: 'Every note becomes a guiding line' },
  { startTime: 94, text: 'Build the future one beat at a time' },
  { startTime: 108, text: 'Stay with me through the quiet hours' },
  { startTime: 122, text: '直到黎明把轮廓照亮' },
  { startTime: 136, text: '♪ Outro' }
];

const DEMO_MUSIC_LYRICS_ZH: MediaTimedLine[] = [
  { startTime: 0, text: '♪ 前奏' },
  { startTime: 8, text: '风吹过城市的边缘' },
  { startTime: 22, text: '你在灯光里回头' },
  { startTime: 36, text: '像一段未写完的故事' },
  { startTime: 50, text: '副歌 · 我们在此刻相遇' },
  { startTime: 64, text: '不必追问明天去向哪里' },
  { startTime: 78, text: '只要旋律还在延续' },
  { startTime: 92, text: '心就不会迷失方向' },
  { startTime: 106, text: '♪ 间奏' },
  { startTime: 128, text: '最后一遍 · 和声' },
  { startTime: 148, text: '♪ 尾奏' }
];

const DEMO_PODCAST_TRANSCRIPT: MediaTimedLine[] = [
  { startTime: 0, speaker: '主播', text: '欢迎收听本期节目。' },
  { startTime: 8, speaker: '主播', text: '今天聊聊知识检索如何与多媒体结果结合。' },
  { startTime: 18, speaker: '主播', text: '当用户提问时，除了文字答案，相关图片、音视频也会一并呈现。' },
  { startTime: 32, speaker: '主播', text: '关键是预览体验要专业：能播、能最小化、能看歌词或字幕。' },
  { startTime: 48, speaker: '主播', text: '我们下一期继续深入 SDK 与 OpenAPI 话题。' }
];

function buildProductReviewMinutes(topic: string): MediaTimedLine[] {
  return [
    { startTime: 0, speaker: '系统', text: '【产品评审会议纪要 · AI 语音转写】' },
    { startTime: 3, speaker: '主持人', text: `各位好，今天评审与「${topic}」相关的检索体验方案。` },
    { startTime: 12, speaker: '产品经理', text: '目标是让用户在搜索对话里直接预览图片、视频、音频和音乐。' },
    { startTime: 22, speaker: '设计', text: '音乐播放器需要同步歌词，窗口有歌词时要放大，只让歌词区滚动。' },
    { startTime: 34, speaker: '研发', text: '音频侧支持录音文件的语音转文字字幕，会议纪要场景要能逐句高亮。' },
    { startTime: 46, speaker: '产品经理', text: '点击某一句字幕可以跳转到对应时间点，和歌词交互保持一致。' },
    { startTime: 58, speaker: '测试', text: '请准备横屏、竖屏、带字幕、带歌词的 mock 数据做回归。' },
    { startTime: 70, speaker: '主持人', text: '结论：先做 demo 验证，再接入知识库真实字幕文件。' },
    { startTime: 82, speaker: '系统', text: '【会议结束 · 待整理纪要】' }
  ];
}

function buildWeeklySyncMinutes(topic: string): MediaTimedLine[] {
  return [
    { startTime: 0, speaker: '系统', text: '【周会同步 · 语音转写】' },
    { startTime: 4, speaker: '主持人', text: '先过一下本周与知识库搜索相关的进展。' },
    { startTime: 11, speaker: '研发 A', text: '媒体预览弹窗已支持最小化到右下角持续播放。' },
    { startTime: 20, speaker: '研发 B', text: `${topic} 相关结果里，视频和音频 mock 已覆盖多种画幅。` },
    { startTime: 31, speaker: '产品', text: '下一步重点：音乐歌词、录音会议纪要字幕的体验打磨。' },
    { startTime: 42, speaker: '设计', text: '字幕面板建议区分播客和会议两种视觉层级。' },
    { startTime: 53, speaker: '主持人', text: '好的，会后我会把 action item 发到群里。' },
    { startTime: 62, speaker: '系统', text: '【录音结束】' }
  ];
}

function buildRequirementMinutes(topic: string): MediaTimedLine[] {
  return [
    { startTime: 0, speaker: '系统', text: '【需求澄清会议纪要 · 自动转写】' },
    { startTime: 3, speaker: '业务', text: `我们需要在搜索场景里展示与「${topic}」有关的会议录音。` },
    { startTime: 14, speaker: '业务', text: '用户上传的 MP3 经 ASR 转写后，要能像字幕一样同步播放。' },
    { startTime: 26, speaker: '研发', text: '数据结构用 startTime + speaker + text，前端按时间轴高亮。' },
    { startTime: 38, speaker: '产品', text: '会议类录音标题和标签要明确，例如「会议纪要」「语音转写」。' },
    { startTime: 50, speaker: '业务', text: '纪要里发言人要保留，方便会后检索是谁说的。' },
    { startTime: 62, speaker: '研发', text: '收到，demo 里先模拟三份不同风格的会议纪要。' },
    { startTime: 74, speaker: '系统', text: '【会议结束】' }
  ];
}

function buildInterviewMinutes(topic: string): MediaTimedLine[] {
  return [
    { startTime: 0, speaker: '系统', text: '【访谈录音 · 语音转写】' },
    { startTime: 4, speaker: '采访者', text: `请先介绍一下你们在「${topic}」方向上的实践。` },
    { startTime: 14, speaker: '嘉宾', text: '我们把检索结果里的多媒体预览做成了独立的播放器组件。' },
    { startTime: 25, speaker: '采访者', text: '录音文件如何和字幕联动？' },
    { startTime: 32, speaker: '嘉宾', text: '和歌词一样，按播放进度自动滚动，支持点击跳转。' },
    { startTime: 44, speaker: '采访者', text: '会议纪要场景有什么特别之处？' },
    { startTime: 51, speaker: '嘉宾', text: '需要展示发言人，并标注「会议纪要」「自动转写」等来源信息。' },
    { startTime: 63, speaker: '系统', text: '【访谈结束】' }
  ];
}

const MEETING_BUILDERS = [
  buildProductReviewMinutes,
  buildWeeklySyncMinutes,
  buildRequirementMinutes,
  buildInterviewMinutes
] as const;

export function buildDemoMusicLyrics(seed: number): MediaTimedLine[] {
  if (seed % 3 === 0) return DEMO_MUSIC_LYRICS;
  if (seed % 3 === 1) return DEMO_MUSIC_LYRICS_ALT;
  return DEMO_MUSIC_LYRICS_ZH;
}

export function buildMeetingMinutesTranscript(topic: string, variant = 0): MediaTimedLine[] {
  const builder = MEETING_BUILDERS[Math.abs(variant) % MEETING_BUILDERS.length];
  return builder(topic);
}

/** @deprecated Use buildMeetingMinutesTranscript */
export function buildDemoRecordingTranscript(): MediaTimedLine[] {
  return buildProductReviewMinutes('检索体验');
}

export function buildDemoPodcastTranscript(): MediaTimedLine[] {
  return DEMO_PODCAST_TRANSCRIPT;
}

export function buildDemoKbMusicLyrics(title: string): MediaTimedLine[] {
  return [{ startTime: 0, text: `♪ ${title}` }, ...DEMO_MUSIC_LYRICS.slice(1, 14)];
}

export function buildDemoKbMeetingTranscript(title: string): MediaTimedLine[] {
  const lines = buildProductReviewMinutes(title);
  lines[0] = { ...lines[0], text: `【知识库录音 · ${title} · AI 转写】` };
  return lines;
}

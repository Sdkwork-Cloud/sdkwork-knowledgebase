import { isKnowledgebaseApiAvailable } from 'sdkwork-knowledgebase-pc-core';
import * as KnowledgebaseDocumentApiBridge from './knowledgebaseDocumentApiBridge';
import * as KnowledgeGitImportService from './knowledgeGitImportService';
import * as KnowledgeFileUploadService from './knowledgeFileUploadService';

export interface DocumentMeta {
  id: string;
  title: string;
  type: 'richtext' | 'code' | 'markdown' | 'file' | 'image' | 'audio' | 'video' | 'folder' | 'pdf' | 'music';
  updatedAt: string;
  author: string;
  kbId?: string;
  size?: string;
  url?: string;
  content?: string;
  parentId?: string | null;
  order?: number;
  isPinned?: boolean;
  tags?: string[];
}

export interface FolderNode {
  id: string;
  title: string;
  type: 'folder';
  children: (FolderNode | DocumentMeta)[];
  parentId?: string | null;
  updatedAt?: string;
  author?: string;
  isPinned?: boolean;
  tags?: string[];
}

export interface KnowledgeBase {
  id: string;
  title: string;
  icon?: string;
  avatar?: string;
  type?: 'team' | 'personal' | 'public';
  isDeployed?: boolean;
  deployedUrl?: string;
  customDomain?: string;
  siteLogo?: string;
  siteName?: string;
  // Model Settings
  provider?: string;
  modelName?: string;
  temperature?: number;
  maxTokens?: number;
  systemPrompt?: string;
  // Permissions Settings
  publicPermission?: 'none' | 'read' | 'write' | 'admin';
  guestLinkEnabled?: boolean;
}

export interface MarketKnowledgeBase {
  id: string;
  title: string;
  icon: string;
  description: string;
  author: string;
  tags: string[];
  subscribersCount: number;
  documentsCount: number;
  provider: string;
  modelName: string;
  isSubscribed?: boolean;
}

// ============== MOCK DATA STORAGE ==============

async function withApiFallback<T>(
  apiCall: () => Promise<T>,
  mockCall: () => Promise<T>,
): Promise<T> {
  if (!isKnowledgebaseApiAvailable()) {
    return mockCall();
  }
  return apiCall();
}

let mockKbs: { team: KnowledgeBase[], personal: KnowledgeBase[], public: KnowledgeBase[] } = {
  team: [
    { id: '1', title: 'Product Docs', icon: '🚀', type: 'team', provider: 'Google', modelName: 'gemini-1.5-flash', temperature: 0.7, maxTokens: 2048, publicPermission: 'none', guestLinkEnabled: false },
    { id: '2', title: 'Engineering', icon: '💻', type: 'team', provider: 'Google', modelName: 'gemini-1.5-pro', temperature: 0.5, maxTokens: 4096, publicPermission: 'none', guestLinkEnabled: false },
    { id: '3', title: 'Design Resources', icon: '🎨', type: 'team', provider: 'DeepSeek', modelName: 'deepseek-chat', temperature: 0.8, maxTokens: 2048, publicPermission: 'none', guestLinkEnabled: false },
    { id: 't4', title: 'Marketing Campaign 2026', icon: '📢', type: 'team' },
    { id: 't5', title: 'Sales Playbooks', icon: '📈', type: 'team' },
    { id: 't6', title: 'Customer Success', icon: '🤝', type: 'team' },
    { id: 't7', title: 'HR & Policies', icon: '📋', type: 'team' },
    { id: 't8', title: 'Legal & Compliance', icon: '⚖️', type: 'team' },
    { id: 't9', title: 'Research & Dev', icon: '🔬', type: 'team' },
    { id: 't10', title: 'Operations Manual', icon: '⚙️', type: 'team' },
    { id: 't11', title: 'Q1 Strategy Planning', icon: '🎯', type: 'team' },
    { id: 't12', title: 'Security Audits', icon: '🔒', type: 'team' },
    { id: 't13', title: 'Cloud Infrastructure', icon: '☁️', type: 'team' }
  ],
  personal: [
    { id: '4', title: 'My Notes', icon: '📓', type: 'personal', provider: 'Google', modelName: 'gemini-1.5-flash', temperature: 0.7, maxTokens: 2048, publicPermission: 'none', guestLinkEnabled: false },
    { id: '5', title: 'Ideas', icon: '💡', type: 'personal', provider: 'DeepSeek', modelName: 'deepseek-reasoner', temperature: 0.6, maxTokens: 4096, publicPermission: 'none', guestLinkEnabled: false },
    { id: 'p1', title: 'Journal 2026', icon: '📝', type: 'personal' },
    { id: 'p2', title: 'Reading List', icon: '📚', type: 'personal' },
    { id: 'p3', title: 'Travel Plans', icon: '✈️', type: 'personal' },
    { id: 'p4', title: 'Recipes', icon: '🍳', type: 'personal' },
    { id: 'p5', title: 'Fitness Tracking', icon: '🏃‍♂️', type: 'personal' },
    { id: 'p6', title: 'Language Learning', icon: '🗣️', type: 'personal' },
    { id: 'p7', title: 'Side Projects', icon: '🛠️', type: 'personal' },
    { id: 'p8', title: 'App Concepts', icon: '📱', type: 'personal' },
    { id: 'p9', title: 'Financial Planning', icon: '💰', type: 'personal' },
    { id: 'p10', title: 'Daily Logs', icon: '📅', type: 'personal' },
    { id: 'p11', title: 'Code Snippets', icon: '👨‍💻', type: 'personal' }
  ],
  public: [
    { id: '6', title: 'Community Guides', icon: '🌍', type: 'public', provider: 'Google', modelName: 'gemini-1.5-flash', temperature: 0.7, maxTokens: 2048, publicPermission: 'read', guestLinkEnabled: true },
    { id: '7', title: 'Shared SDK Specs', icon: '📖', type: 'public', provider: 'DeepSeek', modelName: 'deepseek-chat', temperature: 0.4, maxTokens: 4096, publicPermission: 'write', guestLinkEnabled: true },
    { id: 'pb1', title: 'Open Source Projects', icon: '🌐', type: 'public' },
    { id: 'pb2', title: 'Public FAQs', icon: '❓', type: 'public' },
    { id: 'pb3', title: 'API Documentation', icon: '🔌', type: 'public' },
    { id: 'pb4', title: 'Release Notes', icon: '📝', type: 'public' },
    { id: 'pb5', title: 'Style Guides', icon: '💅', type: 'public' },
    { id: 'pb6', title: 'Design System', icon: '🎨', type: 'public' },
    { id: 'pb7', title: 'Tutorials & Examples', icon: '🎓', type: 'public' },
    { id: 'pb8', title: 'Ecosystem Plugins', icon: '🧩', type: 'public' },
    { id: 'pb9', title: 'Architecture Patterns', icon: '🏛️', type: 'public' },
    { id: 'pb10', title: 'Best Practices', icon: '⭐', type: 'public' },
    { id: 'pb11', title: 'Accessibility Standards', icon: '♿', type: 'public' }
  ]
};

let mockMarketKbs: MarketKnowledgeBase[] = [
  { id: '6', title: 'Community Guides', icon: '🌍', description: '适用于社区所有新老成员的公共操作、入职指南及常问问题（FAQ）合集。', author: '社区管理委员会', tags: ['社区规范', '新手帮助'], subscribersCount: 1205, documentsCount: 16, provider: 'Google', modelName: 'gemini-3.5-flash' },
  { id: '7', title: 'Shared SDK Specs', icon: '📖', description: '官方开放平台核心 SDK 说明文档、调用限制及安全证书链验证策略。', author: '开放平台团队', tags: ['技术框架', 'API指南'], subscribersCount: 840, documentsCount: 9, provider: 'DeepSeek', modelName: 'deepseek-chat' },
  { id: 'm1', title: 'A股证券知识库(每日更新)', icon: '📈', description: '涵盖最新A股、美股各行业深度研究报告，每日早盘前更新主力资金动向与题材复盘指引。', author: '证券投研组', tags: ['金融证券', '每日复盘'], subscribersCount: 1542, documentsCount: 48, provider: 'Google', modelName: 'gemini-3.5-pro' },
  { id: 'm2', title: 'AI工具学习资料', icon: '🤖', description: '全面微调收录大模型微调流程、提示词进阶工程技巧与AI工具提效秘籍。', author: '智脑工院', tags: ['人工智能', '提效工具'], subscribersCount: 1332, documentsCount: 41, provider: 'DeepSeek', modelName: 'deepseek-reasoner' },
  { id: 'm3', title: '硅创联.MCP精选', icon: '🔌', description: 'Model Context Protocol（MCP）精选服务端资源、核心生态包以及一键导入指南。', author: '硅创联社区', tags: ['MCP协议', '生态工具'], subscribersCount: 840, documentsCount: 9, provider: 'DeepSeek', modelName: 'deepseek-chat' },
  { id: 'm4', title: 'React 19 & Next.js 实战集锦', icon: '⚛️', description: '包含 React 源码级解析、React Server Components 精髓、以及多款高质量脚手架工程参考。', author: '前端研习社', tags: ['编程开发', '前端'], subscribersCount: 2105, documentsCount: 64, provider: 'Google', modelName: 'gemini-3.5-flash' },
  { id: 'm5', title: '出境自驾旅行全攻略', icon: '🧭', description: '自驾路线、租车手续、保险购买、多国交规提示等实战干货。', author: '旅行发烧友社群', tags: ['旅行生活', '探险攻略'], subscribersCount: 712, documentsCount: 19, provider: 'DeepSeek', modelName: 'deepseek-chat' },
];

let mockDocs: DocumentMeta[] = [
  { id: 'doc1', title: 'Q3 Roadmap.md', type: 'markdown', kbId: '1', updatedAt: new Date().toISOString(), author: 'Alice', content: '# Q3 Roadmap\n\nThis is a mock document.\n\n## Goals\n- Feature A\n- Feature B\n\n```js\nconsole.log("Hello from markdown!");\n```' },
  { id: 'doc2', title: 'Architecture.ts', type: 'code', kbId: '1', updatedAt: new Date().toISOString(), author: 'Bob', content: 'export const Architecture = {\n  version: "1.0.0",\n  services: ["auth", "db"]\n};' },
  { id: 'doc-html-1', title: 'LandingPage.html', type: 'code', kbId: '1', updatedAt: new Date().toISOString(), author: 'System', content: '<!DOCTYPE html>\n<html lang="en">\n<head>\n    <meta charset="UTF-8">\n    <meta name="viewport" content="width=device-width, initial-scale=1.0">\n    <title>SaaS Landing Page</title>\n    <script src="https://cdn.tailwindcss.com"></script>\n</head>\n<body class="bg-gray-50">\n    <nav class="bg-white shadow-sm">\n        <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">\n            <div class="flex justify-between h-16 items-center">\n                <div class="text-xl font-bold text-blue-600">SaaS Platform</div>\n                <div class="space-x-4">\n                    <a href="#" class="text-gray-500 hover:text-gray-900">Features</a>\n                    <a href="#" class="text-gray-500 hover:text-gray-900">Pricing</a>\n                    <a href="#" class="bg-blue-600 text-white px-4 py-2 rounded-md hover:bg-blue-700">Get Started</a>\n                </div>\n            </div>\n        </div>\n    </nav>\n    <main class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-16 text-center">\n        <h1 class="text-4xl tracking-tight font-extrabold text-gray-900 sm:text-5xl md:text-6xl">\n            <span class="block">The ultimate platform for</span>\n            <span class="block text-blue-600">your next big idea</span>\n        </h1>\n        <p class="mt-3 max-w-md mx-auto text-base text-gray-500 sm:text-lg md:mt-5 md:text-xl md:max-w-3xl">\n            Everything you need to build, launch, and scale your product, beautifully designed and simple to use.\n        </p>\n        <div class="mt-10 max-w-sm mx-auto sm:max-w-none sm:flex sm:justify-center">\n            <div class="space-y-4 sm:space-y-0 sm:mx-auto sm:inline-grid sm:grid-cols-2 sm:gap-5">\n                <a href="#" class="flex items-center justify-center px-8 py-3 border border-transparent text-base font-medium rounded-md text-white bg-blue-600 hover:bg-blue-700 md:py-4 md:text-lg md:px-10">\n                    Get started now\n                </a>\n                <a href="#" class="flex items-center justify-center px-8 py-3 border border-transparent text-base font-medium rounded-md text-blue-700 bg-blue-100 hover:bg-blue-200 md:py-4 md:text-lg md:px-10">\n                    Live demo\n                </a>\n            </div>\n        </div>\n    </main>\n</body>\n</html>' },
  { id: 'doc3', title: 'Welcome to Knowledge Base', type: 'richtext', kbId: '1', updatedAt: new Date().toISOString(), author: 'System', content: '<h2>Welcome to your new knowledge base.</h2><p>Here you can keep all your rich text notes, code snippets, markdown files and media.</p>' },
  { id: 'folder1', title: 'Meeting Notes', type: 'folder', kbId: '1', updatedAt: new Date().toISOString(), author: 'Alice' },
  { id: 'doc4', title: 'Weekly Sync', type: 'richtext', kbId: '1', parentId: 'folder1', updatedAt: new Date().toISOString(), author: 'Bob', content: '<p>Weekly Sync notes go here.</p>' },
  
  // Word & Excel & PPT & Archive Everyday formats
  { id: 'doc-word-1', title: 'Q2 业务拓展执行方案.docx', type: 'file', kbId: '1', size: '154.20 KB', updatedAt: new Date().toISOString(), author: 'Bob' },
  { id: 'doc-excel-1', title: '2026 第一季度营销预算明细表.xlsx', type: 'file', kbId: '1', size: '1.20 MB', updatedAt: new Date().toISOString(), author: 'Alice' },
  { id: 'doc-excel-2', title: '客户满意度调研数据分析.csv', type: 'file', kbId: '1', size: '54.60 KB', updatedAt: new Date().toISOString(), author: 'System' },
  { id: 'doc-ppt-1', title: '天使轮项目路演演示文稿.pptx', type: 'file', kbId: '1', size: '4.80 MB', updatedAt: new Date().toISOString(), author: 'System' },
  { id: 'doc-zip-1', title: '主站前端切图资源与静态素材.zip', type: 'file', kbId: '1', size: '42.50 MB', updatedAt: new Date().toISOString(), author: 'System' },
  // Plain Text file
  { id: 'doc-txt-1', title: '日常待办与灵感记录备忘.txt', type: 'code', kbId: '1', content: '===== 日常灵感备忘录 =====\n\n1. 极简知识库增加对 Word 和 Excel 文件类型的可视化，提升系统多元属性\n2. 统一全平台 HTML 预览页及主色调，确保视觉体验高度一致\n3. 调试微前端通信和高分辨率视频适配比例效果\n4. 确认前端样式细节，完善 UI 卡片阴影过渡', size: '2.40 KB', updatedAt: new Date().toISOString(), author: 'Me' },

  { id: 'doc5', title: 'Architecture Diagram (Landscape 16:9)', type: 'image', kbId: '1', updatedAt: new Date().toISOString(), author: 'Alice', url: 'https://images.unsplash.com/photo-1544383835-bda2bc66a55d?w=1280&h=720&fit=crop&q=80' },
  { id: 'doc5-1', title: 'Aesthetic Textile (Square 1:1)', type: 'image', kbId: '1', updatedAt: new Date().toISOString(), author: 'Alice', url: 'https://images.unsplash.com/photo-1618005182384-a83a8bd57fbe?w=800&h=800&fit=crop&q=80' },
  { id: 'doc5-2', title: 'Mobile UI Screen (Portrait 9:16)', type: 'image', kbId: '1', updatedAt: new Date().toISOString(), author: 'Eve', url: 'https://images.unsplash.com/photo-1512941937669-90a1b58e7e9c?w=720&h=1280&fit=crop&q=80' },
  { id: 'doc5-3', title: 'Cinema Concept (UltraWide 21:9)', type: 'image', kbId: '1', updatedAt: new Date().toISOString(), author: 'Bob', url: 'https://images.unsplash.com/photo-1506744038136-46273834b3fb?w=1400&h=600&fit=crop&q=80' },
  { id: 'doc5-4', title: 'Workspace Shot (Classic 4:3)', type: 'image', kbId: '1', updatedAt: new Date().toISOString(), author: 'System', url: 'https://images.unsplash.com/photo-1531538606174-0f90ff5dce83?w=800&h=600&fit=crop&q=80' },
  
  // Extra diverse image focal length and ratio presets
  { id: 'img-prop-1', title: '运营设计插画 - 奇幻森林 (Portrait 3:4)', type: 'image', kbId: '1', url: 'https://images.unsplash.com/photo-1518709268805-4e9042af9f23?w=768&h=1024&fit=crop&q=80', updatedAt: new Date().toISOString(), author: 'Design Lab' },
  { id: 'img-prop-2', title: '产品主视觉原画 - 蒸汽朋克 (Landscape 3:2)', type: 'image', kbId: '1', url: 'https://images.unsplash.com/photo-1501854140801-50d01698950b?w=1200&h=800&fit=crop&q=80', updatedAt: new Date().toISOString(), author: 'Art Director' },
  { id: 'img-prop-3', title: '全景太空概念看板 (Panoramic 16:5)', type: 'image', kbId: '1', url: 'https://images.unsplash.com/photo-1451187580459-43490279c0fa?w=1600&h=500&fit=crop&q=80', updatedAt: new Date().toISOString(), author: 'Sci-Fi Unit' },
  { id: 'img-prop-4', title: '温暖午后咖啡 (Classic 3:2)', type: 'image', kbId: '1', url: 'https://images.unsplash.com/photo-1509042239860-f550ce710b93?w=1200&h=800&fit=crop&q=80', updatedAt: new Date().toISOString(), author: 'Me' },

  { id: 'doc6', title: 'User Instructions.pdf', type: 'pdf', kbId: '1', updatedAt: new Date().toISOString(), author: 'Alice', url: '/samples/user-instructions.pdf' },
  { id: 'doc6-remote', title: 'Research Paper (Remote URL).pdf', type: 'pdf', kbId: '1', updatedAt: new Date().toISOString(), author: 'Alice', url: 'https://raw.githubusercontent.com/mozilla/pdf.js/ba2edeae/web/compressed.tracemonkey-pldi-09.pdf' },
  { id: 'doc7', title: 'Frontend Structure.json', type: 'code', kbId: '1', updatedAt: new Date().toISOString(), author: 'System', content: '{\n  "src": ["components", "services", "utils"],\n  "public": ["assets"]\n}' },
  
  { id: 'doc9', title: 'Product Demo (Widescreen 16:9).mp4', type: 'video', kbId: '1', updatedAt: new Date().toISOString(), author: 'Marketing', url: 'http://commondatastorage.googleapis.com/gtv-videos-bucket/sample/BigBuckBunny.mp4' },
  { id: 'doc9-1', title: 'TikTok Stream (Portrait 9:16).mp4', type: 'video', kbId: '1', updatedAt: new Date().toISOString(), author: 'Vlog Team', url: 'https://assets.mixkit.co/videos/preview/mixkit-forest-stream-in-the-sunlight-529-large.mp4' },
  { id: 'doc9-2', title: 'Cyberpunk Theme (Cinema 21:9).mp4', type: 'video', kbId: '1', updatedAt: new Date().toISOString(), author: 'Concept Team', url: 'https://assets.mixkit.co/videos/preview/mixkit-stars-in-space-background-1611-large.mp4' },
  { id: 'doc9-3', title: 'Planet Loop (Square 1:1).mp4', type: 'video', kbId: '1', updatedAt: new Date().toISOString(), author: 'SciTech', url: 'https://assets.mixkit.co/videos/preview/mixkit-rotating-planet-earth-loop-1793-large.mp4' },
  
  // Extra detailed video ratios and formats
  { id: 'vid-prop-1', title: '城市雨夜街景 (Portrait 9:16).mov', type: 'video', kbId: '1', url: 'https://assets.mixkit.co/videos/preview/mixkit-reflection-of-neon-lights-on-wet-asphalt-34241-large.mp4', updatedAt: new Date().toISOString(), author: 'Video Lab' },
  { id: 'vid-prop-2', title: '宁静沙滩日落 (Widescreen 16:9).mkv', type: 'video', kbId: '1', url: 'https://assets.mixkit.co/videos/preview/mixkit-sun-setting-over-the-sea-off-the-beach-14002-large.mp4', updatedAt: new Date().toISOString(), author: 'Marketing' },
  { id: 'vid-prop-3', title: '水流穿梭溪石 (Square 1:1).avi', type: 'video', kbId: '1', url: 'https://assets.mixkit.co/videos/preview/mixkit-water-splashing-on-mossy-rocks-in-a-forest-42291-large.mp4', updatedAt: new Date().toISOString(), author: 'SciTech' },

  { id: 'doc10', title: 'Interview Recording.mp3', type: 'audio', kbId: '1', updatedAt: new Date().toISOString(), author: 'Alice', url: 'https://cdn.pixabay.com/download/audio/2022/10/25/audio_220e8b15d9.mp3?filename=piano-moment-9835.mp3' },
  { id: 'doc-music-1', title: 'Retro Cyberpunk Beats.mp3', type: 'music', kbId: '1', updatedAt: new Date().toISOString(), author: 'Music Lab', url: 'https://cdn.pixabay.com/download/audio/2022/03/10/audio_cbf448ccaa.mp3?filename=cyberpunk-2099-10656.mp3' },
  { id: 'doc-music-2', title: 'Lofi Coffee Shop Study.mp3', type: 'music', kbId: '1', updatedAt: new Date().toISOString(), author: 'Lofi Chill', url: 'https://cdn.pixabay.com/download/audio/2021/11/23/audio_a150c950ef.mp3?filename=lofi-study-11219.mp3' },
  { id: 'folder2', title: 'Design Assets', type: 'folder', kbId: '3', updatedAt: new Date().toISOString(), author: 'Design Team' },
  { id: 'doc8', title: 'Brand Guidelines.md', type: 'markdown', kbId: '3', parentId: 'folder2', updatedAt: new Date().toISOString(), author: 'Eve', content: '# Brand Guidelines\n\n### Colors\n- **Primary**: `#000000`\n- **Secondary**: `#ffffff`\n' },
  { id: 'p-doc1', title: '读书笔记 - 《重构》.md', type: 'markdown', kbId: '4', updatedAt: new Date().toISOString(), author: 'Me', content: '# 读书笔记 - 《重构》\n\n- 任何一个傻瓜都能写出计算机可以理解的代码。\n- 唯有写出人类容易理解的代码，才是优秀的程序员。' },
  { id: 'p-doc2', title: '周四技术分享纪要.richtext', type: 'richtext', kbId: '4', updatedAt: new Date().toISOString(), author: 'Me', content: '<h2>周四技术分享：RAG架构与微调</h2><p>1. 提炼了目前行业主流的双路召回，通过Denser Retriever召回密集嵌入并用BM25进行稀疏文本匹配融合。</p><p>2. 对提问做语义路由，确定采用精准匹配还是智能生成模型。</p>' },
  { id: 'p-doc3', title: '2026年终总结安排.richtext', type: 'richtext', kbId: '4', updatedAt: new Date().toISOString(), author: 'Me', content: '<h2>2026年终总结与述职安排</h2><p>主要议程：业务成长、架构优化、团队协同、未来规划。</p>' },
  { id: 'p-doc4', title: '旅行灵感.md', type: 'markdown', kbId: '5', updatedAt: new Date().toISOString(), author: 'Me', content: '# 旅行地灵感清单\n\n- 云南大理游：租车自驾，感受苍山洱海的风土人情\n- 川西高原行：摄影装备，高海拨自驾\n- 冰岛极光：一生一次，冬季自驾环岛' },
  { id: 'p-doc5', title: '新App创意 - 智能桌面助手.md', type: 'markdown', kbId: '5', updatedAt: new Date().toISOString(), author: 'Me', content: '# 创意：精美实用的智能桌面组件App\n\n1. 支持自定义小组件大小，拥有极简、中性色调、高对比度黑白灰三个主题\n2. 引入Gemini交互，用户可以直接在小组件上完成日常打卡、记账、灵感记录。' },
  // More docs for kbId: 1 to force scrolling
  { id: 'mock-extra-1', title: 'Project Euler Solutions.md', type: 'markdown', kbId: '1', updatedAt: new Date().toISOString(), author: 'System' },
  { id: 'mock-extra-2', title: 'Daily Standup Logs.richtext', type: 'richtext', kbId: '1', updatedAt: new Date().toISOString(), author: 'Alice' },
  { id: 'mock-extra-3', title: 'UX Research Q1.pdf', type: 'file', kbId: '1', size: '2.5 MB', updatedAt: new Date().toISOString(), author: 'Bob' },
  { id: 'mock-extra-4', title: 'Server Config.txt', type: 'code', kbId: '1', size: '4.1 KB', updatedAt: new Date().toISOString(), author: 'System' },
  { id: 'mock-extra-5', title: 'Logo Asset V2.png', type: 'image', kbId: '1', updatedAt: new Date().toISOString(), author: 'Design Team' },
  { id: 'mock-extra-6', title: 'Bug Tracker dump.csv', type: 'file', kbId: '1', size: '1.2 MB', updatedAt: new Date().toISOString(), author: 'System' },
  { id: 'mock-extra-7', title: 'Feature Specifications.docx', type: 'file', kbId: '1', size: '400 KB', updatedAt: new Date().toISOString(), author: 'Alice' },
  { id: 'mock-extra-8', title: 'Demo Recording.mp4', type: 'video', kbId: '1', updatedAt: new Date().toISOString(), author: 'Sales' },
  { id: 'mock-extra-9', title: 'Theme Variables.css', type: 'code', kbId: '1', size: '8.5 KB', updatedAt: new Date().toISOString(), author: 'System' },
  { id: 'mock-extra-10', title: 'Meeting Transcript.txt', type: 'code', kbId: '1', size: '15 KB', updatedAt: new Date().toISOString(), author: 'AI Recorder' },
  { id: 'mock-extra-11', title: 'Client Feedback.md', type: 'markdown', kbId: '1', updatedAt: new Date().toISOString(), author: 'Bob' },
  { id: 'mock-extra-12', title: 'Expenses Q1.xlsx', type: 'file', kbId: '1', size: '200 KB', updatedAt: new Date().toISOString(), author: 'Alice' },
  { id: 'mock-extra-13', title: 'Offsite Planning.richtext', type: 'richtext', kbId: '1', updatedAt: new Date().toISOString(), author: 'HR' },
  { id: 'mock-extra-14', title: 'Q1 OKRs.pptx', type: 'file', kbId: '1', size: '14.2 MB', updatedAt: new Date().toISOString(), author: 'System' },
  { id: 'mock-extra-15', title: 'Backup Scripts.zip', type: 'file', kbId: '1', size: '80.5 MB', updatedAt: new Date().toISOString(), author: 'System' }
];

const generateId = () => Math.random().toString(36).substring(2, 9);
const delay = (ms: number) => new Promise(resolve => setTimeout(resolve, ms));

/**
 * 前端封装的 Service 接口层 (Mock Implementation)
 * 后续可通过接入后端 SDK 直接替换内部实现即可，前端业务代码无需大改
 */
export class DocumentService {
  /**
   * 获取知识库列表
   */
  static async getKnowledgeBases(): Promise<{ team: KnowledgeBase[], personal: KnowledgeBase[], public: KnowledgeBase[] }> {
    return withApiFallback(
      () => KnowledgebaseDocumentApiBridge.getKnowledgeBases(),
      async () => {
        await delay(300);
        return JSON.parse(JSON.stringify(mockKbs));
      },
    );
  }

  static async createKnowledgeBase(kb: Partial<KnowledgeBase>): Promise<KnowledgeBase> {
    return withApiFallback(
      () => KnowledgebaseDocumentApiBridge.createKnowledgeBase(kb),
      async () => {
        await delay(300);
        const newKb: KnowledgeBase = {
          id: generateId(),
          title: kb.title || 'Untitled',
          icon: kb.icon || '📁',
          type: kb.type || 'team',
          avatar: kb.avatar,
          provider: kb.provider || 'Google',
          modelName: kb.modelName || 'gemini-3.5-flash',
          temperature: kb.temperature !== undefined ? kb.temperature : 0.7,
          maxTokens: kb.maxTokens || 2048,
          systemPrompt: kb.systemPrompt || '',
          publicPermission: kb.publicPermission || (kb.type === 'public' ? 'read' : 'none'),
          guestLinkEnabled: kb.guestLinkEnabled !== undefined ? kb.guestLinkEnabled : (kb.type === 'public'),
        };
        if (newKb.type === 'team') {
          mockKbs.team.push(newKb);
        } else if (newKb.type === 'public') {
          mockKbs.public.push(newKb);
        } else {
          mockKbs.personal.push(newKb);
        }
        return { ...newKb };
      },
    );
  }

  static async hydrateKnowledgeBase(kb: KnowledgeBase): Promise<KnowledgeBase> {
    return withApiFallback(
      () => KnowledgebaseDocumentApiBridge.hydrateKnowledgeBase(kb),
      async () => kb,
    );
  }

  static async updateKnowledgeBase(id: string, updates: Partial<KnowledgeBase>): Promise<KnowledgeBase> {
    return withApiFallback(
      () => KnowledgebaseDocumentApiBridge.updateKnowledgeBase(id, updates),
      async () => {
        await delay(200);
        let updatedKb = null;

        mockKbs.team = mockKbs.team.map(kb => {
          if (kb.id === id) {
            updatedKb = { ...kb, ...updates };
            return updatedKb;
          }
          return kb;
        });

        mockKbs.personal = mockKbs.personal.map(kb => {
          if (kb.id === id) {
            updatedKb = { ...kb, ...updates };
            return updatedKb;
          }
          return kb;
        });

        mockKbs.public = mockKbs.public.map(kb => {
          if (kb.id === id) {
            updatedKb = { ...kb, ...updates };
            return updatedKb;
          }
          return kb;
        });

        if (!updatedKb) throw new Error('KB not found');
        return updatedKb;
      },
    );
  }

  static async deleteKnowledgeBase(id: string): Promise<boolean> {
    return withApiFallback(
      () => KnowledgebaseDocumentApiBridge.deleteKnowledgeBase(id),
      async () => {
        await delay(200);
        mockKbs.team = mockKbs.team.filter(kb => kb.id !== id);
        mockKbs.personal = mockKbs.personal.filter(kb => kb.id !== id);
        mockKbs.public = mockKbs.public.filter(kb => kb.id !== id);
        return true;
      },
    );
  }

  static async getDocuments(kbId: string): Promise<(FolderNode | DocumentMeta)[]> {
    return withApiFallback(
      () => KnowledgebaseDocumentApiBridge.getDocuments(kbId),
      async () => {
        await delay(200);
    // Return all items belonging to the kb, Tree component will assemble them based on parentId
    const docs = mockDocs.filter(d => d.kbId === kbId);
    
    // Simulate folder hierarchy processing for Tree if needed...
    // Actually, react-arborist can take flat array if each node has children field.
    // Wait, the previous implementation used tree assembled data or flat? The frontend component `searchQuery` processing implies `docs` expects a tree structure if there are folders.
    
    // Let's build tree structure for FolderNode:
    const map = new Map<string, any>();
    const roots: any[] = [];
    
    // Convert to deep clones so we can attach children safely
    docs.forEach(doc => {
      map.set(doc.id, { ...doc, children: doc.type === 'folder' ? [] : undefined });
    });
    
    map.forEach(doc => {
      if (doc.parentId && map.has(doc.parentId)) {
        map.get(doc.parentId).children.push(doc);
      } else {
        roots.push(doc);
      }
    });

        return roots;
      },
    );
  }

  static async getDocumentContent(id: string): Promise<string> {
    return withApiFallback(
      () => KnowledgebaseDocumentApiBridge.getDocumentContent(id),
      async () => {
        await delay(150);
        const doc = mockDocs.find(d => d.id === id);
        return doc?.content || '';
      },
    );
  }

  static async saveDocumentContent(id: string, content: string): Promise<boolean> {
    return withApiFallback(
      () => KnowledgebaseDocumentApiBridge.saveDocumentContent(id, content),
      async () => {
        await delay(300);
        const docIndex = mockDocs.findIndex(d => d.id === id);
        if (docIndex >= 0) {
          mockDocs[docIndex] = { ...mockDocs[docIndex], content, updatedAt: new Date().toISOString() };
          return true;
        }
        return false;
      },
    );
  }
  
  static async updateDocument(id: string, updates: Partial<DocumentMeta>): Promise<boolean> {
    return withApiFallback(
      () => KnowledgebaseDocumentApiBridge.updateDocument(id, updates),
      async () => {
        await delay(100);
        const docIndex = mockDocs.findIndex(d => d.id === id);
        if (docIndex >= 0) {
          mockDocs[docIndex] = { ...mockDocs[docIndex], ...updates, updatedAt: new Date().toISOString() };
          return true;
        }
        return false;
      },
    );
  }

  static async createDocument(doc: Partial<DocumentMeta>): Promise<DocumentMeta> {
    return withApiFallback(
      () => KnowledgebaseDocumentApiBridge.createDocument(doc),
      async () => {
        await delay(300);
        const newDoc: DocumentMeta = {
          id: generateId(),
          title: doc.title || 'Untitled',
          type: doc.type || 'richtext',
          kbId: doc.kbId,
          parentId: doc.parentId || null,
          updatedAt: new Date().toISOString(),
          author: doc.author || 'Current User',
          content: doc.content || '',
          url: doc.url,
          size: doc.size,
          order: doc.order,
        };
        mockDocs.push(newDoc);
        return { ...newDoc };
      },
    );
  }

  static async uploadFiles(files: File[], kbId: string, parentId?: string, overrideType?: DocumentMeta['type']): Promise<DocumentMeta[]> {
    if (isKnowledgebaseApiAvailable()) {
      return KnowledgeFileUploadService.uploadKnowledgebaseFiles(
        files,
        kbId,
        overrideType,
        parentId,
      );
    }

    const folderMap = new Map<string, string>();
    const results: DocumentMeta[] = [];
    const topLevelItems: DocumentMeta[] = [];

    for (const file of files) {
      let currentParentId = parentId || null;
      let isTopLevel = true;
      if ((file as any).webkitRelativePath) {
        const parts = (file as any).webkitRelativePath.split('/');
        parts.pop();
        let pathAccumulator = '';
        for (let i = 0; i < parts.length; i++) {
          const folderName = parts[i];
          pathAccumulator += (pathAccumulator ? '/' : '') + folderName;
          if (!folderMap.has(pathAccumulator)) {
            const folderDoc = await this.createDocument({
              title: folderName,
              type: 'folder',
              kbId,
              parentId: currentParentId
            });
            folderMap.set(pathAccumulator, folderDoc.id);
            if (i === 0) topLevelItems.push(folderDoc);
          }
          currentParentId = folderMap.get(pathAccumulator) || null;
        }
        if (parts.length > 0) isTopLevel = false;
      }

      let type: DocumentMeta['type'] = overrideType || 'file';
      if (!overrideType) {
        if (file.type.startsWith('image/')) type = 'image';
        else if (file.type.startsWith('video/')) type = 'video';
        else if (file.type.startsWith('audio/')) type = 'audio';
        else if (file.type === 'application/pdf' || file.name.endsWith('.pdf')) type = 'pdf';
        else if (file.name.endsWith('.md')) type = 'markdown';
        else if (/\.(ts|js|jsx|tsx|html|htm|css|json|xml|py|java|cpp|c|go|rs|php|rb|swift|kt|sql|sh|yaml|yml)$/i.test(file.name)) type = 'code';
      }
      
      let url = undefined;
      if (['image', 'video', 'audio', 'music', 'file', 'pdf'].includes(type) && URL.createObjectURL) {
        url = URL.createObjectURL(file);
      }

      const doc = await this.createDocument({
        title: file.name,
        type,
        kbId,
        parentId: currentParentId,
        url,
        size: (file.size / 1024).toFixed(2) + ' KB',
      });
      results.push(doc);
      if (isTopLevel) topLevelItems.push(doc);
    }
    return topLevelItems;
  }
  
  static async deleteDocument(id: string): Promise<boolean> {
    return withApiFallback(
      () => KnowledgebaseDocumentApiBridge.deleteDocument(id),
      async () => {
        await delay(200);

        const idsToDelete = new Set<string>([id]);

        let added = true;
        while (added) {
          added = false;
          mockDocs.forEach(d => {
            if (d.parentId && idsToDelete.has(d.parentId) && !idsToDelete.has(d.id)) {
              idsToDelete.add(d.id);
              added = true;
            }
          });
        }

        mockDocs = mockDocs.filter(d => !idsToDelete.has(d.id));
        return true;
      },
    );
  }

  static async getMarketKnowledgeBases(): Promise<MarketKnowledgeBase[]> {
    return withApiFallback(
      async () => [],
      async () => {
        await delay(200);
        return mockMarketKbs.map(mkb => {
          const isSubscribed = mockKbs.public.some(kb => kb.id === mkb.id);
          return { ...mkb, isSubscribed };
        });
      },
    );
  }

  static async subscribeMarketKb(id: string): Promise<boolean> {
    return withApiFallback(
      async () => false,
      async () => {
        await delay(150);
        const item = mockMarketKbs.find(mkb => mkb.id === id);
        if (!item) return false;

        const alreadySubscribed = mockKbs.public.some(kb => kb.id === id);
        if (!alreadySubscribed) {
          mockKbs.public.push({
            id: item.id,
            title: item.title,
            icon: item.icon,
            type: 'public',
            provider: item.provider,
            modelName: item.modelName,
            temperature: 0.7,
            maxTokens: 2048,
            systemPrompt: item.description,
            publicPermission: 'read',
            guestLinkEnabled: true
          });
        }
        return true;
      },
    );
  }

  static async unsubscribeMarketKb(id: string): Promise<boolean> {
    return withApiFallback(
      async () => false,
      async () => {
        await delay(150);
        mockKbs.public = mockKbs.public.filter(kb => kb.id !== id);
        return true;
      },
    );
  }

  static async importGitRepository(
    kbId: string,
    repoUrl: string,
    branch: string = 'main',
    options?: {
      accessToken?: string;
      onProgress?: (progress: KnowledgeGitImportService.GitImportProgress) => void;
    },
  ): Promise<boolean> {
    if (isKnowledgebaseApiAvailable()) {
      const result = await KnowledgeGitImportService.importGitRepository(
        kbId,
        repoUrl,
        branch,
        options?.accessToken,
        options?.onProgress,
      );
      return result.importedCount > 0;
    }

    await delay(1200); // offline demo only
    const repoName = repoUrl.split('/').pop()?.replace('.git', '') || 'repository';
    
    // Create a folder representing the Git repo
    const gFolderId = 'git-folder-' + generateId();
    mockDocs.push({
      id: gFolderId,
      title: `${repoName} (${branch})`,
      type: 'folder',
      kbId: kbId,
      updatedAt: new Date().toISOString(),
      author: 'Git Sync'
    });

    // Create some actual mock files from generic repositories
    mockDocs.push({
      id: generateId(),
      title: 'README.md',
      type: 'markdown',
      kbId: kbId,
      parentId: gFolderId,
      updatedAt: new Date().toISOString(),
      author: 'Git Sync',
      content: `# ${repoName}\n\nThis repository was imported from \`${repoUrl}\` branch \`${branch}\`.\n\n## Getting Started\n\nRun the following commands to set up the project:\n\n\`\`\`sh\nnpm install\nnpm run dev\n\`\`\`\n\n## Project Status\nSynced successfully with continuous tracking enabled.`
    });

    mockDocs.push({
      id: generateId(),
      title: 'package.json',
      type: 'code',
      kbId: kbId,
      parentId: gFolderId,
      updatedAt: new Date().toISOString(),
      author: 'Git Sync',
      content: `{\n  "name": "${repoName.toLowerCase()}",\n  "version": "1.0.0",\n  "private": true,\n  "scripts": {\n    "dev": "vite",\n    "build": "tsc && vite build"\n  },\n  "dependencies": {\n    "react": "^18.3.1",\n    "react-dom": "^18.3.1"\n  }\n}`
    });

    mockDocs.push({
      id: generateId(),
      title: 'src/main.tsx',
      type: 'code',
      kbId: kbId,
      parentId: gFolderId,
      updatedAt: new Date().toISOString(),
      author: 'Git Sync',
      content: `import React from 'react';\nimport ReactDOM from 'react-dom/client';\n\nconsole.log('App successfully mounted and running!');`
    });

    return true;
  }

  static async syncGitRepository(kbId: string, commitMessage: string): Promise<{ success: boolean; hash: string }> {
    if (isKnowledgebaseApiAvailable()) {
      throw new Error('Git repository sync is not available through the Knowledgebase API yet.');
    }

    await delay(1500); // offline demo only
    const randomHash = Math.random().toString(16).substring(2, 10);
    return { success: true, hash: randomHash };
  }

  static async publishWebsite(platform: string, targetId: string): Promise<{ success: boolean; url?: string }> {
    if (isKnowledgebaseApiAvailable()) {
      throw new Error('Website publishing is not available through the Knowledgebase API yet.');
    }

    await delay(1200); // offline demo only
    const platformDomains: Record<string, string> = {
      'vercel': 'vercel.app',
      'netlify': 'netlify.app',
      'gh-pages': 'github.io',
      'feishu': 'feishu.cn/docs',
      'notion': 'notion.site'
    };
    const domain = platformDomains[platform] || 'example.com';
    return { success: true, url: `https://kb-share-${targetId}.${domain}/` };
  }

  static async searchAll(query: string): Promise<{
    kbs: KnowledgeBase[],
    docs: DocumentMeta[]
  }> {
    return withApiFallback(
      () => KnowledgebaseDocumentApiBridge.searchAll(query),
      async () => {
        await delay(300);
        const lowerQuery = query.toLowerCase();
        
        const allKbs = [...mockKbs.team, ...mockKbs.personal, ...mockKbs.public];
        const matchedKbs = allKbs.filter(kb => kb.title.toLowerCase().includes(lowerQuery));
        
        const matchedDocs = mockDocs.filter(doc => 
          (doc.title && doc.title.toLowerCase().includes(lowerQuery)) ||
          (doc.content && doc.content.toLowerCase().includes(lowerQuery))
        );
        
        return {
          kbs: matchedKbs,
          docs: matchedDocs
        };
      },
    );
  }

  static async getRecentDocuments(limit: number = 8): Promise<DocumentMeta[]> {
    return withApiFallback(
      () => KnowledgebaseDocumentApiBridge.getRecentDocuments(limit),
      async () => {
        await delay(100);
        return JSON.parse(JSON.stringify(
          mockDocs
            .filter(d => d.type !== 'folder')
            .sort((a, b) => new Date(b.updatedAt || 0).getTime() - new Date(a.updatedAt || 0).getTime())
            .slice(0, limit)
        ));
      },
    );
  }

  static async touchDocument(id: string): Promise<boolean> {
    return withApiFallback(
      () => KnowledgebaseDocumentApiBridge.touchDocument(id),
      async () => {
        const docIndex = mockDocs.findIndex(d => d.id === id);
        if (docIndex >= 0) {
          mockDocs[docIndex] = { ...mockDocs[docIndex], updatedAt: new Date().toISOString() };
          return true;
        }
        return false;
      },
    );
  }
}


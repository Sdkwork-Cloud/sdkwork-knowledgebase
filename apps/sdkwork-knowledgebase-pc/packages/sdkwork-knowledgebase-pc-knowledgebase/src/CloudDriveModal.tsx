import React, { useState, useEffect, useMemo } from 'react';
import { isBlank, trim } from '@sdkwork/sdkwork-knowledgebase-pc-commons/stringUtils';
import { createPortal } from 'react-dom';
import { useTranslation } from 'react-i18next';
import { 
  X, Cloud, Server, ArrowRightLeft, Folder, FileText, 
  FileSpreadsheet, FileMinus, ChevronRight, Search, LayoutGrid, 
  List, Info, Check, CheckSquare, Square, Home, Users, 
  Clock, Star, ArrowLeft, RefreshCw, Layers, ServerIcon, Download, AlertCircle, Video
} from 'lucide-react';

interface DriveItem {
  id: string;
  name: string;
  type: 'folder' | 'richtext' | 'markdown' | 'file';
  size?: string;
  updatedAt: string;
  ownerString?: string;
  isStarred?: boolean;
  content?: string;
  children?: DriveItem[];
}

interface CloudDriveModalProps {
  isOpen: boolean;
  onClose: () => void;
  onConfirm: (selectedItems: Array<{ title: string; type: string; content?: string }>) => void;
}

const DRIVE_DATA: DriveItem[] = [
  {
    id: 'f-1',
    name: '2026市场调研与行业分析',
    type: 'folder',
    updatedAt: '2026-05-18',
    ownerString: '市场部',
    isStarred: true,
    children: [
      {
        id: 'd-101',
        name: '全球AI搜索服务行业白皮书.md',
        type: 'markdown',
        size: '4.8 MB',
        updatedAt: '2026-05-10',
        ownerString: '市场部',
        content: '# 全球AI搜索服务行业白皮书\n\n该文档包含2026年全球智能搜索、检索增强生成 (RAG) 领域的最新市场调研指标与用户增长趋势。\n\n## 1. 核心观点\n- 混合搜索 (Sparse + Dense) 成为大模型标配；\n- 用户首选对话式、引用清晰、卡片式呈现的回答引擎；\n- 智能知识库的准确召回率比纯索引高出不少。'
      },
      {
        id: 'd-102',
        name: '竞品底层架构与能力对标.richtext',
        type: 'richtext',
        size: '1.2 MB',
        updatedAt: '2026-05-12',
        ownerString: '张经理',
        content: '<h1>竞品底层架构与能力对标</h1><p>通过对市面上前五大智能知识库进行系统拆解，分析其向量数据库、权限分割模型及多轮对话的优劣势。</p><h3>主要改进方向：</h3><ul><li>提升大文本表格在解析时候的层级上下文关联；</li><li>支持直接关联微信聊天文件等网盘信息。</li></ul>'
      },
      {
        id: 'd-103',
        name: 'Q2线上投放渠道转化表.richtext',
        type: 'richtext',
        size: '850 KB',
        updatedAt: '2026-05-15',
        ownerString: '李专员',
        content: '<h1>Q2线上投放渠道转化分析</h1><p>汇总了第二季度所有社交平台、搜索渠道的引流以及注册量细项数据：</p><ul><li><b>头条投放转化率:</b> 2.8% (高转化)</li><li><b>搜索流量ROI:</b> 1.45 (持平)</li><li><b>社交媒体KOL引流成本:</b> 42元/有效线索</li></ul>'
      }
    ]
  },
  {
    id: 'f-2',
    name: '核心产品PRD与技术细节',
    type: 'folder',
    updatedAt: '2026-06-05',
    ownerString: '产品研发团队',
    children: [
      {
        id: 'd-201',
        name: '知识库RAG重构方案及接口文档.md',
        type: 'markdown',
        size: '45 KB',
        updatedAt: '2026-06-04',
        ownerString: 'TechLead',
        content: '# 知识库RAG重构方案及接口文档\n\n为了提高检索准确率，我们将在端侧引入混合搜索 (Hybrid Search) 方案。\n\n## 接口格式\n`POST /api/kb/search`\n参数: `{ query: string, top_k: number }`'
      },
      {
        id: 'd-202',
        name: '核心API网关开发设计规范.richtext',
        type: 'richtext',
        size: '2.1 MB',
        updatedAt: '2026-06-01',
        ownerString: '架构师王五',
        content: '<h2>核心API网关开发设计规范</h2><p>详细说明了微服务架构下的请求路由、动态限流、JWT无状态认证及链路追踪。</p><p>网关作为核心流量入口，确保了所有内部服务不受恶意攻击伤害。</p>'
      },
      {
        id: 'd-203',
        name: '新版智能客户端交互模型.richtext',
        type: 'richtext',
        size: '12.4 MB',
        updatedAt: '2026-06-05',
        ownerString: 'UI设计师',
        content: '<h1>新版智能客户端交互模型说明</h1><p>针对智能助理的双边协同以及拖拽体验进行了优化：</p><ul><li>双栏协同窗口设计，支持无缝联动</li><li>沉浸式侧边聊天与指令面板</li><li>动画过渡效果与状态保持</li></ul>'
      }
    ]
  },
  {
    id: 'f-3',
    name: '财务与行政规范',
    type: 'folder',
    updatedAt: '2026-04-10',
    ownerString: '行政总监',
    isStarred: true,
    children: [
      {
        id: 'd-301',
        name: '企业公章与保密协议签署细则.richtext',
        type: 'richtext',
        size: '450 KB',
        updatedAt: '2026-04-01',
        ownerString: '法务部',
        content: '<h2>企业公章与保密协议签署细则</h2><p>1. 所有对外保密协议必须经过法务二审。</p><p>2. 实体公章的外带需在OA系统提前48小时发起申请，并指派监管人。</p>'
      }
    ]
  },
  {
    id: 'd-1',
    name: '全员安全合规及办公规范.md',
    type: 'markdown',
    size: '12 KB',
    updatedAt: '2026-06-11',
    ownerString: 'HRBP',
    content: '# 全员安全合规及办公规范\n\n- 密码每季度强制更新，要求包含大写字母及特殊符号。\n- 离开座位须通过快捷键 Win+L 锁屏，保护工作资料安全。'
  },
  {
    id: 'd-2',
    name: '团队敏捷协作指南与流程看板.markdown',
    type: 'markdown',
    size: '1.4 MB',
    updatedAt: '2026-06-08',
    ownerString: 'ScrumMaster',
    isStarred: true,
    content: '# 团队敏捷协作指南与流程看板\n\n1. 每日站会于上午9:30举行，严格限时15分钟。\n2. 双周为一个迭代，每周五进行迭代评审与回顾。'
  },
  {
    id: 'd-3',
    name: '2026春季校园招聘产品经理岗白皮书.pdf',
    type: 'file',
    size: '12.5 MB',
    updatedAt: '2026-06-10',
    ownerString: 'HRBP'
  },
  {
    id: 'd-4',
    name: '公众号配套演示组件设计模板合集.zip',
    type: 'file',
    size: '248.6 MB',
    updatedAt: '2026-06-11',
    ownerString: '设计总监'
  },
  {
    id: 'd-5',
    name: 'SDKWork智能科技官方核心亮点解密宣传片.mp4',
    type: 'file',
    size: '42.1 MB',
    updatedAt: '2026-06-09',
    ownerString: '视频组'
  }
];

const renderFileIcon = (item: DriveItem) => {
  const isFolder = item.type === 'folder';
  if (isFolder) {
    return (
      <div className="p-1.5 bg-amber-500/10 text-amber-600 dark:text-amber-500 rounded-lg shrink-0">
        <Folder size={16} />
      </div>
    );
  }
  if (item.type === 'markdown') {
    return (
      <div className="p-1.5 bg-emerald-500/10 text-emerald-600 dark:text-emerald-500 rounded-lg shrink-0">
        <FileText size={16} />
      </div>
    );
  }
  if (item.type === 'richtext') {
    return (
      <div className="p-1.5 bg-blue-500/10 text-blue-600 dark:text-blue-500 rounded-lg shrink-0">
        <FileSpreadsheet size={16} />
      </div>
    );
  }
  
  const nameLower = item.name.toLowerCase();
  if (nameLower.endsWith('.pdf')) {
    return (
      <div className="p-1.5 bg-red-500/10 text-red-600 dark:text-red-500 rounded-lg shrink-0">
        <FileText size={16} />
      </div>
    );
  }
  if (nameLower.endsWith('.zip') || nameLower.endsWith('.rar')) {
    return (
      <div className="p-1.5 bg-purple-500/10 text-purple-600 dark:text-purple-500 rounded-lg shrink-0">
        <Layers size={16} />
      </div>
    );
  }
  if (nameLower.endsWith('.mp4') || nameLower.endsWith('.mov')) {
    return (
      <div className="p-1.5 bg-orange-500/10 text-orange-600 dark:text-orange-500 rounded-lg shrink-0">
        <Video size={16} />
      </div>
    );
  }
  
  return (
    <div className="p-1.5 bg-zinc-500/10 text-zinc-600 dark:text-zinc-500 rounded-lg shrink-0">
      <FileSpreadsheet size={16} />
    </div>
  );
};

const renderGridIcon = (item: DriveItem) => {
  if (item.type === 'folder') {
    return <Folder size={32} className="text-amber-500 fill-amber-500/10 mb-2 shrink-0" />;
  }
  if (item.type === 'markdown') {
    return <FileText size={32} className="text-emerald-500 mb-2 shrink-0" />;
  }
  if (item.type === 'richtext') {
    return <FileSpreadsheet size={32} className="text-blue-500 mb-2 shrink-0" />;
  }
  const nameLower = item.name.toLowerCase();
  if (nameLower.endsWith('.pdf')) {
    return <FileText size={32} className="text-red-500 mb-2 shrink-0" />;
  }
  if (nameLower.endsWith('.zip')) {
    return <Layers size={32} className="text-purple-500 mb-2 shrink-0" />;
  }
  if (nameLower.endsWith('.mp4') || nameLower.endsWith('.mov')) {
    return <Video size={32} className="text-orange-500 mb-2 shrink-0" />;
  }
  return <FileSpreadsheet size={32} className="text-zinc-500 mb-2 shrink-0" />;
};

export function CloudDriveModal({ isOpen, onClose, onConfirm }: CloudDriveModalProps) {
  const { t } = useTranslation('cloudDrive');
  const [activeTab, setActiveTab] = useState<'my-drive' | 'shared' | 'recent' | 'starred'>('my-drive');
  const [searchQuery, setSearchQuery] = useState('');
  const [viewMode, setViewMode] = useState<'list' | 'grid'>('list');
  const [currentFolderId, setCurrentFolderId] = useState<string | null>(null);
  const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set());
  const [isSyncing, setIsSyncing] = useState(false);
  const [feedbackMsg, setFeedbackMsg] = useState<string | null>(null);

  // Clear selections when switching tab/folder
  useEffect(() => {
    setSelectedIds(new Set());
  }, [activeTab, currentFolderId]);

  useEffect(() => {
    if (!isOpen) {
      setTimeout(() => {
        setActiveTab('my-drive');
        setCurrentFolderId(null);
        setSelectedIds(new Set());
        setSearchQuery('');
        setFeedbackMsg(null);
        setIsSyncing(false);
      }, 200);
    }
  }, [isOpen]);

  // Navigate folder helper
  const { currentFolder, breadcrumbs } = useMemo(() => {
    if (!currentFolderId) return { currentFolder: null, breadcrumbs: [] as DriveItem[] };
    
    // Find folder path recursively
    const findPath = (items: DriveItem[], targetId: string, path: DriveItem[]): { folder: DriveItem; path: DriveItem[] } | null => {
      for (const item of items) {
        if (item.id === targetId) {
          return { folder: item, path: [...path, item] };
        }
        if (item.type === 'folder' && item.children) {
          const result = findPath(item.children, targetId, [...path, item]);
          if (result) return result;
        }
      }
      return null;
    };

    const searchResult = findPath(DRIVE_DATA, currentFolderId, []);
    return searchResult
      ? { currentFolder: searchResult.folder, breadcrumbs: searchResult.path }
      : { currentFolder: null, breadcrumbs: [] };
  }, [currentFolderId]);

  // All files at current view (flattened or nested)
  const allCurrentTabFiles = useMemo(() => {
    if (activeTab === 'starred') {
      const result: DriveItem[] = [];
      const traverse = (items: DriveItem[]) => {
        items.forEach(i => {
          if (i.isStarred) result.push(i);
          if (i.children) traverse(i.children);
        });
      };
      traverse(DRIVE_DATA);
      return result;
    }

    if (activeTab === 'recent') {
      const result: DriveItem[] = [];
      const traverse = (items: DriveItem[]) => {
        items.forEach(i => {
          if (i.type !== 'folder') result.push(i);
          if (i.children) traverse(i.children);
        });
      };
      traverse(DRIVE_DATA);
      return result.sort((a, b) => b.updatedAt.localeCompare(a.updatedAt)).slice(0, 5);
    }

    if (activeTab === 'shared') {
      return [
        {
          id: 'shared-1',
          name: '行业前沿智能向量召回与算法研究报告.md',
          type: 'markdown',
          size: '3.1 MB',
          updatedAt: '2026-05-11',
          ownerString: '算法研发组',
          isStarred: false,
          content: '# 行业前沿智能向量召回与算法研究报告\n\n该文档深入讨论了基于高维稀疏特征与稠密实数嵌入的多路检索方法。'
        },
        {
          id: 'shared-2',
          name: '多云协同数据存储及加速规划.richtext',
          type: 'richtext',
          size: '4.5 MB',
          updatedAt: '2026-06-03',
          ownerString: '基础架构部',
          isStarred: true,
          content: '<h1>多云协同数据存储及加速规划</h1><p>分析了跨节点部署和边缘本地加速的设计理念，力争将网络访问延迟降到10ms以下。</p>'
        }
      ] as DriveItem[];
    }

    // Default: my-drive inside currentFolder
    return currentFolder ? currentFolder.children || [] : DRIVE_DATA;
  }, [activeTab, currentFolder]);

  // Filtered by Search Query
  const displayedFiles = useMemo(() => {
    if (isBlank(searchQuery)) return allCurrentTabFiles;
    const query = searchQuery.toLowerCase().trim();
    return allCurrentTabFiles.filter(file => file.name.toLowerCase().includes(query));
  }, [allCurrentTabFiles, searchQuery]);

  // Selection handlers
  const handleToggleSelect = (item: DriveItem, e: React.MouseEvent) => {
    e.stopPropagation();
    const next = new Set(selectedIds);
    if (next.has(item.id)) {
      next.delete(item.id);
    } else {
      next.add(item.id);
    }
    setSelectedIds(next);
  };

  const handleSelectAll = () => {
    const selectable = displayedFiles;
    if (selectedIds.size === selectable.length) {
      setSelectedIds(new Set());
    } else {
      setSelectedIds(new Set(selectable.map(s => s.id)));
    }
  };

  const getSelectedItemObjects = (): DriveItem[] => {
    const selected: DriveItem[] = [];
    const traverse = (items: DriveItem[]) => {
      items.forEach(i => {
        if (selectedIds.has(i.id)) {
          selected.push(i);
        }
        if (i.children) traverse(i.children);
      });
    };
    traverse(DRIVE_DATA);
    
    // Check shared drives as well
    const sharedSlice = [
      {
        id: 'shared-1',
        name: '行业前沿智能向量召回与算法研究报告.md',
        type: 'markdown' as const,
        size: '3.1 MB',
        updatedAt: '2026-05-11',
        ownerString: '算法研发组',
        isStarred: false,
        content: '# 行业前沿智能向量召回与算法研究报告\n\n该文档深入讨论了基于高维稀疏特征与稠密实数嵌入的多路检索方法。'
      },
      {
        id: 'shared-2',
        name: '多云协同数据存储及加速规划.richtext',
        type: 'richtext' as const,
        size: '4.5 MB',
        updatedAt: '2026-06-03',
        ownerString: '基础架构部',
        isStarred: true,
        content: '<h1>多云协同数据存储及加速规划</h1><p>分析了跨节点部署和边缘本地加速的设计理念，力争将网络访问延迟降到10ms以下。</p>'
      }
    ];
    sharedSlice.forEach(s => {
      if (selectedIds.has(s.id)) {
        selected.push(s as DriveItem);
      }
    });

    return selected;
  };

  const handleImportClick = () => {
    const objects = getSelectedItemObjects();
    if (objects.length === 0) return;

    setIsSyncing(true);
    setFeedbackMsg(t('syncingFeedback'));

    setTimeout(() => {
      // Map Drive item properties to Workspace DB document properties
      const mapped = objects.map(o => ({
        title: o.name,
        type: o.type,
        content: o.content || `从云端硬盘导入的资源 "${o.name}"。`,
        size: o.size || (o.type === 'folder' ? '文件夹格式' : '--'),
        updatedAt: o.updatedAt,
        owner: o.ownerString || t('enterpriseOwned')
      }));

      onConfirm(mapped);
      setIsSyncing(false);
      onClose();
    }, 1800);
  };

  const currentSelectableFiles = displayedFiles;

  if (!isOpen) return null;

  return createPortal(
    <div className="fixed inset-0 z-[300] bg-zinc-950/40 flex items-center justify-center backdrop-blur-md p-4">
      <div className="w-[1400px] h-[896px] max-w-[95vw] max-h-[95vh] bg-[var(--color-kb-editor)] rounded-2xl shadow-[0_24px_64px_-16px_rgba(0,0,0,0.25)] border border-[var(--color-kb-panel-border)] flex flex-col overflow-hidden animate-in fade-in zoom-in-95 duration-200">
        
        {/* Header */}
        <div className="h-16 border-b border-zinc-200/80 dark:border-[var(--color-kb-panel-border)] flex items-center justify-between px-6 bg-[#fafafa] dark:bg-[var(--color-kb-panel)]/30 shrink-0 z-10 shadow-sm">
          <div className="flex items-center gap-3">
            <div className="p-2 bg-gradient-to-tr from-cyan-50 dark:from-cyan-500/20 to-blue-50 dark:to-blue-500/20 border border-cyan-100 dark:border-transparent text-cyan-600 dark:text-cyan-500 rounded-xl shadow-inner">
              <Cloud size={20} strokeWidth={2.5} className="animate-pulse" />
            </div>
            <div>
              <h3 className="text-[15px] font-extrabold tracking-tight text-zinc-900 dark:text-[var(--color-kb-text-heading)] leading-tight">{t('myEnterpriseDrive')}</h3>
              <p className="text-[11.5px] font-medium text-zinc-500 dark:text-[var(--color-kb-text-muted)] tracking-wide">{t('driveDescription')}</p>
            </div>
          </div>
          <button 
            onClick={onClose} 
            className="text-zinc-400 hover:text-red-500 hover:bg-red-50 dark:text-[var(--color-kb-text-muted)] dark:hover:bg-red-500/10 p-2 rounded-xl transition-all active:scale-95"
          >
            <X size={16} strokeWidth={2.5} />
          </button>
        </div>

        {/* Search and Navigation Bar - Google Drive Styling */}
        <div className="h-14 border-b border-[var(--color-kb-panel-border)] bg-[var(--color-kb-panel)]/10 px-6 flex items-center justify-between gap-4">
          {/* Breadcrumbs */}
          <div className="flex items-center gap-1 text-[13px] text-[var(--color-kb-text-muted)] overflow-hidden">
            <button 
              onClick={() => { setActiveTab('my-drive'); setCurrentFolderId(null); }}
              className={`hover:text-[var(--color-kb-accent)] flex items-center gap-1.5 transition-colors shrink-0 font-medium ${!currentFolderId ? 'text-[var(--color-kb-text-heading)] font-semibold' : ''}`}
            >
              <Home size={15} />
              <span>{t('myDrive')}</span>
            </button>

            {breadcrumbs.map((crumb, idx) => (
              <React.Fragment key={crumb.id}>
                <ChevronRight size={14} className="opacity-60 shrink-0" />
                <button
                  onClick={() => setCurrentFolderId(crumb.id)}
                  className={`hover:text-[var(--color-kb-accent)] transition-colors truncate max-w-[120px] font-medium ${idx === breadcrumbs.length - 1 ? 'text-[var(--color-kb-text-heading)] font-semibold' : ''}`}
                >
                  {crumb.name}
                </button>
              </React.Fragment>
            ))}
          </div>

          {/* Search bar inside header */}
          <div className="flex items-center gap-3 w-[360px]">
            <div className="flex-1 flex items-center bg-[var(--color-kb-panel)] border border-[var(--color-kb-panel-border)] hover:border-[var(--color-kb-accent)]/50 focus-within:ring-1 focus-within:ring-[var(--color-kb-accent)] focus-within:border-[var(--color-kb-accent)] px-3 py-1.5 rounded-xl transition-all h-9">
              <Search size={14} className="text-[var(--color-kb-text-muted)] mr-2 shrink-0" />
              <input
                type="text"
                placeholder={t('searchPlaceholder')}
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                className="bg-transparent border-none outline-none text-[13px] font-medium text-[var(--color-kb-text-heading)] placeholder-[var(--color-kb-text-muted)] w-full focus:ring-0 focus:outline-none focus:border-none"
              />
              {searchQuery && (
                <button 
                  onClick={() => setSearchQuery('')}
                  className="text-[var(--color-kb-text-muted)] hover:text-[var(--color-kb-accent)] p-0.5 shrink-0"
                >
                  <X size={12} />
                </button>
              )}
            </div>

            {/* Layout switch */}
            <div className="flex items-center bg-[var(--color-kb-panel)] border border-[var(--color-kb-panel-border)] rounded-xl p-0.5 h-9 shrink-0 shadow-inner">
              <button
                onClick={() => setViewMode('list')}
                className={`p-1.5 rounded-lg transition-all ${viewMode === 'list' ? 'bg-[var(--color-kb-editor)] text-[var(--color-kb-accent)] shadow-sm' : 'text-[var(--color-kb-text-muted)] hover:text-current'}`}
                title={t('listView')}
              >
                <List size={14} />
              </button>
              <button
                onClick={() => setViewMode('grid')}
                className={`p-1.5 rounded-lg transition-all ${viewMode === 'grid' ? 'bg-[var(--color-kb-editor)] text-[var(--color-kb-accent)] shadow-sm' : 'text-[var(--color-kb-text-muted)] hover:text-current'}`}
                title={t('gridView')}
              >
                <LayoutGrid size={14} />
              </button>
            </div>
          </div>
        </div>

        {/* Content Body */}
        <div className="flex-1 flex min-h-0">
          
          {/* Left Sidebar */}
          <div className="w-[180px] border-r border-[var(--color-kb-panel-border)] bg-[var(--color-kb-panel)]/20 p-3 space-y-1 shrink-0 flex flex-col justify-between">
            <div className="space-y-1">
              <button
                onClick={() => { setActiveTab('my-drive'); setCurrentFolderId(null); }}
                className={`w-full flex items-center gap-3 px-3 py-2.5 rounded-xl text-[13px] font-semibold transition-all ${activeTab === 'my-drive' ? 'bg-[var(--color-kb-accent)]/10 text-[var(--color-kb-accent)] shadow-sm' : 'text-[var(--color-kb-text)] hover:bg-[var(--color-kb-panel-hover)]'}`}
              >
                <ServerIcon size={16} />
                <span>{t('myFiles')}</span>
              </button>
              <button
                onClick={() => { setActiveTab('shared'); setCurrentFolderId(null); }}
                className={`w-full flex items-center gap-3 px-3 py-2.5 rounded-xl text-[13px] font-semibold transition-all ${activeTab === 'shared' ? 'bg-[var(--color-kb-accent)]/10 text-[var(--color-kb-accent)] shadow-sm' : 'text-[var(--color-kb-text)] hover:bg-[var(--color-kb-panel-hover)]'}`}
              >
                <Users size={16} />
                <span>{t('sharedWithMe')}</span>
              </button>
              <button
                onClick={() => { setActiveTab('recent'); setCurrentFolderId(null); }}
                className={`w-full flex items-center gap-3 px-3 py-2.5 rounded-xl text-[13px] font-semibold transition-all ${activeTab === 'recent' ? 'bg-[var(--color-kb-accent)]/10 text-[var(--color-kb-accent)] shadow-sm' : 'text-[var(--color-kb-text)] hover:bg-[var(--color-kb-panel-hover)]'}`}
              >
                <Clock size={16} />
                <span>{t('recentAccess')}</span>
              </button>
              <button
                onClick={() => { setActiveTab('starred'); setCurrentFolderId(null); }}
                className={`w-full flex items-center gap-3 px-3 py-2.5 rounded-xl text-[13px] font-semibold transition-all ${activeTab === 'starred' ? 'bg-[var(--color-kb-accent)]/10 text-[var(--color-kb-accent)] shadow-sm' : 'text-[var(--color-kb-text)] hover:bg-[var(--color-kb-panel-hover)]'}`}
              >
                <Star size={16} />
                <span>{t('starredFiles')}</span>
              </button>
            </div>

            {/* Cloud Drive Status info */}
            <div className="p-3 bg-[var(--color-kb-accent)]/5 rounded-xl border border-[var(--color-kb-accent)]/10 text-[11px] text-[var(--color-kb-text-muted)] space-y-1.5">
              <div className="flex items-center gap-1.5 text-[var(--color-kb-accent)] font-semibold">
                <Layers size={12} />
                <span>{t('quotaLimit')}</span>
              </div>
              <div className="w-full bg-[var(--color-kb-panel-border)] h-1 rounded-full overflow-hidden">
                <div className="bg-[var(--color-kb-accent)] h-full w-[42%]" />
              </div>
              <div className="flex justify-between font-mono">
                <span>4.2 GB Used</span>
                <span>10 GB Total</span>
              </div>
            </div>
          </div>

          {/* Right Main File Explorer Pane */}
          <div className="flex-1 flex flex-col bg-[var(--color-kb-editor)] relative overflow-hidden min-w-0">
            {isSyncing ? (
              <div className="absolute inset-0 z-50 bg-[var(--color-kb-editor)]/90 backdrop-blur-sm flex flex-col items-center justify-center text-center p-6">
                <RefreshCw size={42} className="text-[var(--color-kb-accent)] animate-spin mb-4" />
                <h4 className="text-[15px] font-bold text-[var(--color-kb-text-heading)] mb-1">{t('syncingTitle')}</h4>
                <p className="text-[12px] text-[var(--color-kb-text-muted)] max-w-sm">{feedbackMsg}</p>
              </div>
            ) : null}

            {displayedFiles.length === 0 ? (
              <div className="flex-1 flex flex-col items-center justify-center p-8 text-center select-none">
                <FileMinus size={40} className="text-[var(--color-kb-text-muted)] opacity-60 mb-3" />
                <h4 className="text-[14px] font-bold text-[var(--color-kb-text-heading)]">{t('emptyFolderTitle')}</h4>
                <p className="text-[12px] text-[var(--color-kb-text-muted)] mt-1">{t('emptyFolderDesc')}</p>
              </div>
            ) : viewMode === 'list' ? (
              // List View Mode
              <div className="flex-1 overflow-y-auto w-full">
                <table className="w-full text-left text-[12px] border-collapse relative">
                  <thead className="sticky top-0 bg-[var(--color-kb-editor)] shadow-[0_1px_0_0_rgba(0,0,0,0.05)] border-b border-[var(--color-kb-panel-border)] z-10 select-none">
                    <tr className="text-[var(--color-kb-text-muted)] font-semibold">
                      <th className="w-12 pl-6 py-3">
                        {currentSelectableFiles.length > 0 && (
                          <button 
                            onClick={handleSelectAll} 
                            className="p-1 hover:bg-[var(--color-kb-panel-hover)] rounded-md text-[var(--color-kb-accent)] transition-all"
                            title={t('selectAll')}
                          >
                            {selectedIds.size === currentSelectableFiles.length ? (
                              <CheckSquare size={16} />
                            ) : (
                              <Square size={16} />
                            )}
                          </button>
                        )}
                      </th>
                      <th className="py-3 font-semibold text-[13px] text-[var(--color-kb-text-heading)]">{t('name')}</th>
                      <th className="py-3 w-32 font-semibold">{t('updatedAt')}</th>
                      <th className="py-3 w-28 font-semibold">{t('owner')}</th>
                      <th className="py-3 w-24 font-semibold pr-6 text-right">{t('size')}</th>
                    </tr>
                  </thead>
                  <tbody className="divide-y divide-[var(--color-kb-panel-border)]/50">
                    {displayedFiles.map(item => {
                      const isFolder = item.type === 'folder';
                      const isSelected = selectedIds.has(item.id);
                      return (
                        <tr 
                          key={item.id}
                          onClick={() => {
                            if (isFolder) {
                              setCurrentFolderId(item.id);
                            }
                          }}
                          className={`hover:bg-[var(--color-kb-panel-hover)]/60 transition-colors group cursor-pointer ${isSelected ? 'bg-[var(--color-kb-accent)]/5' : ''}`}
                        >
                          <td className="pl-6 py-3" onClick={(e) => e.stopPropagation()}>
                            <button 
                              onClick={(e) => handleToggleSelect(item, e)}
                              className="p-1 hover:text-[var(--color-kb-accent)] transition-all"
                            >
                              {isSelected ? (
                                <CheckSquare size={15} className="text-[var(--color-kb-accent)]" />
                              ) : (
                                <Square size={15} className="text-[var(--color-kb-text-muted)]/70 group-hover:text-[var(--color-kb-text-muted)]" />
                              )}
                            </button>
                          </td>
                          <td className="py-3 pr-4 font-medium text-[13px] text-[var(--color-kb-text-heading)]">
                            <div className="flex items-center gap-3">
                              {renderFileIcon(item)}
                              <span className="truncate max-w-[340px] group-hover:text-[var(--color-kb-accent)] transition-colors">{item.name}</span>
                              {item.isStarred && (
                                <Star size={12} className="fill-amber-400 stroke-amber-400 shrink-0" />
                              )}
                              {isFolder && (
                                <span className="ml-auto opacity-0 group-hover:opacity-100 flex items-center text-[10px] text-[var(--color-kb-accent)] font-semibold gap-0.5 shrink-0 transition-all">
                                  <span>{t('clickToEnter')}</span>
                                  <ChevronRight size={10} />
                                </span>
                              )}
                            </div>
                          </td>
                          <td className="py-3 text-[var(--color-kb-text-muted)] font-mono">{item.updatedAt}</td>
                          <td className="py-3 text-[var(--color-kb-text-muted)]">{item.ownerString || t('me')}</td>
                          <td className="py-3 pr-6 text-right text-[var(--color-kb-text-muted)] font-mono">{item.size || t('folderSize')}</td>
                        </tr>
                      );
                    })}
                  </tbody>
                </table>
              </div>
            ) : (
              // Grid View Mode
              <div className="flex-1 overflow-y-auto p-6">
                <div className="grid grid-cols-4 gap-4">
                  {displayedFiles.map(item => {
                    const isFolder = item.type === 'folder';
                    const isSelected = selectedIds.has(item.id);
                    return (
                      <div
                        key={item.id}
                        onClick={() => {
                          if (isFolder) {
                            setCurrentFolderId(item.id);
                          }
                        }}
                        className={`border border-[var(--color-kb-panel-border)] rounded-2xl p-4 flex flex-col justify-between hover:border-[var(--color-kb-accent)] hover:shadow-md transition-all cursor-pointer group h-[120px] relative ${isSelected ? 'bg-[var(--color-kb-accent)]/[0.04] !border-[var(--color-kb-accent)]' : 'bg-[var(--color-kb-panel)]/10'}`}
                      >
                        {/* Selector checkbox */}
                        <button
                          onClick={(e) => handleToggleSelect(item, e)}
                          className="absolute top-3 left-3 opacity-0 group-hover:opacity-100 focus:opacity-100 transition-opacity"
                        >
                          {isSelected ? (
                            <CheckSquare size={16} className="text-[var(--color-kb-accent)] opacity-100" />
                          ) : (
                            <Square size={16} className="text-[var(--color-kb-text-muted)]" />
                          )}
                        </button>

                        <div className="flex justify-end gap-1.5 text-[var(--color-kb-text-muted)]">
                          {item.isStarred && (
                            <Star size={13} className="fill-amber-400 stroke-amber-400" />
                          )}
                        </div>

                        <div className="flex flex-col items-center justify-center text-center py-1">
                          {renderGridIcon(item)}
                          <p className="text-[12px] font-semibold text-[var(--color-kb-text-heading)] truncate max-w-[150px] w-full px-1">{item.name}</p>
                        </div>

                        <div className="flex items-center justify-between mt-2 pt-2 border-t border-[var(--color-kb-panel-border)]/50 text-[10px] text-[var(--color-kb-text-muted)] font-mono leading-none">
                          <span>{item.updatedAt}</span>
                          <span>{item.size || t('folderSize')}</span>
                        </div>
                      </div>
                    );
                  })}
                </div>
              </div>
            )}
          </div>
        </div>

        {/* Footer Drawer showing details and actions */}
        <div className="h-16 border-t border-[var(--color-kb-panel-border)] bg-[var(--color-kb-panel)] flex items-center justify-between px-6 shrink-0 shadow-[0_-5px_15px_rgba(0,0,0,0.02)] select-none">
          <div className="flex items-center gap-2">
            {selectedIds.size > 0 ? (
              <div className="flex items-center gap-2.5 text-[var(--color-kb-accent)] font-semibold text-[13px]">
                <CheckSquare size={16} />
                <span>{t('selectedCount')}</span>
                <span className="text-[11px] text-[var(--color-kb-text-muted)] font-normal ml-2">{t('incompatibleNoticePrefix')}<b>{t('wechatAppletCard')}</b>{t('incompatibleNoticeSuffix')}</span>
              </div>
            ) : (
              <div className="flex items-center gap-2 text-[var(--color-kb-text-muted)] text-[12px]">
                <AlertCircle size={15} />
                <span>{t('selectHint')}</span>
              </div>
            )}
          </div>

          <div className="flex items-center gap-3">
            <button
              onClick={onClose}
              disabled={isSyncing}
              className="px-5 py-2 text-[13px] font-medium text-[var(--color-kb-text-heading)] hover:bg-[var(--color-kb-panel-hover)] border border-[var(--color-kb-panel-border)] rounded-xl transition-all disabled:opacity-50"
            >
              {t('cancel', { defaultValue: '取消' })}
            </button>
            <button
              onClick={handleImportClick}
              disabled={selectedIds.size === 0 || isSyncing}
              className="px-6 py-2 text-[13px] font-medium bg-[var(--color-kb-accent)] hover:bg-[var(--color-kb-accent-hover)] text-white font-semibold rounded-xl shadow-[0_4px_12px_rgba(37,99,235,0.2)] hover:shadow-[0_6px_16px_rgba(37,99,235,0.3)] transition-all disabled:opacity-40 disabled:cursor-not-allowed flex items-center gap-1.5"
            >
              <Download size={14} />
              <span>{t('importCount')}</span>
            </button>
          </div>
        </div>

      </div>
    </div>,
    document.body
  );
}

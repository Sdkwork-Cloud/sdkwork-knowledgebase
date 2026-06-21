import React, { useState, useRef, useEffect } from 'react';
import { isBlank, trim } from '@sdkwork/sdkwork-knowledgebase-pc-commons/stringUtils';
import { useLocation, useNavigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { 
  X, ChevronUp, ChevronDown, Plus, Trash2, Image as ImageIcon, 
  Settings2, Eye, FileText, RotateCcw, RotateCw, Paintbrush, 
  Bold, Italic, Underline, Strikethrough, AlignLeft, AlignCenter, 
  AlignRight, AlignJustify, List, ListOrdered, Quote, Search,
  Sparkles, Notebook, Video, ArrowLeft, ArrowUp, ArrowDown, Cloud,
  Check, Globe, Key, Upload, Folder, Tags, Wand2
} from 'lucide-react';
import { getKnowledgebaseTenantId, readRegisteredSpaces } from 'sdkwork-knowledgebase-pc-core';
import { DocumentMeta } from './services/document';
import { AiAssistantPanel } from './AiAssistantPanel';
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger } from './components/ui/dropdown-menu';
import { marked } from 'marked';

import { WechatWidgetTemplates } from './utils/wechatWidgetTemplates';
import { TiptapEditor } from './TiptapEditor';
import { WechatArticleSettings } from './WechatArticleSettings';
import { WechatPreviewModal } from './WechatPreviewModal';
import { WechatSendPreviewModal } from './WechatSendPreviewModal';
import { NotesAppModal } from './NotesAppModal';
import { CloudDriveModal } from './CloudDriveModal';
import { WechatImageCropperModal } from './components/WechatImageCropperModal';
import { WechatAiImageModal } from './components/WechatAiImageModal';
import { WechatScanModal } from './components/WechatScanModal';
import { InsertToolsMenu, DropdownItem, DropdownDivider } from './components/InsertToolsMenu';
import { AssetLibraryModal, AssetType } from './components/AssetLibraryModal';
import { OfficialAccountModal } from './components/OfficialAccountModal';
import { WechatAppletModal } from './components/WechatAppletModal';
import { WechatArticleListThumb } from './components/WechatArticleListThumb';
import { InsertToolConfigModal } from './components/InsertToolConfigModal';
import { TypographyModal } from './components/TypographyModal';
import { WechatPublishModal } from './components/WechatPublishModal';

import { ExtractCoverFromBodyModal } from './components/ExtractCoverFromBodyModal';

import { WechatService, OfficialAccount, WechatArticle } from './services/wechat';
import { AIService } from './services/ai';

import { toast } from './components/ui/toast-manager';
import { useLocalStorage } from '@packages/sdkwork-knowledgebase-pc-commons/src';

export interface WechatPublishPageProps {
  documents?: DocumentMeta[];
  onClose?: () => void;
}

// 顶部插入工具栏
const INSERT_TOOLS = [
  '图片', '视频', '音频', '超链接', '小程序', '卡券', '模板', 
  '投票', '搜索', '地理位置', '视频号', '问答', '收入变现', 
  '账号名片', '礼物', '...'
];

function getWechatWordCount(content: string | undefined): number {
  if (!content) return 0;
  // HTML tags strip
  let text = content.replace(/<[^>]*>/g, ' ');
  // Decode HTML entities
  text = text.replace(/&nbsp;/gi, ' ')
             .replace(/&lt;/gi, '<')
             .replace(/&gt;/gi, '>')
             .replace(/&amp;/gi, '&');
  
  text = text.trim();
  if (!text) return 0;

  let count = 0;
  
  // 1. Chinese/CJK characters + full-width punctuation
  const cjkRegex = /[\u4e00-\u9fa5\u3000-\u303f\uff00-\uffef]/g;
  const cjkMatches = text.match(cjkRegex) || [];
  count += cjkMatches.length;
  
  // Strip them away to count western components
  const nonCjkText = text.replace(/[\u4e00-\u9fa5\u3000-\u303f\uff00-\uffef]/g, ' ');
  
  // 2. English words and alphanumeric words
  const engWordsRegex = /[a-zA-Z0-9]+'?[a-zA-Z0-9]*/g;
  const engWordsMatches = nonCjkText.match(engWordsRegex) || [];
  count += engWordsMatches.length;
  
  // 3. Half-width single symbols
  const symbolText = nonCjkText.replace(/[a-zA-Z0-9]/g, ' ');
  const symbolRegex = /[^\s]/g;
  const symbolMatches = symbolText.match(symbolRegex) || [];
  count += symbolMatches.length;

  return count;
}

function processDocumentContent(doc: DocumentMeta): string {
  if (!doc.content && !doc.url) return '';
  if (doc.type === 'markdown') {
    return marked.parse(doc.content || '', { async: false }) as string;
  }
  if (doc.type === 'code') {
    return `<pre><code>${(doc.content || '').replace(/</g, '&lt;').replace(/>/g, '&gt;')}</code></pre>`;
  }
  if (doc.type === 'image' && doc.url) {
    return `<img src="${doc.url}" alt="${doc.title || ''}" />`;
  }
  if (doc.type === 'video' && doc.url) {
    return `<video src="${doc.url}" controls></video>`;
  }
  if (doc.type === 'audio' && doc.url) {
    return `<audio src="${doc.url}" controls></audio>`;
  }
  return doc.content || '';
}

export function WechatPublishPage({ documents: defaultDocuments = [], onClose }: WechatPublishPageProps) {
  const location = useLocation();
  const { t } = useTranslation('editor');
  const navigate = useNavigate();
  const documents = location.state?.documents || defaultDocuments;
  const cloudDriveSpaceId = React.useMemo(() => {
    const tenantId = getKnowledgebaseTenantId();
    if (!tenantId) {
      return null;
    }
    const spaces = readRegisteredSpaces(tenantId);
    return spaces[0] ? String(spaces[0].spaceId) : null;
  }, []);

  const [articles, setArticles] = useState<WechatArticle[]>(
    documents.length > 0 ? documents.map((d, index) => ({
       id: d.id,
       title: d.title || `${t('newArticle', { defaultValue: '新文章' })} ${index + 1}`,
       author: d.author || '',
       content: processDocumentContent(d),
       cover: '',
       abstract: ''
     })) : [{
       id: `new-${Date.now()}`,
       title: t('newArticle', { defaultValue: '新文章' }),
       author: '',
       content: '',
       cover: '',
       abstract: ''
     }]
  );
  
  const [selectedIndex, setSelectedIndex] = useState(0);
  const selectedArticle = articles[selectedIndex];

  // Official Accounts state and default values block with grouping support
  const [oaGroups, setOaGroups] = useState<string[]>(t('oaGroups', { returnObjects: true }) as string[]);
  const [selectedGroupFilter, setSelectedGroupFilter] = useState<string>('all');
  const [showGroupManager, setShowGroupManager] = useState<boolean>(false);
  const [newGroupNameInput, setNewGroupNameInput] = useState<string>('');

  const [officialAccounts, setOfficialAccounts] = useState<OfficialAccount[]>([]);
  const [selectedOfficialAccountIds, setSelectedOfficialAccountIds] = useState<string[]>(['1']);
  const [authorHistory, setAuthorHistory] = useLocalStorage<string[]>('wechat_author_history', []);
  const [isOfficialAccountModalOpen, setIsOfficialAccountModalOpen] = useState(false);
  const [isWechatAppletModalOpen, setIsWechatAppletModalOpen] = useState(false);
  
  const selectedOfficialAccounts = officialAccounts.filter(oa => selectedOfficialAccountIds.includes(oa.id));
  const currentOaName = selectedOfficialAccounts[0]?.name || '';

  const getFallbackAuthor = () => {
    let parsedHistory: string[] = [];
    try { 
      parsedHistory = JSON.parse(window.localStorage.getItem('wechat_author_history') || '[]'); 
    } catch(e){}
    return parsedHistory.length > 0 ? parsedHistory[0] : (selectedOfficialAccounts[0]?.name || '');
  };

  useEffect(() => {
    WechatService.getOfficialAccounts().then(accounts => {
      setOfficialAccounts(accounts);
      const selected = accounts.filter(oa => selectedOfficialAccountIds.includes(oa.id));
      const oaName = selected[0]?.name || '';
      
      let parsedHistory: string[] = [];
      try { parsedHistory = JSON.parse(window.localStorage.getItem('wechat_author_history') || '[]'); } catch(e){}
      const fallbackAuthor = parsedHistory.length > 0 ? parsedHistory[0] : oaName;

      setArticles(prev => prev.map(a => {
        if (!a.author) {
          return { ...a, author: fallbackAuthor };
        }
        return a;
      }));
    });
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const moveUp = (index: number) => {
    if (index === 0) return;
    const newArr = [...articles];
    const temp = newArr[index - 1];
    newArr[index - 1] = newArr[index];
    newArr[index] = temp;
    setArticles(newArr);
    if (selectedIndex === index) setSelectedIndex(index - 1);
    else if (selectedIndex === index - 1) setSelectedIndex(index);
  };

  const moveDown = (index: number) => {
    if (index === articles.length - 1) return;
    const newArr = [...articles];
    const temp = newArr[index + 1];
    newArr[index + 1] = newArr[index];
    newArr[index] = temp;
    setArticles(newArr);
    if (selectedIndex === index) setSelectedIndex(index + 1);
    else if (selectedIndex === index + 1) setSelectedIndex(index);
  };

  const removeArticle = (index: number) => {
    if (articles.length === 1) return;
    const newArr = articles.filter((_, i) => i !== index);
    setArticles(newArr);
    if (selectedIndex >= newArr.length) {
      setSelectedIndex(newArr.length - 1);
    }
  };

  const updateSelectedArticle = (updates: Partial<WechatArticle>) => {
    const newArr = [...articles];
    newArr[selectedIndex] = { ...newArr[selectedIndex], ...updates };
    setArticles(newArr);
  };

  const [isPreviewOpen, setIsPreviewOpen] = useState(false);
  const [isSendPreviewOpen, setIsSendPreviewOpen] = useState(false);
  const [isSendingPreview, setIsSendingPreview] = useState(false);
  const [previewWechatId, setPreviewWechatId] = useState('');

  const handleConfirmSendPreview = async (recipients: string[]) => {
    setIsSendingPreview(true);
    try {
      const currentAccountId = selectedOfficialAccounts[0]?.id || '';
      await WechatService.sendPreview(currentAccountId, recipients, articles);
    } finally {
      setIsSendingPreview(false);
    }
  };
  const [isPublishing, setIsPublishing] = useState(false);
  const [isPublishModalOpen, setIsPublishModalOpen] = useState(false);
  const [isAIOpen, setIsAIOpen] = useLocalStorage('wechat-publish-ai-open', false);
  const [isNotesModalOpen, setIsNotesModalOpen] = useState(false);
  const [isCloudDriveModalOpen, setIsCloudDriveModalOpen] = useState(false);

  // States for standard image cropper
  const [isCropperOpen, setIsCropperOpen] = useState(false);
  const [tempCoverToCrop, setTempCoverToCrop] = useState('');
  const [cropperSource, setCropperSource] = useState<'body' | 'gallery' | null>(null);

  // States for typesetting (formatting) progress
  const [isFormattingInProgress, setIsFormattingInProgress] = useState(false);
  const [formattingStep, setFormattingStep] = useState('');

  // States for cover upload and actions
  const [isCoverFromBodyOpen, setIsCoverFromBodyOpen] = useState(false);
  const [assetLibraryOpen, setAssetLibraryOpen] = useState(false);
  const [assetLibraryTab, setAssetLibraryTab] = useState<AssetType>('image');
  const [isWechatScanOpen, setIsWechatScanOpen] = useState(false);
  const [isAiCoverOpen, setIsAiCoverOpen] = useState(false);

  // States for scan & AI simulation
  const [scanStatus, setScanStatus] = useState<'pending' | 'scanning' | 'success'>('pending');
  const [scannedCover, setScannedCover] = useState('');
  const [extractedBodyImages, setExtractedBodyImages] = useState<string[]>([]);
  const [selectedBodyImage, setSelectedBodyImage] = useState<string>('');
  const [aiStyle, setAiStyle] = useState<'illustration' | 'photography' | 'abstract' | 'minimalist'>('abstract');
  const [aiGenerating, setAiGenerating] = useState(false);
  const [aiGenStep, setAiGenStep] = useState('');

  // Dropdown states for inserting tools
  const [imageActionTarget, setImageActionTarget] = useState<'cover' | 'editor'>('cover');

  // Active Tool Insertion states
  const [activeInsertType, setActiveInsertType] = useState<string | null>(null);

  // Floating Actions Capsule Position Tracker
  const outerRef = useRef<HTMLDivElement>(null);
  const scrollContainerRef = useRef<HTMLDivElement>(null);
  const [capsuleTop, setCapsuleTop] = useState<number | null>(null);

  const updateCapsulePosition = () => {
    requestAnimationFrame(() => {
      if (!scrollContainerRef.current || !outerRef.current) return;
      const activeEl = scrollContainerRef.current.querySelector('[data-selected="true"]');
      if (activeEl) {
        const parentRect = outerRef.current.getBoundingClientRect();
        const activeRect = activeEl.getBoundingClientRect();
        const relativeTop = (activeRect.top + activeRect.height / 2) - parentRect.top;
        setCapsuleTop(relativeTop);
      } else {
        setCapsuleTop(null);
      }
    });
  };

  useEffect(() => {
    updateCapsulePosition();
    const container = scrollContainerRef.current;
    if (container) {
      container.addEventListener('scroll', updateCapsulePosition);
    }
    window.addEventListener('resize', updateCapsulePosition);
    return () => {
      if (container) {
        container.removeEventListener('scroll', updateCapsulePosition);
      }
      window.removeEventListener('resize', updateCapsulePosition);
    };
  }, [selectedIndex, articles]);

  // Form states for various dynamic widgets
  const [widgetTitle, setWidgetTitle] = useState('');
  const [widgetSubtitle, setWidgetSubtitle] = useState('');
  const [widgetUrl, setWidgetUrl] = useState('');
  const [widgetQuota, setWidgetQuota] = useState<string>(t('widgetQuotaTemplate'));
  const [widgetMerchant, setWidgetMerchant] = useState<string>(t('widgetMerchantTemplate'));
  const [widgetCondition, setWidgetCondition] = useState<string>(t('widgetConditionTemplate'));
  const [widgetQuestion, setWidgetQuestion] = useState<string>(t('widgetQuestionTemplate'));
  const [widgetAnswer, setWidgetAnswer] = useState<string>(t('widgetAnswerTemplate'));
  const [widgetOptions, setWidgetOptions] = useState<string[]>(t('widgetOptionsTemplate', { returnObjects: true }) as string[]);
  const [widgetHtml, setWidgetHtml] = useState<string>('');

  // Reference for active Tiptap editor
  const [activeEditor, setActiveEditor] = useState<any>(null);

  const insertHtmlToEditor = (html: string) => {
    if (activeEditor && !activeEditor.isDestroyed && typeof activeEditor.chain === 'function') {
      try {
        activeEditor.chain().focus().insertContent(html).run();
      } catch (e) {
        updateSelectedArticle({
          content: (selectedArticle?.content || '') + html
        });
      }
    } else {
      updateSelectedArticle({
        content: (selectedArticle?.content || '') + html
      });
    }
  };

  const insertImageInputRef = useRef<HTMLInputElement>(null);
  const domainVerifyInputRef = useRef<HTMLInputElement>(null);
  const handleInsertImageUpload = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (file) {
      // Mock upload delay and use object URL
      setTimeout(() => {
        insertHtmlToEditor(`<p><img src="${URL.createObjectURL(file)}" alt="Local Uploaded Image" style="border-radius: 12px; max-width: 100%; margin: 16px 0; border: 1px solid var(--color-kb-panel-border);" /></p>`);
      }, 500);
    }
  };

  const handleSelectCoverFromBody = () => {
    setImageActionTarget('cover');
    const htmlContent = selectedArticle?.content || '';
    const imgRegex = /<img[^>]+src=["']([^"']+)["']/g;
    const images: string[] = [];
    let match;
    while ((match = imgRegex.exec(htmlContent)) !== null) {
      if (match[1]) {
        images.push(match[1]);
      }
    }
    setExtractedBodyImages(images);
    setIsCoverFromBodyOpen(true);
  };

  const handleSelectCoverFromGallery = () => {
    setImageActionTarget('cover');
    setAssetLibraryTab('image');
    setAssetLibraryOpen(true);
  };


  const handleWechatScanUpload = () => {
    setImageActionTarget('cover');
    setScanStatus('pending');
    setScannedCover('');
    setIsWechatScanOpen(true);
  };

  const handleAiCoverGenerate = () => {
    setImageActionTarget('cover');
    setIsAiCoverOpen(true);
  };

  const triggerScanSimulation = async () => {
    setScanStatus('scanning');
    await new Promise(r => setTimeout(r, 1200));
    const mobileCovers = [
      'https://images.unsplash.com/photo-1544716278-ca5e3f4abd8c?w=800&q=80',
      'https://images.unsplash.com/photo-1498050108023-c5249f4df085?w=800&q=80',
      'https://images.unsplash.com/photo-1517694712202-14dd9538aa97?w=800&q=80'
    ];
    const picked = mobileCovers[Math.floor(Math.random() * mobileCovers.length)];
    setScannedCover(picked);
    setScanStatus('success');
    if (imageActionTarget === 'editor') {
      insertHtmlToEditor(`<p><img src="${picked}" alt="Scanned Image" style="border-radius: 12px; max-width: 100%; margin: 16px 0; border: 1px solid var(--color-kb-panel-border);" /></p>`);
    } else {
      setTempCoverToCrop(picked);
      setIsCropperOpen(true);
    }
    setTimeout(() => setIsWechatScanOpen(false), 900);
  };

  const triggerAiCoverGeneration = async () => {
    setAiGenerating(true);
    const steps = t('aiGenSteps', { returnObjects: true }) as string[];
    for (let i = 0; i < steps.length; i++) {
      setAiGenStep(steps[i]);
      await new Promise(r => setTimeout(r, 450));
    }
    
    const styleArtMap = {
      illustration: 'https://images.unsplash.com/photo-1618005182384-a83a8bd57fbe?w=800&q=80',
      photography: 'https://images.unsplash.com/photo-1451187580459-43490279c0fa?w=800&q=80',
      abstract: 'https://images.unsplash.com/photo-1541701494587-cb58502866ab?w=800&q=80',
      minimalist: 'https://images.unsplash.com/photo-1470071459604-3b5ec3a7fe05?w=800&q=80'
    };
    
    const outcome = styleArtMap[aiStyle] || styleArtMap.abstract;
    if (imageActionTarget === 'editor') {
      insertHtmlToEditor(`<p><img src="${outcome}" alt="AI Image" style="border-radius: 12px; max-width: 100%; margin: 16px 0; border: 1px solid var(--color-kb-panel-border);" /></p>`);
    } else {
      setTempCoverToCrop(outcome);
      setIsCropperOpen(true);
    }
    setAiGenerating(false);
    setIsAiCoverOpen(false);
  };

  const handleInsertToolClick = (toolName: string) => {
    if (toolName === t('toolImage', { defaultValue: '图片' })) {
      setActiveInsertType('image');
    } else if (toolName === t('toolVideo', { defaultValue: '视频' })) {
      const widgetUrl = 'https://sample-videos.com/video321/mp4/720/big_buck_bunny_720p_1mb.mp4';
      insertHtmlToEditor(`<video src="${widgetUrl}" controls></video>`);
    } else if (toolName === t('toolAudio', { defaultValue: '音频' })) {
      const widgetUrl = 'https://www.soundhelix.com/examples/mp3/SoundHelix-Song-1.mp3';
      insertHtmlToEditor(`<audio src="${widgetUrl}" controls></audio>`);
    } else if (toolName === t('toolLink', { defaultValue: '超链接' })) {
      setWidgetTitle(t('widgetTitleLink'));
      setWidgetUrl('https://id.sdkwork.com');
      setActiveInsertType('link');
    } else if (toolName === t('toolMiniprogram', { defaultValue: '小程序' })) {
      setIsWechatAppletModalOpen(true);
    } else if (toolName === t('toolCoupons', { defaultValue: '卡券' })) {
      setWidgetQuota(t('widgetQuota2'));
      setWidgetMerchant(t('widgetMerchant2'));
      setWidgetCondition(t('widgetCondition2'));
      setActiveInsertType('coupons');
    } else if (toolName === t('toolTemplates', { defaultValue: '模板' })) {
      setActiveInsertType('templates');
    } else if (toolName === t('toolVote', { defaultValue: '投票' })) {
      setWidgetTitle(t('widgetVoteTitle'));
      setWidgetOptions(t('widgetVoteOptions', { returnObjects: true }) as string[]);
      setActiveInsertType('vote');
    } else if (toolName === t('toolSearch', { defaultValue: '搜索' })) {
      setWidgetTitle(t('widgetSearchTitle'));
      setActiveInsertType('search');
    } else if (toolName === t('toolLocation', { defaultValue: '地理位置' })) {
      setWidgetTitle(t('widgetLocationTitle'));
      setWidgetSubtitle(t('widgetLocationSubtitle'));
      setActiveInsertType('location');
    } else if (toolName === t('toolChannel', { defaultValue: '视频号' })) {
      setWidgetTitle(t('widgetChannelTitle'));
      setActiveInsertType('channel');
    } else if (toolName === t('toolQa', { defaultValue: '问答' })) {
      setWidgetQuestion(t('widgetQaQuestion'));
      setWidgetAnswer(t('widgetQaAnswer'));
      setActiveInsertType('qa');
    } else if (toolName === t('toolAd', { defaultValue: '收入变现' })) {
      setActiveInsertType('ad');
    } else if (toolName === t('toolCard', { defaultValue: '账号名片' })) {
      setWidgetTitle(t('widgetCardTitle'));
      setWidgetSubtitle(t('widgetCardSubtitle'));
      setActiveInsertType('card');
    } else if (toolName === t('toolGifts', { defaultValue: '礼物' })) {
      setWidgetTitle(t('widgetGiftsTitle'));
      setActiveInsertType('gifts');
    } else if (toolName === 'html' || toolName === t('toolHtml', { defaultValue: 'HTML代码' })) {
      setWidgetHtml('');
      setActiveInsertType('html');
    } else {
      const fallbackHtml = `
        <div style="border: 1px dashed var(--color-kb-panel-border); padding: 12px; margin: 16px 0; border-radius: 8px; text-align: center; color: var(--color-kb-text-muted); font-size: 13px;">
          ${t('widgetPlaceholder', { toolName })}
        </div>
      `;
      insertHtmlToEditor(fallbackHtml);
    }
  };

  const handleInsertConfirm = () => {
    switch (activeInsertType) {
      case 'image': {
        const pickedImage = 'https://images.unsplash.com/photo-1542281286-9e0a16bb7366?auto=format&fit=crop&w=600&q=80';
        insertHtmlToEditor(`<p><img src="${pickedImage}" alt="Gallery Cover" style="border-radius: 12px; max-width: 100%; margin: 16px 0; border: 1px solid var(--color-kb-panel-border);" /></p>`);
        break;
      }
      case 'link': {
        insertHtmlToEditor(WechatWidgetTemplates.link(widgetTitle, widgetUrl));
        break;
      }
      case 'coupons': {
        insertHtmlToEditor(WechatWidgetTemplates.coupons(widgetQuota, widgetMerchant, widgetCondition));
        break;
      }
      case 'templates': {
        insertHtmlToEditor(WechatWidgetTemplates.templates());
        break;
      }
      case 'vote': {
        insertHtmlToEditor(WechatWidgetTemplates.vote(widgetTitle, widgetOptions));
        break;
      }
      case 'search': {
        insertHtmlToEditor(WechatWidgetTemplates.search(widgetTitle));
        break;
      }
      case 'location': {
        insertHtmlToEditor(WechatWidgetTemplates.location(widgetTitle, widgetSubtitle));
        break;
      }
      case 'channel': {
        insertHtmlToEditor(WechatWidgetTemplates.channel(widgetTitle));
        break;
      }
      case 'qa': {
        insertHtmlToEditor(WechatWidgetTemplates.qa(widgetQuestion, widgetAnswer));
        break;
      }
      case 'ad': {
        insertHtmlToEditor(WechatWidgetTemplates.ad());
        break;
      }
      case 'card': {
        insertHtmlToEditor(WechatWidgetTemplates.card(widgetTitle, widgetSubtitle));
        break;
      }
      case 'gifts': {
        insertHtmlToEditor(WechatWidgetTemplates.gifts());
        break;
      }
      case 'html': {
        if (widgetHtml.trim()) {
          insertHtmlToEditor(widgetHtml);
        }
        break;
      }
    }
    setActiveInsertType(null);
  };

  const handleImportNotes = (items: Array<{ title: string; type: string; content?: string }>) => {
    const fallbackAuthor = getFallbackAuthor();
    const newArticles = items.map((item, index) => ({
      id: `note-${Date.now()}-${index}`,
      title: item.title,
      author: fallbackAuthor,
      content: item.content || '',
      cover: '',
      abstract: ''
    }));
    setArticles([...articles, ...newArticles]);
    setSelectedIndex(articles.length);
    setIsNotesModalOpen(false);
  };

  const handleImportCloudDrive = (items: Array<{ title: string; type: string; content?: string; size?: string; updatedAt?: string; owner?: string }>) => {
    // Filter items:
    // Supported natively as new articles: markdown, richtext, and HTML
    const supported = items.filter(i => i.type === 'markdown' || i.type === 'richtext' || i.type === 'html' || (i.type === 'file' && i.title.toLowerCase().endsWith('.html')));
    // Unsupported as native texts: folder, file (which includes custom extensions)
    const unsupported = items.filter(i => i.type === 'folder' || (i.type === 'file' && !i.title.toLowerCase().endsWith('.html')));

    let updatedArticles = [...articles];
    let nextSelectedIndex = selectedIndex;

    // 1. If we have supported files, create new articles for them
    if (supported.length > 0) {
      const newArticles = supported.map((item, index) => {
        let textContent = item.content || '';
        if (item.type === 'markdown') {
          try {
            textContent = marked.parse(textContent) as string;
          } catch (_) {
            textContent = `<p>${textContent}</p>`;
          }
        }
        const cleanTitle = item.title.replace(/\.(md|markdown|richtext|html)$/i, '');
        return {
          id: `drive-${Date.now()}-${index}`,
          title: cleanTitle,
          author: t('driveAuthor'),
          content: textContent,
          cover: '',
          abstract: t('driveAbstract', { title: cleanTitle })
        };
      });
      // Append and select the first newly created article
      nextSelectedIndex = updatedArticles.length;
      updatedArticles = [...updatedArticles, ...newArticles];
      setArticles(updatedArticles);
      setSelectedIndex(nextSelectedIndex);
    }

    // 2. If we have unsupported files (like folders, pdf, zip, videos),
    // they represent WeChat Applet Cards to insert into the currently selected article's editor.
    // If there is no article yet, let's make sure we have one active article to insert into.
    if (unsupported.length > 0) {
      if (updatedArticles.length === 0) {
        // Create an empty placeholder article first
        const placeholderArticle = {
          id: `drive-holder-${Date.now()}`,
          title: t('driveHolderTitle'),
          author: t('driveHolderAuthor'),
          content: '',
          cover: '',
          abstract: t('driveHolderAbstract')
        };
        updatedArticles = [placeholderArticle];
        nextSelectedIndex = 0;
        setArticles(updatedArticles);
        setSelectedIndex(0);
      }

      // Generate the beautiful WeChat Applet Card HTML
      const cardsHtml = unsupported.map((item) => {
        const itemType = item.type;
        const icon = itemType === 'folder' ? '📂' : '📎';
        
        // Custom branding and colors based on extension
        const nameLower = item.title.toLowerCase();
        let extensionBg = 'rgba(7, 193, 96, 0.08)';
        let extensionColor = '#07c160';
        let extensionLabel = t('zipArchive', { defaultValue: 'ZIP 归档' });
        let actionBtnText = t('previewFile', { defaultValue: '预览文件' });

        if (itemType === 'folder') {
          extensionBg = 'rgba(217, 119, 6, 0.08)';
          extensionColor = '#d97706';
          extensionLabel = t('sharedFolder', { defaultValue: '共享文件夹' });
          actionBtnText = t('enterDriveFolder', { defaultValue: '进入网盘文件夹' });
        } else if (nameLower.endsWith('.pdf')) {
          extensionBg = 'rgba(239, 68, 68, 0.08)';
          extensionColor = '#ef4444';
          extensionLabel = t('pdfDocument', { defaultValue: 'PDF 电子档' });
          actionBtnText = t('fastPreviewOnline', { defaultValue: '在线极速预览' });
        } else if (nameLower.endsWith('.zip')) {
          extensionBg = 'rgba(147, 51, 234, 0.08)';
          extensionColor = '#9333ea';
          extensionLabel = t('zipPackage', { defaultValue: 'ZIP 软件包' });
          actionBtnText = t('extractedResource', { defaultValue: '提取资源内容' });
        } else if (nameLower.endsWith('.mp4') || nameLower.endsWith('.mov')) {
          extensionBg = 'rgba(249, 115, 22, 0.08)';
          extensionColor = '#f97316';
          extensionLabel = t('mp4Video', { defaultValue: 'MP4 超清音视频' });
          actionBtnText = t('hdTheaterPlayback', { defaultValue: '高清剧场版播放' });
        }

        const sizeText = item.size ? t('fileSizeFormat', { size: item.size, defaultValue: `文件大小：${item.size}` }) : t('fullFolderStructure', { defaultValue: '同步类型：全量目录结构' });
        const timeText = item.updatedAt ? t('updateTimeFormat', { time: item.updatedAt, defaultValue: ` · 更新：${item.updatedAt}` }) : '';

        return WechatWidgetTemplates.appletCard(item.title, icon, extensionBg, extensionColor, sizeText, timeText, actionBtnText);
      }).join('\n');

      // Insert HTML into the active editor or back up selected draft
      if (activeEditor) {
        activeEditor.commands.insertContent(cardsHtml);
      } else {
        const targetArticle = updatedArticles[nextSelectedIndex];
        if (targetArticle) {
          targetArticle.content = (targetArticle.content || '') + cardsHtml;
          setArticles(updatedArticles);
        }
      }
    }

    setIsCloudDriveModalOpen(false);
  };
  
  // Resizable AI panel logic
  const [aiWidth, setAiWidth] = useLocalStorage('wechat-publish-ai-width', 420);
  const [isDraggingAi, setIsDraggingAi] = useState(false);
  const aiWidthRef = useRef(aiWidth);

  useEffect(() => {
    aiWidthRef.current = aiWidth;
  }, [aiWidth]);

  useEffect(() => {
    const handleMouseMove = (e: MouseEvent) => {
      if (!isDraggingAi) return;
      const newWidth = window.innerWidth - e.clientX;
      if (newWidth > 240 && newWidth < 800) {
        setAiWidth(newWidth);
      }
    };
    const handleMouseUp = () => {
      setIsDraggingAi(false);
    };

    if (isDraggingAi) {
      document.addEventListener('mousemove', handleMouseMove);
      document.addEventListener('mouseup', handleMouseUp);
      document.body.style.cursor = 'col-resize';
      document.body.style.userSelect = 'none';
    } else {
      document.removeEventListener('mousemove', handleMouseMove);
      document.removeEventListener('mouseup', handleMouseUp);
      document.body.style.cursor = '';
      document.body.style.userSelect = '';
    }

    return () => {
      document.removeEventListener('mousemove', handleMouseMove);
      document.removeEventListener('mouseup', handleMouseUp);
    };
  }, [isDraggingAi]);
  
  const handlePublish = () => {
    setIsPublishModalOpen(true);
  };

  const handleConfirmPublishWithOptions = async (options: {
    sendNotification: boolean;
    groupNotification: boolean;
    selectedGroupId: string;
    scheduleTime: string | null;
  }) => {
    setIsPublishing(true);
    try {
      await WechatService.publishArticles(selectedOfficialAccountIds, articles, options);
      if (onClose) {
        setTimeout(() => {
          onClose();
        }, 1000);
      } else {
        setTimeout(() => {
          navigate(-1);
        }, 1000);
      }
    } catch (e) {
      console.error(e);
      throw e;
    } finally {
      setIsPublishing(false);
    }
  };

  const [isTypographyModalOpen, setIsTypographyModalOpen] = useState(false);
  const [isRewritingStreaming, setIsRewritingStreaming] = useState(false);

  const handleAutoFormat = () => {
    if (!selectedArticle) return;
    setIsTypographyModalOpen(true);
  };

  const handleStreamingRewrite = async () => {
    if (!selectedArticle) return;
    setIsRewritingStreaming(true);
    try {
      await AIService.streamRewrite(selectedArticle.content || '', (chunk) => {
        updateSelectedArticle({ content: chunk });
        if (activeEditor) {
          activeEditor.commands.setContent(chunk);
        }
      });
      toast.success(t('rewriteSuccess'));
    } catch (e) {
      console.error(e);
      toast.error(t('rewriteError'));
    } finally {
      setIsRewritingStreaming(false);
    }
  };

  const handleStreamingContinue = async () => {
    if (!selectedArticle) return;
    setIsRewritingStreaming(true);
    try {
      const originalContent = selectedArticle.content || '';
      const continuationChunks = [
        `<p><br></p>`,
        `<p><strong>💡 延伸洞察与行动路线 (AI 增补)：</strong></p>`,
        `<p>面对上述新趋势，第一步核心策略在于实现低成本、高精度的快速落地试验。其次，要注重建立完善的反馈闭环，以此在多维度的应用场景中最大化释放业务价值。此外，技术团队应当确保接口层面的充分解耦，使整体框架在未来高并发、长连接、高算力周期的演进中，依然拥有极佳的韧性与横向扩展空间。</p>`,
        `<p>总结来说，真正的技术升级不光是框架的更迭，更是技术认知体系的全面重组。唯有时刻保持前沿敏锐度，方能保持数字化长跑的领先优势。</p>`
      ];
      
      let addedContent = "";
      for (const paragraph of continuationChunks) {
        const chars = paragraph.split("");
        for (let j = 0; j < chars.length; j += 4) {
          const nextLetters = chars.slice(j, j + 4).join("");
          addedContent += nextLetters;
          const targetContent = originalContent + addedContent;
          
          updateSelectedArticle({ content: targetContent });
          if (activeEditor) {
            activeEditor.commands.setContent(targetContent);
          }
          await new Promise(r => setTimeout(r, 12));
        }
        addedContent += "\n";
        await new Promise(r => setTimeout(r, 80));
      }
      toast.success('文章流式续写成功！');
    } catch (e) {
      console.error(e);
      toast.error('续写失败');
    } finally {
      setIsRewritingStreaming(false);
    }
  };

  const handleCreateNewArticleAndWrite = async (title: string, promptTopic?: string) => {
    const newId = `new-${Date.now()}`;
    const defaultTitle = title || '智能新篇章';
    const newArt: WechatArticle = {
      id: newId,
      title: defaultTitle,
      author: 'AI 智能助手',
      content: `<p>正在思考并流式起草《${defaultTitle}》...</p>`,
      cover: '',
      abstract: '基于 AI 智能体一键生成的深度前沿洞察文稿大纲。'
    };
    
    // Add to articles list and select it
    const updatedArticles = [...articles, newArt];
    setArticles(updatedArticles);
    const newIndex = updatedArticles.length - 1;
    setSelectedIndex(newIndex);
    
    setIsRewritingStreaming(true);
    try {
      await new Promise(r => setTimeout(r, 600)); // nice UX delay
      
      const contentParagraphs = [
        `<h1>🌐 ${defaultTitle}：AI 智能时代的生产力范式跃迁</h1>`,
        `<p>当下，人工智能技术浪潮正以破竹之势席卷全球。从单一维度的辅助生成，到多模态语义对齐的自主行动体（Autonomous Agent），科技的发展正以前所未有的速率重写人类商业与日常协作的范式。</p>`,
        `<h2>💡 核心洞察：从机械自动化迈入认知自主化</h2>`,
        `<p>传统的软件工程偏向于规定清晰逻辑和既定流程的分发。而现代 AI 赋能的业务模型，则通过内置的多推理步骤、自动反思以及在不确定场景下的感知决策，颠覆了我们对传统工具的定义。</p>`,
        `<div style="margin: 20px 0; background-color: var(--color-kb-panel-hover); border: 1.5px solid var(--color-kb-panel-border); border-left: 5px solid var(--color-kb-accent); border-radius: 12px; padding: 16px;" class="kb-mcp-block">
           <span style="color: #ffffff; font-size: 12px; font-weight: bold; background-color: var(--color-kb-accent); padding: 3px 10px; border-radius: 6px;">🎯 核心金句</span>
           <p style="font-size: 13px; color: var(--color-kb-text); line-height: 1.6; margin: 8px 0 0 0;">
             “技术升维的终极目的从来不是机械地堆砌指令，而是为人性的创意释放提供更为呼吸感、无感知的赋能沙盒。”
           </p>
         </div>`,
        `<h2>🚀 引领未来：如何布局智能化资产沉淀</h2>`,
        `<p>针对未来的智能化运营，企业及个人均须确立以下三大行动立足点：首先是建立深度的知识库对齐；其次是训练高执行力的定制化轻量模型；最后是勇于设计端到端的闭环业务节点。只有如此，才能在未来的长跑中，长久锁定价值链的核心优势。</p>`
      ];
      
      let draftedContent = "";
      for (let i = 0; i < contentParagraphs.length; i++) {
        const part = contentParagraphs[i];
        const chars = part.split("");
        for (let j = 0; j < chars.length; j += 4) {
          const nextLetters = chars.slice(j, j + 4).join("");
          draftedContent += nextLetters;
          
          setArticles(prev => {
            const copy = [...prev];
            const idx = copy.findIndex(a => a.id === newId);
            if (idx !== -1) {
              copy[idx] = { ...copy[idx], content: draftedContent };
            }
            return copy;
          });
          
          await new Promise(r => setTimeout(r, 10));
        }
        draftedContent += "\n";
        await new Promise(r => setTimeout(r, 60));
      }
      toast.success('新文章流式起草完成！');
    } catch (e) {
      console.error(e);
      toast.error('新文章起草失败');
    } finally {
      setIsRewritingStreaming(false);
    }
  };

  const coverInputRef = React.useRef<HTMLInputElement>(null);
  const handleCoverUpload = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (file) {
      setTimeout(() => {
        setTempCoverToCrop(URL.createObjectURL(file));
        setIsCropperOpen(true);
      }, 500);
    }
  };

  return (
    <div className="w-screen h-screen bg-[var(--color-kb-panel)] flex flex-col font-sans text-[var(--color-kb-text)] overflow-hidden">
      {/* Top Header */}
      <div className="h-12 bg-[var(--color-kb-panel)] flex items-center justify-between px-4 flex-shrink-0 z-40 w-full relative border-b border-[var(--color-kb-panel-border)] shadow-xs">
        <div className="flex items-center min-w-[200px]">
          <button 
            onClick={() => onClose ? onClose() : navigate(-1)} 
            className="p-1 px-1.5 mr-2.5 hover:bg-[var(--color-kb-panel-hover)] rounded-lg text-[var(--color-kb-text-muted)] hover:text-[var(--color-kb-text-heading)] transition-colors"
            title={t('back')}
          >
            <ArrowLeft size={18} />
          </button>
          <div className="w-6 h-6 rounded-full bg-[var(--color-kb-accent)] flex items-center justify-center mr-2">
            <svg viewBox="0 0 24 24" className="w-4 h-4 text-white" fill="currentColor">
              <path d="M8.5 13.5c-.83 0-1.5-.67-1.5-1.5s.67-1.5 1.5-1.5 1.5.67 1.5 1.5-.67 1.5-1.5 1.5zm7 0c-.83 0-1.5-.67-1.5-1.5s.67-1.5 1.5-1.5 1.5.67 1.5 1.5-.67 1.5-1.5 1.5zm-3.5 4c3.87 0 7-3.13 7-7s-3.13-7-7-7-7 3.13-7 7 3.13 7 7 7zm0 2C6.48 19.5 2 15.02 2 9.5S6.48-1 12-1 22 3.48 22 9.5 17.52 19.5 12 19.5z"/>
            </svg>
          </div>
          <span className="text-sm font-semibold text-[var(--color-kb-text-heading)]">{t('titleOAEditor')}</span>
        </div>
        
        <div className="flex-1 flex justify-center items-center overflow-visible min-w-0 pr-4">
          <div className="flex items-center space-x-3 max-w-full overflow-visible">
            {/* Tiptap Rich Formatting Toolbar Portal */}
            <div id="editor-toolbar-portal" className="flex items-center min-w-0 overflow-visible"></div>
            
            {/* Visual Divider */}
            <div className="w-px h-4 bg-[var(--color-kb-panel-border)] shrink-0 hidden md:block"></div>

            {/* Dropdown Tools insertion lists */}
            <div className="flex items-center space-x-1 text-sm text-[var(--color-kb-text-muted)] overflow-visible">
              <InsertToolsMenu 
                tools={[
                  {
                    name: t('miniprogram'),
                    hasDropdown: false,
                    onClick: () => handleInsertToolClick(t('toolMiniprogram', { defaultValue: '小程序' }))
                  },
                  {
                    name: t('hyperlink'),
                    hasDropdown: false,
                    onClick: () => handleInsertToolClick(t('toolLink', { defaultValue: '超链接' }))
                  },
                  {
                    name: t('templates'),
                    hasDropdown: false,
                    onClick: () => handleInsertToolClick(t('toolTemplates', { defaultValue: '模板' }))
                  },
                  {
                    name: t('more'),
                    hasDropdown: true,
                    onClick: () => {},
                    dropdownContent: (
                      <div className="grid grid-cols-2 gap-1 p-1">
                        {[
                          { label: t('vote'), key: '投票', icon: '📊' },
                          { label: t('search'), key: '搜索', icon: '🔍' },
                          { label: t('location'), key: '地理位置', icon: '📍' },
                          { label: t('channel'), key: '视频号', icon: '📱' },
                          { label: t('qa'), key: '问答', icon: '❓' },
                          { label: t('ad'), key: '收入变现', icon: '💰' },
                          { label: t('card'), key: '账号名片', icon: '📇' },
                          { label: t('coupons'), key: '卡券', icon: '🎫' },
                          { label: t('gifts'), key: '礼物', icon: '🎁' },
                          { label: t('toolHtml', { defaultValue: 'HTML代码' }), key: 'html', icon: '💻' }
                        ].map((item) => (
                          <div 
                            key={item.key}
                            onClick={() => {
                              handleInsertToolClick(item.key);
                              // Close menu is handled by Dropdown state inside InsertToolsMenu
                            }}
                            className="px-2 py-1.5 text-xs text-[var(--color-kb-text)] hover:bg-[var(--color-kb-panel-hover)] flex flex-col items-center justify-center gap-1 cursor-pointer rounded-xl transition-all font-semibold relative z-10 hover:scale-105 active:scale-95"
                          >
                            <span className="text-[16px]">{item.icon}</span>
                            <span className="text-[10px] scale-90 origin-center whitespace-nowrap">{item.label}</span>
                          </div>
                        ))}
                      </div>
                    )
                  }
                ]} 
              />
            </div>
          </div>
        </div>

        <div className="flex items-center min-w-[200px] justify-end space-x-2">
          <button onClick={() => setIsAIOpen(!isAIOpen)} className={`px-2.5 py-1 flex items-center rounded-md transition-colors ${isAIOpen ? 'bg-[var(--color-kb-accent)]/10 text-[var(--color-kb-accent)]' : 'hover:bg-[var(--color-kb-panel-hover)] text-[var(--color-kb-text-muted)]'}`} title={t('aiPanelTitle')}>
            <Sparkles size={14} className="mr-1" />
            <span className="text-xs font-semibold">{t('aiPanelTitle')}</span>
          </button>
        </div>
      </div>

      {/* Main Content Area: Left list, Center Editor, Right Actions */}
      <div className="flex-1 flex overflow-hidden">
        {/* Left: Article List */}
        <div 
          ref={outerRef}
          className="w-[240px] md:w-[260px] lg:w-[280px] xl:w-[320px] flex-shrink-0 flex flex-col bg-[var(--color-kb-panel)] border-r border-[var(--color-kb-panel-border)] z-20 relative overflow-visible hidden md:flex transition-all duration-300"
        >
          <div 
            ref={scrollContainerRef}
            onScroll={updateCapsulePosition}
            className="flex-1 flex flex-col overflow-x-hidden overflow-y-auto no-scrollbar"
          >
            <div 
              onClick={() => {
                setIsOfficialAccountModalOpen(true);
              }}
              className="p-4 border-b border-[var(--color-kb-panel-border)] flex items-center justify-between bg-[var(--color-kb-panel-hover)] sticky top-0 z-[21] backdrop-blur-sm bg-opacity-95 hover:bg-[var(--color-kb-panel)] cursor-pointer transition-all group select-none active:scale-[0.98]"
              title={t('switchManageOA')}
            >
              <div className="flex items-center min-w-0 mr-1">
                <div className="w-8 h-8 rounded-lg bg-[#07c160]/10 text-[#07c160] flex items-center justify-center mr-2.5 font-bold text-base shadow-sm shrink-0 border border-[#07c160]/20">
                  {selectedOfficialAccounts.length > 0 ? selectedOfficialAccounts[0].avatar : '👥'}
                </div>
                <div className="flex flex-col min-w-0">
                  <span className="font-semibold text-[var(--color-kb-text-heading)] text-[13.5px] truncate tracking-wide group-hover:text-[#07c160] transition-colors leading-tight">
                    {selectedOfficialAccounts.length === 0 
                      ? t('noOASelected') 
                      : selectedOfficialAccounts[0].name
                    }
                  </span>
                  <span className="text-[10px] text-[var(--color-kb-text-muted)] mt-0.5 leading-none">
                    {selectedOfficialAccounts.length > 1 
                      ? `已选 ${selectedOfficialAccounts.length} ${t('oaCountSuffix')}` 
                      : t('selectedOneOA')
                    }
                  </span>
                </div>
              </div>
              <ChevronDown size={14} className="text-zinc-400 group-hover:text-[#07c160] group-hover:translate-y-0.5 transition-all shrink-0" />
            </div>
            
            <div className="p-3">
              <div className="flex flex-col focus:outline-none border border-[var(--color-kb-panel-border)] rounded-xl overflow-hidden shadow-sm bg-[var(--color-kb-panel)] divide-y divide-[var(--color-kb-panel-border)]">
                {articles.map((article, idx) => (
                  <WechatArticleListThumb 
                    key={article.id}
                    article={article}
                    idx={idx}
                    isSelected={selectedIndex === idx}
                    onClick={() => setSelectedIndex(idx)}
                    isLast={idx === articles.length - 1}
                  />
                ))}
              </div>
            </div>

            {articles.length < 8 && (
              <div className="px-3 pb-4">
                <div className="relative group">
                  <button className="w-full flex items-center justify-center py-3 rounded-xl border border-dashed border-[var(--color-kb-panel-border)] text-[var(--color-kb-text-muted)] hover:text-[#07c160] hover:border-[#07c160] hover:bg-[#07c160]/5 transition-colors text-[13.5px] font-medium bg-[var(--color-kb-panel)] shadow-sm">
                    <Plus size={16} className="mr-1.5" /> {t('createNewContent')}
                  </button>
                  <div className="absolute left-0 top-full pt-1 hidden group-hover:block z-[210] w-full">
                    <div className="bg-[var(--color-kb-panel)] border border-[var(--color-kb-panel-border)] rounded-xl shadow-xl text-[13px] overflow-hidden">
                      <button 
                        onClick={() => {
                          setArticles([...articles, { id: `new-${Date.now()}`, title: t('placeholderArticleTitle'), author: getFallbackAuthor(), content: '', cover: '', abstract: '' }]);
                          setSelectedIndex(articles.length);
                        }}
                        className="w-full flex items-center px-4 py-3 hover:bg-[var(--color-kb-panel-hover)] text-[var(--color-kb-text)] transition-colors text-left font-medium"
                      >
                        <FileText size={15} className="mr-3 opacity-70" /> {t('writeNewArticle')}
                      </button>
                      <button 
                        onClick={() => setIsNotesModalOpen(true)}
                        className="w-full flex items-center px-4 py-3 hover:bg-[var(--color-kb-panel-hover)] text-[var(--color-kb-text)] transition-colors text-left font-medium border-t border-[var(--color-kb-panel-border)] group/btn"
                      >
                        <Notebook size={15} className="mr-3 opacity-70 group-hover/btn:text-[#07c160]" /> {t('importFromNotes')}
                      </button>
                      <button 
                        onClick={() => setIsCloudDriveModalOpen(true)}
                        className="w-full flex items-center px-4 py-3 hover:bg-[var(--color-kb-panel-hover)] text-[var(--color-kb-text)] transition-colors text-left font-medium border-t border-[var(--color-kb-panel-border)] group/btn"
                      >
                        <Cloud size={15} className="mr-3 opacity-70 group-hover/btn:text-[#07c160]" /> {t('importFromCloudDrive')}
                      </button>
                    </div>
                  </div>
                </div>
              </div>
            )}

            <div className="mt-auto p-4 flex items-center text-[var(--color-kb-text-muted)] text-sm cursor-pointer hover:text-[var(--color-kb-text)] border-t border-[var(--color-kb-panel-border)] sticky bottom-0 bg-[var(--color-kb-panel)] mt-auto">
                <span>{t('historyVersions')}</span>
                <ChevronDown size={14} className="ml-1" />
            </div>
          </div>

          {/* Floating Actions Capsule - Rendered outside of scroll context to avoid overflow clipping */}
          {selectedIndex !== -1 && capsuleTop !== null && capsuleTop > 73 && capsuleTop < (outerRef.current?.getBoundingClientRect().height || 9999) - 50 && (
            <div 
              style={{ top: `${capsuleTop}px` }}
              onClick={(e) => e.stopPropagation()}
              className="absolute left-[100%] ml-3.5 -translate-y-1/2 bg-[var(--color-kb-panel)] border border-[var(--color-kb-panel-border)] shadow-[0_12px_36px_rgba(0,0,0,0.12)] rounded-full flex flex-col items-center justify-center py-3 px-2 gap-3 z-40 transition-all duration-200 animate-in fade-in zoom-in-95 cursor-default hover:scale-105"
            >
              {/* Triangle Arrow pointing left */}
              <div className="absolute right-full top-1/2 -translate-y-1/2 flex items-center justify-center">
                <div className="w-0 h-0 border-t-[5px] border-t-transparent border-b-[5px] border-b-transparent border-r-[5px] border-r-[var(--color-kb-panel)] relative z-50 mr-[-1px]" />
                <div className="w-0 h-0 border-t-[6px] border-t-transparent border-b-[6px] border-b-transparent border-r-[6px] border-r-[var(--color-kb-panel-border)] absolute z-40" />
              </div>

              <button 
                onClick={(e) => { e.stopPropagation(); moveUp(selectedIndex); }} 
                disabled={selectedIndex === 0} 
                className="text-[var(--color-kb-text-muted)] hover:text-[#07c160] disabled:opacity-20 transition-all p-1.5 hover:bg-[var(--color-kb-panel-hover)] rounded-full hover:scale-110 active:scale-95"
                title={t('moveUp')}
              >
                <ArrowUp size={15} strokeWidth={2.5} />
              </button>
              <button 
                onClick={(e) => { e.stopPropagation(); moveDown(selectedIndex); }} 
                disabled={selectedIndex === articles.length - 1} 
                className="text-[var(--color-kb-text-muted)] hover:text-[#07c160] disabled:opacity-20 transition-all p-1.5 hover:bg-[var(--color-kb-panel-hover)] rounded-full hover:scale-110 active:scale-95"
                title={t('moveDown')}
              >
                <ArrowDown size={15} strokeWidth={2.5} />
              </button>
              <div className="w-4 h-px bg-[var(--color-kb-panel-border)]" />
              <button 
                onClick={(e) => { e.stopPropagation(); removeArticle(selectedIndex); }} 
                disabled={articles.length === 1} 
                className="text-[var(--color-kb-text-muted)] hover:text-red-500 disabled:opacity-20 transition-all p-1.5 hover:bg-red-50 dark:hover:bg-red-500/10 rounded-full hover:scale-110 active:scale-95"
                title={t('delete')}
              >
                <Trash2 size={14} />
              </button>
            </div>
          )}
        </div>

        {/* Center: Editor Area */}
        {selectedArticle && (
          <div className="flex-1 overflow-y-auto bg-[var(--color-kb-editor)] relative scroll-smooth overflow-x-hidden flex flex-col items-center">
            {/* Layout Wrapping Editor Content & Side actions side-by-side on desktop */}
            <div className="w-full max-w-[950px] 2xl:max-w-[1150px] flex flex-row items-start justify-center gap-8 py-12 pl-16 pr-8 pb-32">
              
              {/* Main article Content and Settings */}
              <div className="flex-1 max-w-[700px] 2xl:max-w-[850px] min-w-0">
                <input
                  type="text"
                  value={selectedArticle.title}
                  onChange={e => updateSelectedArticle({ title: e.target.value })}
                  className="w-full text-[32px] font-bold border-none focus:outline-none focus:ring-0 mb-4 placeholder-[var(--color-kb-text-muted)] text-[var(--color-kb-text-heading)] leading-tight bg-transparent tracking-tight"
                  placeholder={t('placeholderTitle')}
                />
                <input
                  list="author-suggestions"
                  type="text"
                  value={selectedArticle.author}
                  onChange={e => updateSelectedArticle({ author: e.target.value })}
                  onBlur={e => {
                    const val = e.target.value.trim();
                    if (val) {
                      setAuthorHistory(prev => Array.from(new Set([val, ...(prev || [])])).slice(0, 10));
                    }
                  }}
                  className="w-full text-[15px] text-[var(--color-kb-text-muted)] font-medium border-none focus:outline-none focus:ring-0 mb-8 placeholder-[var(--color-kb-text-muted)] bg-transparent opacity-80"
                  placeholder={t('placeholderAuthor')}
                />
                <datalist id="author-suggestions">
                  {currentOaName && <option value={currentOaName} />}
                  {(authorHistory || []).filter(a => a !== currentOaName).map(a => <option key={a} value={a} />)}
                </datalist>
                
                <div className="flex-1 flex flex-col w-full h-full min-h-[1000px]">
                  <TiptapEditor 
                    key={selectedArticle.id}
                    initialContent={selectedArticle.content || ''}
                    onChange={(html) => updateSelectedArticle({ content: html })}
                    onEditorReady={(editor) => setActiveEditor(editor)}
                    hideTitle={true}
                    onOpenImageGallery={() => {
                      setImageActionTarget('editor');
                      setAssetLibraryTab('image');
                      setAssetLibraryOpen(true);
                    }}
                    onWechatScan={() => {
                      setImageActionTarget('editor');
                      handleWechatScanUpload();
                    }}
                    onOpenAiImage={() => {
                      setImageActionTarget('editor');
                      setIsAiCoverOpen(true);
                    }}
                    onAudioInsert={() => {
                      insertHtmlToEditor(`<audio src="https://www.soundhelix.com/examples/mp3/SoundHelix-Song-1.mp3" controls></audio>`);
                    }}
                    onAudioGallery={() => {
                      setImageActionTarget('editor');
                      setAssetLibraryTab('audio');
                      setAssetLibraryOpen(true);
                    }}
                    onVideoGallery={() => {
                      setImageActionTarget('editor');
                      setAssetLibraryTab('video');
                      setAssetLibraryOpen(true);
                    }}
                  />
                </div>

                {/* In-view Article Settings */}
                <WechatArticleSettings
                  key={`settings-${selectedArticle.id}`}
                  selectedArticle={selectedArticle}
                  updateSelectedArticle={updateSelectedArticle}
                  coverInputRef={coverInputRef}
                  handleCoverUpload={handleCoverUpload}
                  onSelectCoverFromBody={handleSelectCoverFromBody}
                  onSelectCoverFromGallery={handleSelectCoverFromGallery}
                  onWechatScanUpload={handleWechatScanUpload}
                  onAiCoverGenerate={handleAiCoverGenerate}
                  onCropExistingCover={(url) => {
                    setImageActionTarget('cover');
                    setTempCoverToCrop(url);
                    setIsCropperOpen(true);
                  }}
                />
              </div>

              {/* Editor Right Floating Actions Section */}
              <div className="w-[180px] flex-shrink-0 hidden xl:flex flex-col gap-3 sticky top-4">
                
                {/* Unified Operations Card */}
                <div className="bg-[var(--color-kb-panel)] border border-[var(--color-kb-panel-border)] rounded-2xl p-3 space-y-2 shadow-sm flex flex-col items-center">
                  <div className="text-[10.5px] font-bold text-[var(--color-kb-text-muted)] uppercase tracking-wider w-full text-center border-b border-[var(--color-kb-panel-border)] pb-2 mb-1 select-none">
                    {t('quickTypographyAndRewrite')}
                  </div>

                  {/* One-click format button */}
                  <button 
                    onClick={handleAutoFormat}
                    className="h-10 text-center flex items-center justify-center text-[12.5px] text-[var(--color-kb-text-heading)] hover:text-[#07c160] hover:bg-[#07c160]/5 rounded-xl border border-[var(--color-kb-panel-border)] hover:border-[#07c160]/30 active:scale-95 transition-all w-full font-bold gap-1.5 cursor-pointer shadow-xs"
                  >
                    <FileText size={14} className="opacity-80" />
                    <span>{t('oneClickTypography')}</span>
                  </button>

                  {/* One-click rewrite button */}
                  <button 
                    onClick={handleStreamingRewrite}
                    disabled={isRewritingStreaming}
                    className="h-10 text-center flex items-center justify-center text-[12.5px] text-[var(--color-kb-text-heading)] hover:text-[#07c160] hover:bg-[#07c160]/5 rounded-xl border border-[var(--color-kb-panel-border)] hover:border-[#07c160]/30 active:scale-95 transition-all w-full font-bold gap-1.5 cursor-pointer shadow-sm disabled:opacity-50 disabled:cursor-not-allowed"
                  >
                    <Wand2 size={14} className={isRewritingStreaming ? "animate-spin text-[#07c160]" : "opacity-80"} />
                    <span>{isRewritingStreaming ? t('rewritingAI') : t('oneClickAIRewrite')}</span>
                  </button>
                  
                  {/* Article settings quick scroll */}
                  <button 
                    onClick={() => {
                      document.getElementById('article-settings-anchor')?.scrollIntoView({ behavior: 'smooth' });
                    }}
                    className="h-10 text-center flex items-center justify-center text-[12.5px] text-[var(--color-kb-text-heading)] hover:text-[var(--color-kb-accent)] hover:bg-[var(--color-kb-accent)]/5 rounded-xl border border-[var(--color-kb-panel-border)] hover:border-[var(--color-kb-accent)]/30 active:scale-95 transition-all w-full font-bold gap-1.5 cursor-pointer"
                  >
                    <Settings2 size={14} className="opacity-80" />
                    <span>{t('articleSettings')}</span>
                  </button>
                </div>

                {/* Diagnostics and Info summary card */}
                <div className="bg-[var(--color-kb-panel)] border border-[var(--color-kb-panel-border)] rounded-2xl p-4.5 space-y-3 shadow-xs">
                  <div className="text-[11px] font-bold text-[var(--color-kb-text-heading)] uppercase tracking-wider border-b border-[var(--color-kb-panel-border)] pb-2 mb-1">
                    {t('sidebarTitle')}
                  </div>
                  <div className="text-[11.5px] text-[var(--color-kb-text-muted)] space-y-2">
                    <div className="flex justify-between">
                      <span>{t('statWordCount')}</span>
                      <strong className="text-[var(--color-kb-text)] font-mono">{t('statUnitWords', { count: getWechatWordCount(selectedArticle?.content) })}</strong>
                    </div>
                    <div className="flex justify-between">
                      <span>{t('statParagraphs')}</span>
                      <strong className="text-[var(--color-kb-text)] font-mono">{t('statUnitParagraphs', { count: (selectedArticle.content?.match(/<p>/g) || []).length })}</strong>
                    </div>
                    <div className="flex justify-between">
                      <span>{t('statImages')}</span>
                      <strong className="text-[var(--color-kb-text)] font-mono">{t('statUnitImages', { count: extractedBodyImages.length })}</strong>
                    </div>
                    <div className="flex justify-between">
                      <span>{t('statSafety')}</span>
                      <strong className="text-emerald-500 font-bold">100%</strong>
                    </div>
                  </div>
                </div>

                {isFormattingInProgress && (
                  <div className="bg-[var(--color-kb-panel)] border border-amber-500/20 rounded-xl p-3 text-center animate-pulse shadow-sm">
                    <p className="text-[10px] font-bold text-amber-500 animate-bounce">{t('formattingModel')}</p>
                    <p className="text-[9.5px] text-[var(--color-kb-text-muted)] mt-1">{formattingStep}</p>
                  </div>
                )}
              </div>

            </div>

          </div>
        )}

        {/* Right: Floating Actions (Optional) */}
        {!selectedArticle && !isAIOpen && <div className="flex-1 bg-[var(--color-kb-editor)]"></div>}

        {isAIOpen && (
          <div className="z-10 flex">
            <AiAssistantPanel 
              aiWidth={aiWidth} 
              isDraggingAi={isDraggingAi} 
              onMouseDownDrag={() => setIsDraggingAi(true)} 
              onClose={() => setIsAIOpen(false)} 
              docContent={selectedArticle?.content}
              docs={documents}
              activeDoc={documents.find((d: any) => d.id === selectedArticle?.id)}
              selectedArticle={selectedArticle}
              onUpdateArticle={updateSelectedArticle}
              onTriggerStreamRewrite={handleStreamingRewrite}
              onTriggerStreamContinue={handleStreamingContinue}
              onTriggerCreateNewArticle={handleCreateNewArticleAndWrite}
              onInsertHtml={(html) => {
                if (activeEditor && !activeEditor.isDestroyed && typeof activeEditor.chain === 'function') {
                  try {
                    activeEditor.chain().focus().insertContent(html).run();
                  } catch (e) {
                    console.error('Failed to insert HTML to editor:', e);
                  }
                }
              }}
            />
          </div>
        )}
      </div>

      {/* Bottom Footer Fixed inside modal */}
      <div className="h-[56px] bg-[var(--color-kb-panel)] border-t border-[var(--color-kb-panel-border)] flex items-center justify-between px-6 shadow-[0_-4px_24px_rgba(0,0,0,0.02)] z-20 flex-shrink-0 backdrop-blur-md bg-opacity-90">
        <div className="text-[var(--color-kb-text-muted)] text-[13px]">{t('body')}{t('wordCount', { count: getWechatWordCount(selectedArticle?.content) })}</div>
        <div className="flex items-center gap-2.5">
            <button 
              onClick={() => toast.success(t('saveSuccess', { defaultValue: '保存成功' }))} 
              className="h-9 px-4 flex items-center justify-center text-[13px] font-semibold text-[var(--color-kb-text-heading)] bg-[var(--color-kb-editor)] border border-[var(--color-kb-panel-border)] hover:bg-[var(--color-kb-panel-hover)] rounded-lg shadow-sm transition-all focus:outline-none"
            >
              {t('saveAsDraft')}
            </button>
            <button 
              onClick={() => setIsSendPreviewOpen(true)} 
              className="h-9 px-4 flex items-center justify-center text-[13px] font-semibold text-[var(--color-kb-text-heading)] bg-[var(--color-kb-editor)] border border-[var(--color-kb-panel-border)] hover:bg-[var(--color-kb-panel-hover)] rounded-lg shadow-sm transition-all focus:outline-none"
            >
              {t('sendPreview')}
            </button>
            <button 
              onClick={() => setIsPreviewOpen(true)} 
              className="h-9 px-4 flex items-center justify-center text-[13px] font-semibold text-[var(--color-kb-text-heading)] bg-[var(--color-kb-editor)] border border-[var(--color-kb-panel-border)] hover:bg-[var(--color-kb-panel-hover)] rounded-lg shadow-sm transition-all focus:outline-none"
            >
              {t('mobilePreview')}
            </button>
            <button 
              disabled={isPublishing} 
              onClick={handlePublish} 
              className="h-9 px-5 flex items-center justify-center text-[13px] font-bold text-white bg-[var(--color-kb-accent)] hover:bg-[var(--color-kb-accent-hover)] border border-transparent rounded-lg shadow-sm shadow-[var(--color-kb-accent)]/10 transition-all disabled:opacity-50 active:scale-[0.98] focus:outline-none tracking-wide"
            >
              {isPublishing ? t('publishingToOA') : t('publishToOA')}
            </button>
        </div>
      </div>

      {/* Preview Modal */}
      <WechatPreviewModal 
        isOpen={isPreviewOpen}
        onClose={() => setIsPreviewOpen(false)}
        selectedArticle={selectedArticle}
      />

      {/* Send Preview Modal */}
      <WechatSendPreviewModal
        isOpen={isSendPreviewOpen}
        onClose={() => setIsSendPreviewOpen(false)}
        previewWechatId={previewWechatId}
        setPreviewWechatId={setPreviewWechatId}
        isSending={isSendingPreview}
        onConfirmSend={handleConfirmSendPreview}
      />

      {/* Wechat Publish Modal */}
      <WechatPublishModal
        isOpen={isPublishModalOpen}
        onClose={() => setIsPublishModalOpen(false)}
        isPublishing={isPublishing}
        onConfirmPublish={handleConfirmPublishWithOptions}
        officialAccountName={selectedOfficialAccounts[0]?.name}
        officialAccountType={selectedOfficialAccounts[0]?.type}
      />

      {/* Import Modals */}
      <NotesAppModal 
        isOpen={isNotesModalOpen}
        onClose={() => setIsNotesModalOpen(false)}
        onConfirm={handleImportNotes}
      />

      <CloudDriveModal
        isOpen={isCloudDriveModalOpen}
        onClose={() => setIsCloudDriveModalOpen(false)}
        spaceId={cloudDriveSpaceId}
        onConfirm={handleImportCloudDrive}
      />

      {/* Smart Image Crop Modal Overlay */}
      <WechatImageCropperModal
        isOpen={isCropperOpen}
        imageSrc={tempCoverToCrop}
        onClose={() => {
          setIsCropperOpen(false);
          setTempCoverToCrop('');
          setCropperSource(null);
        }}
        onBack={() => {
          if (cropperSource === 'body') setIsCoverFromBodyOpen(true);
          if (cropperSource === 'gallery') {
            setAssetLibraryTab('image');
            setAssetLibraryOpen(true);
          }
          setIsCropperOpen(false);
        }}
        onConfirm={(cropData) => {
          updateSelectedArticle({
            cover: cropData.cover,
            coverZoom: cropData.coverZoom,
            coverOffsetX: cropData.coverOffsetX,
            coverOffsetY: cropData.coverOffsetY,
            coverAspect: cropData.coverAspect
          });
        }}
      />

      {/* 1. Select cover from body text modal */}
      <ExtractCoverFromBodyModal
        isOpen={isCoverFromBodyOpen}
        onClose={() => {
          setIsCoverFromBodyOpen(false);
          setSelectedBodyImage('');
        }}
        extractedBodyImages={extractedBodyImages}
        selectedBodyImage={selectedBodyImage}
        setSelectedBodyImage={setSelectedBodyImage}
        onConfirm={() => {
          if (selectedBodyImage) {
            if (imageActionTarget === 'editor') {
              insertHtmlToEditor(`<p><img src="${selectedBodyImage}" alt="Inserted Image" style="max-width: 100%; height: auto;" /></p>`);
              setIsCoverFromBodyOpen(false);
              setSelectedBodyImage('');
            } else {
              setTempCoverToCrop(selectedBodyImage);
              setCropperSource('body');
              setIsCropperOpen(true);
              setIsCoverFromBodyOpen(false);
              setSelectedBodyImage('');
            }
          }
        }}
      />

      {/* 2. Unified Asset Library modal */}
      <AssetLibraryModal
        isOpen={assetLibraryOpen}
        onClose={() => setAssetLibraryOpen(false)}
        initialTab={assetLibraryTab}
        title={imageActionTarget === 'editor' ? t('select_editor_asset_title', { defaultValue: '选择要插入文章的素材' }) : t('asset_library_cover_title', { defaultValue: '图片库精选封面' })}
        onSelect={(item) => {
          if (imageActionTarget === 'editor') {
            if (item.type === 'image') {
              insertHtmlToEditor(`<p><img src="${item.url}" alt="${item.title}" style="border-radius: 12px; max-width: 100%; margin: 16px 0; border: 1px solid var(--color-kb-panel-border);" /></p>`);
            } else if (item.type === 'video') {
              insertHtmlToEditor(`<video src="${item.url || 'https://sample-videos.com/video321/mp4/720/big_buck_bunny_720p_1mb.mp4'}" controls></video>`);
            } else if (item.type === 'audio') {
              insertHtmlToEditor(`<audio src="${item.url || 'https://www.soundhelix.com/examples/mp3/SoundHelix-Song-1.mp3'}" controls></audio>`);
            }
            setAssetLibraryOpen(false);
          } else {
            // Must be selecting cover image
            if (item.type === 'image') {
              setTempCoverToCrop(item.url);
              setIsCropperOpen(true);
              setAssetLibraryOpen(false);
            }
          }
        }}
      />

      {/* 3. WeChat scanning code upload modal */}
      <WechatScanModal 
        isOpen={isWechatScanOpen}
        scanStatus={scanStatus}
        scannedCover={scannedCover}
        onClose={() => setIsWechatScanOpen(false)}
        triggerScanSimulation={triggerScanSimulation}
      />

      {/* 4. AI Match Cover modal */}
      <WechatAiImageModal 
        isOpen={isAiCoverOpen}
        onClose={() => setIsAiCoverOpen(false)}
        onConfirm={(url) => {
          if (imageActionTarget === 'cover') {
            setTempCoverToCrop(url);
            setIsCropperOpen(true);
          } else {
            insertHtmlToEditor(`
              <div style="margin: 20px 0; text-align: center;">
                <img src="${url}" style="max-width: 100%; border-radius: 8px;" />
              </div>
            `);
          }
          setIsAiCoverOpen(false);
        }}
      />

      {/* 5. Tool Bar Insertion configuration widgets popup */}
      <InsertToolConfigModal
        activeInsertType={activeInsertType}
        setActiveInsertType={setActiveInsertType}
        widgetTitle={widgetTitle}
        setWidgetTitle={setWidgetTitle}
        widgetSubtitle={widgetSubtitle}
        setWidgetSubtitle={setWidgetSubtitle}
        widgetUrl={widgetUrl}
        setWidgetUrl={setWidgetUrl}
        widgetQuota={widgetQuota}
        setWidgetQuota={setWidgetQuota}
        widgetMerchant={widgetMerchant}
        setWidgetMerchant={setWidgetMerchant}
        widgetCondition={widgetCondition}
        setWidgetCondition={setWidgetCondition}
        widgetQuestion={widgetQuestion}
        setWidgetQuestion={setWidgetQuestion}
        widgetAnswer={widgetAnswer}
        setWidgetAnswer={setWidgetAnswer}
        widgetOptions={widgetOptions}
        setWidgetOptions={setWidgetOptions}
        onConfirm={handleInsertConfirm}
      />

      <TypographyModal
        isOpen={isTypographyModalOpen}
        onClose={() => setIsTypographyModalOpen(false)}
        originalContent={selectedArticle?.content || ''}
        articleTitle={selectedArticle?.title || ''}
        onConfirm={(formattedHtml) => {
          if (selectedArticle) {
            updateSelectedArticle({ content: formattedHtml });
            if (activeEditor) {
              activeEditor.commands.setContent(formattedHtml);
            }
          }
          setIsTypographyModalOpen(false);
        }}
      />

      {/* Import Modals */}
      <NotesAppModal 
        isOpen={isNotesModalOpen}
        onClose={() => setIsNotesModalOpen(false)}
        onConfirm={handleImportNotes}
      />

      {/* Official Account Manager and Selector Modal */}
      <OfficialAccountModal
        isOpen={isOfficialAccountModalOpen}
        onClose={() => setIsOfficialAccountModalOpen(false)}
        initialOfficialAccounts={officialAccounts}
        initialSelectedAccountIds={selectedOfficialAccountIds}
        initialOaGroups={oaGroups}
        onConfirm={async (data) => {
          setOfficialAccounts(data.officialAccounts);
          setSelectedOfficialAccountIds(data.selectedOfficialAccountIds);
          setOaGroups(data.oaGroups);
          await WechatService.saveOfficialAccounts(data.officialAccounts);
          setIsOfficialAccountModalOpen(false);
        }}
      />

      <input 
        type="file" 
        accept="image/*" 
        ref={insertImageInputRef} 
        onChange={handleInsertImageUpload} 
        className="hidden" 
      />

      {isWechatAppletModalOpen && (
        <WechatAppletModal 
          onClose={() => setIsWechatAppletModalOpen(false)}
          onConfirm={(data) => {
             if (activeEditor) {
                activeEditor.commands.insertContent({
                  type: 'wechatMiniprogram',
                  attrs: {
                    'data-miniprogram-type': data.displayType,
                    'data-miniprogram-title': data.displayType === 'card' ? data.cardTitle : data.textContent,
                    'data-miniprogram-imageurl': data.imageUrl,
                    'data-miniprogram-path': data.link,
                    'data-miniprogram-nickname': '小程序'
                  }
                });
                if (data.displayType === 'card' || data.displayType === 'image') {
                  activeEditor.commands.insertContent('<p></p>');
                }
             } else {
                toast.info(t('positionCursorFirst'), 2000);
             }
             setIsWechatAppletModalOpen(false);
          }}
        />
      )}
    </div>
  );
}


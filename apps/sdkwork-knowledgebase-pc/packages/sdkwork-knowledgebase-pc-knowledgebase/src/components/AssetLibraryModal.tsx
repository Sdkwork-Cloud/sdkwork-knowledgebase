import React, { useState, useEffect } from 'react';
import { X, Search, Image as ImageIcon, Music, Video, Plus, Check } from 'lucide-react';
import { useTranslation } from 'react-i18next';

export type AssetType = 'image' | 'audio' | 'video';

export interface AssetLibraryModalProps {
  isOpen: boolean;
  onClose: () => void;
  onSelect: (item: { url: string; title: string; type: AssetType; duration?: string }) => void;
  initialTab?: AssetType;
  title?: string;
}

export function AssetLibraryModal({
  isOpen,
  onClose,
  onSelect,
  initialTab = 'image',
  title
}: AssetLibraryModalProps) {
  const { t } = useTranslation('common');
  const displayTitle = title || t('assetLibrary');

  const [activeTab, setActiveTab] = useState<AssetType>(initialTab);
  const [searchQuery, setSearchQuery] = useState('');

  // Reset tab when reopened with different initialTab
  useEffect(() => {
    if (isOpen) {
      setActiveTab(initialTab);
      setSearchQuery('');
    }
  }, [isOpen, initialTab]);

  if (!isOpen) return null;

  const mockImages = [
    { title: '极光闪耀', url: 'https://images.unsplash.com/photo-1542281286-9e0a16bb7366?auto=format&fit=crop&w=900&h=383&q=80' },
    { title: '秋日私语', url: 'https://images.unsplash.com/photo-1506744626753-1fa28f621b56?auto=format&fit=crop&w=900&h=383&q=80' },
    { title: '冰川冷调', url: 'https://images.unsplash.com/photo-1464822759023-fed622ff2c3b?auto=format&fit=crop&w=900&h=383&q=80' },
    { title: '城市灯火', url: 'https://images.unsplash.com/photo-1449844908441-8829872d2607?auto=format&fit=crop&w=900&h=383&q=80' },
    { title: '晨木森林', url: 'https://images.unsplash.com/photo-1448375240586-882707db888b?auto=format&fit=crop&w=900&h=383&q=80' },
    { title: '静寂湖面', url: 'https://images.unsplash.com/photo-1470071459604-3b5ec3a7fe05?auto=format&fit=crop&w=900&h=383&q=80' },
  ];

  const mockAudios = [
    { title: '深度思考 · 通用人工智能时代的生存指南', duration: '12:45', size: '14MB', author: 'AI小助手', url: 'https://www.soundhelix.com/examples/mp3/SoundHelix-Song-1.mp3' },
    { title: '清晨冥想曲 · 放松心情', duration: '05:30', size: '6MB', author: '音乐库', url: 'https://www.soundhelix.com/examples/mp3/SoundHelix-Song-2.mp3' },
    { title: '新产品发布会预热解说', duration: '02:15', size: '2.5MB', author: '营销组', url: 'https://www.soundhelix.com/examples/mp3/SoundHelix-Song-3.mp3' },
    { title: '自然白噪音 - 淅沥春雨', duration: '30:00', size: '35MB', author: '内置素材', url: 'https://www.soundhelix.com/examples/mp3/SoundHelix-Song-4.mp3' },
  ];

  const mockVideos = [
    { title: '2026 AI全栈开发者成长体系白皮书', duration: '45:00', size: '450MB', cover: 'https://images.unsplash.com/photo-1517245386807-bb43f82c33c4?auto=format&fit=crop&w=300&h=200&q=80', url: 'https://sample-videos.com/video321/mp4/720/big_buck_bunny_720p_1mb.mp4' },
    { title: '企业知识库平台 - 功能演示', duration: '03:20', size: '32MB', cover: 'https://images.unsplash.com/photo-1460925895917-afdab827c52f?auto=format&fit=crop&w=300&h=200&q=80', url: 'https://sample-videos.com/video321/mp4/720/big_buck_bunny_720p_1mb.mp4' },
    { title: '团队建设 - 青岛之旅回顾', duration: '10:15', size: '150MB', cover: 'https://images.unsplash.com/photo-1506869640319-a1a5606089ce?auto=format&fit=crop&w=300&h=200&q=80', url: 'https://sample-videos.com/video321/mp4/720/big_buck_bunny_720p_1mb.mp4' },
  ];

  return (
    <div className="fixed inset-0 bg-zinc-950/40 backdrop-blur-md z-[300] flex items-center justify-center p-4 animate-in fade-in duration-200">
      <div className="bg-[var(--color-kb-panel)] border border-[var(--color-kb-panel-border)] rounded-2xl max-w-4xl w-full flex flex-col h-[85vh] shadow-2xl overflow-hidden animate-in zoom-in-95 duration-200">
        {/* Header */}
        <div className="px-6 py-4 border-b border-[var(--color-kb-panel-border)] flex items-center justify-between shrink-0 bg-[var(--color-kb-editor)]">
          <div className="flex items-center gap-6">
            <span className="text-lg font-bold text-[var(--color-kb-text-heading)]">{displayTitle}</span>
            <div className="flex bg-[var(--color-kb-panel-hover)] p-1 rounded-lg">
              <button 
                onClick={() => setActiveTab('image')}
                className={`px-4 py-1.5 rounded-md text-sm font-medium flex items-center gap-2 transition-colors ${activeTab === 'image' ? 'bg-[var(--color-kb-editor)] text-[var(--color-kb-accent)] shadow-sm' : 'text-[var(--color-kb-text-muted)] hover:text-[var(--color-kb-text)]'}`}
              >
                <ImageIcon size={16} />{t('imageStr')}
              </button>
              <button 
                onClick={() => setActiveTab('audio')}
                className={`px-4 py-1.5 rounded-md text-sm font-medium flex items-center gap-2 transition-colors ${activeTab === 'audio' ? 'bg-[var(--color-kb-editor)] text-[var(--color-kb-accent)] shadow-sm' : 'text-[var(--color-kb-text-muted)] hover:text-[var(--color-kb-text)]'}`}
              >
                <Music size={16} />{t('audioStr')}
              </button>
              <button 
                onClick={() => setActiveTab('video')}
                className={`px-4 py-1.5 rounded-md text-sm font-medium flex items-center gap-2 transition-colors ${activeTab === 'video' ? 'bg-[var(--color-kb-editor)] text-[var(--color-kb-accent)] shadow-sm' : 'text-[var(--color-kb-text-muted)] hover:text-[var(--color-kb-text)]'}`}
              >
                <Video size={16} />{t('videoStr')}
              </button>
            </div>
          </div>
          <button onClick={onClose} className="p-2 text-[var(--color-kb-text-muted)] hover:text-[var(--color-kb-text-heading)] hover:bg-[var(--color-kb-panel-hover)] rounded-full transition-colors">
            <X size={20} />
          </button>
        </div>

        {/* Action Bar */}
        <div className="px-6 py-3 border-b border-[var(--color-kb-panel-border)] flex items-center justify-between shrink-0">
          <div className="relative w-64">
            <Search size={14} className="absolute left-3 top-1/2 -translate-y-1/2 text-[var(--color-kb-text-muted)]" />
            <input 
              type="text" 
              placeholder={t('searchAsset')}
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="w-full pl-9 pr-3 py-1.5 text-sm bg-[var(--color-kb-editor)] border border-[var(--color-kb-panel-border)] rounded-full text-[var(--color-kb-text)] focus:outline-none focus:border-[var(--color-kb-accent)] transition-colors"
            />
          </div>
          <button className="px-4 py-1.5 text-sm font-semibold text-white bg-[var(--color-kb-accent)] hover:bg-[var(--color-kb-accent-hover)] rounded-lg flex items-center shadow-sm transition-colors">
            <Plus size={16} className="mr-1.5" /> {t('localUpload')}
          </button>
        </div>

        {/* Content Area */}
        <div className="flex-1 overflow-y-auto p-6 bg-[var(--color-kb-panel)]">
          {activeTab === 'image' && (
            <div className="grid grid-cols-2 xl:grid-cols-3 gap-6">
              {mockImages.filter(img => img.title.includes(searchQuery)).map((item, i) => (
                <div 
                  key={i} 
                  className="group relative aspect-[900/383] rounded-xl overflow-hidden cursor-pointer border border-transparent hover:border-[var(--color-kb-panel-border)] shadow-sm hover:shadow-md transition-all bg-[var(--color-kb-editor)]"
                  onClick={() => onSelect({ ...item, type: 'image' })}
                >
                  <img src={item.url} referrerPolicy="no-referrer" alt={item.title} className="w-full h-full object-cover group-hover:scale-105 transition-transform duration-300" />
                  <div className="absolute inset-x-0 bottom-0 bg-gradient-to-t from-black/80 to-transparent p-3 pt-6 flex items-end justify-between opacity-0 group-hover:opacity-100 transition-opacity">
                    <span className="text-white text-sm font-medium truncate pr-2">{item.title}</span>
                    <span className="w-6 h-6 rounded-full bg-[var(--color-kb-accent)] text-white flex items-center justify-center shrink-0 shadow-lg">
                      <Plus size={14} />
                    </span>
                  </div>
                </div>
              ))}
            </div>
          )}

          {activeTab === 'audio' && (
            <div className="space-y-3 max-w-3xl mx-auto">
              {mockAudios.filter(audio => audio.title.includes(searchQuery)).map((item, i) => (
                <div 
                  key={i} 
                  className="group flex items-center justify-between p-4 rounded-xl bg-[var(--color-kb-editor)] border border-[var(--color-kb-panel-border)] hover:border-[var(--color-kb-accent)]/50 hover:shadow-sm cursor-pointer transition-all"
                  onClick={() => onSelect({ ...item, type: 'audio' })}
                >
                  <div className="flex items-center gap-4 min-w-0">
                    <div className="w-10 h-10 rounded-full bg-[var(--color-kb-accent)]/10 text-[var(--color-kb-accent)] flex items-center justify-center shrink-0 relative overflow-hidden group-hover:bg-[var(--color-kb-accent)] group-hover:text-white transition-colors">
                      <Music size={18} className="relative z-10" />
                    </div>
                    <div className="min-w-0 flex-1">
                      <h4 className="text-sm font-semibold text-[var(--color-kb-text-heading)] truncate mb-0.5">{item.title}</h4>
                      <div className="flex items-center text-xs text-[var(--color-kb-text-muted)] gap-3">
                        <span>{item.author}</span>
                        <span className="w-1 h-1 rounded-full bg-gray-400/50"></span>
                        <span>{item.size}</span>
                      </div>
                    </div>
                  </div>
                  <div className="flex items-center gap-4 shrink-0 mt-0">
                    <span className="text-xs font-mono text-[var(--color-kb-text-muted)] bg-[var(--color-kb-panel)] px-2 py-1 rounded">{item.duration}</span>
                    <button className="w-8 h-8 rounded-full border border-[var(--color-kb-panel-border)] text-[var(--color-kb-text-muted)] flex items-center justify-center group-hover:bg-[var(--color-kb-accent)] group-hover:border-[var(--color-kb-accent)] group-hover:text-white transition-all shadow-sm">
                      <Plus size={14} />
                    </button>
                  </div>
                </div>
              ))}
            </div>
          )}

          {activeTab === 'video' && (
            <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-6">
              {mockVideos.filter(video => video.title.includes(searchQuery)).map((item, i) => (
                <div 
                  key={i} 
                  className="group flex flex-col rounded-xl overflow-hidden bg-[var(--color-kb-editor)] border border-[var(--color-kb-panel-border)] hover:border-[var(--color-kb-accent)]/50 hover:shadow-md cursor-pointer transition-all"
                  onClick={() => onSelect({ ...item, type: 'video' })}
                >
                  <div className="aspect-video relative overflow-hidden bg-black/5">
                    <img src={item.cover} alt={item.title} className="w-full h-full object-cover group-hover:scale-105 transition-transform duration-300 opacity-90" />
                    <div className="absolute inset-0 bg-black/20 group-hover:bg-black/10 transition-colors flex items-center justify-center">
                      <div className="w-10 h-10 rounded-full bg-white/20 backdrop-blur-sm border border-white/40 flex items-center justify-center shadow-lg group-hover:scale-110 transition-transform">
                        <Video size={18} className="text-white ml-0.5" />
                      </div>
                    </div>
                    <div className="absolute bottom-2 right-2 px-1.5 py-0.5 bg-black/60 backdrop-blur-md rounded text-[10px] text-white font-mono shadow">
                      {item.duration}
                    </div>
                  </div>
                  <div className="p-3">
                    <h4 className="text-sm font-medium text-[var(--color-kb-text-heading)] line-clamp-2 leading-snug mb-2 group-hover:text-[var(--color-kb-accent)] transition-colors">{item.title}</h4>
                    <div className="flex items-center justify-between text-xs text-[var(--color-kb-text-muted)]">
                      <span>{item.size}</span>
                      <span className="flex items-center text-[var(--color-kb-accent)] font-medium opacity-0 group-hover:opacity-100 transition-opacity translate-y-1 group-hover:translate-y-0 duration-200">
                        {t('useAsset')} <Plus size={12} className="ml-0.5" />
                      </span>
                    </div>
                  </div>
                </div>
              ))}
            </div>
          )}
          
          {/* Empty state if no search results */}
          {((activeTab === 'image' && !mockImages.filter(img => img.title.includes(searchQuery)).length) ||
            (activeTab === 'audio' && !mockAudios.filter(audio => audio.title.includes(searchQuery)).length) ||
            (activeTab === 'video' && !mockVideos.filter(video => video.title.includes(searchQuery)).length)) && (
            <div className="flex flex-col items-center justify-center py-20 text-[var(--color-kb-text-muted)]">
              <Search size={48} className="mb-4 opacity-20" />
              <p>{t('noSearchResult', { query: searchQuery })}</p>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

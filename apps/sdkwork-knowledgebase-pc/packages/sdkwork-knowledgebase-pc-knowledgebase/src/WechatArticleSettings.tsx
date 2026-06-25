import React from 'react';
import { Plus, Settings2, Trash2 } from 'lucide-react';
import { WechatArticle } from './services/wechat';
import type { ReactKeyedComponentProps } from '@sdkwork/sdkwork-knowledgebase-pc-commons/reactKeyedProps';

export interface WechatArticleSettingsProps extends ReactKeyedComponentProps {
  selectedArticle: WechatArticle;
  updateSelectedArticle: (updates: Partial<WechatArticle>) => void;
  coverInputRef: React.RefObject<HTMLInputElement>;
  handleCoverUpload: (e: React.ChangeEvent<HTMLInputElement>) => Promise<void>;
  onSelectCoverFromBody: () => void;
  onSelectCoverFromGallery: () => void;
  onWechatScanUpload: () => void;
  onAiCoverGenerate: () => void;
  onCropExistingCover?: (coverUrl: string) => void;
}

export function WechatArticleSettings({
  selectedArticle,
  updateSelectedArticle,
  coverInputRef,
  handleCoverUpload,
  onSelectCoverFromBody,
  onSelectCoverFromGallery,
  onWechatScanUpload,
  onAiCoverGenerate,
  onCropExistingCover
}: WechatArticleSettingsProps) {
  const [isCoverDropdownOpen, setIsCoverDropdownOpen] = React.useState(false);
  const dropdownRef = React.useRef<HTMLDivElement>(null);

  React.useEffect(() => {
    function handleClickOutside(event: MouseEvent) {
      if (dropdownRef.current && !dropdownRef.current.contains(event.target as Node)) {
        setIsCoverDropdownOpen(false);
      }
    }
    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, []);

  const isOneToOne = selectedArticle.coverAspect === '1:1';
  const scaleFactor = isOneToOne ? (85 / 280) : (200 / 470);

  return (
    <div id="article-settings-anchor" className="mt-12 pt-8 border-t border-[var(--color-kb-panel-border)]">
      <h3 className="text-lg text-[var(--color-kb-text-heading)] font-medium mb-4 flex items-center"><Settings2 size={16} className="mr-2 opacity-50" /> 文章设置</h3>
      
      <div className="bg-[var(--color-kb-panel)] border border-[var(--color-kb-panel-border)] rounded-lg overflow-hidden flex flex-col shadow-sm">
        
        {/* Title & Cover */}
        <div className="p-5 border-b border-[var(--color-kb-panel-border)]">
          <label className="block text-[var(--color-kb-text-heading)] text-sm font-medium mb-3">文章封面</label>
          <div className="flex items-start gap-4">
            <div 
              ref={dropdownRef} 
              className="relative z-[40]"
              onMouseEnter={() => setIsCoverDropdownOpen(true)}
              onMouseLeave={() => setIsCoverDropdownOpen(false)}
            >
              <div 
                onClick={() => {
                  if (selectedArticle.cover) {
                    onCropExistingCover?.(selectedArticle.cover);
                  } else {
                    coverInputRef.current?.click();
                  }
                }}
                className="bg-[var(--color-kb-panel-hover)] rounded-xl border border-dashed border-[var(--color-kb-panel-border)] flex items-center justify-center cursor-pointer hover:border-[var(--color-kb-accent)] hover:text-[var(--color-kb-accent)] transition-all relative group overflow-hidden shadow-sm"
                style={{
                  width: isOneToOne ? '85px' : '200px',
                  height: '85px'
                }}
              >
                {selectedArticle.cover ? (
                  <img 
                    src={selectedArticle.cover} 
                    alt="cover" 
                    className="w-full h-full object-contain animate-fade-in transition-transform duration-200" 
                    style={{
                      transform: `scale(${selectedArticle.coverZoom || 1}) translate(${(selectedArticle.coverOffsetX || 0) * scaleFactor}px, ${(selectedArticle.coverOffsetY || 0) * scaleFactor}px)`,
                    }}
                  />
                ) : (
                  <div className="text-[var(--color-kb-text-muted)] flex flex-col items-center">
                    <Plus size={20} />
                    <span className="text-[11px] mt-1 font-medium">选择封面</span>
                  </div>
                )}
                {/* Micro overlay indicators when hovering cover */}
                {selectedArticle.cover && (
                  <div className="absolute inset-0 bg-black/45 opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center gap-1.5 z-10 text-white text-[11px] font-bold">
                    <span>更改/裁剪封面</span>
                  </div>
                )}
              </div>

              {isCoverDropdownOpen && (
                <div className="absolute left-0 mt-1.5 w-[180px] bg-[var(--color-kb-panel)] border border-[var(--color-kb-panel-border)] rounded-xl shadow-xl py-1.5 z-[50] text-[13.5px] text-[var(--color-kb-text-heading)] font-medium divide-y divide-[var(--color-kb-panel-border)] animate-in fade-in slide-in-from-top-2 duration-150">
                  <div className="py-1">
                    <button 
                      onClick={() => { coverInputRef.current?.click(); setIsCoverDropdownOpen(false); }}
                      className="w-full flex items-center px-4 py-2 hover:bg-[var(--color-kb-panel-hover)] transition-colors text-left text-xs font-semibold"
                    >
                      上传本地图片
                    </button>
                    <button 
                      onClick={() => { onSelectCoverFromBody(); setIsCoverDropdownOpen(false); }}
                      className="w-full flex items-center px-4 py-2 hover:bg-[var(--color-kb-panel-hover)] transition-colors text-left text-xs font-semibold"
                    >
                      从正文选择
                    </button>
                    <button 
                      onClick={() => { onSelectCoverFromGallery(); setIsCoverDropdownOpen(false); }}
                      className="w-full flex items-center px-4 py-2 hover:bg-[var(--color-kb-panel-hover)] transition-colors text-left text-xs font-semibold"
                    >
                      图片库精选
                    </button>
                  </div>
                  <div className="py-1">
                    <button 
                      onClick={() => { onWechatScanUpload(); setIsCoverDropdownOpen(false); }}
                      className="w-full flex items-center px-4 py-2 hover:bg-[var(--color-kb-panel-hover)] transition-colors text-left text-xs font-semibold"
                    >
                      微信扫码上传
                    </button>
                  </div>
                  <div className="py-1">
                    <button 
                      onClick={() => { onAiCoverGenerate(); setIsCoverDropdownOpen(false); }}
                      className="w-full flex items-center px-4 py-2 hover:bg-[var(--color-kb-panel-hover)] text-[var(--color-kb-accent)] transition-colors text-left text-xs font-bold"
                    >
                      AI 一键智能配图
                    </button>
                  </div>
                </div>
              )}
            </div>
            {/* We assume input file is rendered inside parent, or we render it here */}
            <input type="file" ref={coverInputRef} className="hidden" accept="image/*" onChange={handleCoverUpload}/>
            <div className="flex flex-col justify-center gap-1 pt-1">
              <p className="text-[13px] text-[var(--color-kb-text-heading)] font-semibold">推荐从正文提取或上传</p>
              <p className="text-[12px] text-[var(--color-kb-text-muted)]">
                当前比例: <strong className="text-[var(--color-kb-accent)] font-mono">{isOneToOne ? '微信次条 (1:1)' : '头条大图 (2.35:1)'}</strong>
              </p>
              <p className="text-[11px] text-[var(--color-kb-text-muted)]">点击封面区域可随时重新调整裁剪尺寸与平移位置</p>
              
              {selectedArticle.cover && (
                <div className="flex gap-2.5 mt-2 animate-fade-in">
                  <button
                    type="button"
                    onClick={() => {
                      if (selectedArticle.cover) {
                        onCropExistingCover?.(selectedArticle.cover);
                      }
                    }}
                    className="px-2 py-0.5 text-[10px] font-bold text-[#07c160] bg-[#07c160]/5 border border-[#07c160]/20 rounded-md hover:bg-[#07c160]/10 transition-all flex items-center gap-1"
                  >
                    <Settings2 size={10} />
                    调整裁剪
                  </button>
                  <button
                    type="button"
                    onClick={() => updateSelectedArticle({ cover: '', coverZoom: 1, coverOffsetX: 0, coverOffsetY: 0 })}
                    className="px-2 py-0.5 text-[10px] font-bold text-red-500 bg-red-500/5 border border-red-500/10 rounded-md hover:bg-red-500/10 transition-all flex items-center gap-1"
                  >
                    <Trash2 size={10} />
                    移除封面
                  </button>
                </div>
              )}
            </div>
          </div>
        </div>
        
        {/* Abstract */}
        <div className="p-5 border-b border-[var(--color-kb-panel-border)]">
          <label className="block text-[var(--color-kb-text-heading)] text-sm font-medium mb-3 flex items-center justify-between">
            <span>摘要</span>
            <span className="text-xs font-normal text-[var(--color-kb-text-muted)]">{selectedArticle.abstract?.length || 0}/120</span>
          </label>
          <textarea 
            value={selectedArticle.abstract}
            onChange={e => updateSelectedArticle({ abstract: e.target.value })}
            className="w-full h-20 bg-[var(--color-kb-editor)] border border-[var(--color-kb-panel-border)] rounded-xl px-4 py-3 text-[14px] text-[var(--color-kb-text)] focus:outline-none focus:border-[var(--color-kb-accent)] focus:ring-1 focus:ring-[var(--color-kb-accent)] resize-none shadow-sm transition-all"
            placeholder="选填，如果不填写会默认抓取正文前54个字"
            maxLength={120}
          />
        </div>

        {/* Declaration */}
        <div className="p-5 flex items-center justify-between border-b border-[var(--color-kb-panel-border)]">
           <div>
             <div className="text-[var(--color-kb-text-heading)] text-sm font-medium">声明原创</div>
             <div className="text-xs text-[var(--color-kb-text-muted)] mt-1">声明原创后，文章将会有“原创”标识</div>
           </div>
           <label className="relative inline-flex items-center cursor-pointer">
              <input type="checkbox" className="sr-only peer" checked={selectedArticle.isOriginal} onChange={e => updateSelectedArticle({ isOriginal: e.target.checked })} />
              <div className="w-10 h-[22px] bg-[var(--color-kb-panel-border)] peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-[18px] after:w-[18px] after:transition-all peer-checked:bg-[var(--color-kb-accent)]"></div>
           </label>
        </div>

        {/* Comments */}
        <div className="p-5 flex items-center justify-between">
            <label className="block text-[var(--color-kb-text-heading)] text-sm font-medium m-0">留言设置</label>
            <div className="flex gap-4">
              <label className="flex items-center gap-2 cursor-pointer group">
                <div className={`w-3.5 h-3.5 rounded-full border flex items-center justify-center transition-colors ${selectedArticle.commentType === 'everyone' ? 'border-[var(--color-kb-accent)] bg-[var(--color-kb-accent)]' : 'border-[var(--color-kb-text-muted)] group-hover:border-[var(--color-kb-accent)]'}`}>
                  {selectedArticle.commentType === 'everyone' && <div className="w-1 h-1 bg-white rounded-full"></div>}
                </div>
                <input type="radio" className="hidden" checked={selectedArticle.commentType === 'everyone'} onChange={() => updateSelectedArticle({ commentType: 'everyone' })} />
                <span className="text-[13px] text-[var(--color-kb-text)]">所有人可留言</span>
              </label>
              <label className="flex items-center gap-2 cursor-pointer group">
                <div className={`w-3.5 h-3.5 rounded-full border flex items-center justify-center transition-colors ${selectedArticle.commentType === 'follower' ? 'border-[var(--color-kb-accent)] bg-[var(--color-kb-accent)]' : 'border-[var(--color-kb-text-muted)] group-hover:border-[var(--color-kb-accent)]'}`}>
                  {selectedArticle.commentType === 'follower' && <div className="w-1 h-1 bg-white rounded-full"></div>}
                </div>
                <input type="radio" className="hidden" checked={selectedArticle.commentType === 'follower'} onChange={() => updateSelectedArticle({ commentType: 'follower' })} />
                <span className="text-[13px] text-[var(--color-kb-text)]">仅关注后可留言</span>
              </label>
              <label className="flex items-center gap-2 cursor-pointer group">
                <div className={`w-3.5 h-3.5 rounded-full border flex items-center justify-center transition-colors ${selectedArticle.commentType === 'none' ? 'border-[var(--color-kb-accent)] bg-[var(--color-kb-accent)]' : 'border-[var(--color-kb-text-muted)] group-hover:border-[var(--color-kb-accent)]'}`}>
                  {selectedArticle.commentType === 'none' && <div className="w-1 h-1 bg-white rounded-full"></div>}
                </div>
                <input type="radio" className="hidden" checked={selectedArticle.commentType === 'none'} onChange={() => updateSelectedArticle({ commentType: 'none' })} />
                <span className="text-[13px] text-[var(--color-kb-text)]">不允许留言</span>
              </label>
            </div>
        </div>
      </div>
    </div>
  );
}

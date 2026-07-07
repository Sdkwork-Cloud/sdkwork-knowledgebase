import React, { useState, useEffect, useCallback, useRef } from 'react';
import { useTranslation } from 'react-i18next';
import { createPortal } from 'react-dom';
import { 
  X, Globe, Copy, Check, Upload, ExternalLink, Settings, 
  Trash2, Image as ImageIcon, Sparkles, RefreshCw, AlertCircle, Laptop
} from 'lucide-react';
import { toast } from './components/ui/toast-manager';
import { toastKnowledgebaseError } from './components/ui/toastKnowledgebaseError';
import {
  isKnowledgebaseApiAvailable,
  KnowledgebaseErrorCodes,
  throwKnowledgebaseError,
} from 'sdkwork-knowledgebase-pc-core';
import { KnowledgeBase, DocumentService } from './services/document';

interface DeployWebsiteModalProps {
  isOpen: boolean;
  activeKb: KnowledgeBase | null;
  onClose: () => void;
  onSave: (updates: Partial<KnowledgeBase>) => void;
}

export function DeployWebsiteModal({ isOpen, activeKb, onClose, onSave }: DeployWebsiteModalProps) {
  const { t } = useTranslation(['kb', 'common']);
  const [isDeploying, setIsDeploying] = useState(false);
  const [deployStep, setDeployStep] = useState('');
  const [copied, setCopied] = useState(false);
  
  // Settings Form State
  const [siteName, setSiteName] = useState('');
  const [customDomain, setCustomDomain] = useState('');
  const [logoBase64, setLogoBase64] = useState('');
  const [isDeployed, setIsDeployed] = useState(false);
  const [deployedSiteUrl, setDeployedSiteUrl] = useState('');

  // Drag and drop states
  const [dragActive, setDragActive] = useState(false);
  const fileInputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (activeKb) {
      setSiteName(activeKb.siteName || activeKb.title || '');
      setCustomDomain(activeKb.customDomain || '');
      setLogoBase64(activeKb.siteLogo || activeKb.avatar || '');
      setIsDeployed(!!activeKb.isDeployed);
      setDeployedSiteUrl(activeKb.deployedUrl || `https://${activeKb.id.toLowerCase().replace(/[^a-z0-9-]/g, '') || 'kb'}.sdkwork.com`);
    }
  }, [activeKb, isOpen]);

  if (!isOpen || !activeKb) return null;

  const handleCopy = () => {
    const urlToCopy = customDomain ? `https://${customDomain}` : deployedSiteUrl;
    navigator.clipboard.writeText(urlToCopy);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const processFile = (file: File) => {
    if (!file.type.startsWith('image/')) {
      toast.error('请选择有效的图片，例如 .png 或者 .jpg 文件。');
      return;
    }
    const reader = new FileReader();
    reader.onload = (e) => {
      if (e.target?.result) {
        setLogoBase64(e.target.result as string);
      }
    };
    reader.readAsDataURL(file);
  };

  const handleDrag = (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    if (e.type === "dragenter" || e.type === "dragover") {
      setDragActive(true);
    } else if (e.type === "dragleave") {
      setDragActive(false);
    }
  };

  const handleDrop = (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setDragActive(false);
    if (e.dataTransfer.files && e.dataTransfer.files[0]) {
      processFile(e.dataTransfer.files[0]);
    }
  };

  const triggerFileInput = () => {
    fileInputRef.current?.click();
  };

  const handleFileInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    if (e.target.files && e.target.files[0]) {
      processFile(e.target.files[0]);
    }
  };

  const handleStartDeploy = async () => {
    if (!isKnowledgebaseApiAvailable()) {
      try {
        throwKnowledgebaseError(KnowledgebaseErrorCodes.API_UNAVAILABLE);
      } catch (error) {
        toastKnowledgebaseError(error, t);
      }
      return;
    }
    setIsDeploying(true);
    setDeployStep('正在分析并序列化站点内容...');
    
    try {
      const selectedPlatform = 'vercel';
      const res = await DocumentService.publishWebsite(selectedPlatform, activeKb.id, {
        siteName: siteName || activeKb.title,
        customDomain: customDomain || undefined,
        siteLogo: logoBase64 || undefined,
      });

      if (!res.accepted) {
        setDeployStep('网站部署失败，请检查知识库内容后重试。');
        setIsDeploying(false);
        toast.error('网站部署失败，请检查知识库内容后重试。');
        return;
      }

      setDeployStep('正在激活 SSL 服务证书与配置边缘路由...');
      
      const newUrl = res.url || `https://${siteName.toLowerCase().replace(/\s+/g, '-')}.${selectedPlatform}.app`;
      setDeployedSiteUrl(newUrl);
      
      setIsDeploying(false);
      setIsDeployed(true);
      onSave({
        isDeployed: true,
        siteName: siteName || activeKb.title,
        customDomain: customDomain || undefined,
        siteLogo: logoBase64 || undefined,
        deployedUrl: newUrl
      });
    } catch (e) {
      const detail = e instanceof Error ? e.message : String(e);
      console.error(e);
      setDeployStep(detail);
      setIsDeploying(false);
      toastKnowledgebaseError(e, t);
    }
  };

  const handleSaveSettings = () => {
    onSave({
      siteName: siteName || activeKb.title,
      customDomain: customDomain || undefined,
      siteLogo: logoBase64 || undefined,
    });
    toast.success('网站配置修改成功！');
  };

  const handleStopDeploy = () => {
    if (confirm('确定要卸载脱离当前网站部署吗？脱离后该地址将无法直接访问。')) {
      setIsDeployed(false);
      onSave({
        isDeployed: false,
        customDomain: '',
        siteLogo: ''
      });
    }
  };

  return createPortal(
    <div className="fixed inset-0 z-[300] bg-zinc-950/40 flex items-center justify-center backdrop-blur-md p-4 select-none">
      <div className="w-[580px] bg-white dark:bg-[var(--color-kb-editor)] rounded-2xl shadow-2xl border border-zinc-200/80 dark:border-[var(--color-kb-panel-border)] flex flex-col overflow-hidden animate-in zoom-in-95 duration-200">
        
        {/* Modal Header */}
        <div className="h-16 border-b border-zinc-200/80 dark:border-[var(--color-kb-panel-border)] flex items-center justify-between px-6 bg-[#fafafa] dark:bg-[var(--color-kb-panel)]/30 shrink-0 shadow-sm z-10">
          <div className="flex items-center gap-3">
            <div className="w-9 h-9 rounded-xl bg-indigo-50 dark:bg-indigo-500/10 border border-indigo-100 dark:border-transparent min-w-0 flex items-center justify-center shadow-inner">
              <Globe size={18} className="text-indigo-600 dark:text-indigo-500" strokeWidth={2.5} />
            </div>
            <div>
              <h3 className="text-[15px] font-extrabold text-zinc-900 dark:text-[var(--color-kb-text-heading)] leading-tight tracking-tight">{t('deployToWebsite')}</h3>
              <p className="text-[11.5px] font-medium text-zinc-500 dark:text-[var(--color-kb-text-muted)] tracking-wide">{t('deployToWebsiteDesc')}</p>
            </div>
          </div>
          <button 
            onClick={onClose} 
            className="text-zinc-400 hover:text-red-500 hover:bg-red-50 dark:text-[var(--color-kb-text-muted)] dark:hover:bg-red-500/10 p-2 rounded-xl transition-all active:scale-95 ml-4 shrink-0"
          >
            <X size={16} strokeWidth={2.5} />
          </button>
        </div>

        {/* Modal Content container */}
        <div className="p-6 overflow-y-auto space-y-5 max-h-[500px]">
          {isDeploying ? (
            /* Deploying animation view */
            <div className="py-12 flex flex-col items-center justify-center text-center">
              <div className="relative mb-6">
                <div className="w-16 h-16 rounded-full border-4 border-indigo-500/15 border-t-indigo-500 animate-spin flex items-center justify-center">
                </div>
                <Globe size={24} className="text-indigo-500 absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 animate-pulse" />
              </div>
              <h4 className="text-[14px] font-bold text-[var(--color-kb-text-heading)] mb-1.5">正在打包并部署知识站点</h4>
              <p className="text-[11.5px] text-indigo-500 font-medium font-sans animate-bounce">{deployStep}</p>
              <p className="text-[10px] text-[var(--color-kb-text-muted)] mt-4 max-w-[320px] leading-relaxed">
                正在通过边缘网络加速节点 (Edge Cloud CDN) 进行同步。该部署包含所有的 Markdown 文档、图层及媒体文件。
              </p>
            </div>
          ) : (
            <>
              {/* Show deployed address card if already deployed */}
              {isDeployed ? (
                <div className="bg-indigo-500/[0.03] border border-indigo-500/20 rounded-xl p-4.5 space-y-3">
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-2">
                      <span className="flex h-2.5 w-2.5 relative">
                        <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75"></span>
                        <span className="relative inline-flex rounded-full h-2.5 w-2.5 bg-emerald-500"></span>
                      </span>
                      <strong className="text-[12.5px] font-bold text-indigo-700 dark:text-indigo-400">{t('deployed')}成功并上线</strong>
                    </div>
                    <span className="text-[10.5px] bg-indigo-500/10 text-indigo-600 px-2 py-0.5 rounded-md font-bold font-sans">
                      独立边缘加速节点
                    </span>
                  </div>

                  <div className="flex items-center justify-between gap-3 bg-[var(--color-kb-panel)] border border-[var(--color-kb-panel-border)] p-3 rounded-lg">
                    <div className="flex items-center gap-2 text-[12px] font-medium text-[var(--color-kb-text-heading)] truncate">
                      <Globe size={15} className="text-indigo-500 shrink-0" />
                      <span className="truncate underline cursor-pointer font-sans select-all">
                        {customDomain ? `https://${customDomain}` : deployedSiteUrl}
                      </span>
                    </div>
                    
                    <div className="flex items-center gap-1.5 shrink-0">
                      <button
                        onClick={handleCopy}
                        className="flex items-center gap-1 px-2.5 py-1 text-[11px] font-semibold bg-[var(--color-kb-editor)] hover:bg-[var(--color-kb-panel-hover)] border border-[var(--color-kb-panel-border)] rounded-md transition-all text-[var(--color-kb-text-heading)] shrink-0"
                      >
                        {copied ? (
                          <>
                            <Check size={12} className="text-emerald-500" />
                            <span className="text-emerald-500">已复制!</span>
                          </>
                        ) : (
                          <>
                            <Copy size={12} className="text-[var(--color-kb-text-muted)]" />
                            <span>复制地址</span>
                          </>
                        )}
                      </button>
                      
                      <a 
                        href={customDomain ? `https://${customDomain}` : deployedSiteUrl} 
                        target="_blank" 
                        rel="noreferrer"
                        className="p-1 text-[var(--color-kb-text-muted)] hover:text-indigo-500 border border-[var(--color-kb-panel-border)] hover:bg-[var(--color-kb-panel-hover)] rounded-md transition-all shrink-0"
                      >
                        <ExternalLink size={13} />
                      </a>
                    </div>
                  </div>
                </div>
              ) : (
                <div className="bg-amber-500/[0.02] border border-amber-500/20 rounded-xl p-4 flex gap-3 text-amber-800 dark:text-amber-400">
                  <AlertCircle size={16} className="text-amber-500 shrink-0 mt-0.5" />
                  <div className="space-y-1">
                    <h5 className="text-[12px] font-bold">该知识库尚未部署为网站</h5>
                    <p className="text-[10.5px] opacity-80 leading-relaxed">
                      部署后，您所有的内部协作文档将统一整理发布到专属加速地址中。任何人都可以极速阅览此站点，同时无需担心数据库泄漏。
                    </p>
                  </div>
                </div>
              )}

              {/* Form Settings Block */}
              <div className="space-y-4 pt-1">
                <div className="border-b border-[var(--color-kb-panel-border)]/50 pb-2">
                  <h4 className="text-[12.5px] font-bold text-[var(--color-kb-text-heading)] flex items-center gap-2">
                    <Settings size={14} className="text-indigo-500" />
                    网站基本设置
                  </h4>
                </div>

                {/* Grid Inputs */}
                <div className="space-y-3.5">
                  {/* Site Name and Custom Domain */}
                  <div className="grid grid-cols-2 gap-4">
                    <div className="space-y-1.5">
                      <label className="text-[11px] font-bold text-[var(--color-kb-text-heading)]">
                        网站显示名称 <span className="text-red-500">*</span>
                      </label>
                      <input 
                        type="text" 
                        value={siteName}
                        onChange={(e) => setSiteName(e.target.value)}
                        placeholder={activeKb.title}
                        className="w-full bg-[var(--color-kb-panel)] border border-[var(--color-kb-panel-border)] focus:border-indigo-500 hover:border-[var(--color-kb-panel-border)]/80 rounded-xl px-3.5 py-1.5 text-[12px] font-semibold outline-none focus:ring-1 focus:ring-indigo-500 text-[var(--color-kb-text-heading)]"
                      />
                    </div>

                    <div className="space-y-1.5">
                      <label className="text-[11px] font-bold text-[var(--color-kb-text-heading)] flex items-center justify-between">
                        <span>自定义独立域名</span>
                        <span className="text-[10px] text-[var(--color-kb-text-muted)] font-normal font-sans">（选填）</span>
                      </label>
                      <input 
                        type="text" 
                        value={customDomain}
                        onChange={(e) => setCustomDomain(e.target.value)}
                        placeholder="kb.yourdomain.com"
                        className="w-full bg-[var(--color-kb-panel)] border border-[var(--color-kb-panel-border)] focus:border-indigo-500 hover:border-[var(--color-kb-panel-border)]/80 rounded-xl px-3.5 py-1.5 text-[12px] font-semibold outline-none focus:ring-1 focus:ring-indigo-500 text-[var(--color-kb-text-heading)]"
                      />
                    </div>
                  </div>

                  {/* Logo Drag-and-drop Image Upload block */}
                  <div className="space-y-1.5">
                    <label className="text-[11px] font-bold text-[var(--color-kb-text-heading)]">
                      网站 Logo
                    </label>
                    
                    <div className="flex items-center gap-4">
                      {/* Logo Preview */}
                      <div className="w-16 h-16 rounded-xl border border-[var(--color-kb-panel-border)] shadow-sm bg-[var(--color-kb-panel)] flex items-center justify-center shrink-0 overflow-hidden select-none">
                        {logoBase64 ? (
                          logoBase64.startsWith('data:') || logoBase64.startsWith('http') ? (
                            <img src={logoBase64} alt="Website Logo" className="w-full h-full object-cover" />
                          ) : (
                            <span className="text-3xl leading-none">{logoBase64}</span>
                          )
                        ) : (
                          <ImageIcon size={22} className="text-[var(--color-kb-text-muted)] opacity-40" />
                        )}
                      </div>

                      {/* Drop area with full drag support & manual click uploader */}
                      <div 
                        onDragEnter={handleDrag}
                        onDragOver={handleDrag}
                        onDragLeave={handleDrag}
                        onDrop={handleDrop}
                        className={`flex-1 h-16 rounded-xl border-2 border-dashed flex flex-col items-center justify-center cursor-pointer transition-all ${
                          dragActive 
                            ? 'border-indigo-500 bg-indigo-500/[0.04]' 
                            : 'border-[var(--color-kb-panel-border)] hover:border-indigo-500/50 hover:bg-[var(--color-kb-panel)]/30'
                        }`}
                        onClick={triggerFileInput}
                      >
                        <Upload size={14} className="text-[var(--color-kb-text-muted)] mb-1" />
                        <p className="text-[10px] text-[var(--color-kb-text-muted)] text-center px-4 font-medium">
                          {dragActive ? "松手上传图片 Logo" : "将 Logo 拖拽放置于此，或 🙋 点击这里上传本地图片"}
                        </p>
                        <input
                          ref={fileInputRef}
                          type="file"
                          onChange={handleFileInputChange}
                          accept="image/*"
                          className="hidden"
                        />
                      </div>

                      {/* Delete logo helper */}
                      {logoBase64 && (
                        <button
                          onClick={() => setLogoBase64('')}
                          className="p-1.5 text-red-500 hover:bg-red-500/10 border border-red-500/10 rounded-lg transition-colors shrink-0"
                          title="移除 Logo"
                        >
                          <Trash2 size={13} />
                        </button>
                      )}
                    </div>
                  </div>

                  {/* Standard branding logo presets helper */}
                  <div className="flex items-center gap-1.5 p-2 bg-[var(--color-kb-panel)]/5 border border-[var(--color-kb-panel-border)]/50 rounded-lg shrink-0">
                    <Sparkles size={11.5} className="text-yellow-500 shrink-0" />
                    <span className="text-[10.5px] text-[var(--color-kb-text-muted)] font-medium">
                      推荐预设：
                    </span>
                    <div className="flex gap-1.5">
                      {['💡', '🚀', '📓', '💼', '💻', '🎨'].map(emoji => (
                        <button
                          key={emoji}
                          onClick={() => setLogoBase64(emoji)}
                          className="w-5 h-5 flex items-center justify-center text-[12px] hover:bg-[var(--color-kb-panel-hover)] rounded border border-[var(--color-kb-panel-border)] shadow-xs transition-colors"
                        >
                          {emoji}
                        </button>
                      ))}
                    </div>
                  </div>
                </div>
              </div>
            </>
          )}
        </div>

        {/* Modal Footer */}
        <div className="h-[70px] border-t border-zinc-200/80 dark:border-[var(--color-kb-panel-border)] bg-[#fafafa] dark:bg-[var(--color-kb-panel)] flex items-center justify-between px-6 shrink-0 rounded-b-2xl shadow-[0_-4px_20px_rgba(0,0,0,0.02)] z-10">
          <div>
            {isDeployed ? (
              <button
                onClick={handleStopDeploy}
                className="text-[12.5px] font-bold text-red-500 hover:text-red-600 flex items-center gap-1.5 hover:bg-red-50 px-3 py-1.5 rounded-lg transition-colors active:scale-95"
              >
                <Trash2 size={14} strokeWidth={2.5} />
                <span>{t('offlineSite')}</span>
              </button>
            ) : (
              <span className="text-[11.5px] font-bold text-zinc-500 dark:text-[var(--color-kb-text-muted)] flex items-center gap-1.5">
                <Laptop size={14} strokeWidth={2.5} />
                {t('editableSite')}
              </span>
            )}
          </div>
          
          <div className="flex items-center gap-3">
            <button
              onClick={onClose}
              className="px-5 py-2.5 text-[13px] font-bold text-zinc-600 dark:text-[var(--color-kb-text-heading)] bg-white dark:bg-[var(--color-kb-editor)] hover:bg-zinc-100 dark:hover:bg-[var(--color-kb-panel-hover)] border-2 border-transparent hover:border-zinc-200/80 dark:border-[var(--color-kb-panel-border)] rounded-xl transition-all shadow-sm active:scale-95"
              disabled={isDeploying}
            >
              {t('close')}
            </button>
            {isDeployed ? (
              <button
                onClick={handleSaveSettings}
                className="px-6 py-2.5 text-[13px] font-extrabold bg-[#07C160] hover:bg-[#06ad56] text-white rounded-xl shadow-md transition-all active:scale-95 hover:shadow-lg focus:outline-none focus:ring-4 focus:ring-[#07C160]/20"
                disabled={isDeploying}
              >
                {t('saveConfig')}
              </button>
            ) : (
              <button
                onClick={handleStartDeploy}
                className="px-6 py-2.5 text-[13px] font-extrabold bg-[#07C160] hover:bg-[#06ad56] text-white rounded-xl shadow-md transition-all flex items-center gap-2 active:scale-95 hover:shadow-lg focus:outline-none focus:ring-4 focus:ring-[#07C160]/20 disabled:opacity-40 disabled:grayscale"
                disabled={isDeploying}
              >
                <RefreshCw size={15} strokeWidth={2.5} className={isDeploying ? 'animate-spin' : ''} />
                <span>{t('deployStart')}</span>
              </button>
            )}
          </div>
        </div>

      </div>
    </div>,
    document.body
  );
}

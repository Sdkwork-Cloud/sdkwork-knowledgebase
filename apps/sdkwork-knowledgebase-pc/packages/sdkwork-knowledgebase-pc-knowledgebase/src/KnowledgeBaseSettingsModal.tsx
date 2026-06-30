import React, { useState, useRef, useEffect } from 'react';
import { isBlank, trim } from '@sdkwork/utils';
import { useTranslation } from 'react-i18next';
import { X, Shield, Settings, Sliders, Upload, UserPlus, Globe, Check, AlertCircle } from 'lucide-react';
import { isKnowledgebaseApiAvailable } from 'sdkwork-knowledgebase-pc-core';
import { KnowledgeBase, DocumentService } from './services/document';
import type { KnowledgeSpaceMemberUi } from './services/knowledgeSpaceMembersService';

interface KnowledgeBaseSettingsModalProps {
  kb: KnowledgeBase;
  onClose: () => void;
  onSave: (updates: Partial<KnowledgeBase>) => void;
}

const predefinedIcons = ['📘', '📗', '📕', '📙', '📓', '📁', '🌟', '🚀', '💡', '🔥', '⚙️', '📊', '🌍', '📖'];

interface MemberMock extends KnowledgeSpaceMemberUi {}

export function KnowledgeBaseSettingsModal({ kb, onClose, onSave }: KnowledgeBaseSettingsModalProps) {
  const { t } = useTranslation(['kb', 'common']);
  const [activeTab, setActiveTab] = useState<'basic' | 'permissions' | 'model'>('basic');
  
  // Basic Settings States
  const [title, setTitle] = useState(kb.title);
  const [icon, setIcon] = useState(kb.icon || '📁');
  const [avatar, setAvatar] = useState(kb.avatar || '');
  const [type, setType] = useState<'team' | 'personal' | 'public'>(kb.type || 'team');
  const fileInputRef = useRef<HTMLInputElement>(null);

  // Permissions Settings States
  const [publicPermission, setPublicPermission] = useState<'none' | 'read' | 'write' | 'admin'>(kb.publicPermission || 'none');
  const [guestLinkEnabled, setGuestLinkEnabled] = useState(kb.guestLinkEnabled !== undefined ? kb.guestLinkEnabled : false);
  const [members, setMembers] = useState<MemberMock[]>([]);
  const initialMembersRef = useRef<MemberMock[]>([]);
  const [newMemberEmail, setNewMemberEmail] = useState('');
  const [newMemberRole, setNewMemberRole] = useState<'admin' | 'editor' | 'viewer'>('viewer');

  useEffect(() => {
    const spaceId = Number(kb.id);
    if (!isKnowledgebaseApiAvailable() || !Number.isFinite(spaceId) || spaceId <= 0) {
      initialMembersRef.current = [];
      setMembers([]);
      return;
    }
    let cancelled = false;
    DocumentService.loadKnowledgeSpaceMembers(spaceId)
      .then((loaded) => {
        if (cancelled) {
          return;
        }
        initialMembersRef.current = loaded;
        setMembers(loaded);
      })
      .catch(() => {
        if (!cancelled) {
          initialMembersRef.current = [];
          setMembers([]);
        }
      });
    return () => {
      cancelled = true;
    };
  }, [kb.id]);

  // Model Settings States
  const [provider, setProvider] = useState(kb.provider || 'Google');
  const [modelName, setModelName] = useState(kb.modelName || 'gemini-3.5-flash');
  const [temperature, setTemperature] = useState(kb.temperature !== undefined ? kb.temperature : 0.7);
  const [maxTokens, setMaxTokens] = useState(kb.maxTokens || 2048);
  const [systemPrompt, setSystemPrompt] = useState(kb.systemPrompt || '你是一个资深的知识库智脑助手。请基于已知文档回答用户的问题。如果问题不在文档中，请友好地指出。');

  const handleImageUpload = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (file) {
      const reader = new FileReader();
      reader.onload = (event) => {
        if (event.target?.result) {
          setAvatar(event.target.result as string);
          setIcon('');
        }
      };
      reader.readAsDataURL(file);
    }
  };

  const handleAddMember = (e: React.FormEvent) => {
    e.preventDefault();
    if (isBlank(newMemberEmail)) return;
    const name = newMemberEmail.split('@')[0];
    const capitalizedName = name.charAt(0).toUpperCase() + name.slice(1);
    const newMember: MemberMock = {
      name: capitalizedName,
      email: newMemberEmail,
      role: newMemberRole,
      avatar: isKnowledgebaseApiAvailable()
        ? ''
        : `https://images.unsplash.com/photo-${Math.floor(Math.random() * 100000) + 1500000}?w=100&h=100&fit=crop`,
    };
    setMembers([...members, newMember]);
    setNewMemberEmail('');
  };

  const handleRemoveMember = (idx: number) => {
    setMembers(members.filter((_, i) => i !== idx));
  };

  const handleSaveAll = async () => {
    const spaceId = Number(kb.id);
    if (isKnowledgebaseApiAvailable() && Number.isFinite(spaceId) && spaceId > 0) {
      await DocumentService.syncKnowledgeSpaceMembers(spaceId, members, initialMembersRef.current);
    }
    onSave({
      title,
      icon,
      avatar,
      type,
      publicPermission,
      guestLinkEnabled,
      provider,
      modelName,
      temperature,
      maxTokens,
      systemPrompt
    });
    onClose();
  };

  // Predefined models mapping based on provider
  const modelsForProvider: Record<string, string[]> = {
    Google: ['gemini-3.5-flash', 'gemini-3.5-pro', 'gemini-2.5-flash', 'gemini-2.5-pro', 'gemini-1.5-pro', 'gemini-1.5-flash'],
    OpenAI: ['gpt-4o', 'gpt-4o-mini', 'o1-mini'],
    DeepSeek: ['deepseek-chat', 'deepseek-reasoner']
  };

  const providerChange = (p: string) => {
    setProvider(p);
    const available = modelsForProvider[p];
    if (available && available.length > 0) {
      setModelName(available[0]);
    }
  };

  return (
    <div className="fixed inset-0 bg-zinc-950/40 z-[300] flex items-center justify-center backdrop-blur-md animate-fade-in">
      <div className="bg-white dark:bg-[var(--color-kb-editor)] w-[820px] h-[600px] rounded-2xl shadow-[0_24px_70px_-15px_rgba(0,0,0,0.1)] dark:shadow-[0_24px_70px_-15px_rgba(0,0,0,0.4)] border border-zinc-200/80 dark:border-[var(--color-kb-panel-border)] flex overflow-hidden animate-in fade-in zoom-in-95 duration-300">
        
        {/* Left Navigation Rails / Tab Sidebar */}
        <div className="w-[190px] bg-[#fafafa] dark:bg-[var(--color-kb-panel)] border-r border-zinc-200/80 dark:border-[var(--color-kb-panel-border)] flex flex-col justify-between py-6 shrink-0 relative z-10 shadow-[4px_0_24px_-12px_rgba(0,0,0,0.04)]">
          <div className="flex flex-col space-y-1 px-3">
            <div className="flex items-center gap-3 px-3 mb-6">
              <span className="text-[20px] drop-shadow-sm scale-110">{icon}</span>
              <div className="min-w-0">
                <div className="text-[10px] text-zinc-400 dark:text-[var(--color-kb-text-muted)] leading-none uppercase font-bold tracking-widest mb-1">{t('kbSpace')}</div>
                <div className="text-[14px] font-extrabold text-zinc-800 dark:text-[var(--color-kb-text-heading)] truncate tracking-tight">{title}</div>
              </div>
            </div>

            <button
              onClick={() => setActiveTab('basic')}
              className={`flex items-center gap-2.5 px-3 py-2.5 rounded-xl text-[13px] font-semibold transition-all ${activeTab === 'basic' ? 'bg-zinc-900 text-white dark:bg-[var(--color-kb-accent)] shadow-md' : 'text-zinc-500 hover:text-zinc-900 dark:text-[var(--color-kb-text)] hover:bg-black/5 dark:hover:bg-[var(--color-kb-panel-hover)]'}`}
            >
              <Settings size={15} />
              <span>基础设置</span>
            </button>
            <button
              onClick={() => setActiveTab('permissions')}
              className={`flex items-center gap-2.5 px-3 py-2.5 rounded-xl text-[13px] font-semibold transition-all ${activeTab === 'permissions' ? 'bg-zinc-900 text-white dark:bg-[var(--color-kb-accent)] shadow-md' : 'text-zinc-500 hover:text-zinc-900 dark:text-[var(--color-kb-text)] hover:bg-black/5 dark:hover:bg-[var(--color-kb-panel-hover)]'}`}
            >
              <Shield size={15} />
              <span>权限管理</span>
            </button>
            <button
              onClick={() => setActiveTab('model')}
              className={`flex items-center gap-2.5 px-3 py-2.5 rounded-xl text-[13px] font-semibold transition-all ${activeTab === 'model' ? 'bg-zinc-900 text-white dark:bg-[var(--color-kb-accent)] shadow-md' : 'text-zinc-500 hover:text-zinc-900 dark:text-[var(--color-kb-text)] hover:bg-black/5 dark:hover:bg-[var(--color-kb-panel-hover)]'}`}
            >
              <Sliders size={15} />
              <span>智脑模型设置</span>
            </button>
          </div>

          <div className="px-6">
            <span className="text-[10px] text-zinc-400 dark:text-[var(--color-kb-text-muted)] font-mono font-bold tracking-widest uppercase">v1.2.5 • Premium</span>
          </div>
        </div>

        {/* Right Active Tab Settings Panel Workspace */}
        <div className="flex-1 flex flex-col justify-between overflow-hidden bg-white dark:bg-[var(--color-kb-editor)]">
          <div className="flex items-center justify-between px-8 py-5 border-b border-zinc-200/80 dark:border-[var(--color-kb-panel-border)] bg-transparent dark:bg-[var(--color-kb-panel)]/30 backdrop-blur-sm z-20">
            <div>
              <h3 className="font-extrabold text-[18px] tracking-tight text-zinc-900 dark:text-[var(--color-kb-text-heading)]">
                {activeTab === 'basic' && t('basicSettings')}
                {activeTab === 'permissions' && t('permissionsSettingsDesc')}
                {activeTab === 'model' && t('modelSettingsDesc')}
              </h3>
              <p className="text-[12px] text-zinc-500 dark:text-[var(--color-kb-text-muted)] mt-1 font-medium">
                {activeTab === 'basic' && t('basicSettingsDesc')}
                {activeTab === 'permissions' && t('permissionsSettingsDesc2')}
                {activeTab === 'model' && t('modelSettingsDesc2')}
              </p>
            </div>
            <button onClick={onClose} className="text-zinc-400 dark:text-[var(--color-kb-text-muted)] hover:text-red-500 transition-all p-2 rounded-xl hover:bg-black/5 dark:hover:bg-[var(--color-kb-panel-hover)] group">
              <X size={18} className="group-hover:scale-110 transition-transform" />
            </button>
          </div>

          {/* Core Fields Form Scroll Area */}
          <div className="flex-1 overflow-y-auto px-8 py-8 space-y-8">
            
            {/* TAB: BASIC */}
            {activeTab === 'basic' && (
              <div className="space-y-6 animate-in fade-in duration-300">
                <div className="flex flex-col space-y-2">
                  <label className="text-[13px] font-bold text-zinc-800 dark:text-[var(--color-kb-text-heading)]">{t('kbRename')}</label>
                  <input 
                    type="text" 
                    value={title}
                    onChange={(e) => setTitle(e.target.value)}
                    placeholder={t('kbRenamePlaceholder')} 
                    className="w-full bg-[#fafafa] dark:bg-[var(--color-kb-input-bg)] border border-zinc-200/80 dark:border-[var(--color-kb-panel-border)] rounded-xl px-4 py-3 text-[14px] font-medium text-zinc-900 dark:text-[var(--color-kb-text)] focus:outline-none focus:ring-4 focus:ring-zinc-900/5 dark:focus:ring-[var(--color-kb-accent)]/20 focus:border-zinc-900 dark:focus:border-[var(--color-kb-accent)] transition-all placeholder-zinc-400 shadow-sm"
                  />
                </div>

                <div className="flex flex-col space-y-3">
                  <label className="text-[13px] font-bold text-zinc-800 dark:text-[var(--color-kb-text-heading)]">{t('iconAvatarSelection')}</label>
                  <div className="flex items-start gap-5">
                    <div 
                      onClick={() => fileInputRef.current?.click()}
                      className="w-[72px] h-[72px] rounded-2xl border-2 border-dashed border-zinc-300 dark:border-[var(--color-kb-panel-border)] bg-[#fafafa] dark:bg-[var(--color-kb-panel-hover)] flex items-center justify-center cursor-pointer hover:border-zinc-900 dark:hover:border-[var(--color-kb-accent)] hover:bg-zinc-50 dark:hover:bg-[var(--color-kb-accent)]/5 transition-all overflow-hidden flex-shrink-0 group relative shadow-inner"
                    >
                      {avatar ? (
                        <img src={avatar} alt="Avatar Preview" className="w-full h-full object-cover" />
                      ) : (
                        <span className="text-[32px] drop-shadow-sm group-hover:scale-110 transition-transform">{icon}</span>
                      )}
                      <div className="absolute inset-0 bg-black/40 flex items-center justify-center opacity-0 group-hover:opacity-100 transition-opacity backdrop-blur-[2px]">
                        <Upload size={18} className="text-white drop-shadow-md" />
                      </div>
                    </div>
                    
                    <div className="flex-1 mt-1">
                      <div className="flex flex-wrap gap-2 mb-2">
                        {predefinedIcons.map(item => (
                          <button
                            key={item}
                            type="button"
                            onClick={() => { setIcon(item); setAvatar(''); }}
                            className={`w-9 h-9 flex items-center justify-center text-[18px] rounded-xl border-2 transition-all ${icon === item && !avatar ? 'border-zinc-900 bg-zinc-900/5 dark:border-[var(--color-kb-accent)] dark:bg-[var(--color-kb-accent)]/15 shadow-sm scale-110 z-10' : 'border-zinc-100 dark:border-[var(--color-kb-panel-border)] bg-[#fafafa] dark:bg-[var(--color-kb-panel)] hover:border-zinc-300 dark:hover:bg-[var(--color-kb-panel-hover)]'}`}
                          >
                            {item}
                          </button>
                        ))}
                      </div>
                      <input 
                        type="file" 
                        ref={fileInputRef} 
                        onChange={handleImageUpload} 
                        accept="image/*" 
                        className="hidden" 
                      />
                    </div>
                  </div>
                </div>

                <div className="flex flex-col space-y-3 pt-2">
                  <label className="text-[13px] font-bold text-zinc-800 dark:text-[var(--color-kb-text-heading)]">知识库共享类别区域</label>
                  <div className="grid grid-cols-3 gap-3">
                    {[
                      { key: 'team', title: '团队知识库', desc: '组织成员协作共享', icon: '👥' },
                      { key: 'personal', title: '个人知识库', desc: '私有仅对自己可见', icon: '👤' },
                      { key: 'public', title: '共享知识库', desc: '外部或订阅多人使用', icon: '🌍' }
                    ].map(item => (
                      <button
                        key={item.key}
                        type="button"
                        onClick={() => setType(item.key as any)}
                        className={`flex flex-col items-start p-4 rounded-2xl border-2 text-left transition-all relative overflow-hidden group ${type === item.key ? 'border-zinc-900 bg-zinc-900/5 dark:border-[var(--color-kb-accent)] dark:bg-[var(--color-kb-accent)]/5 text-zinc-900 dark:text-[var(--color-kb-accent)] shadow-sm translate-y-[-2px]' : 'border-zinc-100 dark:border-[var(--color-kb-panel-border)] bg-white dark:bg-[var(--color-kb-panel)] text-zinc-600 dark:text-[var(--color-kb-text)] hover:border-zinc-300 dark:hover:bg-[var(--color-kb-panel-hover)]'}`}
                      >
                        <div className="flex items-center gap-2 font-bold text-[14px] mb-1.5 z-10 w-full drop-shadow-sm">
                          <span className={`${type===item.key ? 'scale-110 transition-transform' : 'grayscale group-hover:grayscale-0 transition-all'}`}>{item.icon}</span>
                          <span className={type === item.key ? 'text-zinc-900 dark:text-[var(--color-kb-text-heading)]' : 'text-zinc-700 dark:text-[var(--color-kb-text)]'}>{item.title}</span>
                          {type === item.key && (
                            <div className="ml-auto w-4 h-4 bg-zinc-900 dark:bg-[var(--color-kb-accent)] rounded-full flex items-center justify-center shadow-md">
                              <Check size={10} className="text-white" strokeWidth={3} />
                            </div>
                          )}
                        </div>
                        <span className="text-[11px] text-zinc-500 dark:text-[var(--color-kb-text-muted)] font-medium z-10">{item.desc}</span>
                      </button>
                    ))}
                  </div>
                </div>
              </div>
            )}

            {/* TAB: PERMISSIONS */}
            {activeTab === 'permissions' && (
              <div className="space-y-8 animate-in fade-in duration-300">
                
                {/* Public Global Authorization setting */}
                <div className="p-5 bg-emerald-50/50 dark:bg-emerald-950/20 border-2 border-emerald-100 dark:border-emerald-900/30 rounded-2xl flex flex-col space-y-4 shadow-sm relative overflow-hidden group">
                  <div className="absolute top-0 right-0 -mt-6 -mr-6 w-32 h-32 bg-emerald-400/10 rounded-full blur-2xl group-hover:bg-emerald-400/20 transition-all duration-500"></div>
                  
                  <div className="flex items-center justify-between relative z-10">
                    <div className="flex items-center gap-3">
                      <div className="w-10 h-10 rounded-xl bg-emerald-100 dark:bg-emerald-900/50 text-emerald-600 dark:text-emerald-400 flex items-center justify-center shadow-inner border border-emerald-200 dark:border-emerald-800">
                        <Globe size={18} strokeWidth={2.5} />
                      </div>
                      <div>
                        <div className="text-[14px] font-extrabold text-emerald-900 dark:text-emerald-300 tracking-tight">所有人（免密与外部链接）公开访问权限</div>
                        <p className="text-[11.5px] text-emerald-700/80 dark:text-emerald-500/80 font-medium">关闭即表示此知识库为内部独享空间</p>
                      </div>
                    </div>
                    <span className="text-[10px] bg-emerald-200 dark:bg-emerald-800 text-emerald-800 dark:text-emerald-300 px-2.5 py-1 rounded-md font-bold uppercase tracking-widest shadow-sm">Public Key</span>
                  </div>

                  <div className="grid grid-cols-4 gap-3 pt-2 relative z-10">
                    {[
                      { key: 'none', label: '🔏 禁止公开', desc: '仅内部账号可用' },
                      { key: 'read', label: '📖 只读访问', desc: '支持外链直接阅读' },
                      { key: 'write', label: '✍️ 协作编写', desc: '无需配置即登可写' },
                      { key: 'admin', label: '🛡️ 完整管理', desc: '可流式配置整库' }
                    ].map(item => (
                      <button
                        key={item.key}
                        type="button"
                        onClick={() => setPublicPermission(item.key as any)}
                        className={`p-3 rounded-xl border-2 text-left flex flex-col transition-all relative overflow-hidden ${publicPermission === item.key ? 'border-emerald-500 bg-white dark:bg-emerald-900/40 text-emerald-700 dark:text-emerald-400 font-bold shadow-md ring-4 ring-emerald-500/10 translate-y-[-2px]' : 'border-emerald-100 dark:border-emerald-900/30 bg-white/60 dark:bg-[var(--color-kb-panel)] hover:border-emerald-300 dark:hover:bg-zinc-800 text-zinc-600 dark:text-zinc-300 font-medium'}`}
                      >
                        <span className="text-[13px]">{item.label}</span>
                        <span className="text-[10px] opacity-75 mt-1 leading-tight">{item.desc}</span>
                        {publicPermission === item.key && (
                          <div className="absolute top-2 right-2 w-3 h-3 bg-emerald-500 rounded-full flex items-center justify-center shadow-sm">
                            <Check size={8} className="text-white" strokeWidth={4} />
                          </div>
                        )}
                      </button>
                    ))}
                  </div>

                  {publicPermission !== 'none' && (
                    <div className="flex items-center justify-between pt-4 border-t border-emerald-100 dark:border-emerald-900/30 relative z-10">
                      <span className="text-[12px] text-emerald-800 dark:text-emerald-400 font-bold flex items-center gap-1.5 bg-white dark:bg-emerald-950/50 px-3 py-1.5 rounded-lg border border-emerald-100 dark:border-emerald-800 text-sm shadow-sm">
                        <AlertCircle size={14} className="text-emerald-500" strokeWidth={2.5} /> 访客通过专属阅读免登码直接进入
                      </span>
                      <label className="relative inline-flex items-center cursor-pointer group">
                        <input 
                          type="checkbox" 
                          checked={guestLinkEnabled} 
                          onChange={(e) => setGuestLinkEnabled(e.target.checked)}
                          className="sr-only peer" 
                        />
                        <div className="w-10 h-5 bg-zinc-300 dark:bg-zinc-600 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-4 after:w-4 after:transition-all peer-checked:bg-emerald-500 shadow-inner"></div>
                        <span className="ml-3 text-[13px] font-extrabold text-emerald-900 dark:text-emerald-300 group-hover:text-emerald-600 transition-colors">开启快捷分享通道</span>
                      </label>
                    </div>
                  )}
                </div>

                {/* Internal Organization / Project Members Roll list */}
                <div className="space-y-4">
                  <div className="text-[14px] font-extrabold text-zinc-900 dark:text-[var(--color-kb-text-heading)] flex items-center gap-2">
                    组织内部成员组权限配置 
                    <span className="text-[11px] font-bold bg-zinc-900 text-white dark:bg-white dark:text-zinc-900 px-2 py-0.5 rounded-full">{members.length}</span>
                  </div>
                  
                  <form onSubmit={handleAddMember} className="flex gap-2">
                    <input 
                      type="email" 
                      placeholder={t('emailPlaceholder')}
                      value={newMemberEmail}
                      onChange={(e) => setNewMemberEmail(e.target.value)}
                      className="flex-1 bg-white dark:bg-[var(--color-kb-input-bg)] border-2 border-zinc-200/80 dark:border-[var(--color-kb-panel-border)] rounded-xl px-4 py-2.5 text-[14px] font-medium text-zinc-900 dark:text-[var(--color-kb-text)] focus:outline-none focus:border-zinc-900 dark:focus:border-[var(--color-kb-accent)] focus:ring-4 focus:ring-zinc-900/5 shadow-sm transition-all placeholder-zinc-400"
                    />
                    <select
                      value={newMemberRole}
                      onChange={(e) => setNewMemberRole(e.target.value as any)}
                      className="bg-[#fafafa] dark:bg-[var(--color-kb-panel)] border-2 border-zinc-200/80 dark:border-[var(--color-kb-panel-border)] rounded-xl px-4 py-2.5 text-[13px] font-bold text-zinc-700 dark:text-[var(--color-kb-text)] focus:outline-none focus:border-zinc-900 active:scale-95 transition-all shadow-sm cursor-pointer"
                    >
                      <option value="viewer">读取者 (Viewer)</option>
                      <option value="editor">{t('editor')}</option>
                      <option value="admin">管理者 (Admin)</option>
                    </select>
                    <button type="submit" className="bg-[var(--color-kb-accent)] text-white hover:bg-[var(--color-kb-accent-hover)] px-6 py-2.5 rounded-xl text-[13px] font-bold flex items-center gap-1.5 transition-all shadow-md active:scale-95 focus:outline-none">
                      <UserPlus size={16} /> 新增
                    </button>
                  </form>

                  <div className="border-2 border-zinc-200/80 dark:border-[var(--color-kb-panel-border)] rounded-2xl overflow-hidden divide-y divide-zinc-100 dark:divide-[var(--color-kb-panel-border)] shadow-sm bg-white dark:bg-[var(--color-kb-editor)]">
                    {members.map((member, idx) => (
                      <div key={member.email} className="flex items-center justify-between px-5 py-3 hover:bg-[#fafafa] dark:hover:bg-zinc-900/50 transition-colors">
                        <div className="flex items-center gap-3 min-w-0">
                          {member.avatar ? (
                            <img src={member.avatar} alt={member.name} className="w-8 h-8 rounded-full object-cover shrink-0 border border-zinc-200/80 shadow-sm" />
                          ) : (
                            <div className="w-8 h-8 rounded-full shrink-0 border border-zinc-200/80 shadow-sm bg-zinc-100 dark:bg-zinc-800 flex items-center justify-center text-[12px] font-bold text-zinc-600 dark:text-zinc-300">
                              {member.name.charAt(0).toUpperCase()}
                            </div>
                          )}
                          <div className="min-w-0">
                            <div className="text-[13.5px] font-extrabold text-zinc-900 dark:text-[var(--color-kb-text)] truncate tracking-tight">{member.name}</div>
                            <div className="text-[11px] text-zinc-500 font-medium truncate tracking-wide">{member.email}</div>
                          </div>
                        </div>

                        <div className="flex items-center gap-4">
                          <select
                            value={member.role}
                            onChange={(e) => {
                              const updated = [...members];
                              updated[idx].role = e.target.value as any;
                              setMembers(updated);
                            }}
                            className="bg-zinc-50 dark:bg-transparent text-[12px] text-zinc-700 font-bold dark:text-[var(--color-kb-text-muted)] hover:text-zinc-900 dark:hover:text-[var(--color-kb-text-heading)] focus:outline-none border border-zinc-200/80 dark:border-zinc-700/50 rounded-lg px-2.5 py-1.5 cursor-pointer"
                          >
                            <option value="admin">{t('admin')}</option>
                            <option value="editor">{t('editor')}</option>
                            <option value="viewer">{t('viewer')}</option>
                          </select>
                          
                          {member.email !== 'alice@company.com' ? (
                            <button 
                              type="button" 
                              onClick={() => handleRemoveMember(idx)} 
                              className="w-8 h-8 flex items-center justify-center text-zinc-400 hover:text-red-600 hover:bg-red-50 rounded-lg transition-all"
                            >
                              <X size={16} strokeWidth={2.5} />
                            </button>
                          ) : (
                            <div className="w-8"></div>
                          )}
                        </div>
                      </div>
                    ))}
                  </div>
                </div>
              </div>
            )}

            {/* TAB: MODEL SETTINGS */}
            {activeTab === 'model' && (
              <div className="space-y-8 animate-in fade-in duration-300 font-sans">
                <div className="bg-gradient-to-br from-indigo-50/50 to-purple-50/30 dark:from-indigo-950/20 dark:to-purple-950/20 border border-indigo-100 dark:border-indigo-900/30 rounded-2xl p-6 shadow-sm relative overflow-hidden group">
                   {/* Background decorative elements */}
                   <div className="absolute top-0 right-0 -mr-8 -mt-8 w-32 h-32 bg-indigo-500/5 dark:bg-indigo-500/10 rounded-full blur-2xl group-hover:bg-indigo-500/10 transition-all duration-500"></div>
                   
                   <div className="grid grid-cols-2 gap-5 relative z-10">
                     {/* Select AI Model Provider */}
                     <div className="flex flex-col space-y-2">
                       <label className="text-[13px] font-extrabold text-indigo-950 dark:text-indigo-100">{t('cloudProvider')}</label>
                       <div className="flex gap-2">
                         {['Google', 'DeepSeek', 'OpenAI'].map(p => (
                           <button
                             key={p}
                             type="button"
                             onClick={() => providerChange(p)}
                             className={`flex-1 py-2 text-[12.5px] rounded-xl border-2 font-bold text-center transition-all ${provider === p ? 'border-indigo-500 bg-white dark:bg-indigo-900/40 text-indigo-700 dark:text-indigo-300 shadow-sm translate-y-[-1px]' : 'border-indigo-100 dark:border-indigo-900/40 bg-white/50 dark:bg-[var(--color-kb-panel)] text-zinc-500 hover:text-zinc-800 hover:border-indigo-300'}`}
                           >
                             {p}
                           </button>
                         ))}
                       </div>
                     </div>

                     {/* Select Model Name */}
                     <div className="flex flex-col space-y-2">
                       <label className="text-[13px] font-extrabold text-indigo-950 dark:text-indigo-100">{t('llmModel')}</label>
                       <select
                         value={modelName}
                         onChange={(e) => setModelName(e.target.value)}
                         className="w-full bg-white dark:bg-[var(--color-kb-input-bg)] border-2 border-indigo-100 dark:border-indigo-900/40 rounded-xl px-4 py-2 text-[13.5px] font-bold text-indigo-900 dark:text-indigo-100 focus:outline-none focus:border-indigo-500 shadow-sm cursor-pointer"
                       >
                         {(modelsForProvider[provider] || []).map(m => (
                           <option key={m} value={m}>{m}</option>
                         ))}
                       </select>
                     </div>
                   </div>
                </div>

                <div className="grid grid-cols-2 gap-5">
                  {/* Temperature Settings */}
                  <div className="p-5 bg-white dark:bg-[var(--color-kb-panel)] border-2 border-zinc-200/80 dark:border-[var(--color-kb-panel-border)] rounded-2xl space-y-3 shadow-sm hover:border-zinc-300 transition-colors">
                    <div className="flex items-center justify-between">
                      <label className="text-[13px] font-extrabold text-zinc-900 dark:text-[var(--color-kb-text-heading)]">{t('temperature')}</label>
                      <span className="font-mono text-[13px] font-bold text-indigo-600 bg-indigo-50 px-2 py-0.5 rounded-md">{temperature.toFixed(2)}</span>
                    </div>
                    <input 
                      type="range" 
                      min="0" 
                      max="1" 
                      step="0.05"
                      value={temperature}
                      onChange={(e) => setTemperature(parseFloat(e.target.value))}
                      className="w-full h-2 bg-zinc-200 dark:bg-zinc-700 rounded-lg appearance-none cursor-pointer accent-indigo-600"
                    />
                    <div className="text-[10.5px] font-bold tracking-wide text-zinc-500 flex justify-between mt-1">
                      <span>0.0 (严谨模式)</span>
                      <span>1.0 (发散创造)</span>
                    </div>
                  </div>

                  {/* Max Tokens settings */}
                  <div className="flex flex-col space-y-2 p-5 bg-white dark:bg-[var(--color-kb-panel)] border-2 border-zinc-200/80 dark:border-[var(--color-kb-panel-border)] rounded-2xl shadow-sm hover:border-zinc-300 transition-colors">
                    <label className="text-[13px] font-extrabold text-zinc-900 dark:text-[var(--color-kb-text-heading)]">单次最大回复 Token 上限</label>
                    <input 
                      type="number" 
                      value={maxTokens}
                      onChange={(e) => setMaxTokens(parseInt(e.target.value) || 128)}
                      min="128"
                      max="16384"
                      className="w-full bg-[#fafafa] dark:bg-[var(--color-kb-input-bg)] border border-zinc-200/80 dark:border-[var(--color-kb-panel-border)] rounded-xl px-4 py-2 text-[14px] font-mono font-bold text-zinc-900 dark:text-[var(--color-kb-text)] focus:outline-none focus:border-zinc-900 shadow-inner"
                    />
                    <p className="text-[10.5px] font-medium text-zinc-500 leading-tight">
                      指定每次产生的文字大小。建议 2048 - 4096 范围。
                    </p>
                  </div>
                </div>

                {/* System Prompt directive */}
                <div className="flex flex-col space-y-2">
                  <label className="text-[14px] font-extrabold text-zinc-900 dark:text-[var(--color-kb-text-heading)]">AI系统行为指令 (System Prompt Instruct)</label>
                  <textarea 
                    rows={4}
                    value={systemPrompt}
                    onChange={(e) => setSystemPrompt(e.target.value)}
                    placeholder={t('systemPromptPlaceholder')}
                    className="w-full bg-[#fafafa] dark:bg-[var(--color-kb-input-bg)] border-2 border-zinc-200/80 dark:border-[var(--color-kb-panel-border)] rounded-2xl p-4 text-[13px] text-zinc-800 dark:text-[var(--color-kb-text)] focus:outline-none focus:border-zinc-900 font-mono resize-none shadow-inner leading-relaxed text-opacity-90"
                  />
                </div>
              </div>
            )}
          </div>

          {/* Bottom Dialog Action footer */}
          <div className="px-8 py-5 border-t border-zinc-200/80 dark:border-[var(--color-kb-panel-border)] bg-[#fafafa] dark:bg-[var(--color-kb-panel)]/30 backdrop-blur-sm flex justify-end gap-3 z-30">
            <button 
              type="button"
              onClick={onClose}
              className="px-6 py-2.5 rounded-xl border-2 border-zinc-200/80 dark:border-[var(--color-kb-panel-border)] text-[13.5px] font-bold text-zinc-600 dark:text-[var(--color-kb-text)] hover:bg-zinc-100 dark:hover:bg-[var(--color-kb-panel-hover)] hover:text-zinc-900 transition-colors shadow-sm focus:outline-none focus:ring-4 focus:ring-zinc-900/5 active:scale-95"
            >
              放弃更改
            </button>
            <button 
              type="button"
              onClick={handleSaveAll}
              className="px-6 py-2.5 rounded-xl bg-[var(--color-kb-accent)] text-white hover:bg-[var(--color-kb-accent-hover)] text-[13.5px] font-extrabold transition-all shadow-md shadow-[var(--color-kb-accent)]/10 active:scale-95 flex items-center gap-2 focus:outline-none focus:ring-4 focus:ring-[var(--color-kb-accent)]/20"
            >
              <Check size={16} strokeWidth={3} /> 保存并应用设置
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}

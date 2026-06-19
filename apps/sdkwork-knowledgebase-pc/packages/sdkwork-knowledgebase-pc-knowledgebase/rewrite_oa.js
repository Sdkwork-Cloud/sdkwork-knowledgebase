const fs = require('fs');

const filepath = './packages/sdkwork-knowledgebase-pc-knowledgebase/src/components/OfficialAccountModal.tsx';

const content = `import React, { useState, useRef } from 'react';
import { 
  X, Settings2, Folder, Tags, Plus, Trash2, Check, Server, Link as LinkIcon, Edit3, MessageSquare, Globe, Key, Upload, Shield
} from 'lucide-react';
import { OfficialAccount } from '../services/wechat';

import { toast } from './ui/toast-manager';

interface OfficialAccountModalProps {
  isOpen: boolean;
  onClose: () => void;
  onConfirm: (data: { 
    officialAccounts: OfficialAccount[]; 
    selectedOfficialAccountIds: string[]; 
    oaGroups: string[];
  }) => void;
  initialOfficialAccounts: OfficialAccount[];
  initialSelectedAccountIds: string[];
  initialOaGroups: string[];
}

export function OfficialAccountModal({
  isOpen,
  onClose,
  onConfirm,
  initialOfficialAccounts,
  initialSelectedAccountIds,
  initialOaGroups
}: OfficialAccountModalProps) {
  // Main states
  const [officialAccounts, setOfficialAccounts] = useState<OfficialAccount[]>(initialOfficialAccounts);
  const [selectedOfficialAccountIds, setSelectedOfficialAccountIds] = useState<string[]>(initialSelectedAccountIds);
  const [oaGroups, setOaGroups] = useState<string[]>(initialOaGroups);

  // Filter/Listing states
  const [selectedGroupFilter, setSelectedGroupFilter] = useState<string>('all');
  const [showGroupManager, setShowGroupManager] = useState<boolean>(false);
  const [newGroupNameInput, setNewGroupNameInput] = useState<string>('');

  // Editing state
  const [oaEditingId, setOaEditingId] = useState<string | null>(null);
  const [activeTab, setActiveTab] = useState<'basic' | 'developer' | 'server'>('basic');
  
  // Applet fields
  const [oaName, setOaName] = useState('');
  const [oaType, setOaType] = useState<'subscription' | 'service'>('subscription');
  const [oaAvatar, setOaAvatar] = useState('🤖');
  const [oaAppId, setOaAppId] = useState('');
  const [oaAppSecret, setOaAppSecret] = useState('');
  const [oaServerUrl, setOaServerUrl] = useState('');
  const [oaToken, setOaToken] = useState('');
  const [oaEncodingAesKey, setOaEncodingAesKey] = useState('');
  const [oaEncryptMode, setOaEncryptMode] = useState<'plain' | 'compatible' | 'safe'>('safe');
  const [oaDomainVerifyFileName, setOaDomainVerifyFileName] = useState('');
  const [oaDomainVerifyFileContent, setOaDomainVerifyFileContent] = useState('');
  const [oaGroup, setOaGroup] = useState<string>('科技数码');

  const domainVerifyInputRef = useRef<HTMLInputElement>(null);

  if (!isOpen) return null;

  const openEditor = (oa?: OfficialAccount) => {
    setActiveTab('basic');
    if (oa) {
      setOaEditingId(oa.id);
      setOaName(oa.name);
      setOaType(oa.type);
      setOaAvatar(oa.avatar);
      setOaAppId(oa.appId);
      setOaAppSecret(oa.appSecret || '');
      setOaServerUrl(oa.serverUrl || '');
      setOaToken(oa.token || '');
      setOaEncodingAesKey(oa.encodingAesKey || '');
      setOaEncryptMode(oa.encryptMode || 'safe');
      setOaDomainVerifyFileName(oa.domainVerifyFileName || '');
      setOaDomainVerifyFileContent(oa.domainVerifyFileContent || '');
      setOaGroup(oa.group || '未分组');
    } else {
      setOaEditingId('new');
      setOaName('');
      setOaType('subscription');
      setOaAvatar('🤖');
      setOaAppId('');
      setOaAppSecret('');
      setOaServerUrl('');
      setOaToken('');
      setOaEncodingAesKey('');
      setOaEncryptMode('safe');
      setOaDomainVerifyFileName('');
      setOaDomainVerifyFileContent('');
      setOaGroup(oaGroups.length > 0 ? oaGroups[0] : '科技数码');
    }
  };

  const closeEditor = () => {
    setOaEditingId(null);
  };

  const handleSaveOA = () => {
    if (!oaName.trim()) {
      toast.error('请填写完整公众号名称');
      return;
    }
    
    const buildOa = (id: string): OfficialAccount => ({
      id,
      name: oaName.trim(),
      type: oaType,
      avatar: oaAvatar,
      appId: oaAppId.trim(),
      appSecret: oaAppSecret.trim(),
      serverUrl: oaServerUrl.trim(),
      token: oaToken.trim(),
      encodingAesKey: oaEncodingAesKey.trim(),
      encryptMode: oaEncryptMode,
      domainVerifyFileName: oaDomainVerifyFileName.trim(),
      domainVerifyFileContent: oaDomainVerifyFileContent.trim(),
      group: oaGroup
    });
    
    let newAccounts;
    if (oaEditingId === 'new') {
      const newId = \`oa-\${Date.now()}\`;
      newAccounts = [...officialAccounts, buildOa(newId)];
      setSelectedOfficialAccountIds([...selectedOfficialAccountIds, newId]);
    } else if (oaEditingId) {
      newAccounts = officialAccounts.map(app => app.id === oaEditingId ? buildOa(oaEditingId) : app);
    } else {
      return;
    }

    setOfficialAccounts(newAccounts);
    closeEditor();
    toast.success('保存成功');
  };

  const handleDeleteOA = (id: string) => {
    if (officialAccounts.length <= 1) {
       toast.error('至少需要保留一个公众号配置');
       return;
    }
    const newAccounts = officialAccounts.filter(app => app.id !== id);
    setOfficialAccounts(newAccounts);
    setSelectedOfficialAccountIds(selectedOfficialAccountIds.filter(x => x !== id));
    if (oaEditingId === id) {
      closeEditor();
    }
  };

  const handleGroupDelete = (grp: string) => {
    if (confirm(\`确认删除「\${grp}」分组吗？该分组下的应用将自动归类到「未分组」。\`)) {
      const newGroups = oaGroups.filter(g => g !== grp);
      const newAccounts = officialAccounts.map(app => app.group === grp ? { ...app, group: '未分组' } : app);
      setOaGroups(newGroups);
      setOfficialAccounts(newAccounts);
      if (selectedGroupFilter === grp) {
        setSelectedGroupFilter('all');
      }
    }
  };

  const handleGroupAdd = () => {
    if (newGroupNameInput.trim()) {
      const trimmedName = newGroupNameInput.trim();
      if (oaGroups.includes(trimmedName)) {
        toast.error('该分组已存在');
        return;
      }
      const newGroups = [...oaGroups, trimmedName];
      setOaGroups(newGroups);
      setNewGroupNameInput('');
    }
  };

  const handleConfirmAndClose = () => {
    onConfirm({
      officialAccounts,
      selectedOfficialAccountIds,
      oaGroups
    });
  };

  const handleFileUpload = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (file) {
      if (!file.name.endsWith('.txt')) {
        toast.error('只能上传 TXT 验证文件');
        return;
      }
      const reader = new FileReader();
      reader.onload = (evt) => {
        setOaDomainVerifyFileContent(evt.target?.result as string);
        setOaDomainVerifyFileName(file.name);
        toast.success('文件上传成功');
      };
      reader.readAsText(file);
    }
  };

  const filteredList = officialAccounts.filter(app => selectedGroupFilter === 'all' || app.group === selectedGroupFilter);

  return (
    <div className="fixed inset-0 bg-zinc-900/40 backdrop-blur-sm z-[600] flex items-center justify-center p-4 md:p-6 animate-in fade-in duration-200">
      <div className={\`bg-white dark:bg-[#0c0c0e] border border-zinc-200 dark:border-zinc-800 rounded-2xl w-full max-w-5xl h-full max-h-[800px] shadow-2xl flex flex-col overflow-hidden transition-all duration-300 \${oaEditingId ? 'scale-[0.98] opacity-60 pointer-events-none' : 'scale-100 opacity-100'}\`}>
        {/* Header */}
        <div className="px-6 py-4 border-b border-zinc-100 dark:border-zinc-800 flex items-center justify-between shrink-0 bg-neutral-50/50 dark:bg-zinc-900/50">
          <div className="flex items-center gap-3">
            <div className="w-10 h-10 rounded-xl bg-emerald-50 dark:bg-emerald-900/20 flex items-center justify-center text-emerald-600 dark:text-emerald-400">
              <Settings2 size={22} />
            </div>
            <div>
              <h2 className="text-lg font-bold text-zinc-900 dark:text-zinc-100">公众号配置管理</h2>
              <p className="text-xs text-zinc-500 dark:text-zinc-400 font-medium">支持多选发送、按粉丝或业务分组智能化管理</p>
            </div>
          </div>
          <button 
            type="button"
            onClick={onClose} 
            className="w-8 h-8 flex items-center justify-center hover:bg-zinc-200 dark:hover:bg-zinc-800 rounded-lg text-zinc-500 cursor-pointer transition-colors"
          >
            <X size={18} />
          </button>
        </div>

        {/* Content Body */}
        <div className="flex flex-1 overflow-hidden relative bg-white dark:bg-[#0c0c0e]">
          {/* Left Sidebar: Groups */}
          <div className="w-64 border-r border-zinc-100 dark:border-zinc-800 bg-neutral-50/30 dark:bg-zinc-900/20 flex flex-col shrink-0">
            <div className="p-4 flex items-center justify-between">
              <span className="text-xs font-bold text-zinc-500 uppercase tracking-wider">业务分组</span>
              <button 
                type="button"
                onClick={() => setShowGroupManager(!showGroupManager)}
                className="w-6 h-6 flex items-center justify-center hover:bg-emerald-100 dark:hover:bg-emerald-900/30 rounded text-emerald-600 dark:text-emerald-400 transition-colors"
                title="管理自定义分组"
              >
                <Plus size={14} />
              </button>
            </div>

            {showGroupManager && (
              <div className="px-4 pb-4 animate-in slide-in-from-top-2">
                <div className="flex bg-white dark:bg-zinc-900 rounded-lg overflow-hidden border border-zinc-200 dark:border-zinc-800 shadow-sm focus-within:border-emerald-500 focus-within:ring-1 focus-within:ring-emerald-500 transition-all">
                  <input 
                    type="text" 
                    value={newGroupNameInput}
                    onChange={(e) => setNewGroupNameInput(e.target.value)}
                    placeholder="新增分组名"
                    className="w-full bg-transparent border-none text-xs px-3 py-2 outline-none dark:text-zinc-200"
                    onKeyDown={(e) => e.key === 'Enter' && handleGroupAdd()}
                  />
                  <button 
                    onClick={handleGroupAdd}
                    className="px-3 text-emerald-600 hover:bg-emerald-50 dark:hover:bg-emerald-900/30 text-xs font-medium"
                  >
                    添加
                  </button>
                </div>
              </div>
            )}

            <div className="flex-1 overflow-y-auto px-3 pb-4 space-y-1">
              <button
                onClick={() => setSelectedGroupFilter('all')}
                className={\`w-full flex items-center justify-between px-3 py-2.5 rounded-xl text-sm transition-all \${
                  selectedGroupFilter === 'all' 
                    ? 'bg-emerald-50 dark:bg-emerald-900/20 text-emerald-700 dark:text-emerald-400 font-bold' 
                    : 'hover:bg-zinc-100 dark:hover:bg-zinc-800 text-zinc-600 dark:text-zinc-400 font-medium'
                }\`}
              >
                <div className="flex items-center gap-2.5">
                  <Folder size={16} className={selectedGroupFilter === 'all' ? 'text-emerald-600' : 'text-zinc-400'} />
                  <span>全部账号</span>
                </div>
                <span className="text-[10px] bg-zinc-200 dark:bg-zinc-800 px-1.5 py-0.5 rounded-full">{officialAccounts.length}</span>
              </button>
              
              {oaGroups.map(g => {
                const count = officialAccounts.filter(a => a.group === g).length;
                const isSelected = selectedGroupFilter === g;
                return (
                  <div key={g} className="group flex items-center relative">
                    <button
                      onClick={() => setSelectedGroupFilter(g)}
                      className={\`w-full flex items-center justify-between px-3 py-2.5 rounded-xl text-sm transition-all \${
                        isSelected 
                          ? 'bg-emerald-50 dark:bg-emerald-900/20 text-emerald-700 dark:text-emerald-400 font-bold' 
                          : 'hover:bg-zinc-100 dark:hover:bg-zinc-800 text-zinc-600 dark:text-zinc-400 font-medium'
                      }\`}
                    >
                      <div className="flex items-center gap-2.5">
                         <Tags size={16} className={isSelected ? 'text-emerald-600' : 'text-zinc-400'} />
                        <span className="truncate max-w-[100px]">{g}</span>
                      </div>
                      <span className="text-[10px] bg-zinc-200 dark:bg-zinc-800 px-1.5 py-0.5 rounded-full">{count}</span>
                    </button>
                    {showGroupManager && (
                      <button 
                        onClick={() => handleGroupDelete(g)}
                        className="absolute right-2 p-1.5 text-zinc-400 hover:text-red-500 hover:bg-red-50 dark:hover:bg-red-900/20 rounded-md opacity-0 group-hover:opacity-100 transition-all"
                      >
                        <Trash2 size={14} />
                      </button>
                    )}
                  </div>
                );
              })}
            </div>
          </div>

          <div className="flex-1 flex flex-col relative overflow-hidden">
            <div className="px-6 py-4 border-b border-zinc-100 dark:border-zinc-800 flex items-center justify-between bg-white dark:bg-[#0c0c0e]">
              <h3 className="text-base font-bold text-zinc-800 dark:text-zinc-100 flex items-center gap-2">
                {selectedGroupFilter === 'all' ? '待发账号列表' : selectedGroupFilter}
                <span className="text-xs font-medium text-zinc-400 bg-zinc-100 dark:bg-zinc-800 px-2 py-0.5 rounded-full">{filteredList.length}</span>
              </h3>
              <button 
                onClick={() => openEditor()}
                className="flex items-center gap-1.5 px-4 py-2 bg-emerald-600 hover:bg-emerald-700 active:bg-emerald-800 text-white text-sm font-bold rounded-xl transition-all shadow-sm shadow-emerald-600/20"
              >
                <Plus size={16} />
                新增关联配置
              </button>
            </div>

            <div className="flex-1 overflow-y-auto p-6 bg-zinc-50/50 dark:bg-[#0c0c0e]/50">
              <div className="grid grid-cols-1 gap-3">
                {filteredList.length === 0 ? (
                  <div className="col-span-full flex flex-col items-center justify-center py-20 border-2 border-dashed border-zinc-200 dark:border-zinc-800 rounded-2xl">
                    <Settings2 size={40} className="text-zinc-300 dark:text-zinc-700 mb-4" />
                    <span className="text-sm font-medium text-zinc-500">该分组下暂无公众号配置。</span>
                  </div>
                ) : (
                  filteredList.map((app) => {
                    const isSelected = selectedOfficialAccountIds.includes(app.id);
                    return (
                      <div 
                        key={app.id} 
                        className={\`group flex items-center bg-white dark:bg-[#131316] border \${isSelected ? 'border-emerald-500 ring-1 ring-emerald-500/50 bg-emerald-50/10' : 'border-zinc-200 dark:border-zinc-800 hover:border-emerald-300 dark:hover:border-emerald-700/50'} rounded-2xl p-4 transition-all cursor-pointer shadow-sm relative\`}
                        onClick={() => {
                          if (isSelected) {
                            setSelectedOfficialAccountIds(selectedOfficialAccountIds.filter(id => id !== app.id));
                          } else {
                            setSelectedOfficialAccountIds([...selectedOfficialAccountIds, app.id]);
                          }
                        }}
                      >
                         <div className={\`w-5 h-5 rounded-md border-2 mr-4 flex items-center justify-center transition-all shrink-0 \${isSelected ? 'bg-emerald-500 border-emerald-500 text-white' : 'border-zinc-300 dark:border-zinc-700'}\`}>
                            {isSelected && <Check size={14} className="stroke-[3]" />}
                         </div>

                         <div className="w-12 h-12 rounded-xl bg-zinc-100 dark:bg-zinc-800 flex items-center justify-center text-2xl shadow-inner shrink-0 mr-4 object-cover overflow-hidden">
                           {app.avatar.length <= 2 ? app.avatar : <img src={app.avatar} alt="avatar" className="w-full h-full object-cover" />}
                         </div>

                         <div className="flex-1 min-w-0 pr-12">
                           <div className="flex items-center gap-2 mb-1">
                             <h4 className="text-base font-bold text-zinc-900 dark:text-zinc-100 truncate">{app.name}</h4>
                             <span className={\`text-[10px] px-2 py-0.5 rounded-full font-bold uppercase tracking-wider shrink-0 \${app.type === 'service' ? 'bg-blue-100 text-blue-700' : 'bg-orange-100 text-orange-700'}\`}>
                               {app.type === 'service' ? '服务号' : '订阅号'}
                             </span>
                           </div>
                           <div className="text-xs text-zinc-500 dark:text-zinc-400 truncate">
                             AppID: {app.appId || '未配置'}
                           </div>
                         </div>

                         <div className="absolute right-4 flex items-center gap-2 opacity-0 group-hover:opacity-100 transition-opacity">
                            <button 
                              onClick={(e) => { e.stopPropagation(); openEditor(app); }}
                              className="p-2 text-zinc-400 hover:text-emerald-600 hover:bg-emerald-50 dark:hover:bg-emerald-900/30 rounded-xl transition-colors"
                            >
                              <Edit3 size={16} />
                            </button>
                         </div>
                      </div>
                    );
                  })
                )}
              </div>
            </div>
            
            {/* Right main panel footer */}
            <div className="p-4 bg-white dark:bg-zinc-950 border-t border-zinc-100 dark:border-zinc-800 flex justify-between items-center shrink-0 shadow-[0_-4px_15px_rgba(0,0,0,0.02)] z-20 relative">
               <div className="flex-1 flex items-center pl-2">
                 <span className="text-sm text-zinc-600 dark:text-zinc-400">已选择 <span className="font-bold text-emerald-600 text-lg px-1">{selectedOfficialAccountIds.length}</span> 个渠道进行群发</span>
               </div>
               <div className="flex items-center gap-3">
                 <button 
                   onClick={onClose} 
                   type="button"
                   className="px-5 py-2.5 text-sm font-bold bg-white dark:bg-zinc-800 border border-zinc-200 dark:border-zinc-700 hover:bg-zinc-50 dark:hover:bg-zinc-700 text-zinc-700 dark:text-zinc-300 rounded-xl transition-all shadow-sm"
                 >
                   取消
                 </button>
                 <button 
                   onClick={handleConfirmAndClose} 
                   type="button"
                   disabled={selectedOfficialAccountIds.length === 0}
                   className="px-6 py-2.5 text-sm font-bold bg-emerald-600 hover:bg-emerald-700 disabled:opacity-40 disabled:hover:bg-emerald-600 text-white rounded-xl shadow-md transition-all flex items-center gap-2 hover:-translate-y-0.5 active:translate-y-0 active:scale-95"
                 >
                   确认配置
                   <Check size={16} strokeWidth={3} />
                 </button>
               </div>
            </div>

          </div>
        </div>
      </div>

      {/* Editing Drawer (Slide-over from RIGHT side of screen) */}
      <div 
        className={\`fixed inset-0 z-[610] bg-zinc-900/40 backdrop-blur-sm transition-opacity duration-300 \${oaEditingId ? 'opacity-100 pointer-events-auto' : 'opacity-0 pointer-events-none'}\`}
        onClick={closeEditor}
      />

      <div 
        className={\`fixed top-0 bottom-0 right-0 w-full max-w-2xl bg-white dark:bg-[#0c0c0e] shadow-2xl border-l border-zinc-200 dark:border-zinc-800 z-[620] flex flex-col transition-transform duration-300 ease-[cubic-bezier(0.16,1,0.3,1)] \${
          oaEditingId ? 'translate-x-0' : 'translate-x-full'
        }\`}
      >
        <div className="px-6 py-5 border-b border-zinc-100 dark:border-zinc-800 flex items-center justify-between shrink-0 bg-neutral-50/50 dark:bg-zinc-900/50">
          <div className="flex items-center gap-3">
             <div className="w-10 h-10 rounded-xl bg-emerald-600 flex items-center justify-center text-white shadow-md shadow-emerald-600/20">
              <Settings2 size={20} />
            </div>
            <div>
              <h2 className="text-lg font-bold text-zinc-900 dark:text-zinc-100">
                {oaEditingId === 'new' ? '新建公众号配置' : \`配置调试: \${oaName}\`}
              </h2>
              <p className="text-xs text-zinc-500 dark:text-zinc-400 font-medium">完善商业化连接配置</p>
            </div>
          </div>
          <div className="flex items-center gap-2">
            <button 
              onClick={closeEditor}
              className="px-4 py-2 text-sm font-bold text-zinc-500 hover:text-zinc-800 dark:hover:text-zinc-200 transition-colors"
            >
              取消
            </button>
            <button 
              onClick={handleSaveOA}
              disabled={!oaName.trim()}
              className="px-6 py-2 text-sm font-bold bg-zinc-900 hover:bg-zinc-800 dark:bg-zinc-100 dark:hover:bg-white text-white dark:text-zinc-900 disabled:opacity-40 rounded-xl shadow-md transition-all active:scale-95 flex items-center gap-2"
            >
              保存配置
            </button>
          </div>
        </div>

        {/* Edit Tabs */}
        <div className="flex px-6 gap-6 border-b border-zinc-100 dark:border-zinc-800 bg-white dark:bg-[#0c0c0e] shrink-0 pt-2">
          {[
            { id: 'basic', label: '基础资料', icon: <Edit3 size={14}/> },
            { id: 'developer', label: '开发者凭据', icon: <Key size={14}/> },
            { id: 'server', label: '消息推送服务器', icon: <Server size={14}/> }
          ].map(tab => (
            <button
              key={tab.id}
              onClick={() => setActiveTab(tab.id as 'basic' | 'developer' | 'server')}
              className={\`flex items-center gap-2 px-1 py-3 text-sm font-bold border-b-2 transition-all \${
                activeTab === tab.id 
                  ? 'border-emerald-600 text-emerald-600' 
                  : 'border-transparent text-zinc-500 hover:text-zinc-800 dark:hover:text-zinc-300'
              }\`}
            >
              {tab.icon} {tab.label}
            </button>
          ))}
        </div>

        <div className="flex-1 overflow-y-auto p-6 md:p-8 bg-zinc-50/30 dark:bg-[#0c0c0e]/20">
          <div className="max-w-xl mx-auto space-y-8 pb-10">

            {activeTab === 'basic' && (
              <div className="space-y-6 animate-in fade-in duration-300 slide-in-from-right-4">
                 
                 <div>
                    <label className="block text-sm font-bold text-zinc-800 dark:text-zinc-200 mb-2">公众号名称 <span className="text-red-500">*</span></label>
                    <input 
                      type="text" value={oaName} onChange={(e) => setOaName(e.target.value)}
                      placeholder="微信公众平台名称"
                      className="w-full bg-white dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-800 rounded-xl px-4 py-3 text-sm font-medium text-zinc-900 dark:text-zinc-100 focus:outline-none focus:border-emerald-500 focus:ring-2 focus:ring-emerald-500/20 transition-all shadow-sm"
                    />
                 </div>

                 <div className="flex gap-6">
                    <div className="flex-1">
                      <label className="block text-sm font-bold text-zinc-800 dark:text-zinc-200 mb-3">账号类型</label>
                      <div className="flex bg-zinc-100 dark:bg-zinc-800/50 p-1 rounded-xl">
                        <button
                          type="button"
                          onClick={() => setOaType('subscription')}
                          className={\`flex-1 py-2 text-sm font-bold rounded-lg transition-all \${oaType === 'subscription' ? 'bg-white dark:bg-zinc-700 shadow-sm text-zinc-900 dark:text-zinc-100 border border-zinc-200 dark:border-zinc-600' : 'text-zinc-500 hover:text-zinc-700'}\`}
                        >
                          订阅号
                        </button>
                        <button
                          type="button"
                          onClick={() => setOaType('service')}
                          className={\`flex-1 py-2 text-sm font-bold rounded-lg transition-all \${oaType === 'service' ? 'bg-white dark:bg-zinc-700 shadow-sm text-zinc-900 dark:text-zinc-100 border border-zinc-200 dark:border-zinc-600' : 'text-zinc-500 hover:text-zinc-700'}\`}
                        >
                          服务号
                        </button>
                      </div>
                    </div>

                    <div className="w-1/2">
                      <label className="block text-sm font-bold text-zinc-800 dark:text-zinc-200 mb-2">默认分类</label>
                      <select 
                        value={oaGroup} 
                        onChange={(e) => setOaGroup(e.target.value)}
                        className="w-full bg-white dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-800 rounded-xl px-4 py-2.5 text-sm font-medium text-zinc-900 dark:text-zinc-100 focus:outline-none focus:border-emerald-500 transition-all shadow-sm appearance-none h-[42px]"
                      >
                         {oaGroups.length === 0 && <option value="未分组">未分组</option>}
                         {oaGroups.map(g => <option key={g} value={g}>{g}</option>)}
                      </select>
                    </div>
                 </div>

                 <div>
                    <label className="block text-sm font-bold text-zinc-800 dark:text-zinc-200 mb-2">应用图标 (Avatar)</label>
                    <div className="flex gap-2 p-2 bg-zinc-100 dark:bg-zinc-900 rounded-xl border border-zinc-200 dark:border-zinc-800 overflow-x-auto hide-scrollbar">
                      {['🤖', '💬', '📢', '📰', '🌐', '💡', '🔥', '✨', '⚡'].map((emoji) => (
                        <button
                          key={emoji}
                          onClick={() => setOaAvatar(emoji)}
                          className={\`w-10 h-10 shrink-0 flex items-center justify-center text-xl rounded-lg border-2 transition-all \${
                            oaAvatar === emoji ? 'border-emerald-500 bg-white shadow-sm scale-110' : 'border-transparent hover:bg-zinc-200 dark:hover:bg-zinc-800'
                          }\`}
                        >
                          {emoji}
                        </button>
                      ))}
                    </div>
                 </div>
              </div>
            )}

            {activeTab === 'developer' && (
              <div className="space-y-6 animate-in fade-in duration-300 slide-in-from-right-4">
                 <div className="bg-emerald-50 dark:bg-emerald-900/10 border border-emerald-100 dark:border-emerald-900/30 p-4 rounded-xl flex gap-3 text-emerald-800 dark:text-emerald-300 text-sm">
                    <Key className="shrink-0 mt-0.5 text-emerald-500" size={16} />
                    <div>请在微信公众平台「配置与开发 - 基本配置」中获取开发者ID (AppID) 和开发者密码 (AppSecret)。</div>
                 </div>

                 <div>
                    <label className="block text-sm font-bold text-zinc-800 dark:text-zinc-200 mb-2">AppID (公众号ID)</label>
                    <input 
                      type="text" value={oaAppId} onChange={(e) => setOaAppId(e.target.value)}
                      placeholder="wx1234567890abcdef"
                      className="w-full bg-white dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-800 rounded-xl px-4 py-3 text-sm font-mono tracking-wide text-zinc-900 dark:text-zinc-100 focus:outline-none focus:border-emerald-500 transition-all shadow-sm"
                    />
                 </div>

                 <div>
                    <label className="block text-sm font-bold text-zinc-800 dark:text-zinc-200 mb-2">AppSecret (开发者密钥)</label>
                    <input 
                      type="password" value={oaAppSecret} onChange={(e) => setOaAppSecret(e.target.value)}
                      placeholder="********************************"
                      className="w-full bg-white dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-800 rounded-xl px-4 py-3 text-sm font-mono tracking-wide text-zinc-900 dark:text-zinc-100 focus:outline-none focus:border-emerald-500 transition-all shadow-sm"
                    />
                 </div>

                 <div className="pt-4 border-t border-zinc-200 dark:border-zinc-800">
                    <h4 className="text-sm font-bold text-zinc-800 dark:text-zinc-200 mb-4 flex items-center gap-2">
                      <Globe size={16} /> 网页授权域名验证 (业务域名/JS接口安全域名)
                    </h4>
                    
                    <div className="border border-zinc-200 dark:border-zinc-800 bg-white dark:bg-zinc-900 p-5 rounded-xl space-y-4">
                      {oaDomainVerifyFileName && oaDomainVerifyFileContent ? (
                        <div className="flex items-center justify-between bg-emerald-50 dark:bg-emerald-900/10 p-3 rounded-lg border border-emerald-100 dark:border-emerald-800">
                          <div className="flex items-center gap-2 text-sm text-emerald-700 dark:text-emerald-400 font-bold">
                             <Check size={16}/>
                             已上传验证文件: <span className="font-mono">{oaDomainVerifyFileName}</span>
                          </div>
                          <button 
                            onClick={() => { setOaDomainVerifyFileName(''); setOaDomainVerifyFileContent(''); }}
                            className="text-xs text-red-500 hover:text-red-700 font-bold px-3 py-1.5 hover:bg-red-50 dark:hover:bg-red-900/30 rounded-md transition-colors"
                          >
                            移除
                          </button>
                        </div>
                      ) : (
                        <div className="flex items-center gap-3">
                          <button 
                            type="button"
                            onClick={() => domainVerifyInputRef.current?.click()}
                            className="w-full border-2 border-dashed border-zinc-300 dark:border-zinc-700 hover:border-emerald-500 text-zinc-500 hover:text-emerald-600 dark:text-zinc-400 py-6 rounded-xl font-medium text-sm transition-colors flex flex-col items-center gap-2"
                          >
                            <Upload size={24} className="mb-2" />
                            <span>点击上传域名验证文件 (.txt)</span>
                            <span className="text-xs bg-zinc-100 dark:bg-zinc-800 text-zinc-500 px-3 py-1 rounded-full">文件内容将被注入系统根目录</span>
                          </button>
                          <input 
                            type="file" 
                            accept=".txt"
                            ref={domainVerifyInputRef}
                            className="hidden"
                            onChange={handleFileUpload}
                          />
                        </div>
                      )}
                    </div>
                 </div>
              </div>
            )}

            {activeTab === 'server' && (
              <div className="space-y-6 animate-in fade-in duration-300 slide-in-from-right-4">
                 
                 <div className="space-y-5">
                    <h4 className="text-base font-bold text-zinc-900 dark:text-zinc-100 flex items-center gap-2">
                       <Server size={16} className="text-emerald-500" />
                       服务器配置信息
                    </h4>

                    <div>
                      <label className="block text-sm font-bold text-zinc-800 dark:text-zinc-200 mb-2">URL (服务器地址)</label>
                      <input 
                        type="text" value={oaServerUrl} onChange={(e) => setOaServerUrl(e.target.value)}
                        placeholder="https://your-domain.com/api/wechat/serve"
                        className="w-full bg-white dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-800 rounded-xl px-4 py-3 text-sm font-mono text-zinc-900 dark:text-zinc-100 focus:border-emerald-500 transition-all shadow-sm"
                      />
                    </div>
                    <div>
                      <label className="block text-sm font-bold text-zinc-800 dark:text-zinc-200 mb-2">Token (令牌)</label>
                      <input 
                        type="text" value={oaToken} onChange={(e) => setOaToken(e.target.value)}
                        placeholder="自定义任意英文或数字"
                        className="w-full bg-white dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-800 rounded-xl px-4 py-3 text-sm font-mono text-zinc-900 dark:text-zinc-100 focus:border-emerald-500 transition-all shadow-sm"
                      />
                    </div>
                    <div>
                      <label className="block text-sm font-bold text-zinc-800 dark:text-zinc-200 mb-2">EncodingAESKey (消息加解密密钥)</label>
                      <input 
                        type="text" value={oaEncodingAesKey} onChange={(e) => setOaEncodingAesKey(e.target.value)}
                        placeholder="43位长度的随机字符"
                        className="w-full bg-white dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-800 rounded-xl px-4 py-3 text-sm font-mono text-zinc-900 dark:text-zinc-100 focus:border-emerald-500 transition-all shadow-sm"
                      />
                    </div>

                    <div>
                      <label className="block text-sm font-bold text-zinc-800 dark:text-zinc-200 mb-3 flex items-center gap-1">
                        <Shield size={14} className="text-zinc-400"/> 消息加解密方式
                      </label>
                      <div className="grid grid-cols-3 gap-3">
                        <label className={\`flex flex-col items-center justify-center p-3 rounded-xl border-2 cursor-pointer transition-all \${oaEncryptMode === 'plain' ? 'border-emerald-500 bg-emerald-50/10' : 'border-zinc-200 dark:border-zinc-800 hover:border-emerald-200'}\`}>
                          <input 
                            type="radio" name="encryptMode" value="plain" className="sr-only"
                            checked={oaEncryptMode === 'plain'} onChange={() => setOaEncryptMode('plain')}
                          />
                          <span className="text-sm font-bold mb-1">明文模式</span>
                          <span className="text-[10px] text-zinc-500 text-center">不加密, 传输最快</span>
                        </label>
                        <label className={\`flex flex-col items-center justify-center p-3 rounded-xl border-2 cursor-pointer transition-all \${oaEncryptMode === 'compatible' ? 'border-emerald-500 bg-emerald-50/10' : 'border-zinc-200 dark:border-zinc-800 hover:border-emerald-200'}\`}>
                          <input 
                            type="radio" name="encryptMode" value="compatible" className="sr-only"
                            checked={oaEncryptMode === 'compatible'} onChange={() => setOaEncryptMode('compatible')}
                          />
                          <span className="text-sm font-bold mb-1">兼容模式</span>
                          <span className="text-[10px] text-zinc-500 text-center">明文/杂文共存</span>
                        </label>
                        <label className={\`flex flex-col items-center justify-center p-3 rounded-xl border-2 cursor-pointer transition-all \${oaEncryptMode === 'safe' ? 'border-emerald-500 bg-emerald-50/10' : 'border-zinc-200 dark:border-zinc-800 hover:border-emerald-200'}\`}>
                          <input 
                            type="radio" name="encryptMode" value="safe" className="sr-only"
                            checked={oaEncryptMode === 'safe'} onChange={() => setOaEncryptMode('safe')}
                          />
                          <span className="text-sm font-bold mb-1">安全模式</span>
                          <span className="text-[10px] text-emerald-600 text-center font-medium">推荐使用</span>
                        </label>
                      </div>
                    </div>
                 </div>
              </div>
            )}

            {oaEditingId && oaEditingId !== 'new' && (
              <div className="mt-12 pt-6 border-t border-zinc-200 dark:border-zinc-800">
                <button
                  type="button"
                  onClick={() => {
                    if (confirm('确认删除该配置信息吗？此操作不可恢复。')) {
                      handleDeleteOA(oaEditingId);
                    }
                  }}
                  className="w-full py-3.5 flex items-center justify-center gap-2 text-red-500 hover:bg-red-50 dark:hover:bg-red-950/30 rounded-xl border border-red-200 dark:border-red-900/50 font-bold transition-all text-sm"
                >
                  <Trash2 size={16} /> 删除此应用配置
                </button>
              </div>
            )}

          </div>
        </div>
      </div>
    </div>
  );
}
`

fs.writeFileSync(filepath, content, 'utf8');
console.log("OA rewritten")

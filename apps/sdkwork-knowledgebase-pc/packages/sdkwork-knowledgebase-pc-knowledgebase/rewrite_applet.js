const fs = require('fs');

const filepath = './packages/sdkwork-knowledgebase-pc-knowledgebase/src/components/AppletManagerModal.tsx';

const content = `import React, { useState } from 'react';
import { 
  X, Folder, Tags, Plus, Trash2, Smartphone, LayoutGrid, Check, Server, Link as LinkIcon, Edit3, MessageSquare
} from 'lucide-react';
import { WechatAppletConfig } from '../services/wechat';

import { toast } from './ui/toast-manager';

interface AppletManagerModalProps {
  isOpen: boolean;
  onClose: () => void;
  onSelect: (applet: WechatAppletConfig) => void;
  initialApplets: WechatAppletConfig[];
  initialGroups: string[];
  onSaveApplets: (applets: WechatAppletConfig[], groups: string[]) => void;
}

export function AppletManagerModal({
  isOpen,
  onClose,
  onSelect,
  initialApplets,
  initialGroups,
  onSaveApplets
}: AppletManagerModalProps) {
  // Main states
  const [applets, setApplets] = useState<WechatAppletConfig[]>(initialApplets);
  const [groups, setGroups] = useState<string[]>(initialGroups);

  // Filter/Listing states
  const [selectedGroupFilter, setSelectedGroupFilter] = useState<string>('all');
  const [showGroupManager, setShowGroupManager] = useState<boolean>(false);
  const [newGroupNameInput, setNewGroupNameInput] = useState<string>('');

  // Editing state
  const [editingId, setEditingId] = useState<string | null>(null);
  const [activeTab, setActiveTab] = useState<'basic' | 'developer' | 'server'>('basic');
  
  // Applet fields
  const [appName, setAppName] = useState('');
  const [appAvatar, setAppAvatar] = useState('📱');
  const [appId, setAppId] = useState('');
  const [appSecret, setAppSecret] = useState('');
  const [appPath, setAppPath] = useState('');
  const [appGroup, setAppGroup] = useState<string>('工具');
  const [appDescription, setAppDescription] = useState('');
  
  const [requestDomain, setRequestDomain] = useState('');
  const [socketDomain, setSocketDomain] = useState('');
  const [uploadDomain, setUploadDomain] = useState('');
  const [downloadDomain, setDownloadDomain] = useState('');
  
  const [msgToken, setMsgToken] = useState('');
  const [msgEncodingAESKey, setMsgEncodingAESKey] = useState('');
  const [msgDataFormat, setMsgDataFormat] = useState<'json' | 'xml'>('json');

  if (!isOpen) return null;

  const openEditor = (applet?: WechatAppletConfig) => {
    setActiveTab('basic');
    if (applet) {
      setEditingId(applet.id);
      setAppName(applet.name);
      setAppAvatar(applet.avatar);
      setAppId(applet.appId);
      setAppSecret(applet.appSecret || '');
      setAppPath(applet.path);
      setAppGroup(applet.group || '未分组');
      setAppDescription(applet.description || '');
      setRequestDomain(applet.requestDomain || '');
      setSocketDomain(applet.socketDomain || '');
      setUploadDomain(applet.uploadDomain || '');
      setDownloadDomain(applet.downloadDomain || '');
      setMsgToken(applet.msgToken || '');
      setMsgEncodingAESKey(applet.msgEncodingAESKey || '');
      setMsgDataFormat(applet.msgDataFormat || 'json');
    } else {
      setEditingId('new');
      setAppName('');
      setAppAvatar('📱');
      setAppId('');
      setAppSecret('');
      setAppPath('');
      setAppGroup(groups.length > 0 ? groups[0] : '未分组');
      setAppDescription('');
      setRequestDomain('');
      setSocketDomain('');
      setUploadDomain('');
      setDownloadDomain('');
      setMsgToken('');
      setMsgEncodingAESKey('');
      setMsgDataFormat('json');
    }
  };

  const closeEditor = () => {
    setEditingId(null);
  };

  const handleSaveApplet = () => {
    if (!appName.trim() || !appId.trim()) {
      toast.error('请填写完整的小程序名称和AppID');
      return;
    }
    
    const buildOa = (id: string): WechatAppletConfig => ({
      id,
      name: appName.trim(),
      avatar: appAvatar,
      appId: appId.trim(),
      appSecret: appSecret.trim(),
      path: appPath.trim(),
      group: appGroup,
      description: appDescription.trim(),
      requestDomain: requestDomain.trim(),
      socketDomain: socketDomain.trim(),
      uploadDomain: uploadDomain.trim(),
      downloadDomain: downloadDomain.trim(),
      msgToken: msgToken.trim(),
      msgEncodingAESKey: msgEncodingAESKey.trim(),
      msgDataFormat
    });
    
    let newApplets;
    if (editingId === 'new') {
      newApplets = [...applets, buildOa(\`applet-\${Date.now()}\`)];
    } else if (editingId) {
      newApplets = applets.map(app => app.id === editingId ? buildOa(editingId) : app);
    } else {
      return;
    }

    setApplets(newApplets);
    onSaveApplets(newApplets, groups);
    closeEditor();
    toast.success('保存成功');
  };

  const handleDeleteApplet = (id: string) => {
    const newApplets = applets.filter(app => app.id !== id);
    setApplets(newApplets);
    onSaveApplets(newApplets, groups);
    if (editingId === id) {
      closeEditor();
    }
  };

  const handleGroupDelete = (grp: string) => {
    if (confirm(\`确认删除「\${grp}」分组吗？该分组下的小程序将自动归类到「未分组」。\`)) {
      const newGroups = groups.filter(g => g !== grp);
      const newApplets = applets.map(app => app.group === grp ? { ...app, group: '未分组' } : app);
      setGroups(newGroups);
      setApplets(newApplets);
      onSaveApplets(newApplets, newGroups);
      if (selectedGroupFilter === grp) {
        setSelectedGroupFilter('all');
      }
    }
  };

  const handleGroupAdd = () => {
    if (newGroupNameInput.trim()) {
      const trimmedName = newGroupNameInput.trim();
      if (groups.includes(trimmedName)) {
        toast.error('该分组已存在');
        return;
      }
      const newGroups = [...groups, trimmedName];
      setGroups(newGroups);
      onSaveApplets(applets, newGroups);
      setNewGroupNameInput('');
    }
  };

  const filteredList = applets.filter(app => selectedGroupFilter === 'all' || app.group === selectedGroupFilter);

  return (
    <div className="fixed inset-0 bg-zinc-900/40 backdrop-blur-sm z-[600] flex items-center justify-center p-4 md:p-6 animate-in fade-in duration-200">
      <div className={\`bg-white dark:bg-[#0c0c0e] border border-zinc-200 dark:border-zinc-800 rounded-2xl w-full max-w-5xl h-full max-h-[800px] shadow-2xl flex flex-col overflow-hidden transition-all duration-300 \${editingId ? 'scale-[0.98] opacity-60 pointer-events-none' : 'scale-100 opacity-100'}\`}>
        {/* Header */}
        <div className="px-6 py-4 border-b border-zinc-100 dark:border-zinc-800 flex items-center justify-between shrink-0 bg-neutral-50/50 dark:bg-zinc-900/50">
          <div className="flex items-center gap-3">
            <div className="w-10 h-10 rounded-xl bg-blue-50 dark:bg-blue-900/20 flex items-center justify-center text-blue-600 dark:text-blue-400">
              <LayoutGrid size={22} />
            </div>
            <div>
              <h2 className="text-lg font-bold text-zinc-900 dark:text-zinc-100">小程序管理中心</h2>
              <p className="text-xs text-zinc-500 dark:text-zinc-400 font-medium">配置与选择分发平台小程序</p>
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
              <span className="text-xs font-bold text-zinc-500 uppercase tracking-wider">全部分组</span>
              <button 
                type="button"
                onClick={() => setShowGroupManager(!showGroupManager)}
                className="w-6 h-6 flex items-center justify-center hover:bg-blue-100 dark:hover:bg-blue-900/30 rounded text-blue-600 dark:text-blue-400 transition-colors"
              >
                <Plus size={14} />
              </button>
            </div>

            {showGroupManager && (
              <div className="px-4 pb-4 animate-in slide-in-from-top-2">
                <div className="flex bg-white dark:bg-zinc-900 rounded-lg overflow-hidden border border-zinc-200 dark:border-zinc-800 shadow-sm focus-within:border-blue-500 focus-within:ring-1 focus-within:ring-blue-500 transition-all">
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
                    className="px-3 text-blue-600 hover:bg-blue-50 dark:hover:bg-blue-900/30 text-xs font-medium"
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
                    ? 'bg-blue-50 dark:bg-blue-900/20 text-blue-700 dark:text-blue-400 font-bold' 
                    : 'hover:bg-zinc-100 dark:hover:bg-zinc-800 text-zinc-600 dark:text-zinc-400 font-medium'
                }\`}
              >
                <div className="flex items-center gap-2.5">
                  <LayoutGrid size={16} className={selectedGroupFilter === 'all' ? 'text-blue-600' : 'text-zinc-400'} />
                  <span>所有小程序</span>
                </div>
                <span className="text-[10px] bg-zinc-200 dark:bg-zinc-800 px-1.5 py-0.5 rounded-full">{applets.length}</span>
              </button>
              
              {groups.map(g => {
                const count = applets.filter(a => a.group === g).length;
                const isSelected = selectedGroupFilter === g;
                return (
                  <div key={g} className="group flex items-center relative">
                    <button
                      onClick={() => setSelectedGroupFilter(g)}
                      className={\`w-full flex items-center justify-between px-3 py-2.5 rounded-xl text-sm transition-all \${
                        isSelected 
                          ? 'bg-blue-50 dark:bg-blue-900/20 text-blue-700 dark:text-blue-400 font-bold' 
                          : 'hover:bg-zinc-100 dark:hover:bg-zinc-800 text-zinc-600 dark:text-zinc-400 font-medium'
                      }\`}
                    >
                      <div className="flex items-center gap-2.5">
                        <Folder size={16} className={isSelected ? 'text-blue-600' : 'text-zinc-400'} />
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
                {selectedGroupFilter === 'all' ? '所有数据' : selectedGroupFilter}
                <span className="text-xs font-medium text-zinc-400 bg-zinc-100 dark:bg-zinc-800 px-2 py-0.5 rounded-full">{filteredList.length}</span>
              </h3>
              <button 
                onClick={() => openEditor()}
                className="flex items-center gap-1.5 px-4 py-2 bg-blue-600 hover:bg-blue-700 active:bg-blue-800 text-white text-sm font-bold rounded-xl transition-all shadow-sm shadow-blue-600/20"
              >
                <Plus size={16} />
                新增关联配置
              </button>
            </div>

            <div className="flex-1 overflow-y-auto p-6 bg-zinc-50/50 dark:bg-[#0c0c0e]/50">
              <div className="grid grid-cols-1 xl:grid-cols-2 gap-4">
                {filteredList.length === 0 ? (
                  <div className="col-span-full flex flex-col items-center justify-center py-20 border-2 border-dashed border-zinc-200 dark:border-zinc-800 rounded-2xl">
                    <Smartphone size={40} className="text-zinc-300 dark:text-zinc-700 mb-4" />
                    <span className="text-sm font-medium text-zinc-500">当前分类暂无数据，请点击上方添加。</span>
                  </div>
                ) : (
                  filteredList.map((app) => (
                    <div 
                      key={app.id} 
                      className="group flex flex-col bg-white dark:bg-[#131316] border border-zinc-200 dark:border-zinc-800 rounded-2xl p-5 hover:border-blue-300 dark:hover:border-blue-700/50 hover:shadow-md transition-all cursor-pointer"
                      onClick={() => onSelect(app)}
                    >
                      <div className="flex items-start justify-between mb-4">
                        <div className="flex items-center gap-3">
                          <div className="w-12 h-12 rounded-xl bg-zinc-100 dark:bg-zinc-800 flex items-center justify-center text-2xl shadow-inner object-cover overflow-hidden">
                            {app.avatar.length <= 2 ? app.avatar : <img src={app.avatar} alt="avatar" className="w-full h-full object-cover" />}
                          </div>
                          <div>
                            <h4 className="text-base font-bold text-zinc-900 dark:text-zinc-100 group-hover:text-blue-600 dark:group-hover:text-blue-400 transition-colors">{app.name}</h4>
                            <div className="flex items-center gap-2 mt-1 shrink-0 flex-wrap">
                               <span className="text-xs px-2 py-0.5 bg-blue-50 dark:bg-blue-900/20 text-blue-600 dark:text-blue-400 rounded-md font-medium">{app.group || '未分组'}</span>
                               <span className="text-xs font-mono text-zinc-500 dark:text-zinc-400">{app.appId}</span>
                            </div>
                          </div>
                        </div>
                        <button 
                          onClick={(e) => { e.stopPropagation(); openEditor(app); }}
                          className="p-2 text-zinc-400 hover:text-blue-600 hover:bg-blue-50 dark:hover:bg-blue-900/30 rounded-xl transition-colors opacity-0 group-hover:opacity-100"
                        >
                          <Edit3 size={16} />
                        </button>
                      </div>
                      
                      <div className="text-xs text-zinc-500 dark:text-zinc-400 line-clamp-2 mt-auto pt-2 border-t border-zinc-100 dark:border-zinc-800">
                        {app.description || '暂无说明介绍...'}
                      </div>
                    </div>
                  ))
                )}
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* Editing Drawer (Slide-over from RIGHT side of screen) */}
      <div 
        className={\`fixed inset-0 z-[610] bg-zinc-900/40 backdrop-blur-sm transition-opacity duration-300 \${editingId ? 'opacity-100 pointer-events-auto' : 'opacity-0 pointer-events-none'}\`}
        onClick={closeEditor}
      />

      <div 
        className={\`fixed top-0 bottom-0 right-0 w-full max-w-2xl bg-white dark:bg-[#0c0c0e] shadow-2xl border-l border-zinc-200 dark:border-zinc-800 z-[620] flex flex-col transition-transform duration-300 ease-[cubic-bezier(0.16,1,0.3,1)] \${
          editingId ? 'translate-x-0' : 'translate-x-full'
        }\`}
      >
        <div className="px-6 py-5 border-b border-zinc-100 dark:border-zinc-800 flex items-center justify-between shrink-0 bg-neutral-50/50 dark:bg-zinc-900/50">
          <div className="flex items-center gap-3">
             <div className="w-10 h-10 rounded-xl bg-blue-600 flex items-center justify-center text-white shadow-md shadow-blue-600/20">
              <Smartphone size={20} />
            </div>
            <div>
              <h2 className="text-lg font-bold text-zinc-900 dark:text-zinc-100">
                {editingId === 'new' ? '新建小程序配置' : \`配置调试: \${appName}\`}
              </h2>
              <p className="text-xs text-zinc-500 dark:text-zinc-400 font-medium">商业化部署环境参数</p>
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
              onClick={handleSaveApplet}
              disabled={!appName.trim() || !appId.trim()}
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
            { id: 'developer', label: '开发者配置', icon: <Key size={14}/> },
            { id: 'server', label: '服务器与通信', icon: <Server size={14}/> }
          ].map(tab => (
            <button
              key={tab.id}
              onClick={() => setActiveTab(tab.id as 'basic' | 'developer' | 'server')}
              className={\`flex items-center gap-2 px-1 py-3 text-sm font-bold border-b-2 transition-all \${
                activeTab === tab.id 
                  ? 'border-blue-600 text-blue-600' 
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
                    <label className="block text-sm font-bold text-zinc-800 dark:text-zinc-200 mb-2">应用名称 <span className="text-red-500">*</span></label>
                    <input 
                      type="text" value={appName} onChange={(e) => setAppName(e.target.value)}
                      placeholder="你的小程序名字"
                      className="w-full bg-white dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-800 rounded-xl px-4 py-3 text-sm font-medium text-zinc-900 dark:text-zinc-100 focus:outline-none focus:border-blue-500 focus:ring-2 focus:ring-blue-500/20 transition-all shadow-sm"
                    />
                 </div>

                 <div className="flex gap-6">
                    <div className="flex-1">
                      <label className="block text-sm font-bold text-zinc-800 dark:text-zinc-200 mb-2">默认分类</label>
                      <select 
                        value={appGroup} 
                        onChange={(e) => setAppGroup(e.target.value)}
                        className="w-full bg-white dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-800 rounded-xl px-4 py-3 text-sm font-medium text-zinc-900 dark:text-zinc-100 focus:outline-none focus:border-blue-500 focus:ring-2 focus:ring-blue-500/20 transition-all shadow-sm appearance-none"
                      >
                         {groups.length === 0 && <option value="未分组">未分组</option>}
                         {groups.map(g => <option key={g} value={g}>{g}</option>)}
                      </select>
                    </div>

                    <div className="w-1/2">
                      <label className="block text-sm font-bold text-zinc-800 dark:text-zinc-200 mb-2">应用图标 (Avatar)</label>
                      <div className="flex gap-2 p-2 bg-zinc-100 dark:bg-zinc-900 rounded-xl border border-zinc-200 dark:border-zinc-800 overflow-x-auto hide-scrollbar">
                        {['📱', '🛒', '🎮', '🍔', '💼', '📅', '🏥', '🛍️'].map((emoji) => (
                          <button
                            key={emoji}
                            onClick={() => setAppAvatar(emoji)}
                            className={\`w-9 h-9 shrink-0 flex items-center justify-center text-lg rounded-lg border-2 transition-all \${
                              appAvatar === emoji ? 'border-blue-500 bg-white shadow-sm scale-110' : 'border-transparent hover:bg-zinc-200 dark:hover:bg-zinc-800'
                            }\`}
                          >
                            {emoji}
                          </button>
                        ))}
                      </div>
                    </div>
                 </div>

                 <div>
                    <label className="block text-sm font-bold text-zinc-800 dark:text-zinc-200 mb-2">默认页面路径 (Path) <span className="text-zinc-400 font-normal ml-2">选填，未填时打开首页</span></label>
                    <input 
                      type="text" value={appPath} onChange={(e) => setAppPath(e.target.value)}
                      placeholder="例如: pages/index/index"
                      className="w-full bg-white dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-800 rounded-xl px-4 py-3 text-sm font-mono text-zinc-900 dark:text-zinc-100 focus:outline-none focus:border-blue-500 focus:ring-2 focus:ring-blue-500/20 transition-all shadow-sm"
                    />
                 </div>

                 <div>
                    <label className="block text-sm font-bold text-zinc-800 dark:text-zinc-200 mb-2">介绍与功能说明</label>
                    <textarea 
                      value={appDescription} onChange={(e) => setAppDescription(e.target.value)}
                      placeholder="简单说明小程序的功能、受众及定位..."
                      rows={3}
                      className="w-full bg-white dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-800 rounded-xl px-4 py-3 text-sm font-medium text-zinc-900 dark:text-zinc-100 focus:outline-none focus:border-blue-500 focus:ring-2 focus:ring-blue-500/20 transition-all shadow-sm resize-none"
                    />
                 </div>
              </div>
            )}

            {activeTab === 'developer' && (
              <div className="space-y-6 animate-in fade-in duration-300 slide-in-from-right-4">
                 <div className="bg-blue-50 dark:bg-blue-900/10 border border-blue-100 dark:border-blue-900/30 p-4 rounded-xl flex gap-3 text-blue-800 dark:text-blue-300 text-sm">
                    <Key className="shrink-0 mt-0.5 text-blue-500" size={16} />
                    <div>请在微信公众平台「开发管理 - 开发设置」中获取凭证。请妥善保管开发者密码，避免泄露。</div>
                 </div>

                 <div>
                    <label className="block text-sm font-bold text-zinc-800 dark:text-zinc-200 mb-2">AppID (小程序ID) <span className="text-red-500">*</span></label>
                    <input 
                      type="text" value={appId} onChange={(e) => setAppId(e.target.value)}
                      placeholder="wx1234567890abcdef"
                      className="w-full bg-zinc-50 dark:bg-zinc-900/50 border border-zinc-200 dark:border-zinc-800 rounded-xl px-4 py-3 text-sm font-mono tracking-wide text-zinc-900 dark:text-zinc-100 focus:outline-none focus:border-blue-500 focus:ring-2 focus:ring-blue-500/20 transition-all shadow-sm"
                    />
                 </div>

                 <div>
                    <label className="block text-sm font-bold text-zinc-800 dark:text-zinc-200 mb-2">AppSecret (小程序密钥)</label>
                    <input 
                      type="password" value={appSecret} onChange={(e) => setAppSecret(e.target.value)}
                      placeholder="********************************"
                      className="w-full bg-zinc-50 dark:bg-zinc-900/50 border border-zinc-200 dark:border-zinc-800 rounded-xl px-4 py-3 text-sm font-mono tracking-wide text-zinc-900 dark:text-zinc-100 focus:outline-none focus:border-blue-500 focus:ring-2 focus:ring-blue-500/20 transition-all shadow-sm"
                    />
                 </div>
              </div>
            )}

            {activeTab === 'server' && (
              <div className="space-y-8 animate-in fade-in duration-300 slide-in-from-right-4">
                 
                 <div className="space-y-5">
                    <h4 className="text-base font-bold text-zinc-900 dark:text-zinc-100 flex items-center gap-2">
                       <LinkIcon size={16} className="text-blue-500" />
                       合法服务器域名
                    </h4>
                    
                    <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                      <div>
                        <label className="block text-xs font-bold text-zinc-500 uppercase tracking-wider mb-2">request合法域名</label>
                        <input 
                          type="text" value={requestDomain} onChange={(e) => setRequestDomain(e.target.value)}
                          placeholder="https://api.example.com"
                          className="w-full bg-white dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-800 rounded-lg px-3 py-2 text-sm font-mono text-zinc-900 dark:text-zinc-100 focus:border-blue-500 transition-colors"
                        />
                      </div>
                      <div>
                        <label className="block text-xs font-bold text-zinc-500 uppercase tracking-wider mb-2">socket合法域名</label>
                        <input 
                          type="text" value={socketDomain} onChange={(e) => setSocketDomain(e.target.value)}
                          placeholder="wss://ws.example.com"
                          className="w-full bg-white dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-800 rounded-lg px-3 py-2 text-sm font-mono text-zinc-900 dark:text-zinc-100 focus:border-blue-500 transition-colors"
                        />
                      </div>
                      <div>
                        <label className="block text-xs font-bold text-zinc-500 uppercase tracking-wider mb-2">uploadFile合法域名</label>
                        <input 
                          type="text" value={uploadDomain} onChange={(e) => setUploadDomain(e.target.value)}
                          placeholder="https://upload.example.com"
                          className="w-full bg-white dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-800 rounded-lg px-3 py-2 text-sm font-mono text-zinc-900 dark:text-zinc-100 focus:border-blue-500 transition-colors"
                        />
                      </div>
                      <div>
                        <label className="block text-xs font-bold text-zinc-500 uppercase tracking-wider mb-2">downloadFile合法域名</label>
                        <input 
                          type="text" value={downloadDomain} onChange={(e) => setDownloadDomain(e.target.value)}
                          placeholder="https://cdn.example.com"
                          className="w-full bg-white dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-800 rounded-lg px-3 py-2 text-sm font-mono text-zinc-900 dark:text-zinc-100 focus:border-blue-500 transition-colors"
                        />
                      </div>
                    </div>
                 </div>

                 <div className="h-px bg-zinc-200 dark:bg-zinc-800 w-full" />

                 <div className="space-y-5">
                    <h4 className="text-base font-bold text-zinc-900 dark:text-zinc-100 flex items-center gap-2">
                       <MessageSquare size={16} className="text-blue-500" />
                       消息推送配置
                    </h4>

                    <div>
                      <label className="block text-sm font-bold text-zinc-800 dark:text-zinc-200 mb-2">Token (令牌)</label>
                      <input 
                        type="text" value={msgToken} onChange={(e) => setMsgToken(e.target.value)}
                        placeholder="英文或数字组成的令牌"
                        className="w-full bg-white dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-800 rounded-xl px-4 py-3 text-sm font-mono text-zinc-900 dark:text-zinc-100 focus:border-blue-500 transition-all shadow-sm"
                      />
                    </div>
                    <div>
                      <label className="block text-sm font-bold text-zinc-800 dark:text-zinc-200 mb-2">EncodingAESKey (消息加解密密钥)</label>
                      <input 
                        type="text" value={msgEncodingAESKey} onChange={(e) => setMsgEncodingAESKey(e.target.value)}
                        placeholder="43位长度的随机字符"
                        className="w-full bg-white dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-800 rounded-xl px-4 py-3 text-sm font-mono text-zinc-900 dark:text-zinc-100 focus:border-blue-500 transition-all shadow-sm"
                      />
                    </div>

                    <div>
                      <label className="block text-sm font-bold text-zinc-800 dark:text-zinc-200 mb-3">数据格式</label>
                      <div className="flex gap-4">
                        <label className="flex items-center gap-2 cursor-pointer bg-white dark:bg-zinc-900 px-4 py-2.5 border border-zinc-200 dark:border-zinc-800 rounded-xl flex-1 hover:border-blue-500 transition-colors">
                          <input 
                            type="radio" name="msgDataFormat" value="json" 
                            checked={msgDataFormat === 'json'} onChange={() => setMsgDataFormat('json')}
                            className="w-4 h-4 text-blue-600 focus:ring-blue-500 border-zinc-300"
                          />
                          <span className="text-sm font-medium">JSON (推荐)</span>
                        </label>
                        <label className="flex items-center gap-2 cursor-pointer bg-white dark:bg-zinc-900 px-4 py-2.5 border border-zinc-200 dark:border-zinc-800 rounded-xl flex-1 hover:border-blue-500 transition-colors">
                          <input 
                            type="radio" name="msgDataFormat" value="xml" 
                            checked={msgDataFormat === 'xml'} onChange={() => setMsgDataFormat('xml')}
                            className="w-4 h-4 text-blue-600 focus:ring-blue-500 border-zinc-300"
                          />
                          <span className="text-sm font-medium">XML (传统模式)</span>
                        </label>
                      </div>
                    </div>
                 </div>
              </div>
            )}

            {editingId && editingId !== 'new' && (
              <div className="mt-12 pt-6 border-t border-zinc-200 dark:border-zinc-800">
                <button
                  type="button"
                  onClick={() => {
                    if (confirm('确认删除该小程序配置信息吗？此操作不可恢复。')) {
                      handleDeleteApplet(editingId);
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
console.log("Applet rewritten")

const fs = require('fs');
const filepath = './packages/sdkwork-knowledgebase-pc-knowledgebase/src/components/AppletManagerModal.tsx';
let txt = fs.readFileSync(filepath, 'utf8');

// The file was truncated. Let's fix it manually.
// First remove the broken extra divs at the end.
const endBroken = `            </div>
          </div>
        </div>
      </div>

      

      
            </div>
    </div>
  );
}`;

const newEnd = `            </div>
            
            {/* PANEL B: Add or Edit View */}
            <div 
              className={\`absolute inset-0 bg-[var(--color-kb-panel)] z-20 flex flex-col transition-transform duration-300 ease-in-out \${
                 editingId !== null ? 'translate-x-0' : 'translate-x-[100%]'
              }\`}
            >
              <div className="p-6 border-b border-[var(--color-kb-panel-border)] flex items-center justify-between shrink-0 bg-[var(--color-kb-panel)]">
                <div className="flex items-center gap-3">
                  <button 
                    type="button"
                    onClick={() => setEditingId(null)}
                    className="p-1.5 hover:bg-neutral-200 dark:hover:bg-zinc-800 rounded-lg text-zinc-500 transition-colors"
                  >
                    <X size={20} />
                  </button>
                  <span className="text-lg font-bold text-[var(--color-kb-text-heading)] flex items-center gap-2">
                    {editingId === 'new' ? '✨ 新增小程序配置' : \`⚙️ 编辑小程序: \${appName}\`}
                  </span>
                </div>
                
                <div className="flex items-center gap-3">
                  {editingId !== 'new' && (
                    <button
                      type="button"
                      onClick={() => {
                        if (confirm('确认删除该小程序配置信息吗？')) {
                          handleDeleteApplet(editingId!);
                        }
                      }}
                      className="px-4 py-2 text-sm text-red-500 hover:bg-red-50 dark:hover:bg-red-950/30 rounded-xl transition-colors font-bold flex items-center gap-1.5"
                    >
                      <Trash2 size={16} />
                      删除配置
                    </button>
                  )}
                  <button 
                    onClick={handleSaveApplet}
                    disabled={!appName.trim() || !appId.trim()}
                    type="button"
                    className="px-6 py-2 text-sm font-bold bg-[#07c160] hover:bg-[#07c160]/90 disabled:opacity-40 text-white rounded-xl shadow-md flex items-center gap-1.5 transition-all hover:-translate-y-0.5 active:translate-y-0"
                  >
                    保存配置
                  </button>
                </div>
              </div>

              <div className="flex-1 overflow-y-auto p-8 relative">
                <div className="max-w-3xl space-y-8 animate-in fade-in slide-in-from-bottom-4 duration-300">
                  <div className="bg-[var(--color-kb-panel)] border border-[var(--color-kb-panel-border)] p-6 rounded-2xl shadow-sm space-y-6">
                    <div className="flex items-center gap-2 mb-2">
                      <div className="w-1 h-4 bg-[#07c160] rounded-full"></div>
                      <h3 className="text-base font-bold">基本信息</h3>
                    </div>
                    
                    <div className="flex flex-col xl:flex-row gap-6">
                      <div className="flex-1">
                        <label className="block text-sm font-bold text-[var(--color-kb-text-heading)] mb-2">小程序名称 <span className="text-red-500">*</span></label>
                        <input 
                          type="text"
                          value={appName}
                          onChange={(e) => setAppName(e.target.value)}
                          placeholder="例如: 某某购物"
                          className="w-full bg-[var(--color-kb-editor)] border-2 border-[var(--color-kb-panel-border)] rounded-xl px-4 py-2.5 text-sm font-medium text-[var(--color-kb-text)] focus:outline-none focus:border-[#07c160] transition-colors"
                        />
                      </div>
                      <div className="flex-1">
                        <label className="block text-sm font-bold text-[var(--color-kb-text-heading)] mb-2">AppID <span className="text-red-500">*</span></label>
                        <input 
                          type="text"
                          value={appId}
                          onChange={(e) => setAppId(e.target.value)}
                          placeholder="例如: wx1234567890abcdef"
                          className="w-full bg-[var(--color-kb-editor)] border-2 border-[var(--color-kb-panel-border)] rounded-xl px-4 py-2.5 text-sm font-mono text-[var(--color-kb-text)] focus:outline-none focus:border-[#07c160] transition-colors"
                        />
                      </div>
                    </div>
                    
                    <div className="flex flex-col xl:flex-row gap-6 pt-2">
                      {/* Avatar Picker */}
                      <div className="flex-2 xl:w-1/2">
                        <label className="block text-sm font-bold text-[var(--color-kb-text-heading)] mb-2">图标表情 (Avatar)</label>
                        <div className="flex gap-2 flex-wrap bg-[var(--color-kb-editor)] p-3 rounded-xl border border-[var(--color-kb-panel-border)]">
                          {['📱', '🛒', '🎮', '🛠️', '🎫', '🍔', '🎨', '💼', '📅', '🏥', '🚗', '🛍️'].map((emoji) => (
                            <button
                              key={emoji}
                              onClick={() => setAppAvatar(emoji)}
                              type="button"
                              className={\`w-10 h-10 rounded-xl text-xl flex items-center justify-center border-2 transition-all \${
                                appAvatar === emoji 
                                  ? 'border-[#07c160] bg-[#07c160]/10 scale-110 shadow-md z-10' 
                                  : 'border-transparent hover:bg-neutral-100 dark:hover:bg-zinc-800'
                              }\`}
                            >
                              {emoji}
                            </button>
                          ))}
                        </div>
                      </div>

                      <div className="flex-1 xl:w-1/2">
                        <label className="block text-sm font-bold text-[var(--color-kb-text-heading)] mb-2">归属分类</label>
                        <div className="flex flex-wrap gap-2 bg-[var(--color-kb-editor)] p-3 rounded-xl border border-[var(--color-kb-panel-border)] min-h-[66px]">
                          {groups.map((g) => (
                            <button
                              key={g}
                              onClick={() => setAppGroup(g)}
                              type="button"
                              className={\`px-4 py-2 rounded-xl border-2 text-sm cursor-pointer select-none transition-all flex items-center gap-1.5 \${
                                appGroup === g 
                                  ? 'border-[#07c160] bg-[#07c160] text-white font-bold shadow-md' 
                                  : 'border-transparent hover:bg-neutral-100 dark:hover:bg-zinc-800 text-zinc-600 dark:text-zinc-300 font-medium'
                              }\`}
                            >
                              <Folder size={14} className={appGroup === g ? 'text-white' : 'text-zinc-400'} />
                              <span>{g}</span>
                            </button>
                          ))}
                        </div>
                      </div>
                    </div>

                    <div className="flex flex-col gap-6 pt-2">
                       <div className="flex-1">
                        <label className="block text-sm font-bold text-[var(--color-kb-text-heading)] mb-2">默认页面路径 (Path) <span className="text-zinc-400 font-normal ml-2">选填，未填时默认打开首页</span></label>
                        <input 
                          type="text"
                          value={appPath}
                          onChange={(e) => setAppPath(e.target.value)}
                          placeholder="例如: pages/index/index"
                          className="w-full bg-[var(--color-kb-editor)] border-2 border-[var(--color-kb-panel-border)] rounded-xl px-4 py-2.5 text-sm font-mono text-[var(--color-kb-text)] focus:outline-none focus:border-[#07c160] transition-colors"
                        />
                      </div>
                      <div className="flex-1">
                        <label className="block text-sm font-bold text-[var(--color-kb-text-heading)] mb-2">描述说明</label>
                        <input 
                          type="text"
                          value={appDescription}
                          onChange={(e) => setAppDescription(e.target.value)}
                          placeholder="小程序的简单描述..."
                          className="w-full bg-[var(--color-kb-editor)] border-2 border-[var(--color-kb-panel-border)] rounded-xl px-4 py-2.5 text-sm font-medium text-[var(--color-kb-text)] focus:outline-none focus:border-[#07c160] transition-colors"
                        />
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            </div>

          </div>
        </div>
      </div>
    </div>
  );
}`;

txt = txt.replace(endBroken, newEnd);
fs.writeFileSync(filepath, txt, 'utf8');
console.log("Fixed!");

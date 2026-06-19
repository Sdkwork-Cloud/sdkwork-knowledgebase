import fs from 'fs';
const file = 'packages/sdkwork-knowledgebase-pc-knowledgebase/src/components/AppletManagerModal.tsx';
let content = fs.readFileSync(file, 'utf-8');

// The block to replace
const oldBlock = `            {showGroupManager && (
              <div className="px-4 pb-4 animate-in slide-in-from-top-2">
                <div className="flex bg-[var(--color-kb-panel)] rounded-lg border border-[var(--color-kb-panel-border)] focus-within:border-[#07c160] transition-colors overflow-hidden">
                  <input 
                    type="text" 
                    value={newGroupNameInput}
                    onChange={(e) => setNewGroupNameInput(e.target.value)}
                    placeholder="新增分组名"
                    className="w-full bg-transparent border-none text-xs px-3 py-2 outline-none text-[var(--color-kb-text-heading)] placeholder:text-[var(--color-kb-text-muted)]"
                    onKeyDown={(e) => e.key === 'Enter' && handleGroupAdd()}
                  />
                  <button 
                     onClick={handleGroupAdd}
                     className="px-3 text-[#07c160] hover:bg-[#07c160]/10 text-xs font-medium"
                  >
                     添加
                  </button>
                </div>
              </div>
            )}`;

const newBlock = ``;

content = content.replace(oldBlock, newBlock);

if (content.includes("setNewGroupNameInput('');")) {
    content = content.replace("setNewGroupNameInput('');", "setNewGroupNameInput('');\\n    setShowGroupManager(false);");
}

const modalInjectionPoint = `          </div>
        </div>
      </div>`;

const newModalCode = `          </div>
        </div>

        {/* 新增分组 Modal */}
        {showGroupManager && (
          <div className="fixed inset-0 z-[60] flex items-center justify-center bg-black/40 backdrop-blur-sm animate-in fade-in duration-200">
            <div className="bg-[var(--color-kb-editor)] border border-[var(--color-kb-panel-border)] rounded-xl w-[320px] shadow-2xl flex flex-col overflow-hidden animate-in zoom-in-95 duration-200">
              <div className="px-5 py-3.5 border-b border-[var(--color-kb-panel-border)] flex items-center justify-between bg-[var(--color-kb-panel)]">
                <h3 className="text-[14px] font-bold text-[var(--color-kb-text-heading)]">新增业务分组</h3>
                <button onClick={() => { setShowGroupManager(false); setNewGroupNameInput(''); }} className="text-[var(--color-kb-text-muted)] hover:text-[var(--color-kb-text)] transition-colors">
                  <X size={16} />
                </button>
              </div>
              <div className="p-5">
                <label className="block text-[12px] font-bold text-[var(--color-kb-text-heading)] mb-2">分组名称</label>
                <input
                  autoFocus
                  type="text"
                  value={newGroupNameInput}
                  onChange={(e) => setNewGroupNameInput(e.target.value)}
                  placeholder="请输入分组名称"
                  className="w-full bg-[var(--color-kb-editor)] border border-[var(--color-kb-panel-border)] rounded-lg px-3 py-2 text-[13px] text-[var(--color-kb-text-heading)] focus:outline-none focus:border-[#07c160] focus:ring-1 focus:ring-[#07c160] transition-all shadow-sm"
                  onKeyDown={(e) => e.key === 'Enter' && handleGroupAdd()}
                />
              </div>
              <div className="px-5 py-3 border-t border-[var(--color-kb-panel-border)] bg-[var(--color-kb-panel)] flex justify-end gap-2">
                <button onClick={() => { setShowGroupManager(false); setNewGroupNameInput(''); }} className="px-4 py-1.5 text-[12px] font-medium text-[var(--color-kb-text-muted)] hover:text-[var(--color-kb-text)] hover:bg-[var(--color-kb-panel-hover)] rounded-md transition-colors">
                  取消
                </button>
                <button 
                  onClick={handleGroupAdd}
                  disabled={!newGroupNameInput.trim()}
                  className="px-4 py-1.5 text-[12px] font-bold text-white bg-[#07c160] hover:bg-[#06ad56] disabled:opacity-50 rounded-md transition-all shadow-sm"
                >
                  确定
                </button>
              </div>
            </div>
          </div>
        )}
      </div>`;

content = content.replace(modalInjectionPoint, newModalCode);
fs.writeFileSync(file, content.replaceAll('\\n', '\n'));
console.log('Applet updated');

const fs = require('fs');
const filepath = './packages/sdkwork-knowledgebase-pc-knowledgebase/src/components/OfficialAccountModal.tsx';
let txt = fs.readFileSync(filepath, 'utf8');

// I am just going to do string replace to fix it structurally.

// 1. Remove the middle section I incorrectly added.
// It starts after `                </div>\n              </div>\n            </div>` (around line 434)
// and ends right before `              <div className="p-6 border-b border-[var(--color-kb-panel-border)] flex items-center justify-between shrink-0 bg-[var(--color-kb-panel)]">`

const target1 = `                </div>
              </div>
            </div>
          </div>
        </div>

        {/* Global Footer (only for list selection) */}
        {oaEditingId === null && (
          <div className="p-5 border-t border-[var(--color-kb-panel-border)] bg-[var(--color-kb-panel-hover)] flex items-center justify-between shrink-0">
            <span className="text-sm font-bold text-[var(--color-kb-text-heading)]">
              已选 <span className="text-[#07c160] mx-1 text-lg">{selectedOfficialAccountIds.length}</span> 个发布账号
            </span>
            <div className="flex items-center gap-3">
              <button 
                type="button"
                onClick={onClose} 
                className="px-5 py-2 text-sm text-[var(--color-kb-text-muted)] hover:text-[var(--color-kb-text)] transition-colors font-bold"
              >
                取消
              </button>
              <button 
                onClick={handleConfirmAndClose}
                disabled={selectedOfficialAccountIds.length === 0}
                type="button"
                className="px-6 py-2 text-sm font-bold bg-[#07c160] hover:bg-[#07c160]/90 disabled:opacity-40 text-white rounded-xl shadow-md transition-transform hover:-translate-y-0.5 active:translate-y-0"
              >
                保存选择并返回
              </button>
            </div>
          </div>
        )}
      </div>

      {/* Backdrop overlay for Drawer */}
      {oaEditingId !== null && (
         <div 
           className="fixed inset-0 bg-black/40 z-[310] transition-opacity duration-300"
           onClick={() => setOaEditingId(null)}
         />
      )}

      {/* PANEL B: Add or Edit View (Drawer) */}
      <div 
        className={\`fixed top-0 bottom-0 w-[560px] bg-white dark:bg-[#0c0c0e] shadow-[10px_0_40px_rgba(0,0,0,0.2)] border-r border-[var(--color-kb-panel-border)] z-[320] flex flex-col transition-transform duration-300 ease-in-out \${
           oaEditingId !== null ? 'left-0 translate-x-0' : 'left-0 -translate-x-full'
        }\`}
      >`;

const replace1 = `                </div>
              </div>
            </div>
          </div>
        </div>
        
        {/* Global Footer (only for list selection) */}
        {oaEditingId === null && (
          <div className="p-4 bg-[var(--color-kb-panel-hover)] border-t border-[var(--color-kb-panel-border)] flex justify-end gap-4 shrink-0 shadow-[0_-4px_15px_rgba(0,0,0,0.03)] z-20 relative">
            <div className="flex-1 flex items-center pl-4">
              <span className="text-sm">已选择 <span className="font-bold text-[#07c160] text-lg px-1">{selectedOfficialAccountIds.length}</span> 个渠道进行群发配置</span>
            </div>
            <button 
              type="button"
              onClick={onClose} 
              className="px-6 py-2.5 text-sm font-bold bg-white dark:bg-zinc-800 border-2 border-[var(--color-kb-panel-border)] hover:bg-neutral-50 dark:hover:bg-zinc-700 text-[var(--color-kb-text-heading)] rounded-xl transition-all shadow-sm"
            >
              取消与还原
            </button>
            <button 
              onClick={handleConfirmAndClose} 
              type="button"
              className="px-8 py-2.5 text-sm font-bold bg-gray-900 dark:bg-gray-100 hover:bg-gray-800 dark:hover:bg-white text-white dark:text-gray-900 rounded-xl shadow-md transition-all flex items-center gap-2 hover:-translate-y-0.5 active:translate-y-0"
            >
              <span>确认配置并返回编辑</span>
              {/* Note: I can't easily import Check from lucide-react if it's missing, but it was already imported. */}
            </button>
          </div>
        )}
      </div>

      {/* Backdrop overlay for Drawer */}
      <div 
        className={\`fixed inset-0 bg-black/40 z-[610] transition-opacity duration-300 \${oaEditingId !== null ? 'opacity-100 pointer-events-auto' : 'opacity-0 pointer-events-none'}\`}
        onClick={() => setOaEditingId(null)}
      />

      {/* PANEL B: Add or Edit View (Drawer) */}
      <div 
        className={\`fixed top-0 bottom-0 w-[560px] bg-white dark:bg-[#0c0c0e] shadow-[20px_0_40px_rgba(0,0,0,0.2)] border-r border-[var(--color-kb-panel-border)] z-[620] flex flex-col transition-transform duration-300 ease-in-out \${
           oaEditingId !== null ? 'left-0 translate-x-0' : 'left-0 -translate-x-full'
        }\`}
      >`;

let newTxt = txt.replace(target1, replace1);

const target2 = `                      </div>
                    </div>
                  </div>
                )}
              </div>
            </div>
          </div>
        </div>

        {/* Global Footer (only for list selection) */}
        {oaEditingId === null && (
          <div className="p-4 bg-[var(--color-kb-panel-hover)] border-t border-[var(--color-kb-panel-border)] flex justify-end gap-4 shrink-0 shadow-[0_-4px_15px_rgba(0,0,0,0.03)] z-20 relative">
            <div className="flex-1 flex items-center pl-4">
              <span className="text-sm">已选择 <span className="font-bold text-[#07c160] text-lg px-1">{selectedOfficialAccountIds.length}</span> 个渠道进行群发配置</span>
            </div>
            <button 
              onClick={onClose} 
              type="button"
              className="px-6 py-2.5 text-sm font-bold bg-white dark:bg-zinc-800 border-2 border-[var(--color-kb-panel-border)] hover:bg-neutral-50 dark:hover:bg-zinc-700 text-[var(--color-kb-text-heading)] rounded-xl transition-all shadow-sm"
            >
              取消与还原
            </button>
            <button 
              onClick={handleConfirmAndClose} 
              type="button"
              className="px-8 py-2.5 text-sm font-bold bg-gray-900 dark:bg-gray-100 hover:bg-gray-800 dark:hover:bg-white text-white dark:text-gray-900 rounded-xl shadow-md transition-all flex items-center gap-2 hover:-translate-y-0.5 active:translate-y-0"
            >
              <span>确认配置并返回编辑</span>
              <Check size={16} strokeWidth={3} />
            </button>
          </div>
        )}
      </div>
    </div>
  );
}`;

const replace2 = `                      </div>
                    </div>
                  </div>
                )}
              </div>
            </div>
    </div>
  );
}`;

newTxt = newTxt.replace(target2, replace2);

if (newTxt === txt) {
  console.log("No changes made!");
} else {
  fs.writeFileSync(filepath, newTxt, 'utf8');
  console.log("Successfully fixed!");
}

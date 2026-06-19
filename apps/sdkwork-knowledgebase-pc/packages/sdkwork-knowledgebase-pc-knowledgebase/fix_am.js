const fs = require('fs');

const filepath = './packages/sdkwork-knowledgebase-pc-knowledgebase/src/components/AppletManagerModal.tsx';
let txt = fs.readFileSync(filepath, 'utf8');

// 1. Remove backdrop
txt = txt.replace(/\{\/\* Backdrop overlay for Drawer \*\/\}\s*<div[^>]+onClick=\{\(\) => setEditingId\(null\)\}[^>]+\/>\s*/g, '');

// 2. Change Panel A
txt = txt.replace(
  'className="absolute inset-0 flex flex-col overflow-hidden transition-opacity duration-300"',
  'className={`absolute inset-0 flex flex-col overflow-hidden transition-all duration-300 ${editingId !== null ? "-translate-x-10 opacity-0 pointer-events-none" : "translate-x-0 opacity-100 pointer-events-auto"}`}'
);

// 3. Change Panel B class
const oldDrawerStart = /\{\/\* PANEL B: Add or Edit View \(Drawer\) \*\/\}[\s\S]*?(?=<div className="p-6 border-b )/;
const newPanelBStart = `{/* PANEL B: Add or Edit View */}
            <div 
              className={\`absolute inset-0 bg-[var(--color-kb-panel)] z-20 flex flex-col transition-transform duration-300 ease-in-out \${
                 editingId !== null ? 'translate-x-0' : 'translate-x-[100%]'
              }\`}
            >
              `;
txt = txt.replace(oldDrawerStart, newPanelBStart);

// 4. Move Panel B inside the Right Main Content !
const panelBMatch = txt.match(/\{\/\* PANEL B: Add or Edit View \*\/\}[\s\S]*?(?=\s*<\/div>\s*<\/div>\s*\);\s*\})/);
if (panelBMatch) {
  const panelBStr = panelBMatch[0];
  txt = txt.replace(panelBStr, '');
  
  // Right Main Content closes right before the end of the Content Body.
  // Content Body ends right before `</div>\n      </div>\n    </div>\n  );\n}`
  const insertTarget = `              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}`;
  const replacement = `              </div>
            </div>
${panelBStr}
          </div>
        </div>
      </div>
    </div>
  );
}`;

  txt = txt.replace(insertTarget, replacement);
  fs.writeFileSync(filepath, txt, 'utf8');
}


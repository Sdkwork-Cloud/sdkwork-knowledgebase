const fs = require('fs');

function fixApplet() {
  const filepath = './packages/sdkwork-knowledgebase-pc-knowledgebase/src/components/AppletManagerModal.tsx';
  let content = fs.readFileSync(filepath, 'utf8');
  
  // Update Panel A wrapper
  content = content.replace(
    /className="absolute inset-0 flex flex-col overflow-hidden transition-opacity duration-300"/g,
    'className={`absolute inset-0 flex flex-col overflow-hidden transition-all duration-300 ${editingId !== null ? "-translate-x-10 opacity-0 pointer-events-none" : "translate-x-0 opacity-100 pointer-events-auto"}`}'
  );
  
  // Remove Backdrop
  const backdropRegex = /\{\/\* Backdrop overlay for Drawer \*\/\}\s*<div[^>]+onClick=\{\(\) => setEditingId\(null\)\}[^>]+\/>/g;
  content = content.replace(backdropRegex, '');
  
  // Update Panel B container
  const panelBRegex = /\{\/\* PANEL B: Add or Edit View \(Drawer\) \*\/\}\s*<div\s+className=\{\`fixed top-0 bottom-0 w-\[560px\] bg-white dark:bg-\[#0c0c0e\] shadow-\[20px_0_40px_rgba\(0,0,0,0\.2\)\] border-r border-\[var\(--color-kb-panel-border\)\] z-\[620\] flex flex-col transition-transform duration-300 ease-in-out \$\{\s*editingId !== null \? 'left-0 translate-x-0' : 'left-0 -translate-x-full'\s*\}\`\}\s*>/g;
  
  const newPanelB = `{/* PANEL B: Add or Edit View */}
            <div 
              className={\`absolute inset-0 bg-[var(--color-kb-panel)] z-20 flex flex-col transition-transform duration-300 ease-in-out \${
                 editingId !== null ? 'translate-x-0' : 'translate-x-[100%]'
              }\`}
            >`;
  
  content = content.replace(panelBRegex, newPanelB);
  
  // Move Panel B inside the right main content
  const regex = /([\s\S]*?)(\s*\}\)\(\)\}\s*<\/div>\s*<\/div>\s*<\/div>\s*)(<\/div>\s*<\/div>\s*)([\s\S]*?)(\{\/\* PANEL B[\s\S]*?<\/div>\s*<\/div>\s*<\/div>\s*<\/div>\s*<\/div>\s*)(<\/div>\s*);\s*\}/;
  const match = content.match(regex);
  if (match) {
    let newContent = match[1] + match[2] + match[5] + "\n          </div>\n        </div>\n      </div>\n    </div>\n  );\n}";
    content = newContent;
  } else {
    console.log("AppletManagerModal regex match failed!");
  }

  fs.writeFileSync(filepath, content, 'utf8');
}

function fixOA() {
  const filepath = './packages/sdkwork-knowledgebase-pc-knowledgebase/src/components/OfficialAccountModal.tsx';
  let content = fs.readFileSync(filepath, 'utf8');
  
  // Update Panel A config
  content = content.replace(
    /className="absolute inset-0 flex flex-col overflow-hidden transition-opacity duration-300"/g,
    'className={`absolute inset-0 flex flex-col overflow-hidden transition-all duration-300 ${oaEditingId !== null ? "-translate-x-10 opacity-0 pointer-events-none" : "translate-x-0 opacity-100 pointer-events-auto"}`}'
  );
  
  // Remove Backdrop
  const backdropRegex = /\{\/\* Backdrop overlay for Drawer \*\/\}\s*<div[^>]+onClick=\{\(\) => setOaEditingId\(null\)\}[^>]+\/>/g;
  content = content.replace(backdropRegex, '');
  
  // Update Panel B container
  const panelBRegex = /\{\/\* PANEL B: Add or Edit View \(Drawer\) \*\/\}\s*<div\s+className=\{\`fixed top-0 bottom-0 w-\[560px\] bg-white dark:bg-\[#0c0c0e\] shadow-\[20px_0_40px_rgba\(0,0,0,0\.2\)\] border-r border-\[var\(--color-kb-panel-border\)\] z-\[620\] flex flex-col transition-transform duration-300 ease-in-out \$\{\s*oaEditingId !== null \? 'left-0 translate-x-0' : 'left-0 -translate-x-full'\s*\}\`\}\s*>/g;
  
  const newPanelB = `{/* PANEL B: Add or Edit View */}
            <div 
              className={\`absolute inset-0 bg-[var(--color-kb-panel)] z-20 flex flex-col transition-transform duration-300 ease-in-out \${
                 oaEditingId !== null ? 'translate-x-0' : 'translate-x-[100%]'
              }\`}
            >`;
            
  content = content.replace(panelBRegex, newPanelB);
  
  const panelBMatch = content.match(/\{\/\* PANEL B: Add or Edit View[\s\S]*?(?=\s*<\/div>\s*<\/div>\s*\);\s*\})/);
  if (panelBMatch) {
    const panelBStr = panelBMatch[0];
    content = content.replace(panelBMatch[0], ''); // Remove from bottom
    
    // Replace exact insert point
    const targetInsert = `              </div>\n            </div>\n          </div>\n        </div>\n        \n        {/* Global Footer`;
    const insertReplacement = `              </div>\n            </div>\n${panelBStr}\n          </div>\n        </div>\n        \n        {/* Global Footer`;
    content = content.replace(targetInsert, insertReplacement);
  } else {
    console.log("OA panel B regex match failed!");
  }
  
  fs.writeFileSync(filepath, content, 'utf8');
}

try { fixApplet(); } catch (e) { console.error('Applet error:', e) }
try { fixOA(); } catch (e) { console.error('OA error:', e) }
console.log("Done");

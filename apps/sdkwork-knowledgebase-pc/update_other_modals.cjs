const fs = require('fs');

const extractAndReplace = (filePath) => {
  let content = fs.readFileSync(filePath, 'utf8');
  
  if (!content.includes('useTranslation')) {
    // import useTranslation cleanly
    content = content.replace(/import React(.*?);/, (match) => {
      // It replaces the whole line to keep it clean. E.g. import React, { useState } from 'react';
      return `${match}\nimport { useTranslation } from 'react-i18next';`;
    });
  }
  
  // Create function hook if not present
  const functionMatch = content.match(/export function (\w+Modal)\([^)]*\)\s*\{/);
  if (functionMatch) {
    const signature = functionMatch[0];
    if (!content.includes('const { t } = useTranslation([')) {
        content = content.replace(signature, `${signature}\n  const { t } = useTranslation(['kb', 'common']);`);
    }
  }

  // DeployWebsiteModal.tsx
  if (filePath.includes('DeployWebsiteModal.tsx')) {
    content = content.replace("部署为独立网站", "{t('deployToWebsite')}");
    content = content.replace("将当前知识库发布为对客友好、支持全文检索的多端自适应网站", "{t('deployToWebsiteDesc')}");
    content = content.replace(/"网站名称"/g, "t('siteName')");
    // We also have label "网站名称"
    content = content.replace(/>网站名称</g, ">{t('siteName')}<");
    content = content.replace("公开显示的名称...", "{t('siteNamePlaceholder')}");
    content = content.replace("主题配色", "{t('themeColor')}");
    content = content.replace("域名配置", "{t('domainConfig')}");
    content = content.replace("自定义域名 (企业版/旗舰版专属)", "{t('customDomain')}");
    content = content.replace("系统分配域名", "{t('systemDomain')}");
    content = content.replace("部署中", "{t('deploying')}");
    content = content.replace("已部署", "{t('deployed')}");
    content = content.replace("关闭", "{t('close')}");
    content = content.replace("立即部署上线", "{t('deployStart')}");
    content = content.replace("保存配置修改", "{t('saveConfig')}");
    content = content.replace("下线网站", "{t('offlineSite')}");
    content = content.replace("可于独立后台重新定制", "{t('editableSite')}");
    content = content.replace("访问网站", "{t('viewSite')}");
  }

  // VersionHistoryModal.tsx
  if (filePath.includes('VersionHistoryModal.tsx')) {
    content = content.replace("版本历史", "{t('versionHistory')}");
    content = content.replace(/\{item\.title\} 的历史记录/g, "{t('historyOf', { title: item.title })}");
    content = content.replace("当前版本", "{t('currentVersion')}");
    content = content.replace("初始版本", "{t('initialVersion')}");
    content = content.replace(/'今天 /g, "t('today') + ' ' + '");
  }

  // NotesAppModal.tsx
  if (filePath.includes('NotesAppModal.tsx')) {
    content = content.replace("导入备忘录", "{t('importNotes')}");
    content = content.replace("选择并同步草稿箱内容", "{t('importNotesDesc')}");
    content = content.replace("勾选即将导入的备忘录内容：", "{t('checkNotes')}");
    content = content.replace("导入后，备忘录内容将被转换为可编辑的知识库文档，并在本地安全存储。", "{t('importingNotesTip')}");
    // 导入 {selectedIds.size} 篇备忘录
    content = content.replace(/导入 \{selectedIds\.size\} 篇备忘录/g, "{t('importCount', { count: selectedIds.size })}");
    content = content.replace("取消</button>", "{t('cancel', { ns: 'common' })}</button>");
  }

  // PersonalKbModal.tsx
  if (filePath.includes('PersonalKbModal.tsx')) {
    content = content.replace("从个人知识库导入", "{t('importFromPersonal')}");
    content = content.replace("一键多选导入您其它的个人知识库文档到当前团队空间", "{t('importFromPersonalDesc')}");
    content = content.replace("个人知识库列表", "{t('personalKbList')}");
    content = content.replace("暂无个人知识库", "{t('noPersonalKb')}");
    // 搜索 "${activeKbDetails?.title || '未选择'}" 内的文档...
    content = content.replace(/搜索 "\$\{activeKbDetails\?\.title \|\| '未选择'\}\" 内的文档\.\.\./g, "${t('searchInKb', { kbName: activeKbDetails?.title || '未选择' })}");
    content = content.replace("正在加载文档列表...", "{t('loadingDocs')}");
    content = content.replace("此库内无可导入文件", "{t('noImportableDocs')}");
    content = content.replace("您可以通过左侧切换其它的个人库，或在此库中新增文档", "{t('noImportableDocsDesc')}");
    content = content.replace(/全选当前页文档 \(\{displayedFiles\.length\}\)/g, "{t('selectAllPage', { count: displayedFiles.length })}");
    content = content.replace("编辑人：", "{t('editorPrefix')}");
    // 已选择 <strong ...>{selectedIds.size}</strong> 篇个人文档
    content = content.replace(/已选择 <strong className="text-zinc-900 dark:text-\[var\(--color-kb-text-heading\)\] font-extrabold font-mono mx-1">\{selectedIds\.size\}<\/strong> 篇个人文档/g, "{t('selectedPersonalDocs', { count: selectedIds.size })}");
    content = content.replace(/导入选中内容 \(\{selectedIds\.size\}\)/g, "{t('importSelected', { count: selectedIds.size })}");
    content = content.replace(">取消</", ">{t('cancel', { ns: 'common' })}</");
  }

  // ChatFileModal.tsx
  if (filePath.includes('ChatFileModal.tsx')) {
    content = content.replace("导入聊天文件", "{t('importChatFile')}");
    content = content.replace("从对话记录中快速提取并归档有价值的文档", "{t('chatFileDesc')}");
    content = content.replace("微信群聊同步", "{t('fromWechatGroup')}");
    content = content.replace("MCP终端输出", "{t('fromMcpConsole')}");
    // 已选择 <strong ...>{selectedIds.size}</strong> 份文件
    content = content.replace(/已选择 <strong className="text-zinc-900 dark:text-\[var\(--color-kb-text-heading\)\] font-extrabold font-mono mx-1">\{selectedIds\.size\}<\/strong> 份文件/g, "{t('selectedChatFiles', { count: selectedIds.size })}");
    content = content.replace(/导入选中文件 \(\{selectedIds\.size\}\)/g, "{t('importSelectedFiles', { count: selectedIds.size })}");
    content = content.replace("选择导入目标知识库", "{t('selectTargetKb')}");
    content = content.replace("目标文件夹 (可选)", "{t('targetFolder')}");
    content = content.replace("默认根目录", "{t('defaultRoot')}");
    content = content.replace(">取消<", ">{t('cancel', { ns: 'common' })}<");
  }

  // CloudDriveModal.tsx
  if (filePath.includes('CloudDriveModal.tsx')) {
    // Already has useTranslation for cloudDrive
    if (content.includes("cloudDrive")) {
        // Just checking if we need to replace some texts that are omitted.
        // It seems t('myEnterpriseDrive') etc is mostly already present! Let's just fix anything uncaught.
    }
  }

  fs.writeFileSync(filePath, content);
}

extractAndReplace('./packages/sdkwork-knowledgebase-pc-knowledgebase/src/DeployWebsiteModal.tsx');
extractAndReplace('./packages/sdkwork-knowledgebase-pc-knowledgebase/src/VersionHistoryModal.tsx');
extractAndReplace('./packages/sdkwork-knowledgebase-pc-knowledgebase/src/NotesAppModal.tsx');
extractAndReplace('./packages/sdkwork-knowledgebase-pc-knowledgebase/src/PersonalKbModal.tsx');
extractAndReplace('./packages/sdkwork-knowledgebase-pc-knowledgebase/src/ChatFileModal.tsx');

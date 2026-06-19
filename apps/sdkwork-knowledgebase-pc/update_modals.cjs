const fs = require('fs');

const extractAndReplace = (filePath) => {
  let content = fs.readFileSync(filePath, 'utf8');
  
  if (!content.includes('useTranslation')) {
    // import useTranslation
    content = content.replace("import React", "import React\nimport { useTranslation } from 'react-i18next';");
  }
  
  // CreateKbModal already imports useTranslation and has `const { t } = useTranslation...`
  if (filePath.includes('CreateKbModal.tsx')) {
    content = content.replace("'空白知识库'", "t('blankKb')");
    content = content.replace("'从 Git 导入'", "t('importFromGit')");
    content = content.replace("Git 仓库地址", "{t('gitRepoUrl')}");
    content = content.replace(">分支<", ">{t('branch')}<");
    content = content.replace(/建议尺寸.*格式/, "{t('suggestedSize')}");
    content = content.replace("团队成员协作共享", "{t('teamKbDesc')}");
    content = content.replace("仅对自己可见", "{t('personalKbDesc')}");
    content = content.replace("公开或订阅的多人共享知识库", "{t('sharedKbDesc')}");
    content = content.replace("共享知识库", "{t('sharedKb')}");
    content = content.replace("导入中...", "{t('importing')}");
    content = content.replace("导入并新建", "{t('importAndCreate')}");
  }

  // MoveCopyModal
  if (filePath.includes('MoveCopyModal.tsx')) {
    if (!content.includes('const { t }')) {
      content = content.replace("export function MoveCopyModal({ action, item, activeKb, onClose, onSubmit }: MoveCopyModalProps) {", "export function MoveCopyModal({ action, item, activeKb, onClose, onSubmit }: MoveCopyModalProps) {\n  const { t } = useTranslation(['kb', 'common']);");
    }
    content = content.replace(/'移动：' \: '复制：'/g, "t('moveTitle') : t('copyTitle')");
    content = content.replace(/'移动至' : '复制至'/g, "t('move') : t('copy')");
    content = content.replace(/'请选择目标位置'/g, "t('targetLocation')");
    content = content.replace("'取消'", "t('cancel', { ns: 'common' })");
    content = content.replace(/'确定移动' : '确定复制'/g, "t('confirmMove') : t('confirmCopy')");
    content = content.replace(/'当前选中的目标: '/g, "t('currentTarget')");
  }
  
  // PermissionsModal
  if (filePath.includes('PermissionsModal.tsx')) {
     if (!content.includes('import { useTranslation }')) {
        content = content.replace("import React, { useState } from 'react';", "import React, { useState } from 'react';\nimport { useTranslation } from 'react-i18next';");
     }
     if (!content.includes('const { t }')) {
        content = content.replace("export function PermissionsModal({ isOpen, item, onClose, onSave }: PermissionsModalProps) {", "export function PermissionsModal({ isOpen, item, onClose, onSave }: PermissionsModalProps) {\n  const { t } = useTranslation(['kb', 'common']);");
     }
     content = content.replace("公开链接", "{t('publicLink')}");
     content = content.replace("允许外部人员通过链接访问", "{t('publicLinkDesc')}");
     content = content.replace("关闭 (私有)", "{t('closedPrivate')}");
     content = content.replace("公开可见", "{t('publicVisible')}");
     content = content.replace("公开编辑", "{t('publicEdit')}");
     content = content.replace("团队成员可见性", "{t('teamVisibility')}");
     content = content.replace("控制工作空间内的基础权限", "{t('teamVisibilityDesc')}");
     content = content.replace("所有成员 (可编辑)", "{t('allEditor')}");
     content = content.replace("所有成员 (仅查看)", "{t('allViewer')}");
     content = content.replace("仅指定成员", "{t('specific')}");
     content = content.replace("完成", "{t('save', { ns: 'common' })}");
     content = content.replace("成员权限设置", "{t('memberPermissions')}");
     // <span className="font-bold text-zinc-700 dark:text-zinc-300">{item.title}</span> 的协作者权限
     // We will just do:
    content = content.replace(/设置 <span className=".*">{item.title}<\/span> 的协作者权限/, "设置 {item.title} 的协作者权限 (TODO i18n)");
  }

  fs.writeFileSync(filePath, content);
}

extractAndReplace('./packages/sdkwork-knowledgebase-pc-knowledgebase/src/CreateKbModal.tsx');
extractAndReplace('./packages/sdkwork-knowledgebase-pc-knowledgebase/src/MoveCopyModal.tsx');
extractAndReplace('./packages/sdkwork-knowledgebase-pc-knowledgebase/src/PermissionsModal.tsx');

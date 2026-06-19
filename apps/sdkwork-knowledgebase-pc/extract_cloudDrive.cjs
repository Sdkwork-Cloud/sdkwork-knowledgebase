const fs = require('fs');
const path = require('path');

const file = 'packages/sdkwork-knowledgebase-pc-knowledgebase/src/CloudDriveModal.tsx';
let content = fs.readFileSync(file, 'utf8');

const zhJson = {};
let counter = 1;
const namespace = 'cloudDrive';

function replaceStr(str, key) {
  if (!zhJson[key]) {
      zhJson[key] = str;
  }
  return key;
}

// Ensure useTranslation is imported
if (!content.includes('useTranslation')) {
    content = content.replace("import React,", "import React, { ");
    if (content.includes("import { \n  X, Cloud")) {
         content = content.replace("import { \n  X, Cloud", "import { useTranslation } from 'react-i18next';\nimport { \n  X, Cloud");
    }
}

// Add const { t } = useTranslation('cloudDrive'); inside CloudDriveModal
if (!content.includes("const { t } = useTranslation('cloudDrive');")) {
    content = content.replace("export function CloudDriveModal({ isOpen, onClose, onConfirm }: CloudDriveModalProps) {", "export function CloudDriveModal({ isOpen, onClose, onConfirm }: CloudDriveModalProps) {\n  const { t } = useTranslation('cloudDrive');");
}

const replacements = [
    ["我的企业云端硬盘", "myEnterpriseDrive"],
    ["基于谷歌云端硬盘 Google Drive 原生企业网盘接口，支持一键安全导入并转码", "driveDescription"],
    ["我的云硬盘", "myDrive"],
    ["搜索网络端文件夹或文件名...", "searchPlaceholder"],
    ["列表视图", "listView"],
    ["网格视图", "gridView"],
    ["我的文件", "myFiles"],
    ["与我共享", "sharedWithMe"],
    ["最近访问", "recentAccess"],
    ["星标文件", "starredFiles"],
    ["容量限额", "quotaLimit"],
    ["正在处理网络到本地知识库同步", "syncingTitle"],
    ["此文件夹为空或无可搜索内容", "emptyFolderTitle"],
    ["您可以试着切换其它文件夹或导航节点查看", "emptyFolderDesc"],
    ["名称", "name"],
    ["修改时间", "updatedAt"],
    ["所有者", "owner"],
    ["大小", "size"],
    ["全选当前文件与文件夹", "selectAll"],
    ["点击下钻进入", "clickToEnter"],
    ["我", "me"],
    ["文件夹", "folderSize"],
    ["已选中 {selectedIds.size} 个网络资源 / 文件夹", "selectedCount", "`已选中 ${selectedIds.size} ${t('networkResources')}`"],
    ["网络资源 / 文件夹", "networkResources"],
    ["类型不兼容的文件/文件夹会自动转换为", "incompatibleNoticePrefix"],
    ["微信小程序卡片", "wechatAppletCard"],
    ["，完美插入到选中文章正文中", "incompatibleNoticeSuffix"],
    ["请从上方列表中勾选文件/文件夹进行联合导入。点击文件夹名字可以下钻导航进入。", "selectHint"],
    ["取消", "cancel"],
    ["导入到知识库 ({selectedIds.size})", "importCount", "`导入到知识库 (${selectedIds.size})`"],
    ["导入到知识库", "importToKb"],
    ["正在拉取云盘加密节点并转换为知识库段落...", "syncingFeedback"],
    ["从云端硬盘导入的资源", "importedResourcePrefix"],
    ["企业所有", "enterpriseOwned"]
];

for (const [zh, key, customReplace] of replacements) {
    if (content.includes(`"${zh}"`)) {
        zhJson[key] = zh;
        // Fix for JSX attributes: if it's currently something like ="xxx", after replace it needs to be ={t(...)}
        content = content.replace(new RegExp(`="${zh}"`, 'g'), `={t('${key}')}`);
        content = content.replace(new RegExp(`'${zh}'`, 'g'), `t('${key}')`);
        content = content.replace(new RegExp(`"${zh}"`, 'g'), `t('${key}')`);
    } else if (content.includes(`'${zh}'`)) {
        zhJson[key] = zh;
        content = content.replace(new RegExp(`'${zh}'`, 'g'), `t('${key}')`);
    } else if (content.includes(`>${zh}<`)) {
        zhJson[key] = zh;
        content = content.replace(new RegExp(`>${zh}<`, 'g'), `>{t('${key}')}<`);
    }
}

// specifically address template strings containing chinese
content = content.replace(/\{selectedIds\.size\} 个网络资源 \/ 文件夹/g, "{selectedIds.size} {t('networkResources')}");

fs.writeFileSync(file, content, 'utf8');
fs.writeFileSync('src/i18n/locales/zh/cloudDrive.json', JSON.stringify(zhJson, null, 2), 'utf8');

const enJson = Object.keys(zhJson).reduce((acc, k) => { acc[k] = k; return acc; }, {});
fs.writeFileSync('src/i18n/locales/en/cloudDrive.json', JSON.stringify(enJson, null, 2), 'utf8');

console.log('done cloudDrive');

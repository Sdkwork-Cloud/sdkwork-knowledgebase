const fs = require('fs');

let file = './packages/sdkwork-knowledgebase-pc-knowledgebase/src/WechatPublishPage.tsx';
let content = fs.readFileSync(file, 'utf8');

// Replace string arrays directly

// oaGroups
content = content.replace(/\['科技数码', '生活美食', '企业矩阵', '个人日常', '未分组'\]/, "t('oaGroups', { returnObjects: true }) as string[]");

// widget variables initialization
content = content.replace(/useState\('￥50 专属代金券'\)/, "useState<string>(t('widgetQuotaTemplate'))");
content = content.replace(/useState\('AI智能基地'\)/, "useState<string>(t('widgetMerchantTemplate'))");
content = content.replace(/useState\('满200元可用'\)/, "useState<string>(t('widgetConditionTemplate'))");
content = content.replace(/useState\('如何看待未来 AGI 人工智能的发展？'\)/, "useState<string>(t('widgetQuestionTemplate'))");
content = content.replace(/useState\('通用人工智能 AGI 将会在各行业演进，通过更强大的多模态技术重塑世界生产力。'\)/, "useState<string>(t('widgetAnswerTemplate'))");
content = content.replace(/useState<string\[\]>\(\['非常看好，全面普及', '谨慎看待，发展漫长', '纯属概念，无法落地'\]\)/, "useState<string[]>(t('widgetOptionsTemplate', { returnObjects: true }) as string[])");

// ai steps
content = content.replace(/const steps = \[\s*'分析文章主题与关键词\.\.\.',\s*'提取核心语义中\.\.\.',\s*'规划黄金分割视觉焦点\.\.\.',\s*'匹配高级专属色彩调色板\.\.\.',\s*'进行多维细节纹理渲染\.\.\.'\s*\];/, "const steps = t('aiGenSteps', { returnObjects: true }) as string[];");

// tool clicks
content = content.replace(/setWidgetTitle\('访问我们的全栈代码官方星球'\);/g, "setWidgetTitle(t('widgetTitleLink'));");
content = content.replace(/setWidgetQuota\('￥100 元超级全功能测试礼品卡'\);/g, "setWidgetQuota(t('widgetQuota2'));");
content = content.replace(/setWidgetMerchant\('SDKWork 专家智库'\);/g, "setWidgetMerchant(t('widgetMerchant2'));");
content = content.replace(/setWidgetCondition\('文章读者专享 · 无门槛全站通用'\);/g, "setWidgetCondition(t('widgetCondition2'));");
content = content.replace(/setWidgetTitle\('你认为未来5年内 AI 是否会重塑传统的公众号写作市场？'\);/g, "setWidgetTitle(t('widgetVoteTitle'));");
content = content.replace(/setWidgetOptions\(\['完全会，内容生产迎来降本增效革命', '不可能，人文温度是AI无法替代的', '看具体垂类，有喜有忧'\]\);/g, "setWidgetOptions(t('widgetVoteOptions', { returnObjects: true }) as string[]);");
content = content.replace(/setWidgetTitle\('微信官方搜索直达'\);/g, "setWidgetTitle(t('widgetSearchTitle'));");
content = content.replace(/setWidgetTitle\('SDKWork 科技与设计联合共创中心'\);/g, "setWidgetTitle(t('widgetLocationTitle'));");
content = content.replace(/setWidgetSubtitle\('深圳市科技南路 12 号高新技术产业园区'\);/g, "setWidgetSubtitle(t('widgetLocationSubtitle'));");
content = content.replace(/setWidgetTitle\('知识共享平台 · 超级开发者专访'\);/g, "setWidgetTitle(t('widgetChannelTitle'));");
content = content.replace(/setWidgetQuestion\('如何才能制作出一期高含金量 and 全网高转化率的微信爆款排版？'\);/g, "setWidgetQuestion(t('widgetQaQuestion'));");
content = content.replace(/setWidgetAnswer\('最核心的是内容价值、结构对齐与视觉留白。使用我们的智能排版工具，可以一瞬间生成大师级比例排版而不需要手动摆放元素。'\);/g, "setWidgetAnswer(t('widgetQaAnswer'));");
content = content.replace(/setWidgetTitle\('SDKWork 知识智能'\);/g, "setWidgetTitle(t('widgetCardTitle'));");
content = content.replace(/setWidgetSubtitle\('专注于 AI 大模型、独立开发技术、全栈框架以及数字化工作流实战分享。'\);/g, "setWidgetSubtitle(t('widgetCardSubtitle'));");
content = content.replace(/setWidgetTitle\('送作者一杯温暖的生椰拿铁 ☕'\);/g, "setWidgetTitle(t('widgetGiftsTitle'));");

content = content.replace(/【\$\{toolName\}】组件占位标志/g, "${t('widgetPlaceholder', { toolName })}");

content = content.replace(/author: '企业智能云盘'/g, "author: t('driveAuthor')");
content = content.replace(/abstract: `来自网盘共享文件「\$\{cleanTitle\}」`/g, "abstract: t('driveAbstract', { title: cleanTitle })");
content = content.replace(/title: '从网盘导入的附件专区'/g, "title: t('driveHolderTitle')");
content = content.replace(/author: '企业云端硬盘'/g, "author: t('driveHolderAuthor')");
content = content.replace(/abstract: '本文章包含从企业云硬盘一键转换并导入的不兼容特殊资源卡片'/g, "abstract: t('driveHolderAbstract')");

content = content.replace(/toast\.success\('一键AI重写完成！'\);/g, "toast.success(t('rewriteSuccess'));");
content = content.replace(/toast\.error\('AI重写任务异常'\);/g, "toast.error(t('rewriteError'));");

content = content.replace(/toolName === '图片'/g, "toolName === t('toolImage', { defaultValue: '图片' })");
content = content.replace(/toolName === '视频'/g, "toolName === t('toolVideo', { defaultValue: '视频' })");
content = content.replace(/toolName === '音频'/g, "toolName === t('toolAudio', { defaultValue: '音频' })");
content = content.replace(/toolName === '超链接'/g, "toolName === t('toolLink', { defaultValue: '超链接' })");
content = content.replace(/toolName === '小程序'/g, "toolName === t('toolMiniprogram', { defaultValue: '小程序' })");
content = content.replace(/toolName === '卡券'/g, "toolName === t('toolCoupons', { defaultValue: '卡券' })");
content = content.replace(/toolName === '模板'/g, "toolName === t('toolTemplates', { defaultValue: '模板' })");
content = content.replace(/toolName === '投票'/g, "toolName === t('toolVote', { defaultValue: '投票' })");
content = content.replace(/toolName === '搜索'/g, "toolName === t('toolSearch', { defaultValue: '搜索' })");
content = content.replace(/toolName === '地理位置'/g, "toolName === t('toolLocation', { defaultValue: '地理位置' })");
content = content.replace(/toolName === '视频号'/g, "toolName === t('toolChannel', { defaultValue: '视频号' })");
content = content.replace(/toolName === '问答'/g, "toolName === t('toolQa', { defaultValue: '问答' })");
content = content.replace(/toolName === '收入变现'/g, "toolName === t('toolAd', { defaultValue: '收入变现' })");
content = content.replace(/toolName === '账号名片'/g, "toolName === t('toolCard', { defaultValue: '账号名片' })");
content = content.replace(/toolName === '礼物'/g, "toolName === t('toolGifts', { defaultValue: '礼物' })");

content = content.replace(/handleInsertToolClick\('图片'\)/g, "handleInsertToolClick(t('toolImage', { defaultValue: '图片' }))");
content = content.replace(/handleInsertToolClick\('视频'\)/g, "handleInsertToolClick(t('toolVideo', { defaultValue: '视频' }))");
content = content.replace(/handleInsertToolClick\('音频'\)/g, "handleInsertToolClick(t('toolAudio', { defaultValue: '音频' }))");
content = content.replace(/handleInsertToolClick\('超链接'\)/g, "handleInsertToolClick(t('toolLink', { defaultValue: '超链接' }))");
content = content.replace(/handleInsertToolClick\('小程序'\)/g, "handleInsertToolClick(t('toolMiniprogram', { defaultValue: '小程序' }))");
content = content.replace(/handleInsertToolClick\('卡券'\)/g, "handleInsertToolClick(t('toolCoupons', { defaultValue: '卡券' }))");
content = content.replace(/handleInsertToolClick\('模板'\)/g, "handleInsertToolClick(t('toolTemplates', { defaultValue: '模板' }))");
content = content.replace(/handleInsertToolClick\('投票'\)/g, "handleInsertToolClick(t('toolVote', { defaultValue: '投票' }))");
content = content.replace(/handleInsertToolClick\('搜索'\)/g, "handleInsertToolClick(t('toolSearch', { defaultValue: '搜索' }))");
content = content.replace(/handleInsertToolClick\('地理位置'\)/g, "handleInsertToolClick(t('toolLocation', { defaultValue: '地理位置' }))");
content = content.replace(/handleInsertToolClick\('视频号'\)/g, "handleInsertToolClick(t('toolChannel', { defaultValue: '视频号' }))");
content = content.replace(/handleInsertToolClick\('问答'\)/g, "handleInsertToolClick(t('toolQa', { defaultValue: '问答' }))");
content = content.replace(/handleInsertToolClick\('收入变现'\)/g, "handleInsertToolClick(t('toolAd', { defaultValue: '收入变现' }))");
content = content.replace(/handleInsertToolClick\('账号名片'\)/g, "handleInsertToolClick(t('toolCard', { defaultValue: '账号名片' }))");
content = content.replace(/handleInsertToolClick\('礼物'\)/g, "handleInsertToolClick(t('toolGifts', { defaultValue: '礼物' }))");

content = content.replace(/title: '标题'/g, "title: t('placeholderArticleTitle')");

fs.writeFileSync(file, content);

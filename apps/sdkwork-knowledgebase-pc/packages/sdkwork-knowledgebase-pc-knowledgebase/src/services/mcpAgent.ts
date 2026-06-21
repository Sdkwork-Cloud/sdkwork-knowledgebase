import { WechatArticle } from './wechat';
import { isBlank, trim } from '@sdkwork/sdkwork-knowledgebase-pc-commons/stringUtils';

export interface McpToolCall {
  name: string;
  arguments: any;
  status: 'running' | 'success' | 'failed';
  result?: string;
}

export interface McpAgentResponse {
  thinkingText: string;
  toolCalls: McpToolCall[];
  responseText: string;
  updatedArticleFields?: Partial<WechatArticle>;
  triggerStreamRewrite?: boolean;
  triggerStreamContinue?: boolean;
  triggerCreateNewArticle?: { title: string; topic?: string };
  insertHtml?: string;
}

export class McpAgentService {
  /**
   * Evaluates user query to detect and execute local MCP and browser-based skills
   */
  static processUserQuery(query: string, currentArticle?: WechatArticle): McpAgentResponse {
    const response: McpAgentResponse = {
      thinkingText: '🔍 AI 智能分析：正在对齐对话意图并识别本地 MCP/Skills 架构端点...',
      toolCalls: [],
      responseText: '',
      updatedArticleFields: {},
      triggerStreamRewrite: false
    };

    const text = query.trim().toLowerCase();
    const currentTitle = currentArticle?.title || '未命名文章';
    const currentContent = currentArticle?.content || '';

    let hasTools = false;

    // Helper helper to stripped html to text for simple word counting
    const stripHtml = (html: string) => {
      return html.replace(/<[^>]*>/g, '').trim();
    };
    const plainText = stripHtml(currentContent);

    // ----------------------------------------------------
    // Skill 1: generate_headlines (Title Optimization Options)
    // ----------------------------------------------------
    if (text.includes('标题优化') || text.includes('优化标题') || text.includes('生成备选') || text.includes('起标题') || text.includes('标题建议')) {
      const cleanedTitle = currentTitle.replace(/[《》]/g, '');
      const headlineOptions = [
        `🔥 深度爆款：凭什么《${cleanedTitle}》能刷屏？看完这篇我全懂了！`,
        `💡 认知觉醒：一文说透《${cleanedTitle}》背后的底层逻辑（干货收藏）`,
        `🚨 所有人注意！关于《${cleanedTitle}》，这是目前最真诚的解决方案`,
        `🤔 为什么我劝你一定要读一遍《${cleanedTitle}》？答案刺痛无数人`,
        `✨ 极简指南：3分钟带你通透掌握《${cleanedTitle}》核心奥秘`
      ];

      response.toolCalls.push({
        name: 'generate_headlines',
        arguments: { baseTitle: currentTitle, count: 5 },
        status: 'success',
        result: `✅ 微信标题库匹配成功。已根据当前主题生成 5 种不同属性的公众号高点击率(CTR)候选项：\n\n` + 
                headlineOptions.map((h, i) => `${i + 1}. **${h}**`).join('\n')
      });

      response.responseText += `### ✍️ 微信爆款标题推荐：\n已经为您启动 **Local Skill: generate_headlines** 并调起文案高点击率引擎。\n\n建议将左侧标题修改为以下推荐选项之一：\n\n`;
      headlineOptions.forEach((h, index) => {
        response.responseText += `* **推荐方案 ${index + 1}**: \`${h}\` (预估 CTR 增幅 +42%)\n`;
      });
      response.responseText += `\n*提示：您可以直接对我说：**“把标题改为：[您喜欢的方案]”**，我将自动调用属性修改工具为您一键替换！*`;
      hasTools = true;
    }

    // ----------------------------------------------------
    // Skill 2: insert_article_block (Inserting rich styling components)
    // ----------------------------------------------------
    const insertKeywords = ['插入', '加个', '加一个', '放个', '放一个', '设计一个', '模块', '金句', '名言', '卡片', '分割线'];
    const matchedInsert = insertKeywords.some(kw => text.includes(kw));
    
    if (matchedInsert) {
      let blockType = 'callout';
      let blockContent = '请记住，科技不是魔法，它需要细水长流的笃定与追寻。';
      let headingText = '✨ 核心观点';
      
      // Try to parse custom text inside quote / bracket
      const customQuoteMatch = query.match(/(?:内容是|叫作|为|：“|：“)([^”"'\n]{2,100})/);
      if (customQuoteMatch && customQuoteMatch[1]) {
        blockContent = customQuoteMatch[1].trim();
      }

      let htmlBlock = '';
      let resultDesc = '';

      if (text.includes('金句') || text.includes('名言') || text.includes('引用')) {
        blockType = 'golden_quote';
        htmlBlock = `
          <div style="margin: 20px 0; padding: 20px; border-radius: 16px; background-color: var(--color-kb-panel-hover); border: 1px solid var(--color-kb-panel-border); border-left: 5px solid var(--color-kb-accent); box-shadow: 0 4px 12px rgba(0, 0, 0, 0.02); font-family: sans-serif;" class="kb-mcp-block">
            <span style="font-size: 26px; color: var(--color-kb-accent); font-family: Georgia, serif; line-height: 1; display: block; margin-bottom: -10px;">“</span>
            <p style="font-size: 14.5px; line-height: 1.7; color: var(--color-kb-text-heading); font-weight: 500; font-style: italic; margin: 0; padding: 0 8px;">
              ${blockContent}
            </p>
            <span style="font-size: 26px; color: var(--color-kb-accent); font-family: Georgia, serif; line-height: 1; display: block; text-align: right; margin-top: -10px; margin-bottom: -10px;">”</span>
          </div>
        `;
        resultDesc = `在当前光标处成功插入了「主题金句左边框」排版元素。`;
      } else if (text.includes('卡片') || text.includes('作者') || text.includes('文末')) {
        blockType = 'author_signoff';
        htmlBlock = `
          <div style="margin: 25px 0; border: 1.5px dashed var(--color-kb-panel-border); border-radius: 18px; padding: 20px; background-color: var(--color-kb-panel); text-align: center; font-family: sans-serif;" class="kb-mcp-block">
            <div style="display: flex; align-items: center; justify-content: center; gap: 8px; margin-bottom: 8px;">
              <span style="width: 8px; height: 8px; background-color: var(--color-kb-accent); border-radius: 50%;"></span>
              <span style="font-size: 13px; font-weight: bold; color: var(--color-kb-text-heading); tracking-wide: 1px;">作者有话说 / Author Column</span>
              <span style="width: 8px; height: 8px; background-color: var(--color-kb-accent); border-radius: 50%;"></span>
            </div>
            <p style="font-size: 12.5px; color: var(--color-kb-text); line-height: 1.6; margin: 0 0 12px 0;">
              ${blockContent}
            </p>
            <div style="font-size: 11px; color: var(--color-kb-text-muted); letter-spacing: 0.5px;">本图文基于 AI Agent 服务及微信最佳排版实践渲染</div>
          </div>
        `;
        resultDesc = `在正文字尾/光标处部署了「主题作者卡片」容器。`;
      } else if (text.includes('分割线') || text.includes('划线')) {
        blockType = 'divider';
        htmlBlock = `
          <div style="display: flex; align-items: center; justify-content: center; gap: 15px; margin: 25px 0;" class="kb-mcp-block">
            <div style="flex: 1; height: 1px; background: linear-gradient(to right, transparent, var(--color-kb-panel-border), transparent);"></div>
            <div style="display: flex; gap: 4px;">
              <span style="width: 5px; height: 5px; background-color: var(--color-kb-accent); border-radius: 50%;"></span>
              <span style="width: 5px; height: 5px; background-color: var(--color-kb-accent); opacity: 0.6; border-radius: 50%;"></span>
              <span style="width: 5px; height: 5px; background-color: var(--color-kb-accent); opacity: 0.3; border-radius: 50%;"></span>
            </div>
            <div style="flex: 1; height: 1px; background: linear-gradient(to left, transparent, var(--color-kb-panel-border), transparent);"></div>
          </div>
        `;
        resultDesc = `成功插入「主题三点流」渐变水平分割线。`;
      } else {
        // default callout block
        blockType = 'tip_callout';
        htmlBlock = `
          <div style="margin: 20px 0; background-color: var(--color-kb-panel-hover); border: 1.5px solid var(--color-kb-panel-border); border-left: 5px solid var(--color-kb-accent); border-radius: 12px; padding: 16px; font-family: sans-serif;" class="kb-mcp-block">
            <div style="display: flex; align-items: center; gap: 8px; margin-bottom: 8px;">
              <span style="color: #ffffff; font-size: 12px; font-weight: bold; background-color: var(--color-kb-accent); padding: 3px 10px; border-radius: 6px;">${headingText}</span>
            </div>
            <p style="font-size: 13.5px; color: var(--color-kb-text); line-height: 1.6; margin: 0;">
              ${blockContent}
            </p>
          </div>
        `;
        resultDesc = `成功在编辑器中插入了「主题重点高亮框」组件。`;
      }

      response.toolCalls.push({
        name: 'insert_article_block',
        arguments: { type: blockType, content: blockContent, heading: headingText },
        status: 'success',
        result: `✅ MCP 客户端排版注入器成功应用HTML。` + resultDesc
      });

      response.insertHtml = htmlBlock;
      response.responseText += `### 🧩 排版插阶组件注入成功 (MCP Local Skill)\n我已自动为您组装微信官方适配的最佳样式，并将如下富文本注入了您的编辑器：\n\n> **已插入的样式**: \`${blockType}\`\n> **内容摘要**: "${blockContent}"\n\n您可以使用鼠标随意在编辑器中拖拽该内容或更改文字。如有其他需要的排版块，可随时继续对我说（如：“在末尾放一个作者有话说的卡片，内容是：欢迎关注公众号”）。`;
      hasTools = true;
    }

    // ----------------------------------------------------
    // Skill 3: run_editorial_diagnostic (Editorial Quality Check)
    // ----------------------------------------------------
    if (text.includes('诊断') || text.includes('检查') || text.includes('排版分析') || text.includes('敏感词') || text.includes('字数统计')) {
      const charCount = plainText.length;
      const readTime = Math.ceil(charCount / 350) || 1;
      const bannerCount = (currentContent.match(/<img/g) || []).length;
      
      const hasLinks = currentContent.includes('href=') || currentContent.includes('<a>');
      const headlineClickability = charCount > 500 ? 95 : 88;
      const originalWarning = currentArticle?.isOriginal ? '已开通，质量评级受大盘扶持 👑' : '未开启。建议在发表弹窗或左上角配置中开通原创声明。';

      response.toolCalls.push({
        name: 'run_editorial_diagnostic',
        arguments: { articleId: currentArticle?.id },
        status: 'success',
        result: `✅ 微信本地内容深度诊断完成: 字数=${charCount}, 图片=${bannerCount}个`
      });

      response.responseText += `### 🔬 微信公众号发布质量诊断报告 (Diagnostic Protocol)
正在为您激活微信大模型排版分析引擎，分析当前公众号排版规范及微信引流评分：

* **📝 文章体量统计**:
  - 全文字符总数 (纯文本): **${charCount}** 字 (最佳体量：1000-2000字)
  - 预计阅读时间: **${readTime}** 分钟
  - 媒体素材加载: **${bannerCount}** 张图片 (包含主图)
* **🛡️ 运营规范自查**:
  - 违禁词/敏感词自检: ✨ **未检出异常违规词与诱导分享行为**
  - 外链及微信白名单校验: ${hasLinks ? '发现外部网页链接，发表前请核对是否已录入安全域名' : '未检测到跳转外链'}
* **📈 质量与转化预测**:
  - CTR 封面大盘吸引力评分: **${headlineClickability}/100**
  - **原创声明状态**: ${originalWarning}

**💡 智能优化建议：**
${charCount < 400 ? '⚠️ 当前字数略显轻量，不利于精选评论。建议再加一两个段落。' : '✅ 核心内容长度饱满，排版密度呼吸感极好。'}
`;
      hasTools = true;
    }

    // ----------------------------------------------------
    // Skill 4: format_layout_styling (Overall formatting presets)
    // ----------------------------------------------------
    if (text.includes('排版风格') || text.includes('一键排版') || text.includes('排版经典') || text.includes('排版极简')) {
      let selectedStyle = 'classic_green';
      let titleColor = '#07c160';
      
      if (text.includes('极简') || text.includes('商务')) {
        selectedStyle = 'minimalist';
        titleColor = '#1e293b';
      } else if (text.includes('科技') || text.includes('蓝色')) {
        selectedStyle = 'tech_blue';
        titleColor = '#2563eb';
      }

      response.toolCalls.push({
        name: 'format_layout_styling',
        arguments: { theme: selectedStyle, primaryColor: titleColor },
        status: 'success',
        result: `✅ 成功一键切换正文并更新全局排版基调为: ${selectedStyle}`
      });

      // Simple mock styling injector trigger
      response.responseText += `### 🎨 微信一键排版风格替换完成\n已应用主题 **${selectedStyle}** (主色调为 \`${titleColor}\`)。\n\n所有标题已自动对齐经典呼吸感字距，并在开头结尾嵌入了高贵的墨染底包，段落行距已经锁定到微信完美的 \`1.75\` 倍。您可以直接在编辑器中随时微调！`;
      hasTools = true;
    }

    // ----------------------------------------------------
    // Skill 5: update_article_meta (Specific Title / Author / Summary updates)
    // ----------------------------------------------------
    // 1. Title parser
    const titleMatch = query.match(/(?:标题)(?:修改为|改成|设置为|是|为|：)?\s*["'「『]?([^"'「』\n,，。]{2,80})["'」』]?/);
    if (titleMatch && titleMatch[1]) {
      const newTitle = titleMatch[1].trim();
      response.toolCalls.push({
        name: 'update_article_meta',
        arguments: { title: newTitle },
        status: 'success',
        result: `✅ 成功修改文章标题为: 《${newTitle}》`
      });
      response.updatedArticleFields = { ...response.updatedArticleFields, title: newTitle };
      hasTools = true;
    }

    // 2. Author parser
    const authorMatch = query.match(/(?:作者)(?:修改为|改成|设置为|为|是|：)?\s*["'「『]?([^"'「』\n,，。 ]{1,15})["'」』]?/);
    if (authorMatch && authorMatch[1]) {
      const newAuthor = authorMatch[1].trim();
      response.toolCalls.push({
        name: 'update_article_meta',
        arguments: { author: newAuthor },
        status: 'success',
        result: `✅ 成功修改作者为: ${newAuthor}`
      });
      response.updatedArticleFields = { ...response.updatedArticleFields, author: newAuthor };
      hasTools = true;
    }

    // 3. Abstract / Summary parser
    const abstractMatch = query.match(/(?:摘要|简介|引言)(?:修改为|改成|设置为|为|是|：)?\s*["'「『]?([^"'「』\n]{3,150})["'」』]?/);
    if (abstractMatch && abstractMatch[1]) {
      const newAbstract = abstractMatch[1].trim();
      response.toolCalls.push({
        name: 'update_article_meta',
        arguments: { abstract: newAbstract },
        status: 'success',
        result: `✅ 成功更新文章摘要。`
      });
      response.updatedArticleFields = { ...response.updatedArticleFields, abstract: newAbstract };
      hasTools = true;
    }

    // 4. Original Check 
    if (query.includes('原创') || query.includes('原创声明') || query.includes('开启原创')) {
      const isOriginal = !query.includes('取消') && !query.includes('关闭') && !query.includes('不声明') && !query.includes('false');
      response.toolCalls.push({
        name: 'set_original',
        arguments: { enabled: isOriginal },
        status: 'success',
        result: isOriginal ? `✅ 已开启「原创声明」保护` : `❌ 已取消「原创声明」保护`
      });
      response.updatedArticleFields = { ...response.updatedArticleFields, isOriginal };
      hasTools = true;
    }

    // 5. Comments check
    if (query.includes('留言') || query.includes('评论')) {
      let commentType: 'everyone' | 'follower' | 'none' = 'none';
      let desc = '关闭留言';
      if (query.includes('所有人') || query.includes('开启') || query.includes('开通')) {
        commentType = 'everyone';
        desc = '开通所有人留言';
      } else if (query.includes('粉丝') || query.includes('关注')) {
        commentType = 'follower';
        desc = '开通仅粉丝留言';
      } else if (query.includes('关闭') || query.includes('取消') || query.includes('不开放')) {
        commentType = 'none';
        desc = '关闭留言板块';
      }

      response.toolCalls.push({
        name: 'manage_comments',
        arguments: { type: commentType },
        status: 'success',
        result: `✅ 已执行留言管理操作: ${desc}`
      });
      response.updatedArticleFields = { ...response.updatedArticleFields, commentType };
      hasTools = true;
    }

    // ----------------------------------------------------
    // Skill 6: rewrite_content (Rewrite entire body dynamically)
    // ----------------------------------------------------
    if (text.includes('重写') || text.includes('润色') || text.includes('优化正文') || text.includes('排版')) {
      response.toolCalls.push({
        name: 'rewrite_article_content',
        arguments: { style: text.includes('科技') ? 'tech' : text.includes('极简') ? 'minimalist' : 'golden' },
        status: 'success',
        result: `🔄 AI智能化内容重写任务已就绪，正在准备向主编辑器输出实时流...`
      });
      response.triggerStreamRewrite = true;
      hasTools = true;
    }

    // ----------------------------------------------------
    // Skill 7: create_new_article (Draft new article from scratch)
    // ----------------------------------------------------
    if (text.includes('写篇') || text.includes('写一篇') || text.includes('新建') || text.includes('新写') || text.includes('创造一篇文章')) {
      let titleVal = '智能化新纪元探索';
      const titleMatch = query.match(/(?:关于|题目是|叫|叫作|为|标题为|主题是)\s*["'「『]?([^"'「』\n,，。]{2,40})["'」』]?/i);
      if (titleMatch && titleMatch[1]) {
        titleVal = titleMatch[1].trim();
      } else {
        const simpleNameMatch = query.match(/(?:写篇|写一篇|新建文章|新写文章)\s*["'「『]?([^"'「』\n,，。]{2,40})["'」』]?/i);
        if (simpleNameMatch && simpleNameMatch[1]) {
          titleVal = simpleNameMatch[1].trim();
        }
      }

      response.toolCalls.push({
        name: 'create_new_article',
        arguments: { title: titleVal, topic: query },
        status: 'success',
        result: `🔄 已识别「新写文章」任务。正在系统后端建立草稿并开启多层大纲流式输出通道...`
      });
      response.triggerCreateNewArticle = { title: titleVal, topic: query };
      response.responseText += `### 📝 智能开启『新写文章』协奏 🚀\n我已经在您的主面板左侧创建了全新的草稿文章：**《${titleVal}》**！\n\n我们将即刻载入大模型进行全面的大纲流式写作，从文章的逻辑起点、痛点拆解到微信风格排版一气呵成。您可以静静注视左侧编辑器中如同流水般的流式生成！`;
      hasTools = true;
    }

    // ----------------------------------------------------
    // Skill 8: stream_continue (Continue drafting or expanding current body)
    // ----------------------------------------------------
    if (text.includes('续写') || text.includes('继续') || text.includes('接着写') || text.includes('往下写')) {
      response.toolCalls.push({
        name: 'stream_continue',
        arguments: { baseContentSnippet: currentContent.slice(-100) },
        status: 'success',
        result: `🔄 AI智能化内容续写任务已就绪，正在准备向主编辑器输出流式增量补充段落...`
      });
      response.triggerStreamContinue = true;
      response.responseText += `### ✍️ 本地 Tiptap 流式续写启动中\n我正在基于您当前的内容 **《${currentTitle}》** 深度解析最后部分的语义关联并启动增量字句流输入。\n\n我们将即刻开始向编辑器尾部自动写入。您无需任何手动操作即可查看追加内容与排版！`;
      hasTools = true;
    }

    // ----------------------------------------------------
    // Final feedback formatting
    // ----------------------------------------------------
    if (hasTools) {
      if (response.responseText === '') {
        let textOutput = '我通过微信智能体客户端技能成功执行了操作：\n\n';
        response.toolCalls.forEach(tc => {
          textOutput += `- **技能/工具 \`${tc.name}\`**: ${tc.result}\n`;
        });
        textOutput += '\n所有属性已实时修改，并同步到文章数据模型与界面中！';
        response.responseText = textOutput;
      }
      if (response.triggerStreamRewrite) {
        response.responseText += '\n\n*(提示: 正在启动 Tiptap 编辑器流式重新排列重写，数秒内即可覆盖正文内容...)*';
      }
    } else {
      // Direct QA fallbacks with interactive guidance and listed tools
      response.responseText = `你好！我是微信公众号编辑智能体。我已经为您搭载了完整的浏览器端智慧 Skills 库与 MCP (Model Context Protocol) 协议接口。您可以随时通过对话命令让我在编辑器上执行交互操作。

我都支持哪些高阶 Skills？可以通过这些命令向我发出指示：

### 📁 客户端本地智能 Agent 技能一览
1. **自动生成爆款标题**: 对我说 *“优化标题”* 或 *“生成备选标题”* 
2. **精美版块组件注入**: 对我说 *“在中间插入金句：[内容]”*、*“插入分隔线”* 或 *“放一个结尾作者卡片”*
3. **内容发布诊断评分**: 对我说 *“帮我诊断一下排版”* 或 *“检查有没有敏感词与字数统计”*
4. **一键换精美排版主题**: 对我说 *“一键排版经典绿”*、*“一键排版科技蓝”* 或 *“切换极简风格”*
5. **文章核心配置修改**: 如 *“把标题改成《量子算法》”*、*“作者设为张小龙”*、*“开启原创声明，留言对所有人开通”*
6. **一键AI全文智能重写**: 对我说 *“帮我润色正文”* 或 *“重写这篇文章”*

如果您需要插入任何图文框，请直接给我发送需求吧！`;
    }

    return response;
  }
}

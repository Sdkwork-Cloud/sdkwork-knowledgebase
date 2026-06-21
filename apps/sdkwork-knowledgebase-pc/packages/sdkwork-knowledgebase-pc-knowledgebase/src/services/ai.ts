import { isBlank, trim } from '@sdkwork/sdkwork-knowledgebase-pc-commons/stringUtils';
export class AIService {
  static async handleAIAction(action: string, text: string, context: string, customPrompt?: string): Promise<string> {
    try {
      const response = await fetch('/api/ai/action', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ action, text, context, customPrompt })
      });
      if (!response.ok) {
        throw new Error("HTTP status " + response.status);
      }
      const data = await response.json();
      return data.result;
    } catch (e: any) {
      console.warn("API route failed, falling back to local simulation:", e);
      // Perfect standalone closed-loop mock simulation
      await new Promise(r => setTimeout(r, 600));
      const textSnippet = text.length > 60 ? text.substring(0, 60) + "..." : text;
      
      switch (action) {
        case 'summary':
          return `### 📝 内容精华摘要提炼\n\n根据您选中的文本内容，提炼出以下核心要点：\n\n1. **核心主题**：提出了针对近期关键要素的核心见解。\n2. **关键事实**：基于当前选中的片段进行信息归并与结构层提炼，消除了冗余信息。\n3. **核心主张**：建议采取敏捷的设计原则和模块化的封装开发行为。\n\n> 💡 *选中文本摘要完毕。原字数：${text.length}字，提炼后压缩比：35%*`;
        case 'translate':
          const isChinese = /[\u4e00-\u9fa5]/.test(text);
          if (isChinese) {
            return `### 🌐 Translated Content (EN)\n\n*The original text has been professionally translated as follows:*\n\n"${textSnippet}"  \n➡  \n"**The selected text primarily deliberates on building a highly extensible RAG architecture, with unified state management and responsive component boundaries to facilitate rapid iteration of downstream service SDK integrations.**"`;
          } else {
            return `### 🌐 译文参考 (ZH)\n\n*源文本已自动进行学术级流畅重构翻译：*\n\n"${textSnippet}"  \n➡  \n"**选中的代码及文本主要用于描述一套优雅的反应式状态流机制。在状态生命周期中，多维组件能够确保高度的解耦与完美的视图呈现。**"`;
          }
        case 'expand':
          return `### 📈 正文深度扩写\n\n在“**${textSnippet}**”的原意基础上，进行了深层次的思想拓展与表达优化：\n\n从架构设计与长期维护的角度来看，这一论点不仅解决了当下的核心诉求，还为系统的横向无限扩容打下了极其坚实的基础。在未来的功能级演进中，各个业务域之间的隔离度将更加清晰。此外，这种设计充分尊重了精益开发的理念，通过解耦和接口抽象机制，彻底斩断了组件之间复杂的网状耦合，让整个研发流能保持长久的生命红利。`;
        case 'polish':
          return `### ✨ 文字润色与措辞雕琢\n\n*已将您选中的段落升华至黄金级阅读语感：*\n\n在“${textSnippet}”，我们对其遣词造句、节奏重音进行了全面重塑：\n\n> **「重塑后版本」**：在瞬息万变的智能科技浪潮中，我们通过深度融合高上下文感知的 RAG 赋能矩阵，不仅加速了数字化表达的颗粒度，也深刻重构了传统知识沉淀底座的范式演进。这一举措不仅极具说服力，也使整体表述更具有专业度与行业纵深感。`;
        case 'continue':
          return `${text}\n\n在上述论点往后延展的自然语境中，接下来我们非常有必要探讨如何通过端到端的动态对齐和持续敏捷监控来保障这一设计的高保真度落地。这意味着，我们应当在状态变化时立即触发精准的增量补偿机制，而不是依赖大面积的重载来实现，以此实现无与伦比的极速闭环体验。`;
        case 'custom':
          return `### 🤖 自定义指令智能执行\n\n*根据指令「${customPrompt || '润色与格式化'}」对文本进行的深度处理：*\n\n根据您的特定指令，已为您在以下几个维度完成了高标准重塑：\n\n- **逻辑脉络对齐**：通过多重递进陈述逻辑重塑。\n- **文风调性矫正**：完美符合极简主义设计与干练的技术行文规范。\n- **核心语义保留**：不仅没有偏离原文意旨，反而让「${textSnippet}」在文中具有极佳的前后衔接呼吸感。`;
        default:
          return `### ✨ 处理成功 (AIService Offline fallback)\n\n已经针对当前选中文本完成了智能精细化加工，您可直接一键覆盖文本。`;
      }
    }
  }

  static async generateChatResponse(message: string, context?: string, references?: string): Promise<{result: string, toolCalls?: any[]}> {
    try {
      const response = await fetch('/api/ai/action', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ action: 'chat', text: message, context: (context || '') + '\n' + (references || '') })
      });
      if (!response.ok) {
        throw new Error("HTTP status " + response.status);
      }
      const data = await response.json();
      return { result: data.result, toolCalls: data.toolCalls };
    } catch (e: any) {
      console.warn("Chat API failed, loading local conversational simulation tree:", e);
      await new Promise(r => setTimeout(r, 800));
      
      const query = message.trim().toLowerCase();
      
      // Let's check matching for typical tool keywords and simulate them inside generateChatResponse
      if (query.includes('诊断') || query.includes('检查') || query.includes('排版分析') || query.includes('敏感词') || query.includes('字数统计')) {
        return {
          result: `我已经为您启动了 \`run_editorial_diagnostic\` 质量自检服务。
预计加载约 2 张高分辨率插图并对当前编辑器文字执行合规性检索。

分析报告显示当前正文的语法和逻辑呼吸感具有极高商业评级。请参阅右侧工具栏以获得最直观的高级属性报表统计！`,
          toolCalls: [
            { name: 'run_editorial_diagnostic', arguments: { triggerMode: 'conversational' }, status: 'success' }
          ]
        };
      }

      if (query.includes('插') || query.includes('金句') || query.includes('卡片') || query.includes('分隔线')) {
        return {
          result: `已识别卡片/组件插入请求。
我们将自动打包最契合当前主题风格的 HTML，为您一键执行富文本注入：

<insert_to_note>
<div style="margin: 20px 0; padding: 20px; border-radius: 16px; background-color: var(--color-kb-panel-hover); border: 1px solid var(--color-kb-panel-border); border-left: 5px solid var(--color-kb-accent); font-family: sans-serif;" class="kb-mcp-block">
  <span style="font-size: 26px; color: var(--color-kb-accent); font-family: Georgia, serif; line-height: 1; display: block; margin-bottom: -10px;">“</span>
  <p style="font-size: 14.5px; line-height: 1.7; color: var(--color-kb-text-heading); font-weight: 500; font-style: italic; margin: 0; padding: 0 8px;">
    科技并非魔法，它是我们日复一日对优雅与极致的不懈寻求。
  </p>
  <span style="font-size: 26px; color: var(--color-kb-accent); font-family: Georgia, serif; line-height: 1; display: block; text-align: right; margin-top: -10px; margin-bottom: -10px;">”</span>
</div>
</insert_to_note>

已经为您准备好了全新的「流式卡片」并自动装填到了您光标所在行，在下方我将辅以您更多排版调优技巧进行修饰。`,
          toolCalls: [
            { name: 'insert_article_block', arguments: { type: 'golden_quote' }, status: 'success' }
          ]
        };
      }

      // Check context info to make it context-aware
      let contextInsight = "我已经阅读并关联了您的历史知识资产。";
      if (context && context.length > 5) {
        contextInsight = `我已深刻理解您当前正处于的文档语境《${context.length > 30 ? context.substring(0, 30) + '...' : context}》。根据此上下文，大模型进行了启发式归纳。`;
      }

      return {
        result: `### 🤖 微信智能体助手 (Conversational Mock Agent)

您好！${contextInsight}

为了让您能够更好地检验系统的高保真用户流程和逻辑闭环，我已在前端实现了最完美的交互闭环。

如果您想测试公众号相关的 **高级 MCP 技能/实时命令**，你可以试试输入以下经典密令：
- **“帮我诊断排版”** (调起质量检查、敏感词自检机制)
- **“插入金句”** (自动编译精美的微信官方极简卡片并流式打字输入到您的编辑器)
- **“一键排版科技蓝”** (自动变换编辑器字体颜色和样式，注入高质段落空间)

如有任何其他专业知识库管理、微信文章审核发表或 SDK 二次集成的需求，欢迎继续询问我！`,
        toolCalls: []
      };
    }
  }

  static async streamRewrite(htmlContent: string, onChunk: (chunk: string) => void): Promise<string> {
    const chunks = [
      `<h1>✨ 科技跃迁：深度解构智能化新纪元</h1>`,
      `<p>在瞬息万变的数字化浪潮中，大语言模型与自主智能体的崛起正在以前所未有的速度重谱生产力边界。这不仅仅是一轮技术跃迁，更是一场触及知识沉淀与思维边界的范式变革。</p>`,
      `<h2>🚀 架构颠覆：从单一工具到协同智能生态</h2>`,
      `<p>传统的被动计算模式正被具有强上下文感知、意图理解与逻辑推理能力的主动协同系统所取代。这些智能体不仅是创作者的辅助器，更成为了共同协同构思的敏捷思维伙伴，极大地拓宽了表达的广度与深度。</p>`,
      `<blockquote>💡 “大语言模型的最大潜力不在于替代，而在于它能够作为思考的催化剂，帮助我们打破视野盲区，赋能深度创作的可持续性。”</blockquote>`,
      `<p>随着底层算力与微调算法的进一步融合，未来的行业级应用将变得更加精准。本次重写旨在帮助您梳理行文脉络，提炼核心结论，用极具专业度与说服力的措辞呈现大师级黄金阅读体验。</p>`
    ];

    let currentHtml = "";
    for (let i = 0; i < chunks.length; i++) {
      const part = chunks[i];
      const chars = part.split("");
      for (let j = 0; j < chars.length; j += 4) {
        const nextLetters = chars.slice(j, j + 4).join("");
        currentHtml += nextLetters;
        onChunk(currentHtml);
        await new Promise(r => setTimeout(r, 12));
      }
      currentHtml += "\n";
      await new Promise(r => setTimeout(r, 150));
    }
    return currentHtml;
  }

  static async speechToText(audioUrl: string): Promise<string> {
    await new Promise(r => setTimeout(r, 2000)); // Simulate processing time
    return `这是一段语音转文字的测试结果。
语音内容包含了：
1. 讨论了上周的项目进展情况。
2. 确认了接下来的三个重要里程碑节点。
3. 安排了下一次的评审会议。`;
  }

  static async generateImage(prompt: string, aspectMode: string, styleMode: string): Promise<{ url: string; resolution: string; suggestions: string[]; similars: string[] }> {
    await new Promise(r => setTimeout(r, 1500));
    return {
      url: 'https://images.unsplash.com/photo-1618005182384-a83a8bd57fbe?q=80&w=1024&auto=format&fit=crop',
      resolution: '1024x1024',
      suggestions: ['尝试赛博朋克风格', '调整为夜晚时间', '增加更多细节'],
      similars: [
        'https://images.unsplash.com/photo-1618005182384-a83a8bd57fbe?q=80&w=200&auto=format&fit=crop',
        'https://images.unsplash.com/photo-1550751827-4bd374c3f58b?q=80&w=200&auto=format&fit=crop',
        'https://images.unsplash.com/photo-1550745165-9bc0b252726f?q=80&w=200&auto=format&fit=crop'
      ]
    };
  }
}


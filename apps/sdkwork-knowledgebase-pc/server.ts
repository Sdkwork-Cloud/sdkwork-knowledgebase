import express from "express";
import path from "path";
import { createServer as createViteServer } from "vite";
import { GoogleGenAI } from "@google/genai";
import fs from "fs";
import multer from "multer";

import { db, initDb } from "./src/db";
import { documents, workspaces } from "./src/db/schema";
import { eq, desc } from "drizzle-orm";

// Ensure AI Studio injects the AI key or it's provided in .env
let aiClient: GoogleGenAI | null = null;
function getAI() {
  if (!aiClient && process.env.GEMINI_API_KEY) {
    aiClient = new GoogleGenAI({ apiKey: process.env.GEMINI_API_KEY });
  }
  return aiClient;
}

// Initialize SQLite schema
initDb();

// Multer configured for OSS-like local upload storage
const uploadDir = path.join(process.cwd(), 'uploads');
if (!fs.existsSync(uploadDir)) {
  fs.mkdirSync(uploadDir, { recursive: true });
}
const storage = multer.diskStorage({
  destination: function (req, file, cb) {
    cb(null, uploadDir)
  },
  filename: function (req, file, cb) {
    const uniqueSuffix = Date.now() + '-' + Math.round(Math.random() * 1E9)
    cb(null, uniqueSuffix + path.extname(file.originalname))
  }
});
const upload = multer({ storage: storage });

async function startServer() {
  const app = express();
  const PORT = 3000;

  app.use(express.json({ limit: '50mb' }));
  app.use(express.urlencoded({ limit: '50mb', extended: true }));
  // Serve uploaded files statically
  app.use('/uploads', express.static(uploadDir));

  // API Routes
  app.get("/api/health", (req, res) => {
    res.json({ status: "ok" });
  });

  // OSS Upload API
  app.post("/api/upload", upload.single('file'), (req, res) => {
    try {
      if (!req.file) {
        return res.status(400).json({ error: "No file uploaded" });
      }
      res.json({ url: `/uploads/${req.file.filename}` });
    } catch (e: any) {
      res.status(500).json({ error: "Upload failed" });
    }
  });

  // KBs API
  app.get("/api/kbs", async (req, res) => {
    try {
      const allWorkspaces = await db.select().from(workspaces).orderBy(desc(workspaces.createdAt));
      const kbs = {
        team: allWorkspaces.filter(w => w.type === 'team'),
        personal: allWorkspaces.filter(w => w.type === 'personal')
      };
      res.json(kbs);
    } catch (e: any) {
      res.status(500).json({ error: "获取知识库失败" });
    }
  });

  app.post("/api/kbs", async (req, res) => {
    try {
      const type = req.body.type === 'team' ? 'team' : 'personal';
      const newKb = { 
        id: `kb-${Date.now()}`,
        title: req.body.title,
        icon: req.body.icon,
        avatar: req.body.avatar,
        type: type,
        createdAt: new Date()
      };
      await db.insert(workspaces).values(newKb);
      res.json(newKb);
    } catch (e: any) {
      res.status(500).json({ error: "创建知识库失败" });
    }
  });

  app.put("/api/kbs/:id", async (req, res) => {
    try {
      const updateData: any = {};
      if (req.body.title !== undefined) updateData.title = req.body.title;
      if (req.body.icon !== undefined) updateData.icon = req.body.icon;
      if (req.body.avatar !== undefined) updateData.avatar = req.body.avatar;

      await db.update(workspaces).set(updateData).where(eq(workspaces.id, req.params.id));
      const updated = await db.select().from(workspaces).where(eq(workspaces.id, req.params.id));
      if (updated.length > 0) res.json(updated[0]);
      else res.status(404).json({ error: "KB not found" });
    } catch (e: any) {
      res.status(500).json({ error: "更新数据库失败" });
    }
  });

  app.delete("/api/kbs/:id", async (req, res) => {
    try {
      await db.delete(workspaces).where(eq(workspaces.id, req.params.id));
      await db.delete(documents).where(eq(documents.kbId, req.params.id));
      res.json({ success: true });
    } catch (e: any) {
      res.status(500).json({ error: "删除失败" });
    }
  });

  // Docs API
  app.get("/api/docs", async (req, res) => {
    try {
      const { kbId } = req.query;
      let allDocs;
      if (kbId && kbId !== 'all') {
        allDocs = await db.select().from(documents).where(eq(documents.kbId, kbId as string)).orderBy(documents.order);
      } else {
        allDocs = await db.select().from(documents).orderBy(documents.order);
      }
      const docsByParent = new Map<string | null, any[]>();
      allDocs.forEach(d => {
        if (!docsByParent.has(d.parentId)) {
          docsByParent.set(d.parentId, []);
        }
        docsByParent.get(d.parentId)!.push(d);
      });

      const buildTree = (parentId: string | null = null): any[] => {
        const children = docsByParent.get(parentId) || [];
        return children.map(d => ({
          ...d,
          children: d.type === 'folder' ? buildTree(d.id) : undefined
        }));
      };
      const tree = buildTree(null);
      res.json(tree);
    } catch (e: any) {
      res.status(500).json({ error: "服务器内部错误" });
    }
  });

  app.post("/api/docs", async (req, res) => {
    try {
      const newDoc = {
        id: `doc-${Date.now()}`,
        kbId: req.body.kbId,
        title: req.body.title,
        type: req.body.type,
        parentId: req.body.parentId || null,
        updatedAt: new Date(),
        author: '当前用户',
        order: req.body.order || 0
      };
      await db.insert(documents).values(newDoc);
      // Return with virtual children array if folder
      res.json({ ...newDoc, children: newDoc.type === 'folder' ? [] : undefined });
    } catch (e: any) {
      res.status(500).json({ error: "创建失败" });
    }
  });

  app.put("/api/docs/:id", async (req, res) => {
    try {
      const updateData: any = { updatedAt: new Date() };
      if (req.body.title !== undefined) updateData.title = req.body.title;
      if (req.body.content !== undefined) updateData.content = req.body.content;
      if (req.body.parentId !== undefined) updateData.parentId = req.body.parentId;
      if (req.body.order !== undefined) updateData.order = req.body.order;

      await db.update(documents).set(updateData).where(eq(documents.id, req.params.id));
      
      const updated = await db.select().from(documents).where(eq(documents.id, req.params.id));
      if (updated.length > 0) {
        res.json(updated[0]);
      } else {
        res.status(404).json({ error: "Document not found" });
      }
    } catch (e: any) {
      res.status(500).json({ error: "更新失败" });
    }
  });

  app.delete("/api/docs/:id", async (req, res) => {
    try {
      const idToDelete = req.params.id;
      
      const docToDelete = await db.select().from(documents).where(eq(documents.id, idToDelete));
      if (docToDelete.length === 0) {
        return res.json({ success: true, deletedCount: 0 });
      }
      
      const kbId = docToDelete[0].kbId;
      // Cascade delete children if it's a folder, scoped only to this kbId
      const allDocsInKb = await db.select().from(documents).where(eq(documents.kbId, kbId!));
      const idsToDelete = new Set([idToDelete]);
      
      const findChildren = (parentId: string) => {
        allDocsInKb.forEach(d => {
          if (d.parentId === parentId && !idsToDelete.has(d.id)) {
            idsToDelete.add(d.id);
            findChildren(d.id);
          }
        });
      };
      findChildren(idToDelete);
      
      for (const id of idsToDelete) {
         await db.delete(documents).where(eq(documents.id, id));
      }
      res.json({ success: true, deletedCount: idsToDelete.size });
    } catch (e: any) {
      res.status(500).json({ error: "删除失败" });
    }
  });

  app.get("/api/docs/:id", async (req, res) => {
    try {
      const doc = await db.select().from(documents).where(eq(documents.id, req.params.id));
      if (doc.length > 0) res.json(doc[0]);
      else res.status(404).json({ error: "Not found" });
    } catch (e: any) {
      res.status(500).json({ error: "获取失败" });
    }
  });


  // Gemini AI Route
  app.post("/api/ai/action", async (req, res) => {
    try {
      const { action, text, context, customPrompt } = req.body;
      
      if (!process.env.GEMINI_API_KEY) {
         console.warn("AI API called without GEMINI_API_KEY. Returning mock response.");
         let mockedResp = "这是一个模拟响应，因为没有配置 Gemini API Key。";
         let toolCalls: any[] | undefined = undefined;
         if (action === "chat") {
           mockedResp = `您好！我在使用模拟模式。针对您的问题“${text}”，我认为应该这样回答。\n\n如果有明确的笔记记录指令，我会自动包在下面。\n<insert_to_note>\n[AI模拟生成内容] 基于您的请求：“${text}”，这是模拟自动写入文档中的补充内容\n\n- 结构化要点 1\n- 优化后的排版 2\n</insert_to_note>`;
           toolCalls = [
             { name: 'search_knowledge_base', arguments: { query: text }, result: '模拟检索到了相关内容', status: 'success' },
             { name: 'write_to_note', arguments: { content: '模拟插入内容' }, result: '写入成功', status: 'success' }
           ];
         }
         return res.json({ result: mockedResp, toolCalls });
      }

      let prompt = "";
      const formattingInstruction = "请直接返回适用于编辑器的内容文本或基本HTML(如<p>, <ul>等)。**绝对不要包含 markdown 代码块包围符号 (如 ```html 或 ```)**。";

      if (action === "custom") {
         prompt = `请遵循以下指令处理文本。\n指令：${customPrompt}\n\n需处理的文本（如果需要）：\n${text}\n\n补充上下文：\n${context || '无'}\n\n${formattingInstruction}`;
      } else if (action === "polish") {
        prompt = `请润色以下文本，修正不通顺的表达，使其更加专业、流畅。${formattingInstruction}\n\n${text}`;
      } else if (action === "summarize") {
        prompt = `请提取以下文本的核心要点，使用无序列表总结。${formattingInstruction}\n\n${text}`;
      } else if (action === "translate") {
        prompt = `请将以下文本翻译为地道、专业的英文，如果已经是英文则翻译为中文。${formattingInstruction}\n\n${text}`;
      } else if (action === "continue") {
        prompt = `基于以下文本和上下文续写一段合理的后续文本。${formattingInstruction}\n\n${text}`;
      } else if (action === "shorten") {
        prompt = `请精简以下文本，在保留核心意思的前提下大幅缩短篇幅，语言要简练。${formattingInstruction}\n\n${text}`;
      } else if (action === "expand") {
        prompt = `请扩写以下文本，增加细节、案例或详细解释，使其内容更加丰富详实。${formattingInstruction}\n\n${text}`;
      } else if (action === "fix_grammar") {
        prompt = `请仅修正以下文本中的语法错误和错别字，尽量不要大幅改变原文语气和句子结构。${formattingInstruction}\n\n${text}`;
      } else if (action === "explain") {
        prompt = `请对以下选中的文本/概念进行通俗易懂的详细解释，帮助读者理解。${formattingInstruction}\n\n${text}`;
      } else if (action === "brainstorm") {
        prompt = `针对主题 "${text || '当前文章'}"，进行头脑风暴，列出3-5个创新的方向、灵感或想法。格式使用 HTML <ul><li>列表。${formattingInstruction}`;
      } else if (action === "outline") {
        prompt = `为主题 "${text || '当前文章'}" 生成一个大纲。格式使用 HTML <ol><li>列表。不要包在markdown中。`;
      } else if (action === "draft") {
        prompt = `为主题 "${text || '新文档'}" 起草几段正文段落。格式使用 HTML <p>标签。不要包在markdown中。\n\n上下文参考：\n${context || ''}`;
      } else if (action === "optimize_struct") {
        prompt = `优化以下 HTML 文本的结构和排版，使其段落分明、层次清晰，适合知识库阅读（只能返回HTML，不要包裹在 \`\`\`html 中）：\n\n${text}`;
      } else if (action === "chat") {
        prompt = `你是一个专业的AI助手。请基于上下文回答问题或与用户聊天。
如果用户的意图是明确要求你【撰写、修改、追加内容到笔记或正文】中（例如：“帮我写一段关于大模型的介绍”，“把刚才说的加到笔记里”，“重写这段笔记”），除了友好的回答外，你必须将准备写入文档的内容包裹在专门的 <insert_to_note></insert_to_note> 标签中，这部分将自动写入编辑器正文（里面可以直接使用基本的HTML如<p>,<ul>,<strong>等）。
如果是普通的对答、闲聊或纯解答知识，不要使用该标签。

用户消息：${text}

当前编辑中的文档上下文：
${context || '无'}`;
      } else {
        return res.status(400).json({ error: "Unknown action" });
      }

      const activeAi = getAI();
      if (!activeAi) {
         throw new Error("AI is not initialized properly");
      }

      const response = await activeAi.models.generateContent({
        model: "gemini-3.5-flash",
        contents: prompt,
      });

      let generatedText = response.text || "";
      
      // cleanup markdown wrapping if present
      generatedText = generatedText.replace(/^```(html)?/i, '').replace(/```$/i, '').trim();

      res.json({ result: generatedText });
    } catch (e: any) {
      console.error("AI Generation Error:", e);
      let text = req.body?.text || "";
      console.warn("Falling back to mock due to AI error.", e.message);
      let mockedResp = `API 请求失败，进入降级模拟模式。失败信息: ${e.message}`;
      let toolCalls: any[] | undefined = undefined;
      if (req.body?.action === "chat") {
        mockedResp = `非常抱歉，网络服务出现异常或大模型请求失败（可能是 Key 限制）。这是一个降级模拟响应。\n（问题：${text}）\n\n若您有插入笔记的需求，以下为您模拟生成了内容：\n<insert_to_note>\n[兜底模拟插入] 发生网络异常，以下为填充内容\n\n1. 模拟处理异常中的意图并重试。\n2. 排查网络和权限\n</insert_to_note>`;
        toolCalls = [
          { name: 'analyze_intent', arguments: { user_input: text }, result: '意图为：需要写作协助', status: 'success' },
          { name: 'insert_content', arguments: { content: '[兜底模拟插入]...' }, result: '内容已成功插入', status: 'success' }
        ];
      }
      return res.json({ result: mockedResp, toolCalls });
    }
  });

  // Vite middleware for development
  if (process.env.NODE_ENV !== "production") {
    const vite = await createViteServer({
      server: { middlewareMode: true },
      appType: "spa",
    });
    app.use(vite.middlewares);
  } else {
    const distPath = path.join(process.cwd(), "dist");
    app.use(express.static(distPath));
    // Support Express 4 fallback
    app.get("*", (req, res) => {
      res.sendFile(path.join(distPath, "index.html"));
    });
  }

  app.listen(PORT, "0.0.0.0", () => {
    console.log(`Server running on http://localhost:${PORT}`);
  });
}

startServer().catch(console.error);


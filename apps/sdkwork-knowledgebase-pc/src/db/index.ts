import Database from 'better-sqlite3';
import { drizzle } from 'drizzle-orm/better-sqlite3';
import * as schema from './schema';
import path from 'path';
import fs from 'fs';

const dbPath = path.join(process.cwd(), 'local.db');
const sqlite = new Database(dbPath);

export const db = drizzle(sqlite, { schema });

// Simple way to ensure tables exist without running migrations for now
export function initDb() {
  sqlite.exec(`
    CREATE TABLE IF NOT EXISTS users (
      id TEXT PRIMARY KEY,
      name TEXT NOT NULL,
      email TEXT UNIQUE,
      avatar TEXT
    );
    CREATE TABLE IF NOT EXISTS workspaces (
      id TEXT PRIMARY KEY,
      title TEXT NOT NULL,
      icon TEXT,
      avatar TEXT,
      type TEXT NOT NULL,
      created_at INTEGER NOT NULL
    );
    CREATE TABLE IF NOT EXISTS documents (
      id TEXT PRIMARY KEY,
      kb_id TEXT,
      title TEXT NOT NULL,
      type TEXT NOT NULL,
      content TEXT,
      parent_id TEXT,
      updated_at INTEGER NOT NULL,
      author TEXT NOT NULL,
      "order" INTEGER DEFAULT 0
    );
  `);
  
  // Seed initial data if empty
  const wsCount = sqlite.prepare("SELECT count(*) as c FROM workspaces").get() as { c: number };
  if (wsCount.c === 0) {
    sqlite.prepare("INSERT INTO workspaces (id, title, icon, type, created_at) VALUES (?, ?, ?, ?, ?)").run('kb1', '产品需求文档', '🚀', 'team', Date.now());
    sqlite.prepare("INSERT INTO workspaces (id, title, icon, type, created_at) VALUES (?, ?, ?, ?, ?)").run('kb2', '技术架构演进', '🛠️', 'team', Date.now());
    sqlite.prepare("INSERT INTO workspaces (id, title, icon, type, created_at) VALUES (?, ?, ?, ?, ?)").run('pkb1', '个人笔记', '📝', 'personal', Date.now());
    
    sqlite.prepare("INSERT INTO documents (id, kb_id, title, type, content, updated_at, author, \"order\") VALUES (?, ?, ?, ?, ?, ?, ?, ?)").run(
      'welcome-doc',
      'kb1',
      '欢迎使用知识库',
      'richtext',
      '<h2>欢迎来到企业级知识库</h2><p>这是一个完全由全栈服务支持的商业级应用，支持多级目录、实时保存和智能AI辅助协作。</p><ul><li><b>智能润色</b>：选中文字，通过AI助手一键改善行文质量。</li><li><b>大纲起草</b>：点击智能菜单内的“起草大纲”。</li></ul>',
      Date.now(),
      '系统管理员',
      0
    );
  }
}

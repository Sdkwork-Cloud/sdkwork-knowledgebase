import { sqliteTable, text, integer } from 'drizzle-orm/sqlite-core';

export const users = sqliteTable('users', {
  id: text('id').primaryKey(),
  name: text('name').notNull(),
  email: text('email').unique(),
  avatar: text('avatar'),
});

export const workspaces = sqliteTable('workspaces', {
  id: text('id').primaryKey(),
  title: text('title').notNull(),
  icon: text('icon'),
  avatar: text('avatar'),
  type: text('type').notNull(), // 'team' | 'personal'
  createdAt: integer('created_at', { mode: 'timestamp' }).notNull(),
});

export const documents = sqliteTable('documents', {
  id: text('id').primaryKey(),
  kbId: text('kb_id').references(() => workspaces.id),
  title: text('title').notNull(),
  type: text('type').notNull(), // 'folder' | 'richtext' | etc.
  content: text('content'),
  parentId: text('parent_id'),
  updatedAt: integer('updated_at', { mode: 'timestamp' }).notNull(),
  author: text('author').notNull(),
  order: integer('order').default(0),
});

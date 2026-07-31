import assert from "node:assert/strict";
import test from "node:test";

import {
  KnowledgeBaseCapabilityUnavailableError,
  KnowledgeBaseService,
} from "./KnowledgeBaseService";

test("knowledge base operations fail closed until the owner SDK is composed", async () => {
  const knowledgeBase = {
    description: "Description",
    icon: "database",
    name: "Knowledge Base",
  };
  const document = {
    author: "user-id",
    category: "general",
    content: "Content",
    kbId: "knowledge-base-id",
    title: "Document",
  };
  const operations = [
    () => KnowledgeBaseService.getKnowledgeBases(),
    () => KnowledgeBaseService.getKnowledgeBase("knowledge-base-id"),
    () => KnowledgeBaseService.createKnowledgeBase(knowledgeBase),
    () => KnowledgeBaseService.updateKnowledgeBase("knowledge-base-id", { name: "Updated" }),
    () => KnowledgeBaseService.deleteKnowledgeBase("knowledge-base-id"),
    () => KnowledgeBaseService.deleteKnowledgeBases(["knowledge-base-id"]),
    () => KnowledgeBaseService.archiveKnowledgeBases(["knowledge-base-id"]),
    () => KnowledgeBaseService.unarchiveKnowledgeBases(["knowledge-base-id"]),
    () => KnowledgeBaseService.getAllDocuments(),
    () => KnowledgeBaseService.getDocumentsByKbId("knowledge-base-id"),
    () => KnowledgeBaseService.getDocument("document-id"),
    () => KnowledgeBaseService.createDocument(document),
    () => KnowledgeBaseService.updateDocument("document-id", { title: "Updated" }),
    () => KnowledgeBaseService.deleteDocument("document-id"),
  ];

  for (const operation of operations) {
    await assert.rejects(operation, KnowledgeBaseCapabilityUnavailableError);
  }
});

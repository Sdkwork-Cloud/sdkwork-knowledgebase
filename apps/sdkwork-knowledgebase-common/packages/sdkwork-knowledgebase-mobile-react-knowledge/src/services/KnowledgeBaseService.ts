export interface KnowledgeBase {
  id: string;
  name: string;
  description: string;
  icon: string;
  color?: string;
  isArchived?: boolean;
  createdAt: string;
  updatedAt: string;
}

export interface KnowledgeDocument {
  id: string;
  kbId: string;
  title: string;
  content: string;
  category: string;
  author: string;
  createdAt: string;
  updatedAt: string;
}

export class KnowledgeBaseCapabilityUnavailableError extends Error {
  constructor() {
    super("Knowledge Base is unavailable because its owner SDK is not composed.");
    this.name = "KnowledgeBaseCapabilityUnavailableError";
  }
}

export class KnowledgeBaseService {
  static async getKnowledgeBases(): Promise<KnowledgeBase[]> {
    throw new KnowledgeBaseCapabilityUnavailableError();
  }

  static async getKnowledgeBase(_id: string): Promise<KnowledgeBase | null> {
    throw new KnowledgeBaseCapabilityUnavailableError();
  }

  static async createKnowledgeBase(
    _knowledgeBase: Omit<KnowledgeBase, "id" | "createdAt" | "updatedAt">,
  ): Promise<KnowledgeBase> {
    throw new KnowledgeBaseCapabilityUnavailableError();
  }

  static async updateKnowledgeBase(
    _id: string,
    _updates: Partial<KnowledgeBase>,
  ): Promise<KnowledgeBase | null> {
    throw new KnowledgeBaseCapabilityUnavailableError();
  }

  static async deleteKnowledgeBase(_id: string): Promise<void> {
    throw new KnowledgeBaseCapabilityUnavailableError();
  }

  static async deleteKnowledgeBases(_ids: string[]): Promise<void> {
    throw new KnowledgeBaseCapabilityUnavailableError();
  }

  static async archiveKnowledgeBases(_ids: string[]): Promise<void> {
    throw new KnowledgeBaseCapabilityUnavailableError();
  }

  static async unarchiveKnowledgeBases(_ids: string[]): Promise<void> {
    throw new KnowledgeBaseCapabilityUnavailableError();
  }

  static async getAllDocuments(): Promise<KnowledgeDocument[]> {
    throw new KnowledgeBaseCapabilityUnavailableError();
  }

  static async getDocumentsByKbId(_knowledgeBaseId: string): Promise<KnowledgeDocument[]> {
    throw new KnowledgeBaseCapabilityUnavailableError();
  }

  static async getDocument(_id: string): Promise<KnowledgeDocument | null> {
    throw new KnowledgeBaseCapabilityUnavailableError();
  }

  static async createDocument(
    _document: Omit<KnowledgeDocument, "id" | "createdAt" | "updatedAt">,
  ): Promise<KnowledgeDocument> {
    throw new KnowledgeBaseCapabilityUnavailableError();
  }

  static async updateDocument(
    _id: string,
    _updates: Partial<KnowledgeDocument>,
  ): Promise<KnowledgeDocument | null> {
    throw new KnowledgeBaseCapabilityUnavailableError();
  }

  static async deleteDocument(_id: string): Promise<void> {
    throw new KnowledgeBaseCapabilityUnavailableError();
  }
}

import React from "react";
import { motion } from "motion/react";
import { ChevronRight } from "lucide-react";
import { KnowledgeDocument } from "../services/KnowledgeBaseService";

interface KnowledgeDocumentCardProps {
  doc: KnowledgeDocument;
  onClick: () => void;
}

export const KnowledgeDocumentCard: React.FC<KnowledgeDocumentCardProps> = ({
  doc,
  onClick,
}) => {
  return (
    <motion.div
      whileHover={{ scale: 1.01 }}
      whileTap={{ scale: 0.98 }}
      onClick={onClick}
      className="bg-white dark:bg-[#1e1e20] p-4 rounded-xl shadow-sm border border-border-color/50 flex flex-col cursor-pointer hover:shadow-md transition-all"
    >
      <div className="flex justify-between items-start mb-2">
        <h3 className="text-[16px] font-semibold text-text-main leading-tight line-clamp-1 pr-4">
          {doc.title}
        </h3>
        <ChevronRight className="w-5 h-5 text-text-sub shrink-0 opacity-50" />
      </div>
      <p className="text-[14px] text-text-sub line-clamp-2 mb-3">
        {doc.content}
      </p>
      <div className="flex items-center justify-between mt-auto pt-3 border-t border-border-color/50">
        <span className="text-[12px] px-2.5 py-1 bg-primary-blue/10 text-primary-blue rounded-md font-medium">
          {doc.category}
        </span>
        <div className="flex items-center gap-2">
          <span className="text-[12px] text-text-sub">{doc.author}</span>
          <span className="text-[12px] text-text-sub/40">•</span>
          <span className="text-[12px] text-text-sub">
            {new Date(doc.createdAt).toLocaleDateString()}
          </span>
        </div>
      </div>
    </motion.div>
  );
};

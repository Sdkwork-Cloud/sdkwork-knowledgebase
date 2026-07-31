import React from "react";
import { motion } from "motion/react";
import { Trash2, Archive, ArchiveRestore, CheckSquare, Square, X } from "lucide-react";
import { useTranslation } from "react-i18next";

interface BatchActionBarProps {
  selectedCount: number;
  totalCount: number;
  isAllSelected: boolean;
  onToggleSelectAll: () => void;
  onBatchDelete: () => void;
  onBatchArchive: () => void;
  onBatchUnarchive: () => void;
  onCancel: () => void;
  isArchivedTab: boolean;
}

export const BatchActionBar: React.FC<BatchActionBarProps> = ({
  selectedCount,
  totalCount,
  isAllSelected,
  onToggleSelectAll,
  onBatchDelete,
  onBatchArchive,
  onBatchUnarchive,
  onCancel,
  isArchivedTab,
}) => {
  const { t } = useTranslation();

  return (
    <motion.div
      initial={{ y: 100, opacity: 0 }}
      animate={{ y: 0, opacity: 1 }}
      exit={{ y: 100, opacity: 0 }}
      className="fixed bottom-0 left-0 right-0 z-40 bg-white dark:bg-[#1c1c1e] border-t border-border-color shadow-2xl px-4 py-3 pb-safe"
    >
      <div className="max-w-md mx-auto flex items-center justify-between gap-2">
        {/* Select All Toggle */}
        <button
          onClick={onToggleSelectAll}
          className="flex items-center gap-1.5 text-[14px] font-medium text-text-main px-2 py-1.5 rounded-lg active:bg-black/5 dark:active:bg-white/5"
        >
          {isAllSelected ? (
            <CheckSquare className="w-5 h-5 text-primary-blue" />
          ) : (
            <Square className="w-5 h-5 text-text-sub" />
          )}
          <span>{isAllSelected ? "取消全选" : "全选"}</span>
        </button>

        <div className="text-[14px] font-semibold text-text-main">
          已选 <span className="text-primary-blue">{selectedCount}</span>/{totalCount}
        </div>

        {/* Actions */}
        <div className="flex items-center gap-2">
          {isArchivedTab ? (
            <button
              disabled={selectedCount === 0}
              onClick={onBatchUnarchive}
              className={`flex items-center gap-1 px-3 py-2 rounded-xl text-[13px] font-medium transition-all ${
                selectedCount > 0
                  ? "bg-amber-500/10 text-amber-600 dark:text-amber-400 border border-amber-500/20 active:scale-95"
                  : "bg-gray-100 dark:bg-gray-800 text-text-sub opacity-50 cursor-not-allowed"
              }`}
            >
              <ArchiveRestore className="w-4 h-4" />
              <span>取消归档</span>
            </button>
          ) : (
            <button
              disabled={selectedCount === 0}
              onClick={onBatchArchive}
              className={`flex items-center gap-1 px-3 py-2 rounded-xl text-[13px] font-medium transition-all ${
                selectedCount > 0
                  ? "bg-amber-500/10 text-amber-600 dark:text-amber-400 border border-amber-500/20 active:scale-95"
                  : "bg-gray-100 dark:bg-gray-800 text-text-sub opacity-50 cursor-not-allowed"
              }`}
            >
              <Archive className="w-4 h-4" />
              <span>归档</span>
            </button>
          )}

          <button
            disabled={selectedCount === 0}
            onClick={onBatchDelete}
            className={`flex items-center gap-1 px-3 py-2 rounded-xl text-[13px] font-medium transition-all ${
              selectedCount > 0
                ? "bg-red-500/10 text-red-600 dark:text-red-400 border border-red-500/20 active:scale-95"
                : "bg-gray-100 dark:bg-gray-800 text-text-sub opacity-50 cursor-not-allowed"
            }`}
          >
            <Trash2 className="w-4 h-4" />
            <span>删除</span>
          </button>

          <button
            onClick={onCancel}
            className="p-2 rounded-xl text-text-sub hover:bg-black/5 dark:hover:bg-white/5 active:scale-95"
            title="退出多选"
          >
            <X className="w-5 h-5" />
          </button>
        </div>
      </div>
    </motion.div>
  );
};

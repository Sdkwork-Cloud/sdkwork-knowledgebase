import React, { useCallback, useEffect, useMemo, useState } from 'react';
import { KnowledgeBaseApp, ToastContainer, TabCacheService } from '@packages/sdkwork-knowledgebase-pc-knowledgebase/src';
import { DocumentService } from '@packages/sdkwork-knowledgebase-pc-knowledgebase/src/services/document';
import { findDocInTree } from '@packages/sdkwork-knowledgebase-pc-knowledgebase/src/utils/docTreeUtils';
import {
  SearchModule,
  type SearchNavigateToFilePayload,
  type SearchNavigateToKbPayload
} from '@packages/sdkwork-knowledgebase-pc-search/src';
import {
  InAppBrowserHost,
  dispatchOpenInAppBrowser,
  setKbNavIntent,
  useLocalStorage
} from '@packages/sdkwork-knowledgebase-pc-commons/src';
import {
  createKnowledgebaseAccountViewModel,
  useKnowledgebaseRuntime,
  useKnowledgebaseSessionSnapshot,
} from 'sdkwork-knowledgebase-pc-core';

import {
  signOutKnowledgebaseSession,
  useHydrateKnowledgebaseAccount,
} from '../../../src/bootstrap/knowledgebaseAccountSession';
import { SettingsModal } from './SettingsModal';
import { GlobalNav } from './GlobalNav';
import { UserProfileModal, DEFAULT_USER_PROFILE, type UserProfile } from './UserProfileModal';

export function AppShell() {
  const runtime = useKnowledgebaseRuntime();
  const sessionSnapshot = useKnowledgebaseSessionSnapshot(runtime.session);
  const account = useMemo(
    () => createKnowledgebaseAccountViewModel(sessionSnapshot),
    [sessionSnapshot],
  );

  const [isSettingsOpen, setIsSettingsOpen] = useState(false);
  const [settingsInitialTab, setSettingsInitialTab] = useState('appearance');
  const [isProfileOpen, setIsProfileOpen] = useState(false);
  const [themePreference, setThemePreference] = useLocalStorage<'light' | 'dark' | 'system'>('app-theme-preference', 'system');
  const [activeTab, setActiveTab] = useLocalStorage<'kb' | 'market' | 'search'>('app-active-tab', 'kb');
  const [activeColor] = useLocalStorage('app-accent-color', '#2563eb');
  const [fontSize] = useLocalStorage('app-font-size', 'normal');
  const [profile, setProfile] = useLocalStorage<UserProfile>('app-user-profile', DEFAULT_USER_PROFILE);

  useHydrateKnowledgebaseAccount(runtime);

  useEffect(() => {
    document.documentElement.style.setProperty('--theme-accent', activeColor);

    if (fontSize === 'small') {
      document.documentElement.style.fontSize = '14px';
    } else if (fontSize === 'large') {
      document.documentElement.style.fontSize = '18px';
    } else {
      document.documentElement.style.fontSize = '16px';
    }
  }, [activeColor, fontSize]);

  useEffect(() => {
    const applyTheme = () => {
      const isDark = themePreference === 'dark'
        || (themePreference === 'system' && window.matchMedia('(prefers-color-scheme: dark)').matches);

      if (isDark) {
        document.documentElement.classList.add('dark');
      } else {
        document.documentElement.classList.remove('dark');
      }
    };

    applyTheme();

    if (themePreference === 'system') {
      const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
      mediaQuery.addEventListener('change', applyTheme);
      return () => mediaQuery.removeEventListener('change', applyTheme);
    }
  }, [themePreference]);

  useEffect(() => {
    if (!account.displayName && !account.email) {
      return;
    }

    const shouldSyncName = profile.name === DEFAULT_USER_PROFILE.name;
    const shouldSyncEmail = profile.email === DEFAULT_USER_PROFILE.email;
    if (!shouldSyncName && !shouldSyncEmail) {
      return;
    }

    setProfile({
      ...profile,
      name: shouldSyncName ? account.displayName : profile.name,
      email: shouldSyncEmail ? (account.email ?? profile.email) : profile.email,
    });
  }, [account.displayName, account.email, profile, setProfile]);

  const handleSignOut = useCallback(() => {
    void signOutKnowledgebaseSession(runtime);
  }, [runtime]);

  const openSettings = useCallback((tab = 'appearance') => {
    setSettingsInitialTab(tab);
    setIsSettingsOpen(true);
  }, []);

  const activateKnowledgeBase = useCallback((kbId: string, kbTitle?: string) => {
    const prev = localStorage.getItem('app-active-kb');
    let newKb: { id: string; title: string };
    if (prev && prev !== 'null') {
      newKb = { ...JSON.parse(prev), id: kbId, title: kbTitle ?? JSON.parse(prev).title ?? '' };
    } else {
      newKb = { id: kbId, title: kbTitle ?? '' };
    }
    localStorage.setItem('app-active-kb', JSON.stringify(newKb));
    window.dispatchEvent(new CustomEvent('local-storage', { detail: { key: 'app-active-kb', value: newKb } }));
    setActiveTab('kb');
  }, [setActiveTab]);

  const handleGoToKb = useCallback((payload: SearchNavigateToKbPayload) => {
    activateKnowledgeBase(payload.kbId, payload.kbTitle);
  }, [activateKnowledgeBase]);

  const handleGoToFile = useCallback(async (payload: SearchNavigateToFilePayload) => {
    setKbNavIntent({
      kbId: payload.kbId,
      kbTitle: payload.kbTitle,
      docId: payload.docId,
      docTitle: payload.title,
      docType: payload.type,
      parentId: payload.parentId,
      author: payload.author,
      updatedAt: payload.updatedAt,
      highlight: true
    });

    let docMeta = {
      id: payload.docId,
      title: payload.title,
      type: payload.type,
      kbId: payload.kbId,
      author: payload.author ?? account.displayName,
      updatedAt: payload.updatedAt ?? new Date().toISOString(),
      parentId: payload.parentId
    };

    try {
      const tree = await DocumentService.getDocuments(payload.kbId);
      const found = findDocInTree(tree, payload.docId);
      if (found) {
        docMeta = {
          ...docMeta,
          ...found,
          id: found.id,
          title: found.title,
          type: found.type
        };
        setKbNavIntent({
          kbId: payload.kbId,
          kbTitle: payload.kbTitle,
          docId: payload.docId,
          docTitle: found.title,
          docType: found.type,
          parentId: found.parentId ?? payload.parentId,
          author: found.author ?? payload.author,
          updatedAt: found.updatedAt ?? payload.updatedAt,
          highlight: true
        });
      }
    } catch {
      /* use payload snapshot */
    }

    TabCacheService.openDoc(payload.kbId, docMeta);
    activateKnowledgeBase(payload.kbId, payload.kbTitle);
  }, [account.displayName, activateKnowledgeBase]);

  const handleOpenWebLink = useCallback((url: string, title?: string) => {
    dispatchOpenInAppBrowser({ url, title });
  }, []);

  return (
    <div className="flex h-screen w-screen overflow-hidden bg-[var(--color-kb-nav)] text-[var(--color-kb-text)] font-sans">
      <GlobalNav
        account={account}
        profile={profile}
        activeTab={activeTab}
        onTabChange={(tab) => setActiveTab(tab as 'kb' | 'market' | 'search')}
        onOpenSettings={() => openSettings('appearance')}
        onOpenProfile={() => setIsProfileOpen(true)}
        onOpenAccountSettings={() => openSettings('account')}
      />
      <div className="flex-1 flex overflow-hidden relative">
        {activeTab === 'search' ? (
          <SearchModule
            onGoToKb={handleGoToKb}
            onGoToFile={handleGoToFile}
            onOpenWebLink={handleOpenWebLink}
          />
        ) : (
          <KnowledgeBaseApp
            activeTab={activeTab}
            onActiveTabChange={setActiveTab as (tab: string) => void}
          />
        )}
      </div>

      <InAppBrowserHost />

      <SettingsModal
        account={account}
        hosting={runtime.config.hosting}
        initialTab={settingsInitialTab}
        isOpen={isSettingsOpen}
        onClose={() => setIsSettingsOpen(false)}
        onSignOut={handleSignOut}
        runtimeConfig={runtime.config}
        setTheme={setThemePreference}
        theme={themePreference}
      />

      <UserProfileModal
        account={account}
        isOpen={isProfileOpen}
        onClose={() => setIsProfileOpen(false)}
      />

      <ToastContainer />
    </div>
  );
}

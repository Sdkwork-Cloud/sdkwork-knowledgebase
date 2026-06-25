import { useEffect, useState } from 'react';

import type { SessionSnapshot, SessionStore } from '../session/sessionStore';

export function useKnowledgebaseSessionSnapshot(session: SessionStore): SessionSnapshot {
  const [snapshot, setSnapshot] = useState(() => session.getSnapshot());

  useEffect(() => {
    setSnapshot(session.getSnapshot());
    return session.subscribe(setSnapshot);
  }, [session]);

  return snapshot;
}

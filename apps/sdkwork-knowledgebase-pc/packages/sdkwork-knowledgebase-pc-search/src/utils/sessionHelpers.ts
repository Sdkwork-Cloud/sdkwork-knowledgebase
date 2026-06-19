import type { SearchSession } from '../types';

export type SearchSessionDateBucket = 'today' | 'yesterday' | 'last7Days' | 'older';

export function getSessionPreview(
  session: SearchSession,
  emptyLabel = '尚未开始对话',
): string {
  const lastUser = [...session.messages].reverse().find((m) => m.role === 'user');
  if (lastUser) return lastUser.content;
  return emptyLabel;
}

export function groupSessionsByDate(sessions: SearchSession[]) {
  const now = new Date();
  const todayStart = new Date(now.getFullYear(), now.getMonth(), now.getDate()).getTime();
  const yesterdayStart = todayStart - 86_400_000;
  const weekStart = todayStart - 7 * 86_400_000;

  const buckets: { key: SearchSessionDateBucket; sessions: SearchSession[] }[] = [
    { key: 'today', sessions: [] },
    { key: 'yesterday', sessions: [] },
    { key: 'last7Days', sessions: [] },
    { key: 'older', sessions: [] },
  ];

  sessions.forEach((session) => {
    const created = new Date(session.createdAt).getTime();
    if (created >= todayStart) buckets[0].sessions.push(session);
    else if (created >= yesterdayStart) buckets[1].sessions.push(session);
    else if (created >= weekStart) buckets[2].sessions.push(session);
    else buckets[3].sessions.push(session);
  });

  return buckets.filter((bucket) => bucket.sessions.length > 0);
}

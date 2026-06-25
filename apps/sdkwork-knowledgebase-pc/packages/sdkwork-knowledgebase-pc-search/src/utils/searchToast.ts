export function showSearchToast(message: string, tone: 'success' | 'default' = 'default') {
  const el = document.createElement('div');
  el.className = `fixed bottom-6 right-6 z-[9999] font-semibold px-4 py-2.5 rounded-xl shadow-lg text-xs flex items-center gap-2 animate-[searchToastIn_0.25s_ease-out] ${
    tone === 'success'
      ? 'bg-emerald-600 text-white'
      : 'bg-zinc-900 text-white dark:bg-zinc-100 dark:text-zinc-900'
  }`;
  el.textContent = message;
  document.body.appendChild(el);
  setTimeout(() => {
    el.style.opacity = '0';
    el.style.transform = 'translateY(8px)';
    el.style.transition = 'opacity 0.2s, transform 0.2s';
    setTimeout(() => el.remove(), 220);
  }, 2400);
}

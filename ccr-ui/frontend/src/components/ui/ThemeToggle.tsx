'use client';

import { Moon, Sun } from 'lucide-react';
import { useState } from 'react';

export default function ThemeToggle() {
  const [theme, setTheme] = useState<'light' | 'dark'>(() => {
    if (typeof window === 'undefined') return 'light';
    const saved = localStorage.getItem('ccr-theme') as 'light' | 'dark' | null;
    const initial = saved || 'light';
    // 立即应用主题
    if (typeof document !== 'undefined') {
      document.documentElement.setAttribute('data-theme', initial);
    }
    return initial;
  });

  const toggleTheme = () => {
    const newTheme = theme === 'dark' ? 'light' : 'dark';
    setTheme(newTheme);
    document.documentElement.setAttribute('data-theme', newTheme);
    localStorage.setItem('ccr-theme', newTheme);
  };

  return (
    <button
      onClick={toggleTheme}
      className="w-10 h-10 rounded-full transition-all hover:rotate-180 hover:scale-110 flex items-center justify-center"
      style={{
        background: 'var(--bg-tertiary)',
        border: '1px solid var(--border-color)',
        color: 'var(--text-secondary)',
      }}
      title={`切换到${theme === 'dark' ? '明亮' : '深色'}模式`}
      aria-label={`切换到${theme === 'dark' ? '明亮' : '深色'}模式`}
    >
      {theme === 'dark' ? (
        <Moon className="w-5 h-5" />
      ) : (
        <Sun className="w-5 h-5" />
      )}
    </button>
  );
}

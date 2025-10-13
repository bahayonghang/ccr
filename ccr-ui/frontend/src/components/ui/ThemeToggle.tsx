'use client';

import { Moon, Sun } from 'lucide-react';
import { useEffect, useState } from 'react';

export default function ThemeToggle() {
  const [theme, setTheme] = useState<'light' | 'dark'>('light');
  const [mounted, setMounted] = useState(false);

  useEffect(() => {
    setMounted(true);
    // 从 localStorage 加载主题
    const savedTheme = (localStorage.getItem('ccr-theme') as 'light' | 'dark') || 'light';
    setTheme(savedTheme);
    document.documentElement.setAttribute('data-theme', savedTheme);
  }, []);

  const toggleTheme = () => {
    const newTheme = theme === 'dark' ? 'light' : 'dark';
    setTheme(newTheme);
    document.documentElement.setAttribute('data-theme', newTheme);
    localStorage.setItem('ccr-theme', newTheme);
  };

  // 防止服务端渲染不匹配
  if (!mounted) {
    return (
      <button
        className="w-10 h-10 rounded-full flex items-center justify-center"
        style={{
          background: 'var(--bg-tertiary)',
          border: '1px solid var(--border-color)',
        }}
        aria-label="Toggle theme"
      >
        <Sun className="w-5 h-5" style={{ color: 'var(--text-secondary)' }} />
      </button>
    );
  }

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


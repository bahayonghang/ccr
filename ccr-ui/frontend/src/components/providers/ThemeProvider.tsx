'use client';

import { useEffect, useState } from 'react';

export function ThemeProvider({ children }: { children: React.ReactNode }) {
  const [mounted, setMounted] = useState(false);

  useEffect(() => {
    setMounted(true);
    // 从 localStorage 加载主题
    const savedTheme = localStorage.getItem('ccr-theme') || 'light';
    document.documentElement.setAttribute('data-theme', savedTheme);
  }, []);

  // 防止服务端渲染时的主题闪烁
  if (!mounted) {
    return <>{children}</>;
  }

  return <>{children}</>;
}


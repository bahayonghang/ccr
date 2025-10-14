'use client';

import { Puzzle, Wrench } from 'lucide-react';
import Navbar from '@/components/layout/Navbar';
import StatusHeader from '@/components/layout/StatusHeader';
import CollapsibleSidebar from '@/components/layout/CollapsibleSidebar';

export default function GeminiCliPluginsPage() {
  return (
    <div style={{ background: 'var(--bg-primary)', minHeight: '100vh', padding: '20px' }}>
      <div className="max-w-[1800px] mx-auto">
        <Navbar />
        <StatusHeader currentConfig="" totalConfigs={0} historyCount={0} />

        <div className="grid grid-cols-[auto_1fr] gap-4">
          <CollapsibleSidebar />

          <main className="rounded-xl p-6 glass-effect" style={{ border: '1px solid var(--border-color)', boxShadow: 'var(--shadow-small)' }}>
            <div className="flex items-center gap-3 mb-6">
              <Puzzle className="w-6 h-6" style={{ color: 'var(--accent-primary)' }} />
              <h1 className="text-2xl font-bold" style={{ color: 'var(--text-primary)' }}>Gemini Cli - 插件管理</h1>
            </div>

            <div className="flex flex-col items-center justify-center py-20">
              <Wrench className="w-24 h-24 mb-6" style={{ color: 'var(--text-muted)', opacity: 0.5 }} />
              <h2 className="text-3xl font-bold mb-4" style={{ color: 'var(--text-primary)' }}>
                功能开发中
              </h2>
              <p className="text-lg mb-2" style={{ color: 'var(--text-secondary)' }}>
                Gemini Cli 插件管理功能正在开发中，敬请期待！
              </p>
              <p className="text-sm" style={{ color: 'var(--text-muted)' }}>
                该功能将支持管理 Gemini Cli 的插件配置
              </p>
            </div>
          </main>
        </div>
      </div>
    </div>
  );
}


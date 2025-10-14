'use client';

import { Workflow } from 'lucide-react';
import Navbar from '@/components/layout/Navbar';
import StatusHeader from '@/components/layout/StatusHeader';
import CollapsibleSidebar from '@/components/layout/CollapsibleSidebar';

export default function IflowCommandsPage() {
  return (
    <div style={{ background: 'var(--bg-primary)', minHeight: '100vh', padding: '20px' }}>
      <div className="max-w-[1800px] mx-auto">
        <Navbar />
        <StatusHeader currentConfig="" totalConfigs={0} historyCount={0} />

        <div className="grid grid-cols-[auto_1fr] gap-4">
          <CollapsibleSidebar />

          <main className="rounded-xl p-6 glass-effect" style={{ border: '1px solid var(--border-color)', boxShadow: 'var(--shadow-small)' }}>
            <div className="flex items-center gap-3 mb-6">
              <Workflow className="w-6 h-6" style={{ color: '#a855f7' }} />
              <h1 className="text-2xl font-bold" style={{ color: 'var(--text-primary)' }}>IFLOW 命令执行</h1>
            </div>

            <div className="flex flex-col items-center justify-center py-20">
              <div className="w-24 h-24 mb-6 rounded-full flex items-center justify-center" style={{ background: 'rgba(168, 85, 247, 0.1)' }}>
                <Workflow className="w-12 h-12" style={{ color: '#a855f7', opacity: 0.5 }} />
              </div>
              <h2 className="text-3xl font-bold mb-4" style={{ color: 'var(--text-primary)' }}>
                功能开发中
              </h2>
              <p className="text-lg mb-2" style={{ color: 'var(--text-secondary)' }}>
                IFLOW CLI 命令执行功能正在开发中，敬请期待！
              </p>
              <p className="text-sm" style={{ color: 'var(--text-muted)' }}>
                该功能将支持通过 Web 界面执行 IFLOW 相关命令
              </p>
            </div>
          </main>
        </div>
      </div>
    </div>
  );
}


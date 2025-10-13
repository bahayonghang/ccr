'use client';

import { useState } from 'react';
import Link from 'next/link';
import { usePathname } from 'next/navigation';
import { Settings, Terminal, ChevronLeft, ChevronRight, Menu } from 'lucide-react';

const navigation = [
  { name: '配置管理', href: '/configs', icon: Settings },
  { name: '命令执行', href: '/commands', icon: Terminal },
];

export default function CollapsibleSidebar() {
  const [collapsed, setCollapsed] = useState(false);
  const pathname = usePathname();

  return (
    <aside
      className={`rounded-xl p-4 h-fit sticky top-5 transition-all duration-300 glass-effect ${
        collapsed ? 'w-16' : 'w-64'
      }`}
      style={{
        border: '1px solid var(--border-color)',
        boxShadow: 'var(--shadow-small)',
      }}
    >
      {/* 切换按钮 */}
      <div className="flex items-center justify-between mb-4">
        {!collapsed && (
          <div
            className="text-xs font-semibold uppercase tracking-wider"
            style={{ color: 'var(--text-secondary)' }}
          >
            导航菜单
          </div>
        )}
        <button
          onClick={() => setCollapsed(!collapsed)}
          className="p-2 rounded-lg transition-all hover:scale-110"
          style={{
            background: 'var(--bg-tertiary)',
            border: '1px solid var(--border-color)',
            color: 'var(--text-secondary)',
          }}
          title={collapsed ? '展开菜单' : '收起菜单'}
          aria-label={collapsed ? '展开菜单' : '收起菜单'}
        >
          {collapsed ? (
            <ChevronRight className="w-4 h-4" />
          ) : (
            <ChevronLeft className="w-4 h-4" />
          )}
        </button>
      </div>

      {/* 导航链接 */}
      <nav className="space-y-2" aria-label="主导航">
        {navigation.map((item) => {
          const isActive = pathname === item.href;
          const Icon = item.icon;

          return (
            <Link
              key={item.href}
              href={item.href}
              className={`flex items-center ${collapsed ? 'justify-center' : 'space-x-3'} px-4 py-3 rounded-lg transition-all relative overflow-hidden group ${
                isActive ? 'text-white' : ''
              }`}
              style={{
                background: isActive
                  ? 'linear-gradient(135deg, var(--accent-primary), var(--accent-secondary))'
                  : 'var(--bg-tertiary)',
                border: `1px solid ${isActive ? 'var(--accent-primary)' : 'var(--border-color)'}`,
                boxShadow: isActive ? '0 0 15px var(--glow-primary)' : undefined,
                color: isActive ? 'white' : 'var(--text-secondary)',
              }}
              title={collapsed ? item.name : undefined}
              aria-current={isActive ? 'page' : undefined}
            >
              {/* 左侧指示器 */}
              <span
                className={`absolute left-0 top-0 w-1 h-full transition-transform ${
                  isActive ? 'scale-y-100' : 'scale-y-0'
                }`}
                style={{ background: 'var(--accent-primary)' }}
                aria-hidden="true"
              />
              
              <Icon className="w-5 h-5 flex-shrink-0" />
              {!collapsed && <span className="font-medium text-sm">{item.name}</span>}
            </Link>
          );
        })}
      </nav>

      {/* 收起状态提示 */}
      {collapsed && (
        <div className="mt-4 text-center">
          <button
            onClick={() => setCollapsed(false)}
            className="p-2 rounded-lg transition-all hover:scale-110"
            style={{
              background: 'var(--bg-tertiary)',
              border: '1px solid var(--border-color)',
              color: 'var(--text-muted)',
            }}
            title="展开菜单"
            aria-label="展开菜单"
          >
            <Menu className="w-4 h-4" />
          </button>
        </div>
      )}
    </aside>
  );
}


import { useState } from 'react';
import { NavLink } from 'react-router-dom';
import { Settings, Terminal, ChevronLeft, ChevronRight, Menu } from 'lucide-react';

const navigation = [
  { name: '配置管理', to: '/configs', icon: Settings },
  { name: '命令执行', to: '/commands', icon: Terminal },
];

export default function CollapsibleSidebar() {
  const [collapsed, setCollapsed] = useState(false);

  return (
    <aside
      className={`rounded-xl p-4 h-fit sticky top-5 transition-all duration-300 ${
        collapsed ? 'w-16' : 'w-64'
      }`}
      style={{
        background: 'var(--bg-card)',
        backdropFilter: 'blur(20px)',
        border: '1px solid var(--border-color)',
        boxShadow: 'var(--shadow-small)',
      }}
    >
      {/* Toggle Button */}
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
        >
          {collapsed ? (
            <ChevronRight className="w-4 h-4" />
          ) : (
            <ChevronLeft className="w-4 h-4" />
          )}
        </button>
      </div>

      {/* Navigation Links */}
      <nav className="space-y-2">
        {navigation.map((item) => (
          <NavLink
            key={item.to}
            to={item.to}
            className={({ isActive }) =>
              `flex items-center ${collapsed ? 'justify-center' : 'space-x-3'} px-4 py-3 rounded-lg transition-all relative overflow-hidden group ${
                isActive ? 'text-white' : ''
              }`
            }
            style={({ isActive }) => ({
              background: isActive
                ? 'linear-gradient(135deg, var(--accent-primary), var(--accent-secondary))'
                : 'var(--bg-tertiary)',
              border: `1px solid ${isActive ? 'var(--accent-primary)' : 'var(--border-color)'}`,
              boxShadow: isActive ? '0 0 15px var(--glow-primary)' : undefined,
              color: isActive ? 'white' : 'var(--text-secondary)',
            })}
            title={collapsed ? item.name : undefined}
          >
            {({ isActive }) => (
              <>
                {/* Left indicator */}
                <span
                  className={`absolute left-0 top-0 w-1 h-full transition-transform ${
                    isActive ? 'scale-y-100' : 'scale-y-0'
                  }`}
                  style={{ background: 'var(--accent-primary)' }}
                />
                
                <item.icon className="w-5 h-5 flex-shrink-0" />
                {!collapsed && <span className="font-medium text-sm">{item.name}</span>}
              </>
            )}
          </NavLink>
        ))}
      </nav>

      {/* Collapsed state hint */}
      {collapsed && (
        <div className="mt-4 text-center">
          <button
            onClick={() => setCollapsed(false)}
            className="p-2 rounded-lg transition-all"
            style={{
              background: 'var(--bg-tertiary)',
              border: '1px solid var(--border-color)',
              color: 'var(--text-muted)',
            }}
            title="展开菜单"
          >
            <Menu className="w-4 h-4" />
          </button>
        </div>
      )}
    </aside>
  );
}


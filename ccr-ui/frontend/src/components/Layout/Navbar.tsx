import { Zap, RefreshCw, Upload, Download, PlusCircle } from 'lucide-react';
import ThemeToggle from '../ThemeToggle';

interface NavbarProps {
  onRefresh?: () => void;
  onImport?: () => void;
  onExport?: () => void;
  onAdd?: () => void;
}

export default function Navbar({ onRefresh, onImport, onExport, onAdd }: NavbarProps) {
  return (
    <nav
      className="rounded-xl mb-5 p-5 relative overflow-hidden"
      style={{
        background: 'var(--bg-card)',
        backdropFilter: 'blur(20px)',
        border: '1px solid var(--border-color)',
        boxShadow: 'var(--shadow-medium)',
      }}
    >
      {/* Bottom gradient line */}
      <div
        className="absolute bottom-0 left-0 w-full h-0.5 opacity-50"
        style={{
          background:
            'linear-gradient(90deg, transparent, var(--accent-primary), var(--accent-secondary), transparent)',
        }}
      />

      <div className="flex items-center justify-between">
        {/* Brand Section */}
        <div className="flex items-center space-x-5">
          <div className="flex flex-col">
            <div className="flex items-center space-x-2">
              <Zap className="w-7 h-7" style={{ color: 'var(--accent-primary)' }} />
              <div
                className="text-3xl font-bold tracking-tight"
                style={{
                  background: 'linear-gradient(135deg, var(--accent-primary), var(--accent-secondary))',
                  WebkitBackgroundClip: 'text',
                  WebkitTextFillColor: 'transparent',
                  backgroundClip: 'text',
                }}
              >
                CCR
              </div>
            </div>
            <div
              className="text-xs font-medium tracking-widest uppercase mt-1"
              style={{ color: 'var(--text-muted)' }}
            >
              Web Console
            </div>
          </div>

          <div
            className="hidden md:block w-px h-12"
            style={{
              background: 'linear-gradient(180deg, transparent, var(--border-color), transparent)',
            }}
          />

          <div className="hidden md:block">
            <div className="text-sm font-semibold" style={{ color: 'var(--text-primary)' }}>
              Claude Code Configuration Switcher
            </div>
            <div className="flex items-center space-x-3 mt-1">
              <span
                className="flex items-center space-x-1 text-xs"
                style={{ color: 'var(--text-secondary)' }}
              >
                <span className="w-1 h-1 rounded-full" style={{ background: 'var(--text-muted)' }} />
                <span>MIT License</span>
              </span>
            </div>
          </div>
        </div>

        {/* Actions */}
        <div className="flex items-center space-x-3">
          <ThemeToggle />
          {onRefresh && (
            <button
              onClick={onRefresh}
              className="px-4 py-2 rounded-lg font-semibold text-sm transition-all flex items-center space-x-2"
              style={{
                background: 'var(--bg-tertiary)',
                color: 'var(--text-primary)',
                border: '1px solid var(--border-color)',
              }}
            >
              <RefreshCw className="w-4 h-4" />
              <span>刷新</span>
            </button>
          )}
          {onImport && (
            <button
              onClick={onImport}
              className="px-4 py-2 rounded-lg font-semibold text-sm transition-all flex items-center space-x-2"
              style={{
                background: 'var(--bg-tertiary)',
                color: 'var(--text-primary)',
                border: '1px solid var(--border-color)',
              }}
            >
              <Upload className="w-4 h-4" />
              <span>导入</span>
            </button>
          )}
          {onExport && (
            <button
              onClick={onExport}
              className="px-4 py-2 rounded-lg font-semibold text-sm transition-all flex items-center space-x-2"
              style={{
                background: 'var(--bg-tertiary)',
                color: 'var(--text-primary)',
                border: '1px solid var(--border-color)',
              }}
            >
              <Download className="w-4 h-4" />
              <span>导出</span>
            </button>
          )}
          {onAdd && (
            <button
              onClick={onAdd}
              className="px-4 py-2 rounded-lg font-semibold text-sm transition-all flex items-center space-x-2 text-white shadow-lg"
              style={{
                background: 'linear-gradient(135deg, var(--accent-primary), var(--accent-secondary))',
                boxShadow: '0 0 20px var(--glow-primary)',
              }}
            >
              <PlusCircle className="w-4 h-4" />
              <span>添加配置</span>
            </button>
          )}
        </div>
      </div>
    </nav>
  );
}



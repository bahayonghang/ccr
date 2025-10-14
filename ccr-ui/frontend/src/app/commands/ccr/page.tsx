'use client';

import { useState, useEffect, useMemo } from 'react';
import { Play, Copy, Trash2, Zap, Sparkles, Gem, Workflow, Check, X } from 'lucide-react';
import AnsiToHtml from 'ansi-to-html';
import { listCommands, executeCommand, listConfigs, getHistory } from '@/lib/api/client';
import type { CommandInfo, CommandResponse } from '@/lib/types';
import Navbar from '@/components/layout/Navbar';
import StatusHeader from '@/components/layout/StatusHeader';
import CollapsibleSidebar from '@/components/layout/CollapsibleSidebar';
import '../terminal-output.css';

type CliClient = 'ccr' | 'qwen' | 'gemini-cli' | 'iflow';

const CLI_CLIENTS = [
  { id: 'ccr' as CliClient, name: 'CCR', icon: Zap, color: 'rgba(139, 92, 246, 0.2)' },
  { id: 'qwen' as CliClient, name: 'Qwen', icon: Sparkles, color: 'rgba(251, 191, 36, 0.2)' },
  { id: 'gemini-cli' as CliClient, name: 'Gemini Cli', icon: Gem, color: 'rgba(59, 130, 246, 0.2)' },
  { id: 'iflow' as CliClient, name: 'IFLOW', icon: Workflow, color: 'rgba(168, 85, 247, 0.2)' },
];

export default function CommandExecutor() {
  const [selectedClient, setSelectedClient] = useState<CliClient>('ccr');
  const [commands, setCommands] = useState<CommandInfo[]>([]);
  const [selectedCommand, setSelectedCommand] = useState<string>('');
  const [args, setArgs] = useState<string>('');
  const [output, setOutput] = useState<CommandResponse | null>(null);
  const [loading, setLoading] = useState(false);
  const [currentConfig, setCurrentConfig] = useState<string>('');
  const [totalConfigs, setTotalConfigs] = useState(0);
  const [historyCount, setHistoryCount] = useState(0);

  useEffect(() => {
    const loadCommands = async () => {
      try {
        // CCR 有实际的命令，其他客户端显示占位符
        if (selectedClient === 'ccr') {
          const data = await listCommands();
          setCommands(data);
          if (data.length > 0) {
            setSelectedCommand(data[0].name);
          }
        } else {
          // 其他CLI客户端的占位符命令
          setCommands([
            {
              name: 'help',
              description: `${CLI_CLIENTS.find(c => c.id === selectedClient)?.name} 帮助命令 (功能开发中)`,
              usage: `${selectedClient} help`,
              examples: [`${selectedClient} help`],
            },
            {
              name: 'version',
              description: `显示 ${CLI_CLIENTS.find(c => c.id === selectedClient)?.name} 版本`,
              usage: `${selectedClient} --version`,
              examples: [`${selectedClient} --version`],
            },
          ]);
          setSelectedCommand('help');
        }
      } catch (err) {
        console.error('Failed to load commands:', err);
      }
    };

    const loadStats = async () => {
      try {
        const configData = await listConfigs();
        setCurrentConfig(configData.current_config);
        setTotalConfigs(configData.configs.length);

        const historyData = await getHistory();
        setHistoryCount(historyData.total);
      } catch (err) {
        console.error('Failed to load stats:', err);
      }
    };

    loadCommands();
    loadStats();
    setOutput(null); // 切换客户端时清除输出
  }, [selectedClient]);

  const handleExecute = async () => {
    if (!selectedCommand) return;

    setLoading(true);
    try {
      if (selectedClient === 'ccr') {
        const argsArray = args
          .split(' ')
          .map((a) => a.trim())
          .filter((a) => a.length > 0);

        const result = await executeCommand({
          command: selectedCommand,
          args: argsArray,
        });

        setOutput(result);
      } else {
        // 其他CLI客户端的模拟输出
        setOutput({
          success: true,
          output: `${CLI_CLIENTS.find(c => c.id === selectedClient)?.name} 命令执行功能正在开发中...\n\n该功能将支持执行 ${selectedClient} 相关命令。`,
          error: '',
          exit_code: 0,
          duration_ms: 100,
        });
      }
    } catch (err) {
      setOutput({
        success: false,
        output: '',
        error: err instanceof Error ? err.message : 'Unknown error',
        exit_code: -1,
        duration_ms: 0,
      });
    } finally {
      setLoading(false);
    }
  };

  const handleCopyOutput = async () => {
    if (!output) return;
    const text = output.output + (output.error ? '\n' + output.error : '');
    try {
      await navigator.clipboard.writeText(text);
      alert('输出已复制到剪贴板！');
    } catch (err) {
      console.error('Failed to copy:', err);
    }
  };

  const handleClearOutput = () => {
    setOutput(null);
  };

  const selectedCommandInfo = commands.find((c) => c.name === selectedCommand);
  const currentClientInfo = CLI_CLIENTS.find(c => c.id === selectedClient);

  // ANSI to HTML converter
  const ansiConverter = useMemo(() => new AnsiToHtml({
    fg: '#e2e8f0',
    bg: '#0f172a',
    newline: true,
    escapeXML: true,
    stream: false,
  }), []);

  // Convert ANSI codes to HTML
  const convertedOutput = useMemo(() => {
    if (!output?.output) return '';
    return ansiConverter.toHtml(output.output);
  }, [output?.output, ansiConverter]);

  return (
    <div style={{ background: 'var(--bg-primary)', minHeight: '100vh', padding: '20px' }}>
      <div className="max-w-[1800px] mx-auto">
        {/* 导航栏 */}
        <Navbar />

        {/* 状态信息头部 */}
        <StatusHeader
          currentConfig={currentConfig}
          totalConfigs={totalConfigs}
          historyCount={historyCount}
        />

        {/* 布局：可折叠侧边栏 + 主命令区域 */}
        <div className="grid grid-cols-[auto_1fr] gap-4">
          {/* 可折叠导航 */}
          <CollapsibleSidebar />

          {/* 主命令区域 */}
          <div className="space-y-4">
            {/* CLI 客户端选择 */}
            <section
              className="rounded-xl p-5 glass-effect"
              style={{
                border: '1px solid var(--border-color)',
                boxShadow: 'var(--shadow-small)',
              }}
            >
              <h2
                className="text-xs font-semibold uppercase tracking-wider mb-3"
                style={{ color: 'var(--text-secondary)' }}
              >
                选择Cli命令行工具
              </h2>
              <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
                {CLI_CLIENTS.map((client) => {
                  const ClientIcon = client.icon;
                  const isSelected = selectedClient === client.id;
                  
                  return (
                    <button
                      key={client.id}
                      onClick={() => setSelectedClient(client.id)}
                      className={`p-4 rounded-lg transition-all hover:scale-[1.02] ${
                        isSelected ? 'ring-2 shadow-lg' : ''
                      }`}
                      style={{
                        background: isSelected
                          ? 'linear-gradient(135deg, var(--accent-primary), var(--accent-secondary))'
                          : 'var(--bg-tertiary)',
                        border: `1px solid ${isSelected ? 'var(--accent-primary)' : 'var(--border-color)'}`,
                        color: isSelected ? 'white' : 'var(--text-primary)',
                        boxShadow: isSelected ? '0 0 20px var(--glow-primary)' : undefined,
                      }}
                    >
                      <div className="flex flex-col items-center gap-2">
                        <ClientIcon className="w-6 h-6" />
                        <span className="text-sm font-semibold">{client.name}</span>
                      </div>
                    </button>
                  );
                })}
              </div>
            </section>

            {/* 命令列表和执行区域 */}
            <div className="grid grid-cols-1 lg:grid-cols-3 gap-4">
              {/* 命令列表 */}
              <aside
                className="lg:col-span-1 rounded-xl p-5 glass-effect"
                style={{
                  border: '1px solid var(--border-color)',
                  boxShadow: 'var(--shadow-small)',
                }}
              >
                <h2
                  className="text-xs font-semibold uppercase tracking-wider mb-4"
                  style={{ color: 'var(--text-secondary)' }}
                >
                  可用命令
                </h2>
                <nav className="space-y-2" aria-label="命令列表">
                  {commands.map((cmd) => (
                    <button
                      key={cmd.name}
                      onClick={() => setSelectedCommand(cmd.name)}
                      className={`w-full text-left px-4 py-3 rounded-lg transition-all hover:scale-[1.02] ${
                        selectedCommand === cmd.name ? 'text-white shadow-lg' : ''
                      }`}
                      style={{
                        background:
                          selectedCommand === cmd.name
                            ? 'linear-gradient(135deg, var(--accent-primary), var(--accent-secondary))'
                            : 'var(--bg-tertiary)',
                        border: `1px solid ${selectedCommand === cmd.name ? 'var(--accent-primary)' : 'var(--border-color)'}`,
                        boxShadow: selectedCommand === cmd.name ? '0 0 15px var(--glow-primary)' : undefined,
                      }}
                      aria-pressed={selectedCommand === cmd.name}
                    >
                      <div className="font-mono font-bold text-sm">{cmd.name}</div>
                      <div
                        className="text-xs mt-1 leading-relaxed"
                        style={{
                          color: selectedCommand === cmd.name ? 'rgba(255,255,255,0.9)' : 'var(--text-secondary)',
                        }}
                      >
                        {cmd.description}
                      </div>
                    </button>
                  ))}
                </nav>
              </aside>

              {/* 命令执行面板 */}
              <main className="lg:col-span-2 space-y-4">
                {/* 命令信息 */}
                {selectedCommandInfo && (
                  <section
                    className="rounded-xl p-5 glass-effect"
                    style={{
                      border: '1px solid var(--border-color)',
                      boxShadow: 'var(--shadow-small)',
                    }}
                  >
                    <div className="flex items-center gap-2 mb-2">
                      {currentClientInfo && <currentClientInfo.icon className="w-5 h-5" />}
                      <h1 className="text-xl font-bold" style={{ color: 'var(--text-primary)' }}>
                        {selectedCommandInfo.name}
                      </h1>
                      <span
                        className="px-2 py-1 rounded text-xs font-semibold"
                        style={{
                          background: 'var(--bg-tertiary)',
                          color: 'var(--text-muted)',
                        }}
                      >
                        {currentClientInfo?.name}
                      </span>
                    </div>
                    <p className="mb-4" style={{ color: 'var(--text-secondary)' }}>
                      {selectedCommandInfo.description}
                    </p>
                    <div className="space-y-3">
                      <div>
                        <span className="text-sm font-semibold" style={{ color: 'var(--text-primary)' }}>
                          Usage:
                        </span>
                        <code
                          className="ml-2 text-sm px-3 py-1 rounded font-mono"
                          style={{ background: 'var(--bg-tertiary)', color: 'var(--accent-primary)' }}
                        >
                          {selectedCommandInfo.usage}
                        </code>
                      </div>
                      <div>
                        <span className="text-sm font-semibold" style={{ color: 'var(--text-primary)' }}>
                          Examples:
                        </span>
                        <ul className="mt-2 space-y-2">
                          {selectedCommandInfo.examples.map((example, idx) => (
                            <li key={idx}>
                              <code
                                className="text-sm px-3 py-2 rounded font-mono block"
                                style={{ background: 'var(--bg-tertiary)', color: 'var(--text-primary)' }}
                              >
                                {example}
                              </code>
                            </li>
                          ))}
                        </ul>
                      </div>
                    </div>
                  </section>
                )}

                {/* 命令输入 */}
                <section
                  className="rounded-xl p-5 glass-effect"
                  style={{
                    border: '1px solid var(--border-color)',
                    boxShadow: 'var(--shadow-small)',
                  }}
                >
                  <h3
                    className="text-xs font-semibold uppercase tracking-wider mb-3"
                    style={{ color: 'var(--text-secondary)' }}
                  >
                    参数 (可选)
                  </h3>
                  <label htmlFor="command-args" className="sr-only">命令参数</label>
                  <input
                    id="command-args"
                    type="text"
                    value={args}
                    onChange={(e) => setArgs(e.target.value)}
                    placeholder="例如: --help 或 anthropic"
                    className="w-full px-4 py-2.5 rounded-lg font-mono text-sm focus:outline-none transition-all"
                    style={{
                      background: 'var(--bg-tertiary)',
                      border: '1px solid var(--border-color)',
                      color: 'var(--text-primary)',
                    }}
                    onFocus={(e) => {
                      e.target.style.borderColor = 'var(--accent-primary)';
                      e.target.style.boxShadow = '0 0 0 3px rgba(139, 92, 246, 0.1)';
                    }}
                    onBlur={(e) => {
                      e.target.style.borderColor = 'var(--border-color)';
                      e.target.style.boxShadow = 'none';
                    }}
                    onKeyDown={(e) => {
                      if (e.key === 'Enter' && !loading) {
                        handleExecute();
                      }
                    }}
                  />
                  <button
                    onClick={handleExecute}
                    disabled={loading}
                    className="mt-3 w-full px-6 py-3 rounded-lg transition-all disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center space-x-2 font-semibold text-sm text-white hover:scale-[1.02]"
                    style={{
                      background: loading
                        ? 'var(--bg-tertiary)'
                        : 'linear-gradient(135deg, var(--accent-primary), var(--accent-secondary))',
                      boxShadow: loading ? 'none' : '0 0 20px var(--glow-primary)',
                    }}
                    aria-label={loading ? '执行中...' : '执行命令'}
                  >
                    {loading ? (
                      <>
                        <div
                          className="w-4 h-4 rounded-full border-2 border-white border-t-transparent animate-spin"
                          aria-hidden="true"
                        />
                        <span>执行中...</span>
                      </>
                    ) : (
                      <>
                        <Play className="w-4 h-4" aria-hidden="true" />
                        <span>执行命令</span>
                      </>
                    )}
                  </button>
                </section>

                {/* 命令输出 - 优化样式 */}
                {output && (
                  <section
                    className="rounded-xl overflow-hidden"
                    style={{
                      border: output.success 
                        ? '2px solid rgba(34, 197, 94, 0.3)' 
                        : '2px solid rgba(239, 68, 68, 0.3)',
                      boxShadow: output.success
                        ? '0 0 30px rgba(34, 197, 94, 0.15)'
                        : '0 0 30px rgba(239, 68, 68, 0.15)',
                    }}
                    aria-live="polite"
                    aria-atomic="true"
                  >
                    {/* 头部 - 更现代的设计 */}
                    <header
                      className="px-5 py-4"
                      style={{
                        background: output.success
                          ? 'linear-gradient(135deg, rgba(34, 197, 94, 0.1), rgba(16, 185, 129, 0.05))'
                          : 'linear-gradient(135deg, rgba(239, 68, 68, 0.1), rgba(220, 38, 38, 0.05))',
                        borderBottom: `1px solid ${output.success ? 'rgba(34, 197, 94, 0.2)' : 'rgba(239, 68, 68, 0.2)'}`,
                      }}
                    >
                      <div className="flex items-center justify-between">
                        <div className="flex items-center space-x-4">
                          {/* 状态图标 */}
                          <div
                            className="flex items-center gap-2 px-3 py-1.5 rounded-lg"
                            style={{
                              background: output.success
                                ? 'rgba(34, 197, 94, 0.15)'
                                : 'rgba(239, 68, 68, 0.15)',
                            }}
                          >
                            {output.success ? (
                              <Check className="w-4 h-4" style={{ color: '#22c55e' }} />
                            ) : (
                              <X className="w-4 h-4" style={{ color: '#ef4444' }} />
                            )}
                            <span
                              className="text-sm font-bold"
                              style={{
                                color: output.success ? '#22c55e' : '#ef4444',
                              }}
                            >
                              {output.success ? '执行成功' : '执行失败'}
                            </span>
                          </div>

                          {/* 退出码 */}
                          <div className="flex items-center gap-2">
                            <span className="text-xs font-semibold" style={{ color: 'var(--text-muted)' }}>
                              Exit Code:
                            </span>
                            <span
                              className="px-2 py-0.5 rounded font-mono text-xs font-bold"
                              style={{
                                background: output.exit_code === 0 ? 'rgba(34, 197, 94, 0.2)' : 'rgba(239, 68, 68, 0.2)',
                                color: output.exit_code === 0 ? '#22c55e' : '#ef4444',
                              }}
                            >
                              {output.exit_code}
                            </span>
                          </div>

                          {/* 执行时间 */}
                          <div className="flex items-center gap-2">
                            <span className="text-xs font-semibold" style={{ color: 'var(--text-muted)' }}>
                              Duration:
                            </span>
                            <span
                              className="px-2 py-0.5 rounded font-mono text-xs font-bold"
                              style={{
                                background: 'rgba(139, 92, 246, 0.15)',
                                color: 'var(--accent-primary)',
                              }}
                            >
                              {output.duration_ms}ms
                            </span>
                          </div>
                        </div>

                        {/* 操作按钮 */}
                        <div className="flex space-x-2">
                          <button
                            onClick={handleCopyOutput}
                            className="p-2 rounded-lg transition-all hover:scale-110"
                            style={{
                              background: 'var(--bg-tertiary)',
                              border: '1px solid var(--border-color)',
                              color: 'var(--accent-primary)',
                            }}
                            title="复制输出"
                            aria-label="复制输出"
                          >
                            <Copy className="w-4 h-4" />
                          </button>
                          <button
                            onClick={handleClearOutput}
                            className="p-2 rounded-lg transition-all hover:scale-110"
                            style={{
                              background: 'var(--bg-tertiary)',
                              border: '1px solid var(--border-color)',
                              color: 'var(--accent-danger)',
                            }}
                            title="清除输出"
                            aria-label="清除输出"
                          >
                            <Trash2 className="w-4 h-4" />
                          </button>
                        </div>
                      </div>
                    </header>

                    {/* 输出内容 - Terminal 风格 */}
                    <div
                      className="terminal-output p-5 font-mono text-sm leading-relaxed"
                      style={{
                        background: '#0f172a',
                        color: '#e2e8f0',
                        minHeight: '200px',
                        maxHeight: '500px',
                        overflowY: 'auto',
                      }}
                    >
                      {output.output && (
                        <div 
                          className="space-y-1"
                          dangerouslySetInnerHTML={{ __html: convertedOutput }}
                        />
                      )}
                      
                      {output.error && (
                        <div
                          className="mt-4 p-4 rounded-lg border-l-4"
                          style={{
                            background: 'rgba(239, 68, 68, 0.1)',
                            borderColor: '#ef4444',
                          }}
                          role="alert"
                        >
                          <div className="flex items-start gap-3">
                            <X className="w-5 h-5 flex-shrink-0 mt-0.5" style={{ color: '#ef4444' }} />
                            <div className="flex-1">
                              <p className="text-sm font-bold mb-1" style={{ color: '#ef4444' }}>
                                Error
                              </p>
                              <pre
                                className="text-sm whitespace-pre-wrap break-words"
                                style={{ color: '#fca5a5' }}
                              >
                                {output.error}
                              </pre>
                            </div>
                          </div>
                        </div>
                      )}

                      {!output.output && !output.error && (
                        <div className="flex items-center justify-center h-32" style={{ color: 'var(--text-muted)' }}>
                          <p className="text-sm">无输出内容</p>
                        </div>
                      )}
                    </div>

                    {/* 底部状态栏 */}
                    <div
                      className="px-5 py-2 flex items-center justify-between text-xs"
                      style={{
                        background: 'var(--bg-tertiary)',
                        borderTop: '1px solid var(--border-color)',
                      }}
                    >
                      <span style={{ color: 'var(--text-muted)' }}>
                        Terminal Output
                      </span>
                      <span style={{ color: 'var(--text-muted)' }}>
                        {output.output?.split('\n').length || 0} lines
                      </span>
                    </div>
                  </section>
                )}
              </main>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

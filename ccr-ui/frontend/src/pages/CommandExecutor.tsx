import { useState, useEffect } from 'react';
import { Play, Copy, Trash2 } from 'lucide-react';
import { Prism as SyntaxHighlighter } from 'react-syntax-highlighter';
import { vscDarkPlus } from 'react-syntax-highlighter/dist/esm/styles/prism';
import { listCommands, executeCommand } from '../api/client';
import type { CommandInfo, CommandResponse } from '../types';
import CollapsibleSidebar from '../components/CollapsibleSidebar';

export default function CommandExecutor() {
  const [commands, setCommands] = useState<CommandInfo[]>([]);
  const [selectedCommand, setSelectedCommand] = useState<string>('');
  const [args, setArgs] = useState<string>('');
  const [output, setOutput] = useState<CommandResponse | null>(null);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    const loadCommands = async () => {
      try {
        const data = await listCommands();
        setCommands(data);
        if (data.length > 0) {
          setSelectedCommand(data[0].name);
        }
      } catch (err) {
        console.error('Failed to load commands:', err);
      }
    };
    loadCommands();
  }, []);

  const handleExecute = async () => {
    if (!selectedCommand) return;

    setLoading(true);
    try {
      const argsArray = args
        .split(' ')
        .map((a) => a.trim())
        .filter((a) => a.length > 0);

      const result = await executeCommand({
        command: selectedCommand,
        args: argsArray,
      });

      setOutput(result);
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

  const handleCopyOutput = () => {
    if (!output) return;
    const text = output.output + (output.error ? '\n' + output.error : '');
    navigator.clipboard.writeText(text);
    alert('输出已复制到剪贴板！');
  };

  const handleClearOutput = () => {
    setOutput(null);
  };

  const selectedCommandInfo = commands.find((c) => c.name === selectedCommand);

  return (
    <div style={{ background: 'var(--bg-primary)', minHeight: '100vh', padding: '20px' }}>
      {/* Background effect */}
      <div className="bg-effect" />

      <div className="max-w-[1800px] mx-auto">
        {/* Layout with collapsible sidebar */}
        <div className="grid grid-cols-[auto_1fr] gap-4">
          {/* Collapsible Navigation */}
          <CollapsibleSidebar />

          {/* Main Command Area */}
          <div className="grid grid-cols-1 lg:grid-cols-3 gap-4">
          {/* Command List */}
          <div
            className="lg:col-span-1 rounded-xl p-5"
            style={{
              background: 'var(--bg-card)',
              backdropFilter: 'blur(20px)',
              border: '1px solid var(--border-color)',
              boxShadow: 'var(--shadow-small)',
            }}
          >
            <h3
              className="text-xs font-semibold uppercase tracking-wider mb-4"
              style={{ color: 'var(--text-secondary)' }}
            >
              可用命令
            </h3>
            <div className="space-y-2">
              {commands.map((cmd) => (
                <button
                  key={cmd.name}
                  onClick={() => setSelectedCommand(cmd.name)}
                  className={`w-full text-left px-4 py-3 rounded-lg transition-all ${
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
            </div>
          </div>

          {/* Command Execution Panel */}
          <div className="lg:col-span-2 space-y-4">
            {/* Command Info */}
            {selectedCommandInfo && (
              <div
                className="rounded-xl p-5"
                style={{
                  background: 'var(--bg-card)',
                  backdropFilter: 'blur(20px)',
                  border: '1px solid var(--border-color)',
                  boxShadow: 'var(--shadow-small)',
                }}
              >
                <h3 className="text-xl font-bold mb-2" style={{ color: 'var(--text-primary)' }}>
                  {selectedCommandInfo.name}
                </h3>
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
              </div>
            )}

            {/* Command Input */}
            <div
              className="rounded-xl p-5"
              style={{
                background: 'var(--bg-card)',
                backdropFilter: 'blur(20px)',
                border: '1px solid var(--border-color)',
                boxShadow: 'var(--shadow-small)',
              }}
            >
              <h4
                className="text-xs font-semibold uppercase tracking-wider mb-3"
                style={{ color: 'var(--text-secondary)' }}
              >
                参数 (可选)
              </h4>
              <input
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
                className="mt-3 w-full px-6 py-3 rounded-lg transition-all disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center space-x-2 font-semibold text-sm text-white"
                style={{
                  background: loading
                    ? 'var(--bg-tertiary)'
                    : 'linear-gradient(135deg, var(--accent-primary), var(--accent-secondary))',
                  boxShadow: loading ? 'none' : '0 0 20px var(--glow-primary)',
                }}
              >
                {loading ? (
                  <>
                    <div className="w-4 h-4 rounded-full border-2 border-white border-t-transparent animate-spin" />
                    <span>执行中...</span>
                  </>
                ) : (
                  <>
                    <Play className="w-4 h-4" />
                    <span>执行命令</span>
                  </>
                )}
              </button>
            </div>

            {/* Command Output */}
            {output && (
              <div
                className="rounded-xl overflow-hidden"
                style={{
                  background: 'var(--bg-card)',
                  backdropFilter: 'blur(20px)',
                  border: '1px solid var(--border-color)',
                  boxShadow: 'var(--shadow-small)',
                }}
              >
                <div
                  className="flex items-center justify-between px-5 py-3"
                  style={{ borderBottom: '1px solid var(--border-color)' }}
                >
                  <div className="flex items-center space-x-4">
                    <h4
                      className="text-xs font-semibold uppercase tracking-wider"
                      style={{ color: 'var(--text-secondary)' }}
                    >
                      输出结果
                    </h4>
                    <span
                      className="px-2 py-1 text-xs font-semibold rounded uppercase tracking-wide"
                      style={{
                        background: output.success ? 'var(--accent-success)' : 'var(--accent-danger)',
                        color: 'white',
                      }}
                    >
                      Exit: {output.exit_code}
                    </span>
                    <span className="text-xs" style={{ color: 'var(--text-muted)' }}>
                      {output.duration_ms}ms
                    </span>
                  </div>
                  <div className="flex space-x-2">
                    <button
                      onClick={handleCopyOutput}
                      className="p-2 rounded transition-colors"
                      style={{ color: 'var(--text-secondary)' }}
                      title="复制输出"
                    >
                      <Copy className="w-4 h-4" />
                    </button>
                    <button
                      onClick={handleClearOutput}
                      className="p-2 rounded transition-colors"
                      style={{ color: 'var(--text-secondary)' }}
                      title="清除输出"
                    >
                      <Trash2 className="w-4 h-4" />
                    </button>
                  </div>
                </div>
                <div>
                  {output.output && (
                    <SyntaxHighlighter
                      language="bash"
                      style={vscDarkPlus}
                      customStyle={{
                        margin: 0,
                        borderRadius: 0,
                        fontSize: '0.75rem',
                        background: 'var(--bg-secondary)',
                      }}
                    >
                      {output.output}
                    </SyntaxHighlighter>
                  )}
                  {output.error && (
                    <div
                      className="p-4"
                      style={{
                        background: 'rgba(239, 68, 68, 0.1)',
                        borderTop: '1px solid var(--accent-danger)',
                      }}
                    >
                      <p className="text-sm font-mono" style={{ color: 'var(--accent-danger)' }}>
                        {output.error}
                      </p>
                    </div>
                  )}
                </div>
              </div>
            )}
          </div>
        </div>
        </div>
      </div>
    </div>
  );
}

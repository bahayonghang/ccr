'use client';

import { useState } from 'react';
import { convertConfig } from '@/lib/api/client';
import type { ConverterRequest, ConverterResponse, CliType } from '@/lib/types';
import { Loader2, ArrowRight, Download, Copy, Upload, FileJson, FileCode, Check, AlertCircle, Home } from 'lucide-react';
import Link from 'next/link';
import Navbar from '@/components/layout/Navbar';
import CollapsibleSidebar from '@/components/layout/CollapsibleSidebar';

const CLI_TYPES: { value: CliType; label: string; description: string }[] = [
  { value: 'claude-code', label: 'Claude Code', description: 'Claude Code CLI configuration (JSON)' },
  { value: 'codex', label: 'Codex', description: 'OpenAI Codex CLI configuration (TOML)' },
  { value: 'gemini', label: 'Gemini', description: 'Google Gemini CLI configuration' },
  { value: 'qwen', label: 'Qwen', description: 'Alibaba Qwen CLI configuration' },
  { value: 'iflow', label: 'iFlow', description: 'iFlow CLI configuration' },
];

export default function ConverterPage() {
  const [sourceFormat, setSourceFormat] = useState<CliType>('claude-code');
  const [targetFormat, setTargetFormat] = useState<CliType>('codex');
  const [configData, setConfigData] = useState('');
  const [convertMcp, setConvertMcp] = useState(true);
  const [convertCommands, setConvertCommands] = useState(true);
  const [convertAgents, setConvertAgents] = useState(true);
  const [isConverting, setIsConverting] = useState(false);
  const [result, setResult] = useState<ConverterResponse | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [successMessage, setSuccessMessage] = useState<string | null>(null);

  const handleFileUpload = (event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    if (file) {
      const reader = new FileReader();
      reader.onload = (e) => {
        const content = e.target?.result as string;
        setConfigData(content);
        setSuccessMessage(`已加载文件: ${file.name}`);
        setTimeout(() => setSuccessMessage(null), 3000);
      };
      reader.onerror = () => {
        setError('读取文件失败');
      };
      reader.readAsText(file);
    }
  };

  const handleConvert = async () => {
    setError(null);
    setSuccessMessage(null);
    setResult(null);

    if (!configData.trim()) {
      setError('请输入或上传配置内容');
      return;
    }

    if (sourceFormat === targetFormat) {
      setError('源格式和目标格式不能相同');
      return;
    }

    setIsConverting(true);

    try {
      const request: ConverterRequest = {
        source_format: sourceFormat,
        target_format: targetFormat,
        config_data: configData,
        convert_mcp: convertMcp,
        convert_commands: convertCommands,
        convert_agents: convertAgents,
      };

      const response = await convertConfig(request);
      setResult(response);
      setSuccessMessage('配置转换成功！');
    } catch (err) {
      setError(err instanceof Error ? err.message : '转换失败');
    } finally {
      setIsConverting(false);
    }
  };

  const handleCopyResult = () => {
    if (result?.converted_data) {
      navigator.clipboard.writeText(result.converted_data);
      setSuccessMessage('已复制到剪贴板');
      setTimeout(() => setSuccessMessage(null), 2000);
    }
  };

  const handleDownloadResult = () => {
    if (result?.converted_data) {
      const blob = new Blob([result.converted_data], { type: 'text/plain' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;

      const extension = result.format === 'json' ? 'json' : result.format === 'toml' ? 'toml' : 'txt';
      const sourceLabel = CLI_TYPES.find(t => t.value === sourceFormat)?.label || sourceFormat;
      const targetLabel = CLI_TYPES.find(t => t.value === targetFormat)?.label || targetFormat;
      a.download = `${sourceLabel}-to-${targetLabel}.${extension}`;

      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(url);

      setSuccessMessage('文件已下载');
      setTimeout(() => setSuccessMessage(null), 2000);
    }
  };

  const handleLoadExample = () => {
    const exampleJson = `{
  "mcpServers": {
    "context7": {
      "command": "npx",
      "args": ["-y", "@upstash/context7-mcp"],
      "env": {
        "API_KEY": "your-api-key-here"
      }
    },
    "filesystem": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-filesystem", "/path/to/allowed/files"]
    }
  }
}`;
    setConfigData(exampleJson);
    setSourceFormat('claude-code');
    setSuccessMessage('已加载示例配置');
    setTimeout(() => setSuccessMessage(null), 3000);
  };

  return (
    <div style={{ background: 'var(--bg-primary)', minHeight: '100vh', padding: '20px' }}>
      <div className="max-w-[1800px] mx-auto">
        <Navbar />

        <div className="grid grid-cols-[auto_1fr] gap-4">
          <CollapsibleSidebar />

          <main className="space-y-6">
            {/* Header */}
            <div className="rounded-xl p-6 glass-effect" style={{ border: '1px solid var(--border-color)', boxShadow: 'var(--shadow-small)' }}>
              <div className="flex items-center justify-between mb-2">
                <h1 className="text-3xl font-bold" style={{ color: 'var(--text-primary)' }}>配置转换器</h1>
                <Link
                  href="/"
                  className="inline-flex items-center gap-2 px-4 py-2 rounded-lg font-medium text-sm transition-colors"
                  style={{
                    background: 'var(--bg-secondary)',
                    color: 'var(--text-secondary)',
                    border: '1px solid var(--border-color)',
                  }}
                >
                  <Home className="w-4 h-4" />
                  <span>返回首页</span>
                </Link>
              </div>
              <p style={{ color: 'var(--text-muted)' }}>
                支持多种 CLI 配置格式之间的互相转换，包括 Claude Code、Codex、Gemini、Qwen、iFlow 等
              </p>
            </div>

            {/* Error/Success Messages */}
            {error && (
              <div className="rounded-lg p-4 flex items-start gap-3" style={{ background: 'rgba(239, 68, 68, 0.1)', border: '1px solid rgb(239, 68, 68)' }}>
                <AlertCircle className="w-5 h-5 flex-shrink-0 mt-0.5" style={{ color: 'rgb(239, 68, 68)' }} />
                <div style={{ color: 'rgb(239, 68, 68)' }}>{error}</div>
              </div>
            )}

            {successMessage && (
              <div className="rounded-lg p-4 flex items-start gap-3" style={{ background: 'rgba(34, 197, 94, 0.1)', border: '1px solid rgb(34, 197, 94)' }}>
                <Check className="w-5 h-5 flex-shrink-0 mt-0.5" style={{ color: 'rgb(34, 197, 94)' }} />
                <div style={{ color: 'rgb(34, 197, 94)' }}>{successMessage}</div>
              </div>
            )}

            {/* Format Selection */}
            <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
              {/* Source Format */}
              <div className="rounded-xl p-6 glass-effect" style={{ border: '1px solid var(--border-color)', boxShadow: 'var(--shadow-small)' }}>
                <div className="flex items-center gap-2 mb-2">
                  <FileJson className="w-5 h-5" style={{ color: 'var(--accent-primary)' }} />
                  <h2 className="text-xl font-bold" style={{ color: 'var(--text-primary)' }}>源格式</h2>
                </div>
                <p className="mb-4" style={{ color: 'var(--text-muted)', fontSize: '14px' }}>选择要转换的配置格式</p>

                <div className="space-y-2">
                  {CLI_TYPES.map((type) => (
                    <div
                      key={type.value}
                      className="p-4 rounded-lg cursor-pointer transition-all"
                      style={{
                        border: sourceFormat === type.value ? '2px solid var(--accent-primary)' : '1px solid var(--border-color)',
                        background: sourceFormat === type.value ? 'rgba(var(--accent-primary-rgb), 0.1)' : 'var(--bg-tertiary)',
                      }}
                      onClick={() => setSourceFormat(type.value)}
                    >
                      <div className="flex items-center justify-between mb-1">
                        <span className="font-medium" style={{ color: 'var(--text-primary)' }}>{type.label}</span>
                        {sourceFormat === type.value && (
                          <span className="px-2 py-0.5 rounded text-xs font-semibold" style={{ background: 'var(--accent-primary)', color: 'white' }}>
                            已选择
                          </span>
                        )}
                      </div>
                      <p className="text-sm" style={{ color: 'var(--text-muted)' }}>{type.description}</p>
                    </div>
                  ))}
                </div>
              </div>

              {/* Target Format */}
              <div className="rounded-xl p-6 glass-effect" style={{ border: '1px solid var(--border-color)', boxShadow: 'var(--shadow-small)' }}>
                <div className="flex items-center gap-2 mb-2">
                  <FileCode className="w-5 h-5" style={{ color: 'var(--accent-secondary)' }} />
                  <h2 className="text-xl font-bold" style={{ color: 'var(--text-primary)' }}>目标格式</h2>
                </div>
                <p className="mb-4" style={{ color: 'var(--text-muted)', fontSize: '14px' }}>选择转换后的配置格式</p>

                <div className="space-y-2">
                  {CLI_TYPES.map((type) => (
                    <div
                      key={type.value}
                      className="p-4 rounded-lg cursor-pointer transition-all"
                      style={{
                        border: targetFormat === type.value && sourceFormat !== type.value ? '2px solid var(--accent-secondary)' : '1px solid var(--border-color)',
                        background: targetFormat === type.value && sourceFormat !== type.value ? 'rgba(var(--accent-secondary-rgb), 0.1)' : 'var(--bg-tertiary)',
                        opacity: sourceFormat === type.value ? 0.5 : 1,
                        cursor: sourceFormat === type.value ? 'not-allowed' : 'pointer',
                      }}
                      onClick={() => {
                        if (sourceFormat !== type.value) {
                          setTargetFormat(type.value);
                        }
                      }}
                    >
                      <div className="flex items-center justify-between mb-1">
                        <span className="font-medium" style={{ color: 'var(--text-primary)' }}>{type.label}</span>
                        {targetFormat === type.value && sourceFormat !== type.value && (
                          <span className="px-2 py-0.5 rounded text-xs font-semibold" style={{ background: 'var(--accent-secondary)', color: 'white' }}>
                            已选择
                          </span>
                        )}
                      </div>
                      <p className="text-sm" style={{ color: 'var(--text-muted)' }}>{type.description}</p>
                    </div>
                  ))}
                </div>
              </div>
            </div>

            {/* Conversion Options */}
            <div className="rounded-xl p-6 glass-effect" style={{ border: '1px solid var(--border-color)', boxShadow: 'var(--shadow-small)' }}>
              <h2 className="text-xl font-bold mb-2" style={{ color: 'var(--text-primary)' }}>转换选项</h2>
              <p className="mb-4" style={{ color: 'var(--text-muted)', fontSize: '14px' }}>选择要转换的配置项</p>

              <div className="flex flex-wrap gap-6">
                <label className="flex items-center gap-2 cursor-pointer">
                  <input
                    type="checkbox"
                    checked={convertMcp}
                    onChange={(e) => setConvertMcp(e.target.checked)}
                    className="w-4 h-4 cursor-pointer"
                  />
                  <span style={{ color: 'var(--text-secondary)' }}>MCP 服务器配置</span>
                </label>
                <label className="flex items-center gap-2 cursor-pointer">
                  <input
                    type="checkbox"
                    checked={convertCommands}
                    onChange={(e) => setConvertCommands(e.target.checked)}
                    className="w-4 h-4 cursor-pointer"
                  />
                  <span style={{ color: 'var(--text-secondary)' }}>Slash 命令配置</span>
                </label>
                <label className="flex items-center gap-2 cursor-pointer">
                  <input
                    type="checkbox"
                    checked={convertAgents}
                    onChange={(e) => setConvertAgents(e.target.checked)}
                    className="w-4 h-4 cursor-pointer"
                  />
                  <span style={{ color: 'var(--text-secondary)' }}>Agents 配置</span>
                </label>
              </div>
            </div>

            {/* Config Input */}
            <div className="rounded-xl p-6 glass-effect" style={{ border: '1px solid var(--border-color)', boxShadow: 'var(--shadow-small)' }}>
              <div className="flex items-center justify-between mb-4">
                <div>
                  <h2 className="text-xl font-bold mb-1" style={{ color: 'var(--text-primary)' }}>配置输入</h2>
                  <p style={{ color: 'var(--text-muted)', fontSize: '14px' }}>粘贴配置内容或上传配置文件</p>
                </div>
                <div className="flex gap-2">
                  <button
                    onClick={handleLoadExample}
                    className="px-3 py-1.5 rounded-lg font-medium text-sm"
                    style={{ background: 'var(--bg-tertiary)', color: 'var(--text-primary)', border: '1px solid var(--border-color)' }}
                  >
                    加载示例
                  </button>
                  <label>
                    <span className="px-3 py-1.5 rounded-lg font-medium text-sm cursor-pointer flex items-center gap-2" style={{ background: 'var(--bg-tertiary)', color: 'var(--text-primary)', border: '1px solid var(--border-color)' }}>
                      <Upload className="w-4 h-4" />
                      上传文件
                    </span>
                    <input
                      type="file"
                      accept=".json,.toml,.yaml,.yml,.txt"
                      className="hidden"
                      onChange={handleFileUpload}
                    />
                  </label>
                </div>
              </div>

              <textarea
                value={configData}
                onChange={(e) => setConfigData(e.target.value)}
                placeholder="在此粘贴配置内容，或点击上传文件按钮..."
                className="w-full px-3 py-2 rounded-lg font-mono text-sm resize-none"
                style={{
                  background: 'var(--bg-tertiary)',
                  border: '1px solid var(--border-color)',
                  color: 'var(--text-primary)',
                  minHeight: '300px'
                }}
              />
              <div className="mt-2 text-sm" style={{ color: 'var(--text-muted)' }}>
                支持的文件格式: JSON, TOML, YAML, TXT
              </div>
            </div>

            {/* Convert Button */}
            <div className="flex justify-center">
              <button
                onClick={handleConvert}
                disabled={isConverting || !configData.trim() || sourceFormat === targetFormat}
                className="px-8 py-3 rounded-lg font-semibold text-white flex items-center gap-2"
                style={{
                  background: isConverting || !configData.trim() || sourceFormat === targetFormat
                    ? 'var(--bg-tertiary)'
                    : 'linear-gradient(135deg, var(--accent-primary), var(--accent-secondary))',
                  boxShadow: isConverting || !configData.trim() || sourceFormat === targetFormat
                    ? 'none'
                    : '0 0 20px var(--glow-primary)',
                  opacity: isConverting || !configData.trim() || sourceFormat === targetFormat ? 0.5 : 1,
                  cursor: isConverting || !configData.trim() || sourceFormat === targetFormat ? 'not-allowed' : 'pointer',
                }}
              >
                {isConverting ? (
                  <>
                    <Loader2 className="w-5 h-5 animate-spin" />
                    转换中...
                  </>
                ) : (
                  <>
                    <ArrowRight className="w-5 h-5" />
                    开始转换
                  </>
                )}
              </button>
            </div>

            {/* Conversion Result */}
            {result && (
              <div className="space-y-6">
                {/* Statistics */}
                <div className="rounded-xl p-6 glass-effect" style={{ border: '1px solid var(--border-color)', boxShadow: 'var(--shadow-small)' }}>
                  <h2 className="text-xl font-bold mb-4" style={{ color: 'var(--text-primary)' }}>转换统计</h2>

                  <div className="grid grid-cols-2 md:grid-cols-5 gap-4 mb-4">
                    <div className="text-center">
                      <div className="text-3xl font-bold" style={{ color: 'var(--accent-primary)' }}>{result.stats?.mcp_servers || 0}</div>
                      <div className="text-sm mt-1" style={{ color: 'var(--text-muted)' }}>MCP 服务器</div>
                    </div>
                    <div className="text-center">
                      <div className="text-3xl font-bold" style={{ color: 'var(--accent-primary)' }}>{result.stats?.slash_commands || 0}</div>
                      <div className="text-sm mt-1" style={{ color: 'var(--text-muted)' }}>Slash 命令</div>
                    </div>
                    <div className="text-center">
                      <div className="text-3xl font-bold" style={{ color: 'var(--accent-primary)' }}>{result.stats?.agents || 0}</div>
                      <div className="text-sm mt-1" style={{ color: 'var(--text-muted)' }}>Agents</div>
                    </div>
                    <div className="text-center">
                      <div className="text-3xl font-bold" style={{ color: 'var(--accent-primary)' }}>{result.stats?.profiles || 0}</div>
                      <div className="text-sm mt-1" style={{ color: 'var(--text-muted)' }}>Profiles</div>
                    </div>
                    <div className="text-center">
                      <div className="text-3xl font-bold" style={{ color: 'var(--accent-primary)' }}>
                        {result.stats?.base_config ? '✓' : '✗'}
                      </div>
                      <div className="text-sm mt-1" style={{ color: 'var(--text-muted)' }}>基础配置</div>
                    </div>
                  </div>

                  {result.warnings && result.warnings.length > 0 && (
                    <div className="rounded-lg p-4" style={{ background: 'rgba(234, 179, 8, 0.1)', border: '1px solid rgb(234, 179, 8)' }}>
                      <div className="font-medium mb-2" style={{ color: 'rgb(234, 179, 8)' }}>转换警告:</div>
                      <ul className="list-disc list-inside space-y-1">
                        {result.warnings.map((warning, index) => (
                          <li key={index} className="text-sm" style={{ color: 'rgb(234, 179, 8)' }}>{warning}</li>
                        ))}
                      </ul>
                    </div>
                  )}
                </div>

                {/* Result Display */}
                <div className="rounded-xl p-6 glass-effect" style={{ border: '1px solid var(--border-color)', boxShadow: 'var(--shadow-small)' }}>
                  <div className="flex items-center justify-between mb-4">
                    <div>
                      <h2 className="text-xl font-bold mb-1" style={{ color: 'var(--text-primary)' }}>转换结果</h2>
                      <p style={{ color: 'var(--text-muted)', fontSize: '14px' }}>格式: {result.format?.toUpperCase()}</p>
                    </div>
                    <div className="flex gap-2">
                      <button
                        onClick={handleCopyResult}
                        className="px-3 py-1.5 rounded-lg font-medium text-sm flex items-center gap-2"
                        style={{ background: 'var(--bg-tertiary)', color: 'var(--text-primary)', border: '1px solid var(--border-color)' }}
                      >
                        <Copy className="w-4 h-4" />
                        复制
                      </button>
                      <button
                        onClick={handleDownloadResult}
                        className="px-3 py-1.5 rounded-lg font-medium text-sm flex items-center gap-2"
                        style={{ background: 'var(--bg-tertiary)', color: 'var(--text-primary)', border: '1px solid var(--border-color)' }}
                      >
                        <Download className="w-4 h-4" />
                        下载
                      </button>
                    </div>
                  </div>

                  <textarea
                    value={result.converted_data}
                    readOnly
                    className="w-full px-3 py-2 rounded-lg font-mono text-sm resize-none"
                    style={{
                      background: 'var(--bg-tertiary)',
                      border: '1px solid var(--border-color)',
                      color: 'var(--text-primary)',
                      minHeight: '400px'
                    }}
                  />
                </div>
              </div>
            )}

            {/* Usage Guide */}
            <div className="rounded-xl p-6 glass-effect" style={{ border: '1px solid var(--border-color)', boxShadow: 'var(--shadow-small)' }}>
              <h2 className="text-xl font-bold mb-4" style={{ color: 'var(--text-primary)' }}>使用说明</h2>

              <div className="space-y-4">
                <div>
                  <h4 className="font-medium mb-2" style={{ color: 'var(--text-secondary)' }}>支持的转换路径</h4>
                  <ul className="list-disc list-inside space-y-1 text-sm" style={{ color: 'var(--text-muted)' }}>
                    <li>Claude Code ↔ Codex (完全支持)</li>
                    <li>其他格式转换功能正在开发中...</li>
                  </ul>
                </div>
                <div>
                  <h4 className="font-medium mb-2" style={{ color: 'var(--text-secondary)' }}>转换说明</h4>
                  <ul className="list-disc list-inside space-y-1 text-sm" style={{ color: 'var(--text-muted)' }}>
                    <li>Claude Code 使用 JSON 格式 (settings.json)</li>
                    <li>Codex 使用 TOML 格式 (config.toml)</li>
                    <li>转换过程会保留所有支持的配置项</li>
                    <li>不支持的配置项会在警告中显示</li>
                  </ul>
                </div>
                <div>
                  <h4 className="font-medium mb-2" style={{ color: 'var(--text-secondary)' }}>注意事项</h4>
                  <ul className="list-disc list-inside space-y-1 text-sm" style={{ color: 'var(--text-muted)' }}>
                    <li>转换前请备份原始配置文件</li>
                    <li>API 密钥等敏感信息需要手动填写</li>
                    <li>建议转换后进行验证测试</li>
                  </ul>
                </div>
              </div>
            </div>
          </main>
        </div>
      </div>
    </div>
  );
}

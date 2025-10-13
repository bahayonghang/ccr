# 贡献指南

感谢你对 CCR UI 项目的关注！我们欢迎所有形式的贡献，包括但不限于代码、文档、测试、问题报告和功能建议。

## 🤝 如何贡献

### 贡献类型

我们欢迎以下类型的贡献：

- **🐛 Bug 修复** - 修复现有功能的问题
- **✨ 新功能** - 添加新的功能特性
- **📚 文档改进** - 改善文档质量和完整性
- **🧪 测试增强** - 添加或改进测试用例
- **🎨 UI/UX 改进** - 改善用户界面和体验
- **⚡ 性能优化** - 提升应用性能
- **🔧 工具改进** - 改善开发工具和流程

### 贡献流程

1. **Fork 项目** - 在 GitHub 上 fork 本项目
2. **创建分支** - 为你的贡献创建一个新分支
3. **开发和测试** - 实现你的更改并确保测试通过
4. **提交 PR** - 创建 Pull Request 并描述你的更改
5. **代码审查** - 等待维护者审查你的代码
6. **合并** - 审查通过后，你的代码将被合并

## 🚀 开发环境设置

### 系统要求

- **Rust 1.70+** (包含 Cargo)
- **Node.js 18+** (包含 npm)
- **Git** 版本控制
- **CCR** 已安装并在 PATH 中可用

### 克隆项目

```bash
# 克隆你的 fork
git clone https://github.com/YOUR_USERNAME/ccr.git
cd ccr/ccr-ui

# 添加上游仓库
git remote add upstream https://github.com/ORIGINAL_OWNER/ccr.git
```

### 安装依赖

```bash
# 使用 Just (推荐)
just install

# 或者手动安装
cd backend && cargo build
cd ../frontend && npm install
cd ../docs && npm install
```

### 启动开发环境

```bash
# 启动完整开发环境
just dev

# 或者分别启动
just dev-backend    # 启动后端服务器
just dev-frontend   # 启动前端开发服务器
just dev-docs       # 启动文档服务器
```

## 📝 编码规范

### 通用规范

- 使用英文编写代码注释和文档
- 保持代码简洁、可读性强
- 遵循项目现有的代码风格
- 为新功能添加适当的测试
- 更新相关文档

### Rust 后端规范

#### 代码风格

```rust
// ✅ 推荐的函数命名和结构
pub async fn get_configs() -> Result<Vec<Config>, ApiError> {
    // 函数实现
}

// ✅ 推荐的错误处理
match execute_ccr_command("list", &[]).await {
    Ok(output) => {
        if output.success {
            // 处理成功情况
        } else {
            // 处理命令执行失败
        }
    }
    Err(e) => {
        log::error!("Failed to execute command: {}", e);
        return Err(ApiError::CcrCommandError(e.to_string()));
    }
}

// ✅ 推荐的结构体定义
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub name: String,
    pub path: String,
    pub is_active: bool,
}
```

#### 格式化和检查

```bash
# 格式化代码
cargo fmt

# 检查代码
cargo clippy -- -D warnings

# 运行测试
cargo test
```

### React 前端规范

#### 组件结构

```typescript
// ✅ 推荐的组件结构
import React, { useState, useEffect } from 'react';
import { SomeType } from '../types';

interface ComponentProps {
  // Props 定义
}

const Component: React.FC<ComponentProps> = ({ prop1, prop2 }) => {
  // 1. Hooks
  const [state, setState] = useState<SomeType | null>(null);
  
  // 2. 副作用
  useEffect(() => {
    // 副作用逻辑
  }, []);
  
  // 3. 事件处理器
  const handleEvent = () => {
    // 处理逻辑
  };
  
  // 4. 条件渲染
  if (!state) {
    return <div>Loading...</div>;
  }
  
  // 5. 主要渲染
  return (
    <div>
      {/* JSX */}
    </div>
  );
};

export default Component;
```

#### 类型定义

```typescript
// ✅ 优先使用 interface
interface Config {
  name: string;
  path: string;
  isActive: boolean;
}

// ✅ 为组件 Props 定义类型
interface ConfigItemProps {
  config: Config;
  onSwitch: (configName: string) => void;
  isLoading?: boolean;
}
```

#### 格式化和检查

```bash
# 格式化代码
npm run lint

# 类型检查
npm run type-check

# 运行测试
npm test
```

## 🧪 测试指南

### 后端测试

#### 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_parse_config_line() {
        let line = "* test-config (/path/to/config)";
        let config = parse_config_line(line).unwrap();
        
        assert_eq!(config.name, "test-config");
        assert_eq!(config.path, "/path/to/config");
        assert!(config.is_active);
    }
    
    #[tokio::test]
    async fn test_execute_ccr_command() {
        // 模拟测试或使用真实命令
        let result = execute_ccr_command("--version", &[]).await;
        assert!(result.is_ok());
    }
}
```

#### 集成测试

```rust
#[cfg(test)]
mod integration_tests {
    use actix_web::{test, App};
    use super::*;
    
    #[actix_web::test]
    async fn test_get_configs_endpoint() {
        let app = test::init_service(
            App::new().configure(configure_routes)
        ).await;
        
        let req = test::TestRequest::get()
            .uri("/api/configs")
            .to_request();
            
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }
}
```

### 前端测试

#### 组件测试

```typescript
import { render, screen, fireEvent } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';
import ConfigItem from '../ConfigItem';

describe('ConfigItem', () => {
  const mockConfig = {
    name: 'test-config',
    path: '/path/to/config',
    isActive: false,
  };

  it('renders config name', () => {
    render(<ConfigItem config={mockConfig} onSwitch={vi.fn()} />);
    expect(screen.getByText('test-config')).toBeInTheDocument();
  });

  it('calls onSwitch when clicked', () => {
    const mockOnSwitch = vi.fn();
    render(<ConfigItem config={mockConfig} onSwitch={mockOnSwitch} />);
    
    fireEvent.click(screen.getByRole('button'));
    expect(mockOnSwitch).toHaveBeenCalledWith('test-config');
  });
});
```

#### API 测试

```typescript
import { describe, it, expect, vi } from 'vitest';
import { getConfigs } from '../api/configService';
import { apiClient } from '../api/client';

vi.mock('../api/client');

describe('configService', () => {
  it('should return configs on success', async () => {
    vi.mocked(apiClient.get).mockResolvedValue({
      data: {
        success: true,
        data: [{ name: 'test', path: '/test', isActive: true }],
      },
    });

    const result = await getConfigs();
    expect(result).toHaveLength(1);
    expect(result[0].name).toBe('test');
  });
});
```

### 运行测试

```bash
# 后端测试
cd backend
cargo test

# 前端测试
cd frontend
npm test

# 覆盖率报告
npm run test:coverage
```

## 📋 提交规范

### Commit 消息格式

我们使用 [Conventional Commits](https://www.conventionalcommits.org/) 规范：

```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

#### 类型 (type)

- `feat`: 新功能
- `fix`: Bug 修复
- `docs`: 文档更新
- `style`: 代码格式化（不影响功能）
- `refactor`: 代码重构
- `test`: 测试相关
- `chore`: 构建过程或辅助工具的变动

#### 示例

```bash
# 新功能
git commit -m "feat(frontend): add config validation UI"

# Bug 修复
git commit -m "fix(backend): handle timeout error in command execution"

# 文档更新
git commit -m "docs: update API documentation"

# 重构
git commit -m "refactor(frontend): extract common API error handling"
```

### 分支命名

- `feature/功能名称` - 新功能开发
- `fix/问题描述` - Bug 修复
- `docs/文档更新` - 文档更新
- `refactor/重构描述` - 代码重构

示例：
```bash
git checkout -b feature/config-validation
git checkout -b fix/command-timeout-handling
git checkout -b docs/api-documentation
```

## 🔍 Pull Request 指南

### PR 标题

使用与 commit 消息相同的格式：

```
feat(frontend): add real-time command output display
fix(backend): resolve memory leak in command executor
docs: improve installation instructions
```

### PR 描述模板

```markdown
## 📝 变更描述

简要描述这个 PR 的目的和实现的功能。

## 🔧 变更类型

- [ ] Bug 修复
- [ ] 新功能
- [ ] 文档更新
- [ ] 代码重构
- [ ] 性能优化
- [ ] 测试改进

## 🧪 测试

- [ ] 添加了新的测试用例
- [ ] 所有现有测试通过
- [ ] 手动测试通过

## 📋 检查清单

- [ ] 代码遵循项目编码规范
- [ ] 自我审查了代码变更
- [ ] 添加了必要的注释
- [ ] 更新了相关文档
- [ ] 没有引入新的警告

## 📸 截图（如适用）

如果是 UI 相关的变更，请提供截图。

## 🔗 相关 Issue

Closes #123
```

### PR 审查流程

1. **自动检查** - CI/CD 流水线会自动运行测试
2. **代码审查** - 至少需要一个维护者的审查
3. **测试验证** - 确保所有测试通过
4. **文档检查** - 验证文档是否需要更新
5. **合并** - 审查通过后合并到主分支

## 🐛 问题报告

### Bug 报告模板

```markdown
## 🐛 Bug 描述

清晰简洁地描述遇到的问题。

## 🔄 复现步骤

1. 进入 '...'
2. 点击 '....'
3. 滚动到 '....'
4. 看到错误

## 🎯 期望行为

描述你期望发生的行为。

## 📸 截图

如果适用，添加截图来帮助解释问题。

## 🖥️ 环境信息

- OS: [e.g. Ubuntu 20.04]
- Browser: [e.g. Chrome 91]
- CCR Version: [e.g. 1.2.3]
- Node.js Version: [e.g. 18.17.0]
- Rust Version: [e.g. 1.70.0]

## 📋 附加信息

添加任何其他有关问题的信息。
```

### 功能请求模板

```markdown
## 🚀 功能描述

清晰简洁地描述你想要的功能。

## 💡 动机

解释为什么这个功能对你或其他用户有用。

## 📝 详细描述

详细描述你希望这个功能如何工作。

## 🎨 可能的实现

如果你有实现想法，请描述。

## 🔄 替代方案

描述你考虑过的任何替代解决方案或功能。

## 📋 附加信息

添加任何其他有关功能请求的信息。
```

## 🏷️ 发布流程

### 版本号规范

我们使用 [Semantic Versioning](https://semver.org/)：

- `MAJOR.MINOR.PATCH` (例如: 1.2.3)
- `MAJOR`: 不兼容的 API 变更
- `MINOR`: 向后兼容的功能新增
- `PATCH`: 向后兼容的问题修复

### 发布检查清单

- [ ] 所有测试通过
- [ ] 文档已更新
- [ ] CHANGELOG.md 已更新
- [ ] 版本号已更新
- [ ] 创建 Git 标签
- [ ] 发布到相应平台

## 🎯 开发最佳实践

### 代码质量

1. **保持简洁** - 编写简洁、可读的代码
2. **单一职责** - 每个函数/组件只做一件事
3. **错误处理** - 适当处理所有可能的错误情况
4. **性能考虑** - 避免不必要的计算和渲染
5. **安全意识** - 注意输入验证和安全漏洞

### 协作规范

1. **及时沟通** - 遇到问题及时在 Issue 中讨论
2. **代码审查** - 认真对待代码审查，给出建设性意见
3. **文档维护** - 保持文档与代码同步
4. **测试覆盖** - 为新功能编写测试
5. **向后兼容** - 尽量保持 API 的向后兼容性

## 📞 获取帮助

如果你在贡献过程中遇到任何问题，可以通过以下方式获取帮助：

- **GitHub Issues** - 提交问题或功能请求
- **GitHub Discussions** - 参与社区讨论
- **代码审查** - 在 PR 中请求帮助和反馈

## 🙏 致谢

感谢所有为 CCR UI 项目做出贡献的开发者！你们的贡献让这个项目变得更好。

### 贡献者

- 查看 [Contributors](https://github.com/your-username/ccr/graphs/contributors) 页面了解所有贡献者

### 特别感谢

- 感谢 CCR 项目提供的基础工具
- 感谢开源社区提供的优秀库和工具
- 感谢所有提供反馈和建议的用户

---

再次感谢你对 CCR UI 项目的关注和贡献！🎉
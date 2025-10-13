# 常见问题解答 (FAQ)

本文档收集了 CCR UI 项目中的常见问题和解决方案。

## 🚀 安装和启动

### Q: 安装依赖时遇到错误怎么办？

**A:** 请按照以下步骤排查：

1. **检查系统要求**
   ```bash
   # 检查 Rust 版本
   rustc --version  # 需要 1.70+
   
   # 检查 Node.js 版本
   node --version   # 需要 18+
   
   # 检查 CCR 是否安装
   ccr --version
   ```

2. **清理缓存重新安装**
   ```bash
   # 清理 Rust 缓存
   cargo clean
   
   # 清理 Node.js 缓存
   npm cache clean --force
   
   # 重新安装
   just install
   ```

3. **检查网络连接**
   - 确保可以访问 crates.io 和 npmjs.com
   - 如果在中国大陆，可能需要配置镜像源

### Q: 启动时提示端口被占用怎么办？

**A:** 可以通过以下方式解决：

1. **查找占用端口的进程**
   ```bash
   # 查看 8081 端口占用情况
   lsof -i :8081
   
   # 查看 5173 端口占用情况
   lsof -i :5173
   ```

2. **使用不同端口启动**
   ```bash
   # 后端使用不同端口
   cargo run -- --port 8082
   
   # 前端修改 vite.config.ts 中的端口配置
   export default defineConfig({
     server: {
       port: 5174
     }
   });
   ```

3. **终止占用进程**
   ```bash
   # 终止占用 8081 端口的进程
   kill -9 $(lsof -t -i:8081)
   ```

### Q: CCR 命令不可用怎么办？

**A:** 请确保 CCR 正确安装：

1. **检查 CCR 安装**
   ```bash
   # 检查 CCR 是否在 PATH 中
   which ccr
   
   # 测试 CCR 命令
   ccr --version
   ```

2. **安装 CCR**
   ```bash
   # 如果未安装，请参考 CCR 项目的安装说明
   # 通常可以通过以下方式安装：
   cargo install ccr
   ```

3. **配置 PATH**
   ```bash
   # 将 CCR 安装目录添加到 PATH
   export PATH="$HOME/.cargo/bin:$PATH"
   
   # 永久配置（添加到 ~/.bashrc 或 ~/.zshrc）
   echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc
   ```

## 🔧 开发问题

### Q: 前端热重载不工作怎么办？

**A:** 尝试以下解决方案：

1. **检查 Vite 配置**
   ```typescript
   // vite.config.ts
   export default defineConfig({
     server: {
       host: '0.0.0.0', // 允许外部访问
       port: 5173,
       watch: {
         usePolling: true, // 在某些系统上需要启用轮询
       },
     },
   });
   ```

2. **清理缓存**
   ```bash
   # 删除 node_modules 和重新安装
   rm -rf node_modules package-lock.json
   npm install
   
   # 清理 Vite 缓存
   rm -rf .vite
   ```

3. **检查文件权限**
   ```bash
   # 确保项目文件有正确的权限
   chmod -R 755 src/
   ```

### Q: TypeScript 类型错误怎么解决？

**A:** 常见的类型错误解决方案：

1. **更新类型定义**
   ```bash
   # 安装最新的类型定义
   npm install --save-dev @types/react @types/react-dom
   ```

2. **检查 tsconfig.json 配置**
   ```json
   {
     "compilerOptions": {
       "strict": true,
       "skipLibCheck": true,
       "moduleResolution": "bundler"
     }
   }
   ```

3. **常见类型问题修复**
   ```typescript
   // ❌ 错误：隐式 any 类型
   const handleClick = (event) => {
     // ...
   };
   
   // ✅ 正确：明确类型
   const handleClick = (event: React.MouseEvent<HTMLButtonElement>) => {
     // ...
   };
   ```

### Q: Rust 编译错误怎么解决？

**A:** 常见编译错误的解决方案：

1. **依赖版本冲突**
   ```bash
   # 更新 Cargo.lock
   cargo update
   
   # 清理并重新构建
   cargo clean && cargo build
   ```

2. **缺少系统依赖**
   ```bash
   # Ubuntu/Debian
   sudo apt-get install build-essential pkg-config libssl-dev
   
   # macOS
   xcode-select --install
   ```

3. **Rust 版本过旧**
   ```bash
   # 更新 Rust
   rustup update
   ```

## 🌐 API 和网络问题

### Q: API 请求失败怎么办？

**A:** 按以下步骤排查：

1. **检查后端服务状态**
   ```bash
   # 检查后端是否运行
   curl http://127.0.0.1:8081/api/system/info
   ```

2. **检查 CORS 配置**
   ```rust
   // 确保 CORS 配置正确
   Cors::default()
       .allowed_origin("http://localhost:5173")
       .allowed_methods(vec!["GET", "POST"])
   ```

3. **检查防火墙设置**
   ```bash
   # 检查端口是否被防火墙阻止
   sudo ufw status
   ```

### Q: 命令执行超时怎么办？

**A:** 可以通过以下方式解决：

1. **增加超时时间**
   ```rust
   // 在 cli_executor.rs 中调整超时时间
   const COMMAND_TIMEOUT: Duration = Duration::from_secs(60); // 增加到 60 秒
   ```

2. **检查 CCR 命令性能**
   ```bash
   # 手动测试命令执行时间
   time ccr list
   ```

3. **优化命令参数**
   - 避免使用会产生大量输出的参数
   - 使用更具体的命令而不是通用命令

## 🎨 UI 和样式问题

### Q: 样式不生效怎么办？

**A:** 检查以下几个方面：

1. **Tailwind CSS 配置**
   ```javascript
   // tailwind.config.js
   module.exports = {
     content: [
       "./src/**/*.{js,ts,jsx,tsx}", // 确保包含所有源文件
     ],
     // ...
   };
   ```

2. **CSS 导入顺序**
   ```typescript
   // main.tsx 中确保正确导入顺序
   import './index.css'; // Tailwind CSS
   import App from './App';
   ```

3. **浏览器缓存**
   ```bash
   # 强制刷新浏览器缓存
   Ctrl+Shift+R (Windows/Linux)
   Cmd+Shift+R (macOS)
   ```

### Q: 深色模式不工作怎么办？

**A:** 检查主题配置：

1. **CSS 变量定义**
   ```css
   :root {
     --bg-primary: #ffffff;
   }
   
   [data-theme="dark"] {
     --bg-primary: #0f172a;
   }
   ```

2. **主题切换逻辑**
   ```typescript
   const toggleTheme = () => {
     const newTheme = theme === 'light' ? 'dark' : 'light';
     setTheme(newTheme);
     document.documentElement.setAttribute('data-theme', newTheme);
   };
   ```

## 🧪 测试问题

### Q: 测试运行失败怎么办？

**A:** 常见测试问题的解决方案：

1. **前端测试环境配置**
   ```typescript
   // vitest.config.ts
   export default defineConfig({
     test: {
       environment: 'jsdom',
       setupFiles: ['./src/test/setup.ts'],
     },
   });
   ```

2. **Mock API 调用**
   ```typescript
   // 在测试中 mock API 客户端
   vi.mock('../api/client', () => ({
     apiClient: {
       get: vi.fn(),
       post: vi.fn(),
     },
   }));
   ```

3. **后端测试配置**
   ```rust
   // 在测试中使用测试配置
   #[cfg(test)]
   mod tests {
       use super::*;
       
       #[tokio::test]
       async fn test_function() {
           // 测试代码
       }
   }
   ```

### Q: 测试覆盖率低怎么办？

**A:** 提高测试覆盖率的方法：

1. **识别未覆盖的代码**
   ```bash
   # 前端覆盖率报告
   npm run test:coverage
   
   # 后端覆盖率报告
   cargo tarpaulin --out Html
   ```

2. **添加缺失的测试**
   - 为每个公共函数添加测试
   - 测试错误处理路径
   - 测试边界条件

## 🚀 部署问题

### Q: 生产构建失败怎么办？

**A:** 检查构建配置：

1. **前端构建问题**
   ```bash
   # 检查构建错误
   npm run build
   
   # 检查 TypeScript 错误
   npm run type-check
   ```

2. **后端构建问题**
   ```bash
   # 使用 release 模式构建
   cargo build --release
   
   # 检查依赖问题
   cargo check
   ```

3. **环境变量配置**
   ```bash
   # 确保生产环境变量正确设置
   VITE_API_BASE_URL=https://api.your-domain.com
   ```

### Q: Docker 部署问题怎么解决？

**A:** 常见 Docker 问题：

1. **构建镜像失败**
   ```dockerfile
   # 确保 Dockerfile 中的路径正确
   COPY Cargo.toml Cargo.lock ./
   COPY src ./src
   ```

2. **容器启动失败**
   ```bash
   # 查看容器日志
   docker logs container_name
   
   # 进入容器调试
   docker exec -it container_name /bin/bash
   ```

3. **网络连接问题**
   ```bash
   # 检查端口映射
   docker run -p 8081:8081 your-image
   ```

## 📊 性能问题

### Q: 应用响应慢怎么办？

**A:** 性能优化建议：

1. **前端性能优化**
   ```typescript
   // 使用 React.memo 优化组件
   const ConfigItem = React.memo<ConfigItemProps>(({ config, onSwitch }) => {
     // 组件实现
   });
   
   // 使用 useMemo 缓存计算结果
   const expensiveValue = useMemo(() => {
     return computeExpensiveValue(data);
   }, [data]);
   ```

2. **后端性能优化**
   ```rust
   // 使用连接池
   // 添加缓存机制
   // 优化数据库查询
   ```

3. **网络优化**
   - 启用 gzip 压缩
   - 使用 CDN
   - 优化 API 响应大小

### Q: 内存使用过高怎么办？

**A:** 内存优化方案：

1. **检查内存泄漏**
   ```typescript
   // 确保清理事件监听器
   useEffect(() => {
     const handler = () => {
       // 处理逻辑
     };
     
     window.addEventListener('resize', handler);
     
     return () => {
       window.removeEventListener('resize', handler);
     };
   }, []);
   ```

2. **优化数据结构**
   - 避免存储不必要的数据
   - 使用适当的数据结构
   - 及时清理缓存

## 🔒 安全问题

### Q: 如何防止安全漏洞？

**A:** 安全最佳实践：

1. **输入验证**
   ```rust
   // 验证用户输入
   fn validate_config_name(name: &str) -> Result<(), String> {
       if name.is_empty() || name.len() > 100 {
           return Err("Invalid config name".to_string());
       }
       Ok(())
   }
   ```

2. **防止命令注入**
   ```rust
   // 避免直接拼接命令字符串
   // 使用参数化命令执行
   ```

3. **HTTPS 配置**
   ```nginx
   # 在生产环境中使用 HTTPS
   server {
       listen 443 ssl;
       ssl_certificate /path/to/cert.pem;
       ssl_certificate_key /path/to/key.pem;
   }
   ```

## 📞 获取更多帮助

如果以上解决方案都无法解决你的问题，可以通过以下方式获取帮助：

### GitHub Issues
- 搜索现有 Issues 看是否有相同问题
- 创建新 Issue 并提供详细信息：
  - 操作系统和版本
  - 软件版本信息
  - 错误日志
  - 复现步骤

### 社区讨论
- 参与 GitHub Discussions
- 查看项目文档
- 阅读源代码注释

### 调试技巧
1. **启用详细日志**
   ```bash
   # 后端
   RUST_LOG=debug cargo run
   
   # 前端
   # 在浏览器开发者工具中查看控制台
   ```

2. **使用调试工具**
   - 浏览器开发者工具
   - React DevTools
   - Rust 调试器

3. **检查系统资源**
   ```bash
   # 检查系统资源使用情况
   htop
   df -h
   free -h
   ```

---

如果你遇到了本 FAQ 中没有涵盖的问题，欢迎提交 Issue 或 PR 来完善这个文档！
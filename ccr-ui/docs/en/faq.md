# Frequently Asked Questions (FAQ)

This document collects common questions and solutions for the CCR UI project.

## 🚀 Installation and Startup

### Q: What to do if I encounter errors when installing dependencies?

**A:** Please follow these troubleshooting steps:

1. **Check system requirements**
   ```bash
   # Check Rust version
   rustc --version  # Requires 1.70+
   
   # Check Node.js version
   node --version   # Requires 18+
   
   # Check if CCR is installed
   ccr --version
   ```

2. **Clean cache and reinstall**
   ```bash
   # Clean Rust cache
   cargo clean
   
   # Clean Node.js cache
   npm cache clean --force
   
   # Reinstall
   just install
   ```

3. **Check network connection**
   - Ensure you can access crates.io and npmjs.com
   - If in mainland China, you may need to configure mirror sources

### Q: What to do if port is already in use during startup?

**A:** You can resolve this by:

1. **Find the process occupying the port**
   ```bash
   # Check port 8081 usage
   lsof -i :8081
   
   # Check port 5173 usage
   lsof -i :5173
   ```

2. **Start with different ports**
   ```bash
   # Backend with different port
   cargo run -- --port 8082
   
   # Frontend - modify port in vite.config.ts
   export default defineConfig({
     server: {
       port: 5174
     }
   });
   ```

3. **Kill the occupying process**
   ```bash
   # Kill process occupying port 8081
   kill -9 $(lsof -t -i:8081)
   ```

### Q: What to do if CCR command is not available?

**A:** Please ensure CCR is correctly installed:

1. **Check CCR installation**
   ```bash
   # Check if CCR is in PATH
   which ccr
   
   # Test CCR command
   ccr --version
   ```

2. **Install CCR**
   ```bash
   # If not installed, refer to CCR project installation instructions
   # Usually can be installed via:
   cargo install ccr
   ```

3. **Configure PATH**
   ```bash
   # Add CCR installation directory to PATH
   export PATH="$HOME/.cargo/bin:$PATH"
   
   # Permanent configuration (add to ~/.bashrc or ~/.zshrc)
   echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc
   ```

## 🔧 Development Issues

### Q: Frontend hot reload not working?

**A:** Try the following solutions:

1. **Restart development server**
   ```bash
   # Stop current server (Ctrl+C)
   # Restart
   just dev-frontend
   ```

2. **Check Vite configuration**
   ```typescript
   // vite.config.ts
   export default defineConfig({
     server: {
       watch: {
         usePolling: true  // Enable polling mode (for WSL/Docker)
       }
     }
   });
   ```

3. **Clear browser cache**
   - Press Ctrl+Shift+R (Windows/Linux)
   - Press Cmd+Shift+R (Mac)

### Q: Backend compilation fails?

**A:** Please check:

1. **Rust version**
   ```bash
   rustc --version  # Must be 1.70+
   ```

2. **Dependency lock file**
   ```bash
   # Delete lock file and rebuild
   rm Cargo.lock
   cargo build
   ```

3. **Check error messages**
   - Read error output carefully
   - Search for solutions on Google or Stack Overflow
   - Check project Issues

## 🌐 API and Connectivity

### Q: Frontend cannot connect to backend?

**A:** Please verify:

1. **Backend is running**
   ```bash
   # Check if backend process is running
   ps aux | grep ccr-ui-backend
   
   # Or check port
   curl http://localhost:8081/api/system/info
   ```

2. **CORS configuration**
   - Backend should allow frontend origin
   - Check Axum CORS middleware configuration

3. **API proxy configuration**
   - Frontend should proxy `/api` requests to backend
   - Check Vite proxy configuration

### Q: API call timeout?

**A:** Possible solutions:

1. **Increase timeout**
   ```typescript
   // In API client
   axios.defaults.timeout = 30000  // 30 seconds
   ```

2. **Check CCR command execution time**
   - Some CCR commands may be slow
   - Consider using async execution

3. **Check network**
   - Ensure backend is accessible
   - Check firewall settings

## 📱 UI and UX

### Q: Theme switch not working?

**A:** Try:

1. **Clear local storage**
   ```javascript
   localStorage.clear()
   ```

2. **Check browser support**
   - Modern browsers (Chrome 90+, Firefox 88+, Safari 14+)
   - Dark mode depends on `prefers-color-scheme`

### Q: Layout issues on mobile?

**A:** Solutions:

1. **Check responsive design**
   - Use browser dev tools mobile mode
   - Test on actual devices

2. **Tailwind breakpoints**
   - Ensure correct responsive classes are used
   - Check `tailwind.config.js` configuration

## 🔐 Security and Permissions

### Q: Cannot execute CCR commands?

**A:** Check:

1. **File permissions**
   ```bash
   # Check CCR executable permissions
   ls -l $(which ccr)
   
   # If needed, add execute permission
   chmod +x $(which ccr)
   ```

2. **User permissions**
   - Ensure current user can execute CCR
   - Check if sudo is required

### Q: Configuration file cannot be read/written?

**A:** Solutions:

1. **Check file permissions**
   ```bash
   # Check config file permissions
   ls -l ~/.ccs_config.toml
   
   # Modify permissions if needed
   chmod 644 ~/.ccs_config.toml
   ```

2. **Check file existence**
   ```bash
   # Create config file if it doesn't exist
   ccr init
   ```

## 📊 Statistics and Data

### Q: Statistics page shows no data?

**A:** Possible reasons:

1. **No API calls yet**
   - Use AI API first
   - Data will be automatically recorded

2. **Statistics file doesn't exist**
   ```bash
   # Check statistics directory
   ls -la ~/.claude/stats/
   ```

3. **Incorrect time range**
   - Try selecting different time range
   - Check if data exists for the selected period

### Q: Cost calculation incorrect?

**A:** Verification steps:

1. **Check pricing configuration**
   - Ensure using latest Anthropic pricing
   - Verify model names match

2. **Check token counts**
   - Compare with API response
   - Verify calculation formula

## 🚀 Performance and Optimization

### Q: Application runs slowly?

**A:** Optimization suggestions:

1. **Check system resources**
   ```bash
   # Check CPU and memory usage
   top
   htop
   ```

2. **Production build**
   ```bash
   # Frontend production build
   npm run build
   
   # Backend release build
   cargo build --release
   ```

3. **Enable caching**
   - Browser caching
   - API response caching
   - Static resource caching

### Q: Large log files?

**A:** Cleanup methods:

1. **Use provided script**
   ```bash
   ./clean-logs.sh
   ```

2. **Manual cleanup**
   ```bash
   # Clean backend logs
   rm -rf backend/logs/*
   
   # Clean frontend logs
   rm -rf frontend/logs/*
   ```

3. **Configure log rotation**
   - Set maximum log file size
   - Set log retention period

## 📚 Documentation and Learning

### Q: Where to find more documentation?

**A:** Documentation resources:

- **Project Documentation** - `/docs` directory in project
- **API Documentation** - `/docs/backend/api.md`
- **Component Documentation** - `/docs/frontend/components.md`
- **Contribution Guide** - `/docs/contributing.md`

### Q: How to learn the technology stack?

**A:** Learning resources:

**Backend (Rust + Axum)**:
- [Rust Official Book](https://doc.rust-lang.org/book/)
- [Axum Documentation](https://docs.rs/axum/)
- [Tokio Tutorial](https://tokio.rs/tokio/tutorial)

**Frontend (Vue 3 + TypeScript)**:
- [Vue 3 Official Docs](https://vuejs.org/)
- [TypeScript Handbook](https://www.typescriptlang.org/docs/)
- [Vite Guide](https://vitejs.dev/guide/)
- [Tailwind CSS Docs](https://tailwindcss.com/docs)

## 🆘 Getting More Help

If you can't find a solution to your problem:

1. **Search Existing Issues**
   - Check [GitHub Issues](https://github.com/your-username/ccr/issues)
   - Someone may have encountered the same problem

2. **Create New Issue**
   - Provide detailed problem description
   - Include error logs
   - Describe reproduction steps

3. **Community Discussion**
   - Join project discussion group
   - Ask questions on Stack Overflow

4. **Contact Maintainers**
   - Create issue on GitHub
   - Send email to project maintainers

---

**Note**: This FAQ will be continuously updated. If you have new questions or solutions, welcome to contribute!


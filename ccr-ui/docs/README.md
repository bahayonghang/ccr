# CCR UI Documentation

This directory contains the multi-language documentation for CCR UI, built with VitePress.

## 📚 Documentation Structure

```
docs/
├── .vitepress/          # VitePress configuration
│   └── config.ts        # Multi-language config
├── zh/                  # Chinese documentation
│   ├── index.md         # Chinese homepage
│   ├── guide/           # User guides
│   ├── frontend/        # Frontend docs
│   ├── backend/         # Backend docs
│   ├── contributing.md  # Contributing guide
│   └── faq.md          # FAQ
├── en/                  # English documentation
│   ├── index.md         # English homepage
│   ├── guide/           # User guides
│   ├── frontend/        # Frontend docs
│   ├── backend/         # Backend docs
│   ├── contributing.md  # Contributing guide
│   └── faq.md          # FAQ
├── public/              # Static assets
│   ├── logo.svg
│   └── favicon.ico
└── index.md            # Root redirect page
```

## 🌐 Supported Languages

- **简体中文 (zh)** - Chinese (Simplified) - Primary language
- **English (en)** - English - Secondary language

## 🚀 Development

### Install Dependencies

```bash
npm install
```

### Start Development Server

```bash
npm run docs:dev
```

Visit `http://localhost:5174`

### Build Documentation

```bash
npm run docs:build
```

Output in `.vitepress/dist/`

### Preview Built Documentation

```bash
npm run docs:preview
```

## 📝 Adding New Documentation

### Add Chinese Documentation

1. Create file in `zh/` directory:
   ```bash
   touch zh/new-doc.md
   ```

2. Add to `.vitepress/config.ts`:
   ```typescript
   // Add to zhNav or zh sidebar
   { text: '新文档', link: '/zh/new-doc' }
   ```

### Add English Documentation

1. Create file in `en/` directory:
   ```bash
   touch en/new-doc.md
   ```

2. Add to `.vitepress/config.ts`:
   ```typescript
   // Add to enNav or en sidebar
   { text: 'New Doc', link: '/en/new-doc' }
   ```

## 🔗 Link Format

### Internal Links

Always use absolute paths from root:

```markdown
<!-- Chinese -->
[快速开始](/guide/getting-started)
[项目结构](/guide/project-structure)

<!-- English -->
[Getting Started](/en/guide/getting-started)
[Project Structure](/en/guide/project-structure)
```

### External Links

```markdown
[GitHub](https://github.com/your-username/ccr)
```

## 🎨 VitePress Features

### Frontmatter

```yaml
---
layout: doc
title: Page Title
description: Page description
---
```

### Alerts

```markdown
::: tip
This is a tip
:::

::: warning
This is a warning
:::

::: danger
This is a danger alert
:::
```

### Code Blocks

````markdown
```typescript
const greeting = 'Hello World'
```
````

### Tables

```markdown
| Header 1 | Header 2 |
|----------|----------|
| Cell 1   | Cell 2   |
```

## 📋 Contributing to Documentation

1. **Edit existing docs**: Directly edit files in `zh/` or `en/` directories
2. **Add new docs**: Follow the structure above
3. **Update config**: Add new pages to navigation and sidebar in `config.ts`
4. **Test locally**: Run `npm run docs:dev` to preview changes
5. **Submit PR**: Create a pull request with your changes

## 🔍 Search

VitePress includes built-in search functionality. It will automatically index all pages in both languages.

## 🌍 Language Detection

The root `index.md` includes a script that automatically redirects users to:
- `/zh/` for Chinese speakers (based on browser language)
- `/en/` for English speakers

Users can manually switch languages using the language selector in the navigation bar.

## 📖 Documentation Guidelines

### Writing Style

- **Chinese**: Use formal written Chinese (书面语)
- **English**: Use clear, concise American English

### Code Examples

- Include practical, working examples
- Add comments to explain complex code
- Use TypeScript for frontend examples
- Use Rust for backend examples

### Images

- Place images in `public/` directory
- Use descriptive file names
- Optimize images for web
- Reference with absolute paths: `/image.png`

## 🛠️ Maintenance

### Update Dependencies

```bash
npm update
```

### Check for Broken Links

```bash
npm run docs:build
# Check build output for warnings
```

### Regenerate Documentation

If you make significant changes to the structure:

1. Clear cache: `rm -rf .vitepress/cache`
2. Rebuild: `npm run docs:build`
3. Test: `npm run docs:preview`

## 📞 Support

- **Issues**: [GitHub Issues](https://github.com/your-username/ccr/issues)
- **Discussions**: [GitHub Discussions](https://github.com/your-username/ccr/discussions)

---

**Last Updated**: November 2, 2025
**VitePress Version**: Latest
**License**: MIT


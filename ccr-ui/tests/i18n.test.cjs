#!/usr/bin/env node

/**
 * i18n Translation Files Validation Test
 *
 * This script validates the consistency and completeness of i18n translation files
 * for the CCR UI frontend application.
 *
 * Test Coverage:
 * - File existence and accessibility
 * - File size comparison (detect major discrepancies)
 * - Namespace extraction and validation
 * - Required namespace presence check
 * - Variable placeholder extraction and overlap analysis
 * - Syntax validation (export default, balanced braces)
 * - Coverage statistics and reporting
 *
 * Exit Codes:
 * - 0: No critical failures (warnings / non-critical mismatches may still exist)
 * - 1: Critical tests failed (missing files, syntax errors, missing required namespaces)
 *
 * Usage:
 *   node tests/i18n.test.cjs
 *   bun run test:i18n
 *
 * @author CCR Development Team
 * @version 1.0.0
 */

const fs = require('fs');
const path = require('path');
const ts = require('typescript');

// ============================================================================
// ANSI Color Codes for Pretty Terminal Output
// ============================================================================

const colors = {
  reset: '\x1b[0m',
  bright: '\x1b[1m',
  dim: '\x1b[2m',

  // Foreground colors
  black: '\x1b[30m',
  red: '\x1b[31m',
  green: '\x1b[32m',
  yellow: '\x1b[33m',
  blue: '\x1b[34m',
  magenta: '\x1b[35m',
  cyan: '\x1b[36m',
  white: '\x1b[37m',

  // Background colors
  bgRed: '\x1b[41m',
  bgGreen: '\x1b[42m',
  bgYellow: '\x1b[43m',
  bgBlue: '\x1b[44m',
};

// Helper functions for colored output
const colorize = (text, color) => `${color}${text}${colors.reset}`;
const success = (text) => colorize(text, colors.green);
const error = (text) => colorize(text, colors.red);
const warning = (text) => colorize(text, colors.yellow);
const info = (text) => colorize(text, colors.cyan);
const cyan = (text) => colorize(text, colors.cyan);
const bold = (text) => colorize(text, colors.bright);
const dim = (text) => colorize(text, colors.dim);

// ============================================================================
// Configuration
// ============================================================================

const I18N_DIR = path.join(__dirname, '../src/i18n/locales');
const ZH_CN_FILE = path.join(I18N_DIR, 'zh-CN.ts');
const EN_US_FILE = path.join(I18N_DIR, 'en-US.ts');
const SRC_DIR = path.join(__dirname, '../src');

// Required namespaces that must exist in both files
const REQUIRED_NAMESPACES = [
  'common',
  'nav',
  'configs',
  'mcp',
  'agents',
  'slashCommands',
  'plugins',
];

// Maximum file size difference ratio (e.g., 0.3 = 30%)
const MAX_SIZE_DIFF_RATIO = 0.35;

// Test results tracker
const testResults = {
  total: 0,
  passed: 0,
  failed: 0,
  warnings: 0,
  critical: 0,
};

// ============================================================================
// Utility Functions
// ============================================================================

/**
 * Print a section header
 */
function printHeader(text) {
  const line = '='.repeat(80);
  console.log('\n' + info(line));
  console.log(bold(info(`  ${text}`)));
  console.log(info(line));
}

/**
 * Print a test result
 */
function printTest(name, passed, message = '', isCritical = false) {
  testResults.total++;

  if (passed) {
    testResults.passed++;
    console.log(success('✓') + ' ' + name);
    if (message) {
      console.log(dim(`  ${message}`));
    }
  } else {
    testResults.failed++;
    if (isCritical) {
      testResults.critical++;
    }
    console.log(error('✗') + ' ' + name);
    if (message) {
      console.log(error(`  ${message}`));
    }
  }
}

/**
 * Print a warning
 */
function printWarning(name, message) {
  testResults.warnings++;
  console.log(warning('⚠') + ' ' + name);
  if (message) {
    console.log(warning(`  ${message}`));
  }
}

/**
 * Read file content safely
 */
function readFileSafe(filePath) {
  try {
    return fs.readFileSync(filePath, 'utf-8');
  } catch (err) {
    return null;
  }
}

/**
 * Get file size in bytes
 */
function getFileSize(filePath) {
  try {
    const stats = fs.statSync(filePath);
    return stats.size;
  } catch (err) {
    return 0;
  }
}

/**
 * Format bytes to human-readable format
 */
function formatBytes(bytes) {
  if (bytes === 0) return '0 Bytes';
  const k = 1024;
  const sizes = ['Bytes', 'KB', 'MB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
}

/**
 * Extract top-level namespaces from TypeScript export default object
 * Uses regex to parse the structure
 */
function extractNamespaces(content) {
  const namespaces = [];

  // Remove comments
  const cleanContent = content.replace(/\/\*[\s\S]*?\*\//g, '')
                              .replace(/\/\/.*/g, '');

  // Find the export default block
  const exportMatch = cleanContent.match(/export\s+default\s+\{([\s\S]*)\}/);
  if (!exportMatch) {
    return namespaces;
  }

  const objectContent = exportMatch[1];

  // Match top-level keys (namespace names)
  // Pattern: word characters followed by colon and opening brace
  const namespacePattern = /^\s*([a-zA-Z_$][a-zA-Z0-9_$]*)\s*:\s*\{/gm;
  let match;

  while ((match = namespacePattern.exec(objectContent)) !== null) {
    namespaces.push(match[1]);
  }

  return namespaces;
}

/**
 * Extract variable placeholders like {name}, {count}, {platform}
 */
function extractPlaceholders(content) {
  const placeholders = new Set();
  const placeholderPattern = /\{([a-zA-Z_][a-zA-Z0-9_]*)\}/g;
  let match;

  while ((match = placeholderPattern.exec(content)) !== null) {
    placeholders.add(match[1]);
  }

  return Array.from(placeholders);
}

/**
 * Count balanced braces in content
 * This checks for balanced object braces, not placeholders
 */
function checkBalancedBraces(content) {
  let openCount = 0;
  let closeCount = 0;

  // Simple character-by-character count
  // This includes placeholders like {name}, but that's okay
  // because we're just checking if the overall structure is balanced
  for (const char of content) {
    if (char === '{') openCount++;
    if (char === '}') closeCount++;
  }

  return {
    balanced: openCount === closeCount,
    open: openCount,
    close: closeCount,
  };
}

/**
 * Check for export default statement
 */
function hasExportDefault(content) {
  return /export\s+default\s+\{/.test(content);
}

/**
 * Extract all leaf string messages from a locale export default object.
 */
function extractLocaleMessages(content, fileName) {
  const sourceFile = ts.createSourceFile(
    fileName,
    content,
    ts.ScriptTarget.Latest,
    true,
    ts.ScriptKind.TS
  );
  const messages = [];
  const stack = [];

  function getPropertyName(node) {
    if (ts.isIdentifier(node) || ts.isStringLiteral(node) || ts.isNumericLiteral(node)) {
      return node.text;
    }

    return null;
  }

  function visitProperty(node) {
    if (!ts.isPropertyAssignment(node)) return;

    const propertyName = getPropertyName(node.name);
    if (!propertyName) return;

    const initializer = node.initializer;
    if (ts.isObjectLiteralExpression(initializer)) {
      stack.push(propertyName);
      initializer.properties.forEach(visitProperty);
      stack.pop();
      return;
    }

    if (ts.isStringLiteral(initializer) || ts.isNoSubstitutionTemplateLiteral(initializer)) {
      const position = sourceFile.getLineAndCharacterOfPosition(initializer.getStart(sourceFile));
      messages.push({
        key: [...stack, propertyName].join('.'),
        value: initializer.text,
        line: position.line + 1,
      });
    }
  }

  function visit(node) {
    if (ts.isExportAssignment(node) && ts.isObjectLiteralExpression(node.expression)) {
      node.expression.properties.forEach(visitProperty);
      return;
    }

    ts.forEachChild(node, visit);
  }

  visit(sourceFile);
  return messages;
}

const EXPECTED_LEAF_COUNT = 4308;

/**
 * Calculate overlap percentage between two arrays
 */
function calculateOverlap(arr1, arr2) {
  const set1 = new Set(arr1);
  const set2 = new Set(arr2);
  const intersection = new Set([...set1].filter(x => set2.has(x)));
  const union = new Set([...set1, ...set2]);

  if (union.size === 0) return 100;
  return ((intersection.size / union.size) * 100).toFixed(2);
}

/**
 * Decode common JS string escapes for static checks
 */
function decodeStringLiteral(raw, quote) {
  let value = raw;

  if (quote === "'") {
    value = value.replace(/\\'/g, "'");
  } else if (quote === '"') {
    value = value.replace(/\\"/g, '"');
  }

  return value.replace(/\\\\/g, '\\');
}

/**
 * Whether an @ usage is explicitly safe in i18n message strings
 */
function isAllowedAtUsage(value, atIndex) {
  // Explicit literal @: {'@'}
  if (value.slice(atIndex - 2, atIndex + 3) === "{'@'}") {
    return true;
  }

  // Allow linked syntax if ever intentionally used in future: @:foo / @.upper:foo
  const nextChar = value[atIndex + 1] || '';
  if (nextChar === ':' || nextChar === '.') {
    return true;
  }

  return false;
}

/**
 * Find unsafe bare @ symbols in translation string literals
 */
function findUnsafeAtSymbols(content) {
  const issues = [];
  const lines = content.split('\n');
  const stringPattern = /(['"])((?:\\.|(?!\1).)*)\1/g;

  for (let index = 0; index < lines.length; index++) {
    const line = lines[index];
    const lineNumber = index + 1;
    const keyMatch = line.match(/^\s*([a-zA-Z_$][a-zA-Z0-9_$]*)\s*:/);
    const key = keyMatch ? keyMatch[1] : '(unknown)';

    let match;
    while ((match = stringPattern.exec(line)) !== null) {
      const quote = match[1];
      const rawValue = match[2];
      const value = decodeStringLiteral(rawValue, quote);

      for (let i = 0; i < value.length; i++) {
        if (value[i] !== '@') continue;
        if (isAllowedAtUsage(value, i)) continue;

        issues.push({
          line: lineNumber,
          key,
          value
        });
      }
    }
  }

  return issues;
}

/**
 * Extract dotted translation keys that contain placeholders
 */
function extractPlaceholderKeysByPath(content) {
  const placeholdersByKey = new Map();
  const stack = [];

  for (const line of content.split('\n')) {
    const trimmed = line.trim();
    if (!trimmed) continue;

    const openMatch = trimmed.match(/^([a-zA-Z_$][a-zA-Z0-9_$]*)\s*:\s*\{$/);
    if (openMatch) {
      stack.push(openMatch[1]);
      continue;
    }

    if (trimmed === '},' || trimmed === '}') {
      stack.pop();
      continue;
    }

    const leafMatch = trimmed.match(/^([a-zA-Z_$][a-zA-Z0-9_$]*)\s*:\s*(['"])((?:\\.|(?!\2).)*)\2\s*,?$/);
    if (!leafMatch) continue;

    const key = [...stack, leafMatch[1]].join('.');
    const placeholders = [...leafMatch[3].matchAll(/\{([a-zA-Z_][a-zA-Z0-9_]*)\}/g)].map((match) => match[1]);
    if (placeholders.length > 0) {
      placeholdersByKey.set(key, placeholders);
    }
  }

  return placeholdersByKey;
}

/**
 * Collect source files for lightweight static checks
 */
function collectSourceFiles(dir, acc = []) {
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    if (entry.name === 'dist' || entry.name === 'node_modules') continue;

    const fullPath = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      collectSourceFiles(fullPath, acc);
      continue;
    }

    if (/\.(ts|tsx)$/.test(entry.name)) {
      acc.push(fullPath);
    }
  }

  return acc;
}

/**
 * Validate simple same-line t/$t placeholder bindings.
 * This intentionally stays conservative to avoid noisy false positives.
 */
function findPlaceholderBindingIssues(files, placeholdersByKey) {
  const issues = [];
  const callPattern = /\$?t\(\s*['"]([^'"]+)['"]\s*,\s*\{([^}]*)\}\s*\)/g;

  for (const file of files) {
    const content = readFileSafe(file);
    if (!content) continue;

    const lines = content.split('\n');
    lines.forEach((line, index) => {
      callPattern.lastIndex = 0;
      let match;
      while ((match = callPattern.exec(line)) !== null) {
        const key = match[1];
        const placeholders = placeholdersByKey.get(key);
        if (!placeholders) continue;

        const passedKeys = match[2]
          .split(',')
          .map((entry) => entry.trim())
          .flatMap((entry) => {
            const explicitMatch = entry.match(/^([a-zA-Z_$][a-zA-Z0-9_$]*)\s*:/);
            if (explicitMatch) {
              return [explicitMatch[1]];
            }

            const shorthandMatch = entry.match(/^([a-zA-Z_$][a-zA-Z0-9_$]*)$/);
            return shorthandMatch ? [shorthandMatch[1]] : [];
          });
        const missing = placeholders.filter((placeholder) => !passedKeys.includes(placeholder));

        if (missing.length > 0) {
          issues.push({
            file: path.relative(path.join(__dirname, '..'), file),
            line: index + 1,
            key,
            expected: placeholders,
            passed: passedKeys,
            missing,
          });
        }
      }
    });
  }

  return issues;
}

// ============================================================================
// Test Suites
// ============================================================================

/**
 * Test 1: File Existence
 */
function testFileExistence() {
  printHeader('Test 1: File Existence Check');

  const zhExists = fs.existsSync(ZH_CN_FILE);
  const enExists = fs.existsSync(EN_US_FILE);

  printTest(
    'zh-CN.ts file exists',
    zhExists,
    zhExists ? `Path: ${ZH_CN_FILE}` : `File not found at: ${ZH_CN_FILE}`,
    true
  );

  printTest(
    'en-US.ts file exists',
    enExists,
    enExists ? `Path: ${EN_US_FILE}` : `File not found at: ${EN_US_FILE}`,
    true
  );

  return zhExists && enExists;
}

/**
 * Test 2: File Size Comparison
 */
function testFileSize() {
  printHeader('Test 2: File Size Comparison');

  const zhSize = getFileSize(ZH_CN_FILE);
  const enSize = getFileSize(EN_US_FILE);

  console.log(info(`  zh-CN.ts: ${formatBytes(zhSize)}`));
  console.log(info(`  en-US.ts: ${formatBytes(enSize)}`));

  const sizeDiff = Math.abs(zhSize - enSize);
  const avgSize = (zhSize + enSize) / 2;
  const diffRatio = avgSize > 0 ? sizeDiff / avgSize : 0;

  console.log(info(`  Size difference: ${formatBytes(sizeDiff)} (${(diffRatio * 100).toFixed(2)}%)`));

  const isReasonable = diffRatio <= MAX_SIZE_DIFF_RATIO;

  printTest(
    'File sizes are reasonably similar',
    isReasonable,
    isReasonable
      ? 'Files are within acceptable size range'
      : `Size difference of ${(diffRatio * 100).toFixed(2)}% exceeds threshold of ${(MAX_SIZE_DIFF_RATIO * 100)}%`,
    false
  );
}

/**
 * Test 3: Namespace Extraction and Validation
 */
function testNamespaces() {
  printHeader('Test 3: Namespace Extraction and Validation');

  const zhContent = readFileSafe(ZH_CN_FILE);
  const enContent = readFileSafe(EN_US_FILE);

  if (!zhContent || !enContent) {
    printTest('Extract namespaces', false, 'Cannot read file content', true);
    return;
  }

  const zhNamespaces = extractNamespaces(zhContent);
  const enNamespaces = extractNamespaces(enContent);

  console.log(info(`  zh-CN namespaces (${zhNamespaces.length}): ${zhNamespaces.join(', ')}`));
  console.log(info(`  en-US namespaces (${enNamespaces.length}): ${enNamespaces.join(', ')}`));

  // Check if namespace counts match
  const countsMatch = zhNamespaces.length === enNamespaces.length;
  printTest(
    'Namespace counts match',
    countsMatch,
    countsMatch
      ? `Both files have ${zhNamespaces.length} namespaces`
      : `zh-CN has ${zhNamespaces.length}, en-US has ${enNamespaces.length}`,
    false
  );

  // Check for missing namespaces
  const zhSet = new Set(zhNamespaces);
  const enSet = new Set(enNamespaces);

  const missingInEn = zhNamespaces.filter(ns => !enSet.has(ns));
  const missingInZh = enNamespaces.filter(ns => !zhSet.has(ns));

  if (missingInEn.length > 0) {
    printWarning(
      'Namespaces in zh-CN but missing in en-US',
      missingInEn.join(', ')
    );
  }

  if (missingInZh.length > 0) {
    printWarning(
      'Namespaces in en-US but missing in zh-CN',
      missingInZh.join(', ')
    );
  }

  if (missingInEn.length === 0 && missingInZh.length === 0) {
    printTest('All namespaces are consistent', true, 'Perfect namespace alignment');
  }

  return { zhNamespaces, enNamespaces };
}

/**
 * Test 4: Required Namespaces Check
 */
function testRequiredNamespaces(namespaceData) {
  printHeader('Test 4: Required Namespaces Check');

  const { zhNamespaces, enNamespaces } = namespaceData || {};

  if (!zhNamespaces || !enNamespaces) {
    printTest('Check required namespaces', false, 'Namespace data not available', true);
    return;
  }

  const zhSet = new Set(zhNamespaces);
  const enSet = new Set(enNamespaces);

  console.log(info(`  Required namespaces: ${REQUIRED_NAMESPACES.join(', ')}`));

  for (const required of REQUIRED_NAMESPACES) {
    const inZh = zhSet.has(required);
    const inEn = enSet.has(required);
    const both = inZh && inEn;

    printTest(
      `Required namespace: ${required}`,
      both,
      both
        ? 'Present in both files'
        : `Missing in: ${!inZh ? 'zh-CN' : ''} ${!inEn ? 'en-US' : ''}`.trim(),
      true
    );
  }
}

/**
 * Test 5: Variable Placeholders Analysis
 */
function testPlaceholders() {
  printHeader('Test 5: Variable Placeholders Analysis');

  const zhContent = readFileSafe(ZH_CN_FILE);
  const enContent = readFileSafe(EN_US_FILE);

  if (!zhContent || !enContent) {
    printTest('Extract placeholders', false, 'Cannot read file content', false);
    return;
  }

  const zhPlaceholders = extractPlaceholders(zhContent);
  const enPlaceholders = extractPlaceholders(enContent);

  console.log(info(`  zh-CN placeholders (${zhPlaceholders.length}): ${zhPlaceholders.slice(0, 10).join(', ')}${zhPlaceholders.length > 10 ? '...' : ''}`));
  console.log(info(`  en-US placeholders (${enPlaceholders.length}): ${enPlaceholders.slice(0, 10).join(', ')}${enPlaceholders.length > 10 ? '...' : ''}`));

  const overlapPercent = calculateOverlap(zhPlaceholders, enPlaceholders);
  console.log(info(`  Placeholder overlap: ${overlapPercent}%`));

  const goodOverlap = overlapPercent >= 80;
  printTest(
    'Placeholder overlap is good',
    goodOverlap,
    goodOverlap
      ? `${overlapPercent}% overlap indicates consistent variable usage`
      : `Only ${overlapPercent}% overlap - check for inconsistent variable names`,
    false
  );

  // Check for placeholders in one file but not the other
  const zhSet = new Set(zhPlaceholders);
  const enSet = new Set(enPlaceholders);

  const onlyInZh = zhPlaceholders.filter(p => !enSet.has(p));
  const onlyInEn = enPlaceholders.filter(p => !zhSet.has(p));

  if (onlyInZh.length > 0) {
    printWarning(
      'Placeholders only in zh-CN',
      onlyInZh.slice(0, 5).join(', ') + (onlyInZh.length > 5 ? '...' : '')
    );
  }

  if (onlyInEn.length > 0) {
    printWarning(
      'Placeholders only in en-US',
      onlyInEn.slice(0, 5).join(', ') + (onlyInEn.length > 5 ? '...' : '')
    );
  }
}

/**
 * Test 6: Syntax Validation
 */
function testSyntax() {
  printHeader('Test 6: Syntax Validation');

  const zhContent = readFileSafe(ZH_CN_FILE);
  const enContent = readFileSafe(EN_US_FILE);

  if (!zhContent || !enContent) {
    printTest('Validate syntax', false, 'Cannot read file content', true);
    return;
  }

  // Test 6.1: Export default statement
  const zhHasExport = hasExportDefault(zhContent);
  const enHasExport = hasExportDefault(enContent);

  printTest(
    'zh-CN has export default statement',
    zhHasExport,
    zhHasExport ? 'Found export default' : 'Missing export default statement',
    true
  );

  printTest(
    'en-US has export default statement',
    enHasExport,
    enHasExport ? 'Found export default' : 'Missing export default statement',
    true
  );

  // Test 6.2: Balanced braces
  const zhBraces = checkBalancedBraces(zhContent);
  const enBraces = checkBalancedBraces(enContent);

  printTest(
    'zh-CN has balanced braces',
    zhBraces.balanced,
    zhBraces.balanced
      ? `${zhBraces.open} opening, ${zhBraces.close} closing`
      : `Unbalanced: ${zhBraces.open} opening, ${zhBraces.close} closing`,
    true
  );

  printTest(
    'en-US has balanced braces',
    enBraces.balanced,
    enBraces.balanced
      ? `${enBraces.open} opening, ${enBraces.close} closing`
      : `Unbalanced: ${enBraces.open} opening, ${enBraces.close} closing`,
    true
  );
}

/**
 * Test 7: i18n @ Literal Safety
 */
function testAtLiteralSafety() {
  printHeader('Test 7: i18n @ Literal Safety');

  const localeFiles = [
    { name: 'zh-CN.ts', path: ZH_CN_FILE },
    { name: 'en-US.ts', path: EN_US_FILE }
  ];

  for (const localeFile of localeFiles) {
    const content = readFileSafe(localeFile.path);
    if (!content) {
      printTest(`Read ${localeFile.name}`, false, 'Cannot read file content', true);
      continue;
    }

    const issues = findUnsafeAtSymbols(content);
    const hasUnsafeAt = issues.length > 0;

    printTest(
      `${localeFile.name} has no unsafe bare @`,
      !hasUnsafeAt,
      hasUnsafeAt
        ? `Found ${issues.length} unsafe @ usage(s)`
        : 'All @ usages are explicit literals or linked syntax',
      true
    );

    if (hasUnsafeAt) {
      const maxPreview = 10;
      issues.slice(0, maxPreview).forEach((issue) => {
        console.log(error(
          `    line ${issue.line}, key ${issue.key}: ${issue.value}`
        ));
      });
      if (issues.length > maxPreview) {
        console.log(error(`    ... and ${issues.length - maxPreview} more`));
      }
    }
  }
}

/**
 * Test 8: Leaf key count (react-i18next runtime reuses the same catalogs)
 */
function testLeafKeyCount() {
  printHeader('Test 8: Leaf Key Count');

  const localeFiles = [
    { name: 'zh-CN.ts', path: ZH_CN_FILE },
    { name: 'en-US.ts', path: EN_US_FILE },
  ];

  const counts = [];
  for (const localeFile of localeFiles) {
    const content = readFileSafe(localeFile.path);
    if (!content) {
      printTest(`Read ${localeFile.name} for leaf key count`, false, 'Cannot read file content', true);
      continue;
    }

    const messages = extractLocaleMessages(content, localeFile.name);
    counts.push(messages.length);
    printTest(
      `${localeFile.name} has ${EXPECTED_LEAF_COUNT} leaf keys`,
      messages.length === EXPECTED_LEAF_COUNT,
      `counted ${messages.length} string leaves`,
      true
    );
  }

  if (counts.length === 2) {
    printTest(
      'zh-CN and en-US leaf key counts match',
      counts[0] === counts[1],
      `${counts[0]} vs ${counts[1]}`,
      true
    );
  }
}

/**
 * Test 9: Placeholder Binding Validation
 */
function testPlaceholderBindings() {
  printHeader('Test 9: Placeholder Binding Validation');

  const zhContent = readFileSafe(ZH_CN_FILE);
  if (!zhContent) {
    printTest('Read zh-CN.ts for binding validation', false, 'Cannot read file content', true);
    return;
  }

  const placeholderMap = extractPlaceholderKeysByPath(zhContent);
  const sourceFiles = collectSourceFiles(SRC_DIR);
  const issues = findPlaceholderBindingIssues(sourceFiles, placeholderMap);

  printTest(
    'Simple i18n placeholder bindings are aligned',
    issues.length === 0,
    issues.length === 0
      ? `Checked ${sourceFiles.length} source files`
      : `Found ${issues.length} binding issue(s)`,
    true
  );

  if (issues.length > 0) {
    issues.slice(0, 10).forEach((issue) => {
      console.log(error(
        `    ${issue.file}:${issue.line} -> ${issue.key} | expected ${issue.expected.join(', ')} | passed ${issue.passed.join(', ') || '(none)'}`
      ));
    });
    if (issues.length > 10) {
      console.log(error(`    ... and ${issues.length - 10} more`));
    }
  }
}

/**
 * Test 10: Coverage Statistics
 */
function testCoverageStats() {
  printHeader('Test 10: Coverage Statistics');

  const zhContent = readFileSafe(ZH_CN_FILE);
  const enContent = readFileSafe(EN_US_FILE);

  if (!zhContent || !enContent) {
    console.log(warning('  Cannot calculate coverage - files not readable'));
    return;
  }

  // Count translation keys (approximate)
  const zhKeys = (zhContent.match(/:\s*['"]/g) || []).length;
  const enKeys = (enContent.match(/:\s*['"]/g) || []).length;

  // Count lines
  const zhLines = zhContent.split('\n').length;
  const enLines = enContent.split('\n').length;

  // Count comments
  const zhComments = (zhContent.match(/\/\/.*/g) || []).length;
  const enComments = (enContent.match(/\/\/.*/g) || []).length;

  console.log(info('\n  Translation Coverage:'));
  console.log(info(`    zh-CN: ${zhKeys} keys, ${zhLines} lines, ${zhComments} comments`));
  console.log(info(`    en-US: ${enKeys} keys, ${enLines} lines, ${enComments} comments`));

  const keyDiff = Math.abs(zhKeys - enKeys);
  const keyDiffPercent = zhKeys > 0 ? ((keyDiff / zhKeys) * 100).toFixed(2) : 0;

  console.log(info(`    Key difference: ${keyDiff} (${keyDiffPercent}%)`));

  const goodCoverage = keyDiffPercent < 5;
  printTest(
    'Translation key coverage is good',
    goodCoverage,
    goodCoverage
      ? 'Key counts are very close'
      : `${keyDiffPercent}% difference in key counts`,
    false
  );
}

// ============================================================================
// Main Test Runner
// ============================================================================

function main() {
  console.log(bold(cyan('\n╔═══════════════════════════════════════════════════════════════════════════════╗')));
  console.log(bold(cyan('║                   i18n Translation Files Validation Test                     ║')));
  console.log(bold(cyan('╚═══════════════════════════════════════════════════════════════════════════════╝')));

  console.log(info('\nTest Configuration:'));
  console.log(info(`  zh-CN file: ${ZH_CN_FILE}`));
  console.log(info(`  en-US file: ${EN_US_FILE}`));
  console.log(info(`  Required namespaces: ${REQUIRED_NAMESPACES.join(', ')}`));
  console.log(info(`  Max size difference: ${(MAX_SIZE_DIFF_RATIO * 100)}%`));

  // Run all tests
  const filesExist = testFileExistence();

  if (!filesExist) {
    console.log(error('\n✗ Critical: Translation files not found. Cannot continue tests.\n'));
    process.exit(1);
  }

  testFileSize();
  const namespaceData = testNamespaces();
  testRequiredNamespaces(namespaceData);
  testPlaceholders();
  testSyntax();
  testAtLiteralSafety();
  testLeafKeyCount();
  testPlaceholderBindings();
  testCoverageStats();

  // Print summary
  printHeader('Test Summary');

  const passRate = testResults.total > 0
    ? ((testResults.passed / testResults.total) * 100).toFixed(2)
    : 0;

  console.log(info('\n  Results:'));
  console.log(`    Total tests:    ${bold(testResults.total)}`);
  console.log(success(`    Passed:         ${testResults.passed}`));

  if (testResults.failed > 0) {
    console.log(error(`    Failed:         ${testResults.failed}`));
  }

  if (testResults.warnings > 0) {
    console.log(warning(`    Warnings:       ${testResults.warnings}`));
  }

  if (testResults.critical > 0) {
    console.log(error(`    Critical:       ${testResults.critical}`));
  }

  console.log(info(`    Pass rate:      ${passRate}%`));

  // Determine exit code
  const exitCode = testResults.critical > 0 ? 1 : 0;

  console.log('\n' + info('─'.repeat(80)));

  if (exitCode === 0 && testResults.failed === 0 && testResults.warnings === 0) {
    console.log(success('\n✓ All tests passed! i18n files are valid and consistent.\n'));
  } else if (exitCode === 0) {
    console.log(warning('\n⚠ Completed without critical failures, but non-blocking issues were found.\n'));
  } else {
    console.log(error('\n✗ Critical tests failed. Please fix the issues above.\n'));
  }

  process.exit(exitCode);
}

// ============================================================================
// Entry Point
// ============================================================================

// Ensure we're in the right directory
const expectedDir = path.join(__dirname, '..');
if (process.cwd() !== expectedDir) {
  console.log(warning(`Changing directory to: ${expectedDir}`));
  process.chdir(expectedDir);
}

// Run tests
main();

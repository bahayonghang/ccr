# i18n Translation Tests

## Overview

This directory contains test scripts for validating the consistency and completeness of i18n translation files.

## Test Files

### i18n.test.cjs

A comprehensive Node.js CommonJS test script that validates the i18n translation files (`zh-CN.ts` and `en-US.ts`).

**Test Coverage:**
- ✓ File existence and accessibility
- ✓ File size comparison (detects major discrepancies)
- ✓ Namespace extraction and validation
- ✓ Required namespace presence check
- ✓ Variable placeholder extraction and overlap analysis
- ✓ Syntax validation (export default, balanced braces)
- ✓ Coverage statistics and reporting

**Exit Codes:**
- `0`: All critical tests passed
- `1`: Critical tests failed (missing files, syntax errors, missing required namespaces)

## Usage

### Run via npm

```bash
# Run the default test
npm test

# Run i18n test specifically
npm run test:i18n
```

### Run directly with Node.js

```bash
node tests/i18n.test.cjs
```

### Run with execution permission

```bash
chmod +x tests/i18n.test.cjs
./tests/i18n.test.cjs
```

## Test Output

The script uses ANSI colors for pretty terminal output:

- 🟢 Green checkmarks (✓) for passed tests
- 🔴 Red crosses (✗) for failed tests
- 🟡 Yellow warnings (⚠) for non-critical issues
- 🔵 Cyan for informational messages

### Sample Output

```
╔═══════════════════════════════════════════════════════════════════════════════╗
║                   i18n Translation Files Validation Test                     ║
╚═══════════════════════════════════════════════════════════════════════════════╝

Test Configuration:
  zh-CN file: /path/to/zh-CN.ts
  en-US file: /path/to/en-US.ts
  Required namespaces: common, nav, configs, mcp, agents, slashCommands, plugins
  Max size difference: 35%

================================================================================
  Test 1: File Existence Check
================================================================================
✓ zh-CN.ts file exists
✓ en-US.ts file exists

[... more tests ...]

================================================================================
  Test Summary
================================================================================

  Results:
    Total tests:    18
    Passed:         17
    Failed:         1
    Warnings:       1
    Pass rate:      94.44%

────────────────────────────────────────────────────────────────────────────────

✓ All tests passed! i18n files are valid and consistent.
```

## Configuration

You can customize the test behavior by modifying constants in `i18n.test.cjs`:

```javascript
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
```

## What Gets Tested

### 1. File Existence
- Checks if both `zh-CN.ts` and `en-US.ts` exist
- **Critical**: If files don't exist, test exits immediately

### 2. File Size Comparison
- Compares file sizes to detect major discrepancies
- Warns if difference exceeds 35% threshold
- **Non-critical**: Large differences suggest incomplete translations

### 3. Namespace Extraction
- Extracts top-level namespaces from both files
- Compares namespace counts and names
- Reports missing namespaces in either file

### 4. Required Namespaces
- Validates that all required namespaces exist in both files
- **Critical**: Missing required namespaces will fail the test

### 5. Variable Placeholders
- Extracts variable placeholders like `{name}`, `{count}`, `{platform}`
- Calculates overlap percentage
- Reports placeholders present in one file but not the other
- **Non-critical**: Low overlap suggests inconsistent variable usage

### 6. Syntax Validation
- Checks for `export default` statement
- Validates balanced braces (opening `{` vs closing `}`)
- **Critical**: Syntax errors will fail the test

### 7. Coverage Statistics
- Counts translation keys, lines, and comments
- Compares key counts between files
- Reports coverage differences

## Common Issues

### Unbalanced Braces
If you see "Unbalanced braces" error:
1. Check for missing or extra `{` or `}` characters
2. Ensure all object and placeholder braces are properly closed
3. Use an editor with bracket matching to find the issue

### Missing Required Namespace
If a required namespace is missing:
1. Add the namespace to the translation file
2. Ensure the namespace exists in both `zh-CN.ts` and `en-US.ts`

### Low Placeholder Overlap
If placeholder overlap is below 80%:
1. Check for inconsistent variable naming (e.g., `{name}` vs `{userName}`)
2. Ensure the same variables are used in both language files
3. This is non-critical but should be reviewed

## Integration with CI/CD

Add to your CI pipeline:

```yaml
# .github/workflows/test.yml
- name: Run i18n tests
  run: npm run test:i18n
```

The script exits with code 0 on success and code 1 on critical failures, making it suitable for CI/CD integration.

## Development

### Adding New Tests

To add a new test:

1. Create a test function following the pattern:
```javascript
function testNewFeature() {
  printHeader('Test N: New Feature Check');

  // Test logic here

  printTest(
    'Test name',
    passed,
    'Optional message',
    isCritical
  );
}
```

2. Call the function in `main()`:
```javascript
function main() {
  // ... existing tests
  testNewFeature();
  // ... summary
}
```

### Modifying Validation Rules

Edit the configuration constants at the top of the file to adjust validation thresholds and requirements.

## License

MIT License - Part of the CCR UI project

## Author

CCR Development Team

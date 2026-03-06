# Multi-Platform Setup Guide

> This page focuses on the CLI setup path for running multiple platforms under one CCR registry.

## Overview

Complete guide for setting up and managing multiple AI CLI platforms with CCR.

## Quick Setup

```bash
# Initialize all platforms
ccr platform init claude
ccr platform init codex
ccr platform init gemini

# Switch between platforms
ccr platform switch claude
ccr add  # Add Claude profile

ccr platform switch codex
ccr add  # Add Codex profile

ccr platform switch gemini
ccr add  # Add Gemini profile
```

## See Also

- [Platform Overview](/en/reference/platforms/)
- [Examples Overview](./index)
- [Quick Start](/en/guide/quick-start)

---

For UI-specific module coverage, continue with [UI Overview](/en/guide/ui-overview).

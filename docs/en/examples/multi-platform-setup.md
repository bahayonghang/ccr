# Multi-Platform Setup Guide

> This page focuses on the CLI setup path for running multiple platforms under one CCR registry.

## Overview

Complete guide for setting up and managing multiple AI CLI platforms with CCR.

## Quick Setup

```bash
# Initialize supported profile templates
ccr claude profile init
ccr codex profile init
ccr grok profile init

# Gemini currently has no profile init command; copy its example manually
mkdir -p ~/.ccr/platforms/gemini
cp examples/gemini/profiles.toml ~/.ccr/platforms/gemini/profiles.toml

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

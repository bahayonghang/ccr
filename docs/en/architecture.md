# Architecture

> **Note:** This documentation is being translated. Please refer to the [Chinese version](./architecture) for complete information.

## Overview

CCR features a strict layered architecture with clear separation of concerns.

## Architecture Layers

```
CLI/Web Layer → Services → Managers → Core/Utils
```

### Components

- **Service Layer**: Business logic
- **Manager Layer**: Data access
- **Web Module**: Axum REST API
- **Core Infrastructure**: File operations, locking, error handling

## Design Patterns

- Atomic file operations
- Multi-process safety via file locking
- Complete audit trail
- Automatic backups

## See Also

- [Quick Start](./quick-start)
- [Configuration Guide](./configuration)
- [Command Reference](./commands/)

---

**Translation in progress.** See [Chinese version](./architecture) for complete architecture documentation.

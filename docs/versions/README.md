# Documentation Versioning

TeachLink documentation supports multiple versions to maintain backward compatibility and provide access to historical documentation.

## Version Structure

```
versions/
├── README.md              # This file
├── current/              # Current version (symlink or copy)
├── v1.0/                 # Version 1.0
│   ├── README.md
│   └── ...
├── v0.9/                 # Version 0.9 (legacy)
│   └── ...
└── CHANGELOG.md          # Version history
```

## Versioning Strategy

### Version Numbering

- **Major versions** (v1.0, v2.0): Significant API changes, new features
- **Minor versions** (v1.1, v1.2): New features, backward compatible
- **Patch versions** (v1.0.1): Bug fixes, documentation updates

### Version Lifecycle

| Status | Description |
|--------|-------------|
| Current | Active development, latest features |
| Supported | Bug fixes, security updates |
| Legacy | Security fixes only |
| Deprecated | No longer maintained |

## Current Versions

| Version | Status | Released | Support Until |
|---------|--------|----------|---------------|
| v1.0 | Current | 2026-02-01 | - |
| v0.9 | Legacy | 2025-08-15 | 2026-08-15 |

## Switching Versions

### Web Interface

Use the version selector in the top navigation bar.

### CLI

```bash
# View specific version
export DOC_VERSION=v1.0

# Use latest
export DOC_VERSION=latest
```

### URL Structure

```
https://docs.teachlink.io/en/v1.0/
https://docs.teachlink.io/en/v0.9/
```

## Contributing Updates

### Updating Documentation

1. Create a branch for your changes
2. Make edits to files in `docs/`
3. Submit a PR with changes

### Adding New Version

When a major version is released:
1. Copy current docs to new version directory
2. Update version references
3. Create version switcher entry
4. Update CHANGELOG.md

## Changelog

See [CHANGELOG.md](./CHANGELOG.md) for detailed version history.

---

*For API versioning, see [API_REFERENCE.md](../API_REFERENCE.md)*

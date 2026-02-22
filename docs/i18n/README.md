# Multilingual Documentation Support

TeachLink documentation is available in multiple languages.

## Supported Languages

| Code | Language | Status |
|------|----------|--------|
| en | English | ✅ Primary |
| es | Spanish | 🟡 In Progress |
| fr | French | 🟡 In Progress |
| de | German | 🟡 Planned |
| zh | Chinese | 🟡 Planned |
| pt | Portuguese | 🟡 Planned |
| ja | Japanese | 🟡 Planned |

## Language Structure

```
i18n/
├── en/                    # English (default)
│   └── README.md
├── es/                    # Spanish
│   └── README.md
├── fr/                    # French
│   └── README.md
├── de/                    # German
│   └── README.md
├── zh/                    # Chinese
│   └── README.md
├── pt/                    # Portuguese
│   └── README.md
└── ja/                    # Japanese
    └── README.md
```

## Contributing Translations

### Translation Process

1. **Choose a language** from the planned list
2. **Check existing translations** in the i18n directory
3. **Create a new branch** for your translation work
4. **Translate content** following the style guide
5. **Submit a PR** for review

### Style Guidelines

- Use formal but accessible language
- Maintain technical terminology consistency
- Follow the original document structure
- Include code examples unchanged

### Translation Priority

1. README.md files
2. API Reference
3. Tutorials
4. Knowledge Base
5. FAQ

## Language Switching

In the web interface, use the language selector in the top navigation to switch between languages.

For CLI tools, set the `DOC_LANG` environment variable:

```bash
export DOC_LANG=es  # Spanish
export DOC_LANG=fr  # French
```

## Translation Status

View the current translation progress in [TRANSLATION_STATUS.md](./TRANSLATION_STATUS.md).

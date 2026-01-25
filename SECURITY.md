# Security Policy

## Our Commitment

The TeachLink team takes security seriously. As a smart contract project handling tokenized rewards and cross-chain bridging, we understand the critical importance of security. We appreciate the security community's efforts in helping us maintain a secure platform.

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| main    | :white_check_mark: |
| testnet | :white_check_mark: |
| < 1.0   | :x:                |

## Reporting a Vulnerability

### For Critical Vulnerabilities

**DO NOT** open a public GitHub issue for security vulnerabilities. Instead:

1. **GitHub Security Advisories** (Preferred)
   - Navigate to [Security Advisories](https://github.com/rinafcode/teachLink_contract/security/advisories/new)
   - Click "Report a vulnerability"
   - Fill out the private report form

2. **Email**
   - Send details to: security@teachlink.io
   - Use PGP encryption if available (key below)
   - Subject: `[SECURITY] Brief description`

### PGP Key

```
-----BEGIN PGP PUBLIC KEY BLOCK-----
[PGP Key will be added upon project launch]
-----END PGP PUBLIC KEY BLOCK-----
```

### What to Include

Please provide as much information as possible:

1. **Vulnerability Type**
   - Smart contract vulnerability
   - Access control issue
   - Reentrancy vulnerability
   - Integer overflow/underflow
   - Denial of service
   - Cross-chain bridge vulnerability
   - Other

2. **Description**
   - Clear explanation of the vulnerability
   - Affected components/functions
   - Potential impact

3. **Reproduction Steps**
   - Step-by-step instructions
   - Proof of concept (if possible)
   - Test environment details

4. **Impact Assessment**
   - Who is affected
   - Potential financial impact
   - Exploitability assessment

5. **Suggested Fix** (optional)
   - Your recommended remediation

## Response Timeline

| Stage | Timeline |
|-------|----------|
| Initial Response | Within 24 hours |
| Triage & Assessment | Within 72 hours |
| Status Update | Weekly until resolved |
| Fix Development | Depends on severity |
| Public Disclosure | After fix is deployed |

## Severity Classification

### Critical (CVSS 9.0-10.0)
- Direct theft of user funds
- Unauthorized minting/burning of tokens
- Bridge manipulation allowing double-spending
- Complete contract takeover

**Response**: Immediate team mobilization, potential emergency pause

### High (CVSS 7.0-8.9)
- Partial loss of funds
- Bypass of access controls
- Manipulation of reward calculations
- Significant denial of service

**Response**: Priority fix within 7 days

### Medium (CVSS 4.0-6.9)
- Limited impact vulnerabilities
- Information disclosure
- Non-critical access control issues
- Temporary denial of service

**Response**: Fix within 30 days

### Low (CVSS 0.1-3.9)
- Minor issues
- Best practice violations
- Theoretical vulnerabilities

**Response**: Fix in next release cycle

## Bug Bounty Program

We offer rewards for responsibly disclosed vulnerabilities:

| Severity | Reward Range |
|----------|-------------|
| Critical | 5,000 - 25,000 TEACH |
| High | 2,000 - 5,000 TEACH |
| Medium | 500 - 2,000 TEACH |
| Low | 100 - 500 TEACH |

### Eligibility

To be eligible for a bounty:

- First reporter of the vulnerability
- Report follows responsible disclosure
- Vulnerability is in-scope (see below)
- No exploitation of the vulnerability
- Compliance with our Code of Conduct

### In-Scope

- TeachLink smart contracts (`contracts/teachlink/`)
- Insurance contracts (`contracts/insurance/`)
- Bridge functionality
- Escrow mechanisms
- Access control and authorization

### Out-of-Scope

- Third-party services and dependencies
- Social engineering attacks
- Physical attacks
- Issues already reported
- Issues in deprecated code
- Frontend/UI vulnerabilities (unless leading to contract exploitation)
- Theoretical vulnerabilities without practical impact

## Security Best Practices

### For Users

1. **Verify Contract Addresses**
   - Always verify you're interacting with official contracts
   - Check addresses against our documentation

2. **Review Transactions**
   - Carefully review transaction details before signing
   - Use hardware wallets for significant amounts

3. **Stay Updated**
   - Follow official channels for security announcements
   - Keep your wallet software updated

### For Developers

1. **Code Review**
   - All code changes require review
   - Security-sensitive changes require additional review

2. **Testing**
   - Comprehensive unit tests
   - Integration tests for cross-module interactions
   - Fuzz testing for edge cases

3. **Audits**
   - Regular security audits by reputable firms
   - Audit reports published in `/docs/audits/`

4. **Dependencies**
   - Regular dependency updates
   - Security scanning of dependencies
   - Minimal dependency footprint

## Security Measures

### Smart Contract Security

- **Access Control**: Role-based permissions with multi-sig for critical functions
- **Input Validation**: Comprehensive input validation on all external functions
- **Reentrancy Guards**: Protection against reentrancy attacks
- **Integer Safety**: Use of checked arithmetic
- **Upgrade Pattern**: Careful consideration of upgradeability (if applicable)

### Operational Security

- **Key Management**: Secure key storage and rotation procedures
- **Monitoring**: Real-time monitoring of contract activity
- **Incident Response**: Documented incident response procedures
- **Emergency Pause**: Ability to pause contracts in emergencies

## Audit History

| Date | Auditor | Scope | Report |
|------|---------|-------|--------|
| TBD | TBD | Full Contract Audit | [Link] |

*Audit reports will be published upon completion.*

## Security Contacts

- **Security Team**: security@teachlink.io
- **Bug Bounty**: bounty@teachlink.io
- **PGP Key**: [Available upon request]

## Acknowledgments

We thank all security researchers who have helped improve TeachLink's security. Contributors who report valid vulnerabilities will be recognized in our [Hall of Fame](docs/governance/HALL_OF_FAME.md).

---

## Updates

This security policy may be updated periodically. Major changes will be announced through official channels.

*Last updated: January 2026*

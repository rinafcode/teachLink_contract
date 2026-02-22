# Frequently Asked Questions - TeachLink

Common questions and answers about the TeachLink platform.

## Table of Contents

1. [General Questions](#general-questions)
2. [Technical Questions](#technical-questions)
3. [Development Questions](#development-questions)
4. [Governance Questions](#governance-questions)
5. [Insurance Questions](#insurance-questions)

---

## General Questions

### What is TeachLink?

TeachLink is a decentralized knowledge-sharing platform built on the Stellar blockchain. It enables educators to tokenize their courses, students to earn rewards for learning, and provides a cross-chain bridge for global accessibility.

### How does TeachLink work?

TeachLink combines several key features:
- **Course Tokenization**: Educators can create tokens representing their courses
- **Reward System**: Students earn tokens for completing courses
- **Cross-Chain Bridge**: Tokens can be bridged to other blockchains
- **Insurance Protection**: Course completion insurance for students
- **Governance**: Community-driven decision making

---

## Technical Questions

### What blockchain does TeachLink use?

TeachLink is built on the Stellar blockchain, using Soroban smart contracts.

### What programming languages are used?

- **Rust**: Smart contract development (Soroban)
- **JavaScript/TypeScript**: Frontend and API integrations
- **Python**: Indexer and data processing

### How do I set up a local development environment?

See the [Installation Guide](../knowledge-base/getting-started/installation.md) for detailed setup instructions.

---

## Development Questions

### How do I create a new course?

1. Deploy a new course token using the TeachLink contract
2. Set up the course metadata (name, description, price)
3. Configure reward distribution
4. Optionally add insurance coverage

### How do I integrate with the bridge?

Use the `bridge_out` and `complete_bridge` functions in the TeachLink contract. See the [API Reference](../API_REFERENCE.md) for details.

---

## Governance Questions

### How do I participate in governance?

Any token holder can:
1. Submit a proposal
2. Vote on active proposals
3. Execute passed proposals

See the [Governance Documentation](../governance/README.md) for detailed instructions.

---

## Insurance Questions

### What does the insurance cover?

The insurance protects students by providing refunds if a course is not completed due to instructor withdrawal or course cancellation.

### How do I file a claim?

1. Navigate to the Insurance section
2. Select the course with the issue
3. Submit a claim with documentation
4. Wait for review and approval

---

## Still Have Questions?

- Check the [Knowledge Base](../knowledge-base/README.md)
- Join our [Community Forum](https://community.teachlink.io)
- Open an [Issue](https://github.com/luhrhenz/rinacode/issues)

---

*Last updated: 2026-02-22*

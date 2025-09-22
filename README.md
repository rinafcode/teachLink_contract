TeachLink Solidity Smart Contracts

TeachLink is a decentralized knowledge-sharing platform.
This repository contains Solidity smart contracts to support core on-chain features such as tokenized learning rewards, proof-of-participation, and educator incentives — all deployed on the Stella ecosystem.

🚀 Project Goals

Enable on-chain user rewards and proof of skill acquisition.

Implement token logic for incentivizing creators and learners.

Build a modular, testable Solidity-based infrastructure compatible with the Stella ecosystem.

🛠️ Getting Started
1. Prerequisites

Ensure the following are installed:

Node.js & npm

Hardhat (or Foundry/Truffle)

Solidity compiler (solc)

Git

2. Initialize the Project
git clone https://github.com/yourorg/teachlink-solidity.git
cd teachlink-solidity
npm install

📁 Project Structure
teachlink-solidity/
├── contracts/
│   └── TeachLinkToken.sol     # Main smart contract
├── test/
│   └── TeachLinkToken.test.js # Unit tests
├── hardhat.config.js          # Hardhat project config
├── .gitignore                 # Ignore build outputs and secrets
└── README.md                  # Project docs

✨ Features

🎓 Course Reward Logic (WIP): Track and distribute token rewards per lesson/module.

🪙 Custom Token Standard: ERC20-based logic adapted to TeachLink’s needs.

🔐 Secure & Modular: Follows Stella ecosystem security practices and modular development patterns.

🧪 Test Driven: Unit tests for validating contract logic.

🧪 How to Build & Test

Build the contracts:

npx hardhat compile


Run unit tests:

npx hardhat test

🧩 Example: Minimal Token Logic (Placeholder)
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

contract TeachLinkToken {
    mapping(address => uint256) public balances;

    function mint(address recipient, uint256 amount) external {
        balances[recipient] += amount;
    }
}

🔧 Deployment Guide

Full deployment instructions are in DEPLOYMENT.md.

🤝 Contributing

Please read our CONTRIBUTING.md for guidelines.

To get started:

git clone https://github.com/yourorg/teachlink-solidity.git
cd teachlink-solidity
npm install

📜 License

This project is licensed under the MIT License.

📬 Join the Community

Telegram

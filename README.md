### 📜 `contract/README.md`

```md
# Teachme Smart Contracts

This repository contains the smart contracts powering Teachme’s earning and reward system.

## 🧱 Stack
- Solidity
- Hardhat
- Ethers.js
- OpenZeppelin

## 🔧 Setup

```bash
git clone https://github.com/your-org/teachme-contract.git
cd teachme-contract
npm install
cp .env.example .env
npx hardhat compile
🧪 Testing
bash
Copy
Edit
npx hardhat test
🔐 Features
Token reward system for knowledge sharing

Access control for content creators

Payment/earning logic

Gas-efficient smart contracts

📄 Deployment
bash
Copy
Edit
npx hardhat run scripts/deploy.js --network <network>

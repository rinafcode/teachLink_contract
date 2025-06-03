TeachLink Cairo Smart Contracts TeachLink is a decentralized knowledge-sharing
platform. This repository contains Cairo 1.0 smart contracts to support core
on-chain features such as tokenized learning rewards, proof-of-participation,
and educator incentives—all deployed on StarkNet.

🚀 Project Goals Enable on-chain user rewards and proof of skill acquisition.

Implement token logic for incentivizing creators and learners.

Build a modular, testable Cairo-based infrastructure compatible with StarkNet.

🛠️ Getting Started

1. Prerequisites Ensure the following are installed:

Scarb

Cairo 1.0 compiler (cairo-test, cairo-run, etc.)

StarkNet CLI

Git

✅ You can install Scarb via:

bash curl --proto '=https' --tlsv1.2 -sSf https://install.scarb.sh | sh 2.
Initialize the Project bash

scarb new teachlink-cairo cd teachlink-cairo 📁 Project Structure bash

teachlink-cairo/ ├── src/ │ └── lib.cairo # Main contract entry point ├── tests/
│ └── test_basic.cairo # Unit tests ├── Scarb.toml # Scarb project config ├──
.gitignore # Ignore build outputs and secrets └── README.md # Project docs ✨
Features 🎓 Course Reward Logic (WIP): Track and distribute token rewards per
lesson/module.

🪙 Custom Token Standard: Optional ERC20-like logic adapted to TeachLink’s
needs.

🔐 Secure & Modular: Follow StarkNet security practices and modular development.

🧪 Test Driven: Cairo test framework support for validating logic.

🧪 How to Build & Test Build the Project bash

scarb build Run Unit Tests bash

scarb test 🧩 Example: Minimal Token Logic (Placeholder) rust Copy Edit
#[contract] mod teachlink_token { #[storage] struct Storage { balances:
LegacyMap::<ContractAddress, u256>, }

    #[external]
    fn mint(recipient: ContractAddress, amount: u256) {
        balances::write(recipient, amount);
    }

} 🔧 Deployment Guide Full deployment instructions are in DEPLOYMENT.md

See [`DEPLOYMENT.md`](../DEPLOYMENT.md) 🤝 Contributing Please read our
CONTRIBUTING.md for guidelines.

To get started:

bash Copy Edit git clone https://github.com/yourorg/teachlink-cairo.git cd
teachlink-cairo scarb build 📜 License This project is licensed under the MIT
License.

📬 Contact

## 📬 Join the Community

- [Telegram](t.me/teachlinkOD)

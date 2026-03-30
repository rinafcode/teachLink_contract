# Byzantine Fault Tolerance (BFT) Consensus

## Overview
BFT consensus ensures agreement among distributed nodes even when some act maliciously or fail. It guarantees **safety** (no two honest nodes decide differently) and **liveness** (honest nodes eventually decide).

## Algorithm Steps
1. **Proposal Phase**  
   - A leader proposes a block.
2. **Pre-Prepare Phase**  
   - Leader broadcasts proposal to all validators.
3. **Prepare Phase**  
   - Validators verify and broadcast prepare messages.
4. **Commit Phase**  
   - Validators broadcast commit messages.
5. **Decision**  
   - Once ≥ 2/3 of validators commit, the block is finalized.

## Mathematical Proofs
- **Safety:**  
  With `n` validators, tolerating up to `f` faulty nodes requires `n ≥ 3f + 1`.  
  Proof: Two conflicting blocks cannot both reach ≥ 2/3 commits without overlap of ≥ f+1 honest validators.
- **Liveness:**  
  As long as ≤ f nodes are faulty, honest validators eventually receive enough messages to finalize.

## Flowchart
![BFT Consensus Flow](../diagrams/bft-consensus-flow.png)

## Examples
- **Normal Operation:** Leader proposes block, validators reach consensus in 3 rounds.
- **Faulty Leader:** Honest validators detect invalid proposal, reject, and elect new leader.
- **Network Delay:** Messages delayed, but eventual delivery ensures liveness.

## Use Cases
- Blockchain consensus (Cosmos, Tendermint).
- Distributed databases requiring strong consistency.

# Validator Slashing Algorithms

## Overview
Slashing penalizes validators who act maliciously or negligently. It protects the network by discouraging misbehavior and compensating affected parties.

## Slashing Conditions
1. **Double-Signing:** Validator signs two conflicting blocks.
2. **Downtime:** Validator fails to participate in consensus for a threshold period.
3. **Equivocation:** Validator sends contradictory messages in the same round.

## Algorithm Steps
1. Detect violation via consensus messages or monitoring.
2. Submit evidence to slashing contract/module.
3. Verify evidence cryptographically.
4. Apply penalty:
   - Reduce validator stake.
   - Jail validator (temporary removal).
   - Redistribute slashed funds to honest participants.

## Mathematical Justification
- **Penalty Thresholds:**  
  Slashing fraction `s` must satisfy:  
  `s ≥ (cost of attack) / (potential gain)`  
  Ensures rational validators prefer honest behavior.

## Flowchart
![Validator Slashing Flow](../diagrams/validator-slashing-flow.png)

## Examples
- **Double-Signing:** Validator signs two blocks at height `h`. Evidence submitted, stake reduced by 5%.
- **Downtime:** Validator misses 100 consecutive blocks. Stake reduced, validator jailed.
- **Equivocation:** Contradictory prepare messages detected. Validator slashed.

## Use Cases
- Proof-of-Stake blockchains (Ethereum 2.0, Cosmos).
- Any consensus system requiring strong incentives for honesty.

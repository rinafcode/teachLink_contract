# System Flow Diagrams

This document visualizes the core processes within the TeachLink ecosystem using Mermaid diagrams.

## 🎓 Learner Journey

The process of a learner enrolling in a course and receiving a credential.

```mermaid
sequenceDiagram
    participant Learner
    participant UI
    participant Contract as TeachLink Contract
    participant Stellar as Stellar Ledger

    Learner->>UI: Select Course
    UI->>Contract: Enroll(learner, course_id)
    Contract->>Stellar: Create Escrow
    Stellar-->>Contract: Success
    Contract-->>UI: Enrollment Confirmed
    
    Note over Learner, Stellar: Learning Process...

    Learner->>UI: Complete Final Assessment
    UI->>Contract: SubmitAssessment(answers)
    Contract->>Contract: Verify Score
    alt Passed
        Contract->>Stellar: Release Escrow to Educator
        Contract->>Stellar: Mint Credential NFT
        Contract-->>UI: Course Completed!
    else Failed
        Contract-->>UI: Try Again
    end
```

## 💰 Reward Distribution

How rewards are calculated and distributed to educators and learners.

```mermaid
graph TD
    A[Course Completion] --> B{Verify Eligibility}
    B -- Yes --> C[Calculate Reward Pool]
    B -- No --> D[End]
    C --> E[Educator Share: 70%]
    C --> F[Platform Fee: 5%]
    C --> G[Learner Cashback: 25%]
    E --> H[Transfer to Educator Wallet]
    F --> I[Transfer to DAO Treasury]
    G --> J[Transfer to Learner Wallet]
```

## 🌉 Cross-Chain Bridge Flow

(Placeholder for cross-chain integration logic)

```mermaid
graph LR
    User[User on Ethereum] -- Lock Assets --> Bridge[Ethereum Bridge]
    Bridge -- Relay Event --> TL[TeachLink Contract]
    TL -- Mint Wrapped Asset --> UserTL[User on Stellar]
```

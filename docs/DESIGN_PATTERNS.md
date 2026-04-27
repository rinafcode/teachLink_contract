# TeachLink Design Patterns

This document outlines the standard software design patterns implemented across the TeachLink ecosystem. Utilizing standard design patterns ensures code maintainability, scalability, and consistency for onboarding new contributors.

## 1. Strategy Pattern

**Context**: Conditional branches that change behavior based on a type or flag (e.g., generating explanations for AI recommendations) often result in large, unmaintainable `switch` blocks.
**Usage**: Used in `ExplanationGenerator` (`explainability.ts`) to dynamically select the correct algorithm for extracting dominant signal explanations.

**Example**:
```typescript
export interface ExplanationStrategy {
  matches(dominantSignal: string): boolean;
  extract(rankingSignal: any, userProfile: Types.UserProfile, similarUsers?: string[]): ExplanationResult;
}

class ContentSignalStrategy implements ExplanationStrategy {
  matches(signal: string) { return signal === 'contentSignal'; }
  extract(...) { ... }
}
```

## 2. Builder Pattern

**Context**: Classes requiring multiple configuration options and dependencies (especially when many are optional) often lead to confusing constructors (the "telescoping constructor" anti-pattern).
**Usage**: Implemented for `PrivacyComplianceManager` in `privacy.ts` to cleanly instantiate the manager and supply various privacy engines systematically.

**Example**:
```typescript
const privacyManager = new PrivacyComplianceManagerBuilder()
  .withAnonymizer(new UserAnonymizer())
  .withDifferentialPrivacy(new DifferentialPrivacyEngine(0.5))
  .build();
```

## 3. Facade Pattern

**Context**: Complex subsystems with many interconnected mechanisms shouldn't expose their granular logic to the application's business layer.
**Usage**: `PrivacyComplianceManager` acts as a facade, exposing clean, high-level methods like `processUserDataPrivate` while hiding the interactions between `UserAnonymizer`, `DifferentialPrivacyEngine`, and `DataMinimizer`.

## 4. Dependency Injection (DI)

**Context**: Hardcoding dependencies tightly couples classes and makes unit testing incredibly difficult.
**Usage**: Across the NestJS Indexer and system layers, components are injected via class constructors. 
**Example**: The `LoggerService` is instantiated by the framework and passed dynamically to processors, allowing the testing environment to swap it for a mock logger without modifying the core service.
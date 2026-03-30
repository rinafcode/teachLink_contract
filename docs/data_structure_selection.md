# Data Structure Selection Criteria

To ensure optimal performance and gas usage in the TeachLink contract, follow these criteria when selecting data structures and storage patterns.

## 1. Storage Type Selection

| Storage Type | Use Case | Threshold / Limits |
|--------------|----------|-------------------|
| **Instance** | Core configuration, global state, small lists (< 100 entries). | Shared 64KB limit. Hard limit on contract serialization. |
| **Persistent**| Large datasets, transaction history, user/provider positions, historical records. | Scales with network limits. Each key is separate. |
| **Temporary** | Ephemeral state, short-lived locks, sequence numbers within a tx (rare in Soroban). | Cleared after expiration. |

## 2. Access Pattern Optimization

### Granular Key Pattern (Preferred)
Instead of storing a full `Map<K, V>` in a single storage key, use a unique `DataKey` for each entry.

**Why?**
- **Gas Efficiency**: Loading/saving a single entry costs much less gas than a whole map.
- **Scalability**: Avoids hitting the 64KB/1MB storage limits for individual keys.
- **Replay Protection**: Individual keys for nonces are more efficient and safer.

**Example:**
```rust
// AVOID:
let mut map: Map<u64, Tx> = env.storage().instance().get(&KEY).unwrap();
map.set(id, tx);
env.storage().instance().set(&KEY, &map);

// PREFER:
env.storage().persistent().set(&DataKey::Transaction(id), &tx);
```

### Decoupled Nested Data
Remove large collections from nested structs.

**Why?**
- Loading a struct shouldn't force loading all its historical data or related providers.

**Example:**
- `LiquidityPool` stores metadata (token, chain_id), but `LPPosition` is stored separately under `DataKey::LPPoolProvider(chain_id, provider)`.

## 3. Iteration and Lists

- **Iteration is expensive**: Avoid unbounded iteration in contract calls.
- **Maintain Lists sparingly**: If iteration is truly needed (e.g., for validators), maintain a separate `Vec<K>` in instance storage, but keep it small.
- **Off-chain Indexing**: Prefer off-chain indexing (e.g., via events) for complex queries.

## 4. Selection Flowchart

1. **Is the data a single global config?** → Use `instance()` storage with a `Symbol` key.
2. **Is it a collection of things?**
   - **Fixed/Small count (< 10-20 entries)?** → Large `Map` in `instance()` is acceptable for simplicity.
   - **Dynamic/Varying size?** → Use `DataKey::Variant(Key)` in `persistent()` storage.
3. **Do you need to iterate?**
   - **Yes (small sets only)** → Maintain a `Vec` list + granular keys.
   - **No** → Granular keys only.

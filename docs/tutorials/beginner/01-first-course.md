# Tutorial: Your First Course

In this tutorial, you'll learn how to create your first tokenized course on TeachLink.

## Prerequisites

- Rust toolchain installed
- Stellar CLI installed
- Testnet wallet with XLM

## Step 1: Initialize Your Project

Create a new directory for your course:

```bash
mkdir my-first-course
cd my-first-course
```

## Step 2: Create the Course Contract

Create a new Rust project:

```bash
cargo new --lib course
cd course
```

## Step 3: Define Course Metadata

Add the following to your `src/lib.rs`:

```rust
use soroban_sdk::{Address, Env};

pub struct Course {
    pub name: String,
    pub description: String,
    pub price: i128,
    pub instructor: Address,
}

impl Course {
    pub fn new(name: String, description: String, price: i128, instructor: Address) -> Self {
        Self {
            name,
            description,
            price,
            instructor,
        }
    }
}
```

## Step 4: Build and Deploy

Build your contract:

```bash
cargo build --target wasm32-unknown-unknown --release
```

Deploy to testnet:

```bash
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/course.wasm \
  --source your-wallet-key \
  --network testnet
```

## Step 5: Initialize the Course

```rust
use course::Course;

let course = Course::new(
    "Introduction to Blockchain".to_string(),
    "Learn the fundamentals of blockchain technology".to_string(),
    1000, // Price in XLM (with 7 decimals)
    instructor_address,
);
```

## Next Steps

- [Understanding Tokens](../beginner/02-understanding-tokens.md)
- [Basic Rewards System](../beginner/03-rewards-system.md)

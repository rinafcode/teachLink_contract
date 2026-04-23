use soroban_sdk::{Env, Symbol};

pub fn with_guard<T, E, F>(env: &Env, key: &Symbol, reentrancy_error: E, f: F) -> Result<T, E>
where
    E: Copy,
    F: FnOnce() -> Result<T, E>,
{
    let active = env
        .storage()
        .instance()
        .get::<_, bool>(key)
        .unwrap_or(false);
    if active {
        return Err(reentrancy_error);
    }

    env.storage().instance().set(key, &true);
    let result = f();
    env.storage().instance().set(key, &false);
    result
}

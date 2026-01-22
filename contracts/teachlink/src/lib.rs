#![no_std]

use soroban_sdk::{contract, contractimpl, Env, Symbol};

#[contract]
pub struct TeachLinkContract;

#[contractimpl]
impl TeachLinkContract {
    #[must_use]
    pub fn hello(_env: Env, to: Symbol) -> Symbol {
        to
    }
}

#[cfg(test)]
mod test {
    extern crate std;

    use super::*;

    #[test]
    fn hello_returns_input() {
        let env = Env::default();
        let input = Symbol::new(&env, "teachlink");
        let out = TeachLinkContract::hello(env.clone(), input);
        assert_eq!(out, Symbol::new(&env, "teachlink"));
    }
}

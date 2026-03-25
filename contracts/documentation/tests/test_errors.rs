#![cfg(test)]

use soroban_sdk::{Env, String};

use documentation_contract::{DocumentationContract, DocumentationContractClient, DocumentationError};

fn expect_err<T, E: core::fmt::Debug>(res: Result<T, E>, context: &str) -> E {
    match res {
        Ok(_) => panic!("{}: expected error", context),
        Err(err) => err,
    }
}

#[test]
fn test_article_not_found() {
    let env = Env::default();
    let contract_id = env.register(DocumentationContract, ());
    let client = DocumentationContractClient::new(&env, &contract_id);

    let err = expect_err(
        client.get_article(&String::from_str(&env, "missing")),
        "get_article missing",
    );
    assert_eq!(err, DocumentationError::ArticleNotFound);
}

#[test]
fn test_faq_not_found() {
    let env = Env::default();
    let contract_id = env.register(DocumentationContract, ());
    let client = DocumentationContractClient::new(&env, &contract_id);

    let err = expect_err(
        client.get_faq(&String::from_str(&env, "missing")),
        "get_faq missing",
    );
    assert_eq!(err, DocumentationError::FaqNotFound);
}

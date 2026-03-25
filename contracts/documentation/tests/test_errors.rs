#![cfg(test)]

use soroban_sdk::{ConversionError, Env, InvokeError, String};

use teachlink_documentation::{DocumentationContract, DocumentationContractClient, DocumentationError};

fn expect_contract_err<T, E: Copy + core::fmt::Debug>(
    res: Result<Result<T, ConversionError>, Result<E, InvokeError>>,
    context: &str,
) -> E {
    match res {
        Ok(Ok(_)) => panic!("{}: expected error", context),
        Ok(Err(err)) => panic!("{}: conversion error {:?}", context, err),
        Err(Ok(err)) => err,
        Err(Err(err)) => panic!("{}: invoke error {:?}", context, err),
    }
}

#[test]
fn test_article_not_found() {
    let env = Env::default();
    let contract_id = env.register(DocumentationContract, ());
    let client = DocumentationContractClient::new(&env, &contract_id);

    let err = expect_contract_err(
        client.try_get_article(&String::from_str(&env, "missing")),
        "get_article missing",
    );
    assert_eq!(err, DocumentationError::ArticleNotFound);
}

#[test]
fn test_faq_not_found() {
    let env = Env::default();
    let contract_id = env.register(DocumentationContract, ());
    let client = DocumentationContractClient::new(&env, &contract_id);

    let err = expect_contract_err(
        client.try_get_faq(&String::from_str(&env, "missing")),
        "get_faq missing",
    );
    assert_eq!(err, DocumentationError::FaqNotFound);
}

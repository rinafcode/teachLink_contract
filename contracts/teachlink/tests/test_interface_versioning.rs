#![allow(clippy::needless_pass_by_value)]

use soroban_sdk::testutils::Address as _;
use soroban_sdk::{Address, Env};
use teachlink_contract::{ContractSemVer, TeachLinkBridge, TeachLinkBridgeClient};

fn setup_client(env: &Env) -> (TeachLinkBridgeClient<'_>, Address, Address, Address) {
    let contract_id = env.register(TeachLinkBridge, ());
    let client = TeachLinkBridgeClient::new(env, &contract_id);

    let token_admin = Address::generate(env);
    let sac = env.register_stellar_asset_contract_v2(token_admin);
    let token = sac.address();

    let admin = Address::generate(env);
    let fee_recipient = Address::generate(env);

    env.mock_all_auths();
    client.initialize(&token, &admin, &1, &fee_recipient);

    (client, token, admin, fee_recipient)
}

#[test]
fn interface_version_defaults_follow_semver() {
    let env = Env::default();
    let (client, _, _, _) = setup_client(&env);

    let status = client.get_interface_version_status();

    assert_eq!(status.current, ContractSemVer::new(1, 0, 0));
    assert_eq!(status.minimum_compatible, ContractSemVer::new(1, 0, 0));
    assert!(client.is_interface_compatible(&ContractSemVer::new(1, 0, 0)));
    assert!(!client.is_interface_compatible(&ContractSemVer::new(2, 0, 0)));
}

#[test]
fn interface_version_range_enforces_compatibility_window() {
    let env = Env::default();
    let (client, _, _, _) = setup_client(&env);

    env.mock_all_auths();
    client.set_interface_version(&ContractSemVer::new(1, 3, 0), &ContractSemVer::new(1, 1, 0));

    assert!(!client.is_interface_compatible(&ContractSemVer::new(1, 0, 9)));
    assert!(client.is_interface_compatible(&ContractSemVer::new(1, 1, 0)));
    assert!(client.is_interface_compatible(&ContractSemVer::new(1, 2, 5)));
    assert!(client.is_interface_compatible(&ContractSemVer::new(1, 3, 0)));
    assert!(!client.is_interface_compatible(&ContractSemVer::new(1, 3, 1)));
    assert!(!client.is_interface_compatible(&ContractSemVer::new(2, 0, 0)));
}

#[test]
fn interface_version_rejects_invalid_ranges() {
    let env = Env::default();
    let (client, _, _, _) = setup_client(&env);

    env.mock_all_auths();
    let invalid_major = client
        .try_set_interface_version(&ContractSemVer::new(2, 0, 0), &ContractSemVer::new(1, 9, 9));
    assert!(invalid_major.is_err());

    let invalid_order = client
        .try_set_interface_version(&ContractSemVer::new(1, 1, 0), &ContractSemVer::new(1, 2, 0));
    assert!(invalid_order.is_err());
}

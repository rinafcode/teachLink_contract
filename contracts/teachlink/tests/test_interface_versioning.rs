#![allow(clippy::needless_pass_by_value)]

mod common;

use soroban_sdk::testutils::Address as _;
use soroban_sdk::{Address, Env};
use teachlink_contract::{ContractSemVer, TeachLinkBridge, TeachLinkBridgeClient};
use common::{register_bridge_client, register_sac_token, random_address, test_env};

fn setup_client(env: &Env) -> (TeachLinkBridgeClient<'_>, Address, Address, Address) {
    env.mock_all_auths();
    let client = register_bridge_client(env);
    let token = register_sac_token(env);

    let admin = random_address(env);
    let fee_recipient = random_address(env);

    client.initialize(&token, &admin, &1, &fee_recipient);

    (client, token, admin, fee_recipient)
}

#[test]
fn interface_version_defaults_follow_semver() {
    let env = test_env();
    let (client, _, _, _) = setup_client(&env);

    let status = client.get_interface_version_status();

    assert_eq!(status.current, ContractSemVer::new(1, 0, 0));
    assert_eq!(status.minimum_compatible, ContractSemVer::new(1, 0, 0));
    assert!(client.is_interface_compatible(&ContractSemVer::new(1, 0, 0)));
    assert!(!client.is_interface_compatible(&ContractSemVer::new(2, 0, 0)));
}

#[test]
fn interface_version_range_enforces_compatibility_window() {
    let env = test_env();
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
    let env = test_env();
    let (client, _, _, _) = setup_client(&env);

    env.mock_all_auths();
    let invalid_major = client
        .try_set_interface_version(&ContractSemVer::new(2, 0, 0), &ContractSemVer::new(1, 9, 9));
    assert!(invalid_major.is_err());

    let invalid_order = client
        .try_set_interface_version(&ContractSemVer::new(1, 1, 0), &ContractSemVer::new(1, 2, 0));
    assert!(invalid_order.is_err());
}

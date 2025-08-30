use starknet::{ContractAddress, contract_address_const, get_block_timestamp};
use snforge_std::{declare, ContractClassTrait, start_prank, stop_prank, CheatTarget};

use super::super::contracts::certificates::CourseNFTCertificate;
use super::super::contracts::certificates::interfaces::ICourseNFTCertificate::{
    ICourseNFTCertificateDispatcher, ICourseNFTCertificateDispatcherTrait,
    CertificateDetails, CourseRequirements, CourseInfo
};

fn OWNER() -> ContractAddress {
    contract_address_const::<'owner'>()
}

fn INSTRUCTOR() -> ContractAddress {
    contract_address_const::<'instructor'>()
}

fn STUDENT() -> ContractAddress {
    contract_address_const::<'student'>()
}

fn OTHER_USER() -> ContractAddress {
    contract_address_const::<'other_user'>()
}

fn deploy_contract() -> ICourseNFTCertificateDispatcher {
    let contract = declare("CourseNFTCertificate");
    let constructor_calldata = array![
        OWNER().into(),
        'MarketX Certificates',
        'MXCERT',
        'https://api.marketx.com/certificates/'
    ];
    let contract_address = contract.deploy(@constructor_calldata).unwrap();
    ICourseNFTCertificateDispatcher { contract_address }
}

fn create_sample_requirements() -> CourseRequirements {
    CourseRequirements {
        min_completion_percentage: 80,
        required_assignments: 5,
        min_quiz_score: 75,
        required_participation: 10,
        custom_requirements: 'additional_requirements',
    }
}

#[test]
fn test_deploy_contract() {
    let contract = deploy_contract();
    // Contract should be deployed successfully
    assert(contract.contract_address != contract_address_const::<0>(), 'Contract not deployed');
}
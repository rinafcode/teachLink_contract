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


#[test]
fn test_register_course() {
    let contract = deploy_contract();
    let requirements = create_sample_requirements();
    
    start_prank(CheatTarget::One(contract.contract_address), INSTRUCTOR());
    
    contract.register_course(1, INSTRUCTOR(), requirements);
    
    stop_prank(CheatTarget::One(contract.contract_address));
    
    // Verify course was registered
    // Note: We would need getter functions to verify this properly
}

#[test]
fn test_issue_certificate() {
    let contract = deploy_contract();
    let requirements = create_sample_requirements();
    
    // Register course first
    start_prank(CheatTarget::One(contract.contract_address), INSTRUCTOR());
    contract.register_course(1, INSTRUCTOR(), requirements);
    
    // Issue certificate
    let certificate_id = contract.issue_certificate(
        STUDENT(),
        1,
        INSTRUCTOR(),
        'completion_data_hash',
        'https://api.marketx.com/certificates/1'
    );
    
    stop_prank(CheatTarget::One(contract.contract_address));
    
    assert(certificate_id == 1, 'Wrong certificate ID');
    
    // Verify certificate details
    let certificate = contract.get_certificate_details(certificate_id);
    assert(certificate.student == STUDENT(), 'Wrong student');
    assert(certificate.course_id == 1, 'Wrong course ID');
    assert(certificate.instructor == INSTRUCTOR(), 'Wrong instructor');
    assert(!certificate.is_revoked, 'Certificate should not be revoked');
}

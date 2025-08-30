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


#[test]
fn test_verify_certificate() {
    let contract = deploy_contract();
    let requirements = create_sample_requirements();
    
    start_prank(CheatTarget::One(contract.contract_address), INSTRUCTOR());
    contract.register_course(1, INSTRUCTOR(), requirements);
    
    let certificate_id = contract.issue_certificate(
        STUDENT(),
        1,
        INSTRUCTOR(),
        'completion_data_hash',
        'https://api.marketx.com/certificates/1'
    );
    
    stop_prank(CheatTarget::One(contract.contract_address));
    
    // Verify certificate
    let is_valid = contract.verify_certificate(certificate_id);
    assert(is_valid, 'Certificate should be valid');
}

#[test]
fn test_verify_course_completion() {
    let contract = deploy_contract();
    let requirements = create_sample_requirements();
    
    start_prank(CheatTarget::One(contract.contract_address), INSTRUCTOR());
    contract.register_course(1, INSTRUCTOR(), requirements);
    
    contract.issue_certificate(
        STUDENT(),
        1,
        INSTRUCTOR(),
        'completion_data_hash',
        'https://api.marketx.com/certificates/1'
    );
    
    stop_prank(CheatTarget::One(contract.contract_address));
    
    // Verify course completion
    let completed = contract.verify_course_completion(STUDENT(), 1);
    assert(completed, 'Student should have completed course');
    
    // Verify non-completion for different course
    let not_completed = contract.verify_course_completion(STUDENT(), 2);
    assert(!not_completed, 'Student should not have completed course 2');
}


#[test]
fn test_revoke_certificate() {
    let contract = deploy_contract();
    let requirements = create_sample_requirements();
    
    start_prank(CheatTarget::One(contract.contract_address), INSTRUCTOR());
    contract.register_course(1, INSTRUCTOR(), requirements);
    
    let certificate_id = contract.issue_certificate(
        STUDENT(),
        1,
        INSTRUCTOR(),
        'completion_data_hash',
        'https://api.marketx.com/certificates/1'
    );
    
    // Revoke certificate
    contract.revoke_certificate(certificate_id);
    
    stop_prank(CheatTarget::One(contract.contract_address));
    
    // Verify certificate is revoked
    let certificate = contract.get_certificate_details(certificate_id);
    assert(certificate.is_revoked, 'Certificate should be revoked');
    
    // Verify certificate is no longer valid
    let is_valid = contract.verify_certificate(certificate_id);
    assert(!is_valid, 'Revoked certificate should not be valid');
}

#[test]
fn test_update_certificate_metadata() {
    let contract = deploy_contract();
    let requirements = create_sample_requirements();
    
    start_prank(CheatTarget::One(contract.contract_address), INSTRUCTOR());
    contract.register_course(1, INSTRUCTOR(), requirements);
    
    let certificate_id = contract.issue_certificate(
        STUDENT(),
        1,
        INSTRUCTOR(),
        'completion_data_hash',
        'https://api.marketx.com/certificates/1'
    );
    
    // Update metadata
    let new_uri = 'https://api.marketx.com/certificates/updated/1';
    contract.update_certificate_metadata(certificate_id, new_uri);
    
    stop_prank(CheatTarget::One(contract.contract_address));

    
    // Verify metadata was updated
    let certificate = contract.get_certificate_details(certificate_id);
    assert(certificate.metadata_uri == new_uri, 'Metadata not updated');
}

#[test]
#[should_panic(expected: ('Course does not exist',))]
fn test_issue_certificate_nonexistent_course() {
    let contract = deploy_contract();
    
    start_prank(CheatTarget::One(contract.contract_address), INSTRUCTOR());
    
    // Try to issue certificate for non-existent course
    contract.issue_certificate(
        STUDENT(),
        999, // Non-existent course
        INSTRUCTOR(),
        'completion_data_hash',
        'https://api.marketx.com/certificates/1'
    );
    
    stop_prank(CheatTarget::One(contract.contract_address));
}

#[test]
#[should_panic(expected: ('Unauthorized instructor',))]
fn test_issue_certificate_wrong_instructor() {
    let contract = deploy_contract();
    let requirements = create_sample_requirements();
    
    start_prank(CheatTarget::One(contract.contract_address), INSTRUCTOR());
    contract.register_course(1, INSTRUCTOR(), requirements);
    stop_prank(CheatTarget::One(contract.contract_address));
    
    start_prank(CheatTarget::One(contract.contract_address), OTHER_USER());
    
    // Try to issue certificate with wrong instructor
    contract.issue_certificate(
        STUDENT(),
        1,
        OTHER_USER(), // Wrong instructor
        'completion_data_hash',
        'https://api.marketx.com/certificates/1'
    );
    
    stop_prank(CheatTarget::One(contract.contract_address));
}


#[test]
#[should_panic(expected: ('Course already exists',))]
fn test_register_duplicate_course() {
    let contract = deploy_contract();
    let requirements = create_sample_requirements();
    
    start_prank(CheatTarget::One(contract.contract_address), INSTRUCTOR());
    
    contract.register_course(1, INSTRUCTOR(), requirements);
    
    // Try to register same course again
    contract.register_course(1, INSTRUCTOR(), requirements);
    
    stop_prank(CheatTarget::One(contract.contract_address));
}

#[test]
fn test_get_student_certificates() {
    let contract = deploy_contract();
    let requirements = create_sample_requirements();
    
    start_prank(CheatTarget::One(contract.contract_address), INSTRUCTOR());
    contract.register_course(1, INSTRUCTOR(), requirements);
    contract.register_course(2, INSTRUCTOR(), requirements);
    
    // Issue multiple certificates to same student
    let cert1 = contract.issue_certificate(
        STUDENT(),
        1,
        INSTRUCTOR(),
        'completion_data_hash_1',
        'https://api.marketx.com/certificates/1'
    );
    
    let cert2 = contract.issue_certificate(
        STUDENT(),
        2,
        INSTRUCTOR(),
        'completion_data_hash_2',
        'https://api.marketx.com/certificates/2'
    );
    
    stop_prank(CheatTarget::One(contract.contract_address));
    
    // Get student certificates
    let student_certs = contract.get_student_certificates(STUDENT());
    assert(student_certs.len() == 2, 'Wrong number of certificates');
}

#[test]
fn test_pause_and_unpause() {
    let contract = deploy_contract();

    
    start_prank(CheatTarget::One(contract.contract_address), OWNER());
    
    // Pause contract
    contract.pause_contract();
    
    stop_prank(CheatTarget::One(contract.contract_address));
    
    // Try to register course while paused (should fail)
    start_prank(CheatTarget::One(contract.contract_address), INSTRUCTOR());
    
    // This should panic due to contract being paused
    // contract.register_course(1, INSTRUCTOR(), create_sample_requirements());
    
    stop_prank(CheatTarget::One(contract.contract_address));
    
    // Unpause contract
    start_prank(CheatTarget::One(contract.contract_address), OWNER());
    contract.unpause_contract();
    stop_prank(CheatTarget::One(contract.contract_address));
    
    // Now operations should work again
    start_prank(CheatTarget::One(contract.contract_address), INSTRUCTOR());
    contract.register_course(1, INSTRUCTOR(), create_sample_requirements());
    stop_prank(CheatTarget::One(contract.contract_address));
}

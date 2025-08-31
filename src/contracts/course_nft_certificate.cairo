use starknet::ContractAddress;

#[starknet::interface]
trait ICourseNFTCertificate<TContractState> {
    // Certificate Management
    fn issue_certificate(
        ref self: TContractState,
        student: ContractAddress,
        course_id: u256,
        instructor: ContractAddress,
        completion_data: felt252,
        metadata_uri: felt252
    ) -> u256;
    
    fn revoke_certificate(ref self: TContractState, certificate_id: u256);
    fn update_certificate_metadata(ref self: TContractState, certificate_id: u256, new_uri: felt252);
    
    // Verification Functions
    fn verify_certificate(self: @TContractState, certificate_id: u256) -> bool;
    fn verify_course_completion(
        self: @TContractState, 
        student: ContractAddress, 
        course_id: u256
    ) -> bool;
    
    // Query Functions
    fn get_certificate_details(self: @TContractState, certificate_id: u256) -> CertificateDetails;
    fn get_student_certificates(self: @TContractState, student: ContractAddress) -> Array<u256>;
    fn get_course_certificates(self: @TContractState, course_id: u256) -> Array<u256>;
    fn get_instructor_issued_certificates(self: @TContractState, instructor: ContractAddress) -> Array<u256>;
    
    // Course Management
    fn register_course(
        ref self: TContractState,
        course_id: u256,
        instructor: ContractAddress,
        requirements: CourseRequirements
    );
    fn update_course_requirements(
        ref self: TContractState,
        course_id: u256,
        requirements: CourseRequirements
    );

    
    // Integration Functions
    fn set_identity_contract(ref self: TContractState, contract_address: ContractAddress);
    fn set_reputation_contract(ref self: TContractState, contract_address: ContractAddress);
    
    // Admin Functions
    fn pause_contract(ref self: TContractState);
    fn unpause_contract(ref self: TContractState);
    fn upgrade_contract(ref self: TContractState, new_implementation: felt252);
}

#[derive(Drop, Serde, starknet::Store)]
struct CertificateDetails {
    certificate_id: u256,
    student: ContractAddress,
    course_id: u256,
    instructor: ContractAddress,
    issue_timestamp: u64,
    completion_data: felt252,
    metadata_uri: felt252,
    is_revoked: bool,
    verification_hash: felt252,
}

#[derive(Drop, Serde, starknet::Store)]
struct CourseRequirements {
    min_completion_percentage: u8,
    required_assignments: u32,
    min_quiz_score: u8,
    required_participation: u32,
    custom_requirements: felt252,
}

#[derive(Drop, Serde, starknet::Store)]
struct CourseInfo {
    course_id: u256,
    instructor: ContractAddress,
    requirements: CourseRequirements,
    is_active: bool,
    total_certificates_issued: u32,
    creation_timestamp: u64,
}

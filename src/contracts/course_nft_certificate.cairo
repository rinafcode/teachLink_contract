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

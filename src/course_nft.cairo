#[starknet::contract]
mod CourseNFTCertificate {
    use starknet::{
        ContractAddress, get_caller_address, get_contract_address, get_block_timestamp,
        contract_address_const
    };
    use core::pedersen::pedersen;
    use core::poseidon::poseidon_hash_span;
    use core::array::ArrayTrait;
    use openzeppelin::token::erc721::{ERC721Component, ERC721HooksEmptyImpl};
    use openzeppelin::introspection::src5::SRC5Component;
    use openzeppelin::access::ownable::OwnableComponent;
    use openzeppelin::security::pausable::PausableComponent;
    use openzeppelin::upgrades::UpgradeableComponent;

    use super::interfaces::ICourseNFTCertificate::{
        ICourseNFTCertificate, CertificateDetails, CourseRequirements, CourseInfo
    };
    use super::libraries::CertificateVerification::{
        CompletionProof, VerificationResult, CertificateVerification
    };

    component!(path: ERC721Component, storage: erc721, event: ERC721Event);
    component!(path: SRC5Component, storage: src5, event: SRC5Event);
    component!(path: OwnableComponent, storage: ownable, event: OwnableEvent);
    component!(path: PausableComponent, storage: pausable, event: PausableEvent);
    component!(path: UpgradeableComponent, storage: upgradeable, event: UpgradeableEvent);

    #[abi(embed_v0)]
    impl ERC721Impl = ERC721Component::ERC721Impl<ContractState>;
    #[abi(embed_v0)]
    impl ERC721MetadataImpl = ERC721Component::ERC721MetadataImpl<ContractState>;
    #[abi(embed_v0)]
    impl ERC721CamelOnly = ERC721Component::ERC721CamelOnlyImpl<ContractState>;
    #[abi(embed_v0)]
    impl SRC5Impl = SRC5Component::SRC5Impl<ContractState>;
    #[abi(embed_v0)]
    impl OwnableImpl = OwnableComponent::OwnableImpl<ContractState>;
    #[abi(embed_v0)]
    impl PausableImpl = PausableComponent::PausableImpl<ContractState>;
    #[abi(embed_v0)]
    impl UpgradeableImpl = UpgradeableComponent::UpgradeableImpl<ContractState>;

    impl ERC721InternalImpl = ERC721Component::InternalImpl<ContractState>;
    impl OwnableInternalImpl = OwnableComponent::InternalImpl<ContractState>;
    impl PausableInternalImpl = PausableComponent::InternalImpl<ContractState>;
    impl UpgradeableInternalImpl = UpgradeableComponent::InternalImpl<ContractState>;

    #[storage]
    struct Storage {
        #[substorage(v0)]
        erc721: ERC721Component::Storage,
        #[substorage(v0)]
        src5: SRC5Component::Storage,
        #[substorage(v0)]
        ownable: OwnableComponent::Storage,
        #[substorage(v0)]
        pausable: PausableComponent::Storage,
        #[substorage(v0)]
        upgradeable: UpgradeableComponent::Storage,
        
        // Certificate storage
        certificates: LegacyMap<u256, CertificateDetails>,
        student_certificates: LegacyMap<ContractAddress, Array<u256>>,
        course_certificates: LegacyMap<u256, Array<u256>>,
        instructor_certificates: LegacyMap<ContractAddress, Array<u256>>,
        
        // Course storage
        courses: LegacyMap<u256, CourseInfo>,
        course_exists: LegacyMap<u256, bool>,

        
        // Integration contracts
        identity_contract: ContractAddress,
        reputation_contract: ContractAddress,
        
        // Counters
        next_certificate_id: u256,
        total_certificates: u256,
        total_courses: u256,
        
        // Anti-fraud measures
        certificate_hashes: LegacyMap<felt252, bool>,
        revoked_certificates: LegacyMap<u256, bool>,
    }

    #[event]
    #[derive(Drop, starknet::Event)]
    enum Event {
        #[flat]
        ERC721Event: ERC721Component::Event,
        #[flat]
        SRC5Event: SRC5Component::Event,
        #[flat]
        OwnableEvent: OwnableComponent::Event,
        #[flat]
        PausableEvent: PausableComponent::Event,
        #[flat]
        UpgradeableEvent: UpgradeableComponent::Event,
        
        CertificateIssued: CertificateIssued,
        CertificateRevoked: CertificateRevoked,
        CourseRegistered: CourseRegistered,
        CourseUpdated: CourseUpdated,
        MetadataUpdated: MetadataUpdated,
    }

    #[derive(Drop, starknet::Event)]
    struct CertificateIssued {
        certificate_id: u256,
        student: ContractAddress,
        course_id: u256,
        instructor: ContractAddress,
        timestamp: u64,
    }

    #[derive(Drop, starknet::Event)]
    struct CertificateRevoked {
        certificate_id: u256,
        revoked_by: ContractAddress,
        timestamp: u64,
    }

    
    #[derive(Drop, starknet::Event)]
    struct CourseRegistered {
        course_id: u256,
        instructor: ContractAddress,
        timestamp: u64,
    }

    #[derive(Drop, starknet::Event)]
    struct CourseUpdated {
        course_id: u256,
        updated_by: ContractAddress,
        timestamp: u64,
    }

    #[derive(Drop, starknet::Event)]
    struct MetadataUpdated {
        certificate_id: u256,
        new_uri: felt252,
        updated_by: ContractAddress,
    }

    #[constructor]
    fn constructor(
        ref self: ContractState,
        owner: ContractAddress,
        name: felt252,
        symbol: felt252,
        base_uri: felt252
    ) {
        self.erc721.initializer(name, symbol, base_uri);
        self.ownable.initializer(owner);
        self.next_certificate_id.write(1);
        self.total_certificates.write(0);
        self.total_courses.write(0);
    }

    #[abi(embed_v0)]
    impl CourseNFTCertificateImpl of ICourseNFTCertificate<ContractState> {
        fn issue_certificate(
            ref self: ContractState,
            student: ContractAddress,
            course_id: u256,
            instructor: ContractAddress,
            completion_data: felt252,
            metadata_uri: felt252
        ) -> u256 {
            self.pausable.assert_not_paused();
            
            // Verify course exists and instructor is authorized
            assert(self.course_exists.read(course_id), 'Course does not exist');
            let course_info = self.courses.read(course_id);
            assert(course_info.instructor == instructor, 'Unauthorized instructor');
            assert(course_info.is_active, 'Course is not active');
            
            // Verify caller is authorized (instructor or contract owner)
            let caller = get_caller_address();
            assert(
                caller == instructor || caller == self.ownable.owner(),
                'Unauthorized certificate issuer'
            );
            
            // Generate certificate ID and verification hash
            let certificate_id = self.next_certificate_id.read();
            let timestamp = get_block_timestamp();
            let verification_hash = CertificateVerification::generate_certificate_hash(
                student, course_id, instructor, completion_data, timestamp
            );
            
            // Ensure certificate hash is unique (prevent duplicates)
            assert(!self.certificate_hashes.read(verification_hash), 'Certificate already exists');
            
            // Create certificate details
            let certificate = CertificateDetails {
                certificate_id,
                student,
                course_id,
                instructor,
                issue_timestamp: timestamp,
                completion_data,
                metadata_uri,
                is_revoked: false,
                verification_hash,
            };

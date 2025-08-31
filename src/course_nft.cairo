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

            // Store certificate
            self.certificates.write(certificate_id, certificate);
            self.certificate_hashes.write(verification_hash, true);
            
            // Update mappings
            let mut student_certs = self.student_certificates.read(student);
            student_certs.append(certificate_id);
            self.student_certificates.write(student, student_certs);
            
            let mut course_certs = self.course_certificates.read(course_id);
            course_certs.append(certificate_id);
            self.course_certificates.write(course_id, course_certs);
            
            let mut instructor_certs = self.instructor_certificates.read(instructor);
            instructor_certs.append(certificate_id);
            self.instructor_certificates.write(instructor, instructor_certs);
            
            // Mint NFT to student
            self.erc721._mint(student, certificate_id);
            self.erc721._set_token_uri(certificate_id, metadata_uri);
            
            // Update counters
            self.next_certificate_id.write(certificate_id + 1);
            self.total_certificates.write(self.total_certificates.read() + 1);
            
            // Update course certificate count
            let mut updated_course = course_info;
            updated_course.total_certificates_issued += 1;
            self.courses.write(course_id, updated_course);
            
            // Emit event
            self.emit(CertificateIssued {
                certificate_id,
                student,
                course_id,
                instructor,
                timestamp,
            });
            
            certificate_id
        }

        fn revoke_certificate(ref self: ContractState, certificate_id: u256) {
            self.pausable.assert_not_paused();
            
            let certificate = self.certificates.read(certificate_id);
            assert(certificate.certificate_id != 0, 'Certificate does not exist');
            assert(!certificate.is_revoked, 'Certificate already revoked');
            
            let caller = get_caller_address();
            // Only instructor, student, or contract owner can revoke
            assert(
                caller == certificate.instructor || 
                caller == certificate.student || 
                caller == self.ownable.owner(),
                'Unauthorized revocation'
            );
            
            // Update certificate status
            let mut updated_certificate = certificate;
            updated_certificate.is_revoked = true;
            self.certificates.write(certificate_id, updated_certificate);
            self.revoked_certificates.write(certificate_id, true);
            
            // Burn the NFT
            self.erc721._burn(certificate_id);
            
            self.emit(CertificateRevoked {
                certificate_id,
                revoked_by: caller,
                timestamp: get_block_timestamp(),
            });
        }

        fn update_certificate_metadata(
            ref self: ContractState, 
            certificate_id: u256, 
            new_uri: felt252
        ) {
            self.pausable.assert_not_paused();
            
            let certificate = self.certificates.read(certificate_id);
            assert(certificate.certificate_id != 0, 'Certificate does not exist');
            assert(!certificate.is_revoked, 'Certificate is revoked');
            
            let caller = get_caller_address();
            assert(
                caller == certificate.instructor || caller == self.ownable.owner(),
                'Unauthorized metadata update'
            );
            // Update metadata URI
            let mut updated_certificate = certificate;
            updated_certificate.metadata_uri = new_uri;
            self.certificates.write(certificate_id, updated_certificate);
            
            // Update NFT metadata
            self.erc721._set_token_uri(certificate_id, new_uri);
            
            self.emit(MetadataUpdated {
                certificate_id,
                new_uri,
                updated_by: caller,
            });
        }

        fn verify_certificate(self: @ContractState, certificate_id: u256) -> bool {
            let certificate = self.certificates.read(certificate_id);
            
            if certificate.certificate_id == 0 {
                return false;
            }
            
            if certificate.is_revoked {
                return false;
            }
            
            // Verify hash integrity
            let computed_hash = CertificateVerification::generate_certificate_hash(
                certificate.student,
                certificate.course_id,
                certificate.instructor,
                certificate.completion_data,
                certificate.issue_timestamp
            );
            
            computed_hash == certificate.verification_hash
        }

        fn verify_course_completion(
            self: @ContractState, 
            student: ContractAddress, 
            course_id: u256
        ) -> bool {
            let student_certs = self.student_certificates.read(student);
            let mut i = 0;
            
            loop {
                if i >= student_certs.len() {
                    break false;
                }
                
                let cert_id = *student_certs.at(i);
                let certificate = self.certificates.read(cert_id);
                
                if certificate.course_id == course_id && !certificate.is_revoked {
                    break true;
                }
                
                i += 1;
            }
        }

        fn get_certificate_details(
            self: @ContractState, 
            certificate_id: u256
        ) -> CertificateDetails {
            self.certificates.read(certificate_id)
        }

        fn get_student_certificates(
            self: @ContractState, 
            student: ContractAddress
        ) -> Array<u256> {
            self.student_certificates.read(student)
        }

        fn get_course_certificates(self: @ContractState, course_id: u256) -> Array<u256> {
            self.course_certificates.read(course_id)
        }

        fn get_instructor_issued_certificates(
            self: @ContractState, 
            instructor: ContractAddress
        ) -> Array<u256> {
            self.instructor_certificates.read(instructor)
        }

        fn register_course(
            ref self: ContractState,
            course_id: u256,
            instructor: ContractAddress,
            requirements: CourseRequirements
        ) {
            self.pausable.assert_not_paused();
            
            let caller = get_caller_address();
            assert(
                caller == instructor || caller == self.ownable.owner(),
                'Unauthorized course registration'
            );
            
            assert(!self.course_exists.read(course_id), 'Course already exists');
            
            let course_info = CourseInfo {
                course_id,
                instructor,
                requirements,
                is_active: true,
                total_certificates_issued: 0,
                creation_timestamp: get_block_timestamp(),
            };
            
            self.courses.write(course_id, course_info);
            self.course_exists.write(course_id, true);
            self.total_courses.write(self.total_courses.read() + 1);
            
            self.emit(CourseRegistered {
                course_id,
                instructor,
                timestamp: get_block_timestamp(),
            });
        }

        fn update_course_requirements(
            ref self: ContractState,
            course_id: u256,
            requirements: CourseRequirements
        ) {
            self.pausable.assert_not_paused();
            
            assert(self.course_exists.read(course_id), 'Course does not exist');
            let mut course_info = self.courses.read(course_id);
            
            let caller = get_caller_address();
            assert(
                caller == course_info.instructor || caller == self.ownable.owner(),
                'Unauthorized course update'
            );
            
            course_info.requirements = requirements;
            self.courses.write(course_id, course_info);
            
            self.emit(CourseUpdated {
                course_id,
                updated_by: caller,
                timestamp: get_block_timestamp(),
            });
        }

        fn set_identity_contract(ref self: ContractState, contract_address: ContractAddress) {
            self.ownable.assert_only_owner();
            self.identity_contract.write(contract_address);
        }

        fn set_reputation_contract(ref self: ContractState, contract_address: ContractAddress) {
            self.ownable.assert_only_owner();
            self.reputation_contract.write(contract_address);
        }

        fn pause_contract(ref self: ContractState) {
            self.ownable.assert_only_owner();
            self.pausable._pause();
        }

        fn unpause_contract(ref self: ContractState) {
            self.ownable.assert_only_owner();
            self.pausable._unpause();
        }

        fn upgrade_contract(ref self: ContractState, new_implementation: felt252) {
            self.ownable.assert_only_owner();
            self.upgradeable._upgrade(new_implementation);
        }
    }
}

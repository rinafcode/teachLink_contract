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

        
#[starknet::contract]
mod ReputationSystem {
    use starknet::{
        ContractAddress, get_caller_address, get_block_timestamp,
        storage::{
            StoragePointerReadAccess, StoragePointerWriteAccess, StoragePathEntry, Map
        }
    };
    use openzeppelin::access::ownable::OwnableComponent;
    use openzeppelin::security::pausable::PausableComponent;
    use openzeppelin::token::erc721::{ERC721Component, ERC721HooksEmptyImpl};
    use openzeppelin::introspection::src5::SRC5Component;
    
    use super::interfaces::IReputationSystem::{
        IReputationSystem, Review, InstructorReputation, ReputationSnapshot, ReviewerProfile
    };
    use super::libraries::ReputationCalculation::{
        ReputationCalculationTrait, WeightingFactors, ScoreComponents
    };

    component!(path: OwnableComponent, storage: ownable, event: OwnableEvent);
    component!(path: PausableComponent, storage: pausable, event: PausableEvent);
    component!(path: ERC721Component, storage: erc721, event: ERC721Event);
    component!(path: SRC5Component, storage: src5, event: SRC5Event);

    // Ownable Mixin
    #[abi(embed_v0)]
    impl OwnableMixinImpl = OwnableComponent::OwnableMixinImpl<ContractState>;
    impl OwnableInternalImpl = OwnableComponent::InternalImpl<ContractState>;

    // Pausable Mixin
    #[abi(embed_v0)]
    impl PausableMixinImpl = PausableComponent::PausableMixinImpl<ContractState>;
    impl PausableInternalImpl = PausableComponent::InternalImpl<ContractState>;

    // ERC721 Mixin
    #[abi(embed_v0)]
    impl ERC721MixinImpl = ERC721Component::ERC721MixinImpl<ContractState>;
    impl ERC721InternalImpl = ERC721Component::InternalImpl<ContractState>;

    #[storage]
    struct Storage {
        #[substorage(v0)]
        ownable: OwnableComponent::Storage,
        #[substorage(v0)]
        pausable: PausableComponent::Storage,
        #[substorage(v0)]
        erc721: ERC721Component::Storage,
        #[substorage(v0)]
        src5: SRC5Component::Storage,
        
        // Core storage
        instructor_reputations: Map<ContractAddress, InstructorReputation>,
        instructor_to_token: Map<ContractAddress, u256>,
        token_to_instructor: Map<u256, ContractAddress>,
        reviews: Map<u256, Review>,
        instructor_reviews: Map<ContractAddress, Array<u256>>,
        reviewer_profiles: Map<ContractAddress, ReviewerProfile>,
        reputation_history: Map<(ContractAddress, u256), ReputationSnapshot>,
        
        // Counters and settings
        next_token_id: u256,
        next_review_id: u256,
        minimum_credibility: u256,
        weighting_factors: WeightingFactors,
        
        // Anti-manipulation
        flagged_reviews: Map<u256, bool>,
        suspicious_activity: Map<ContractAddress, Array<felt252>>,
    }

    #[event]
    #[derive(Drop, starknet::Event)]
    enum Event {
        #[flat]
        OwnableEvent: OwnableComponent::Event,
        #[flat]
        PausableEvent: PausableComponent::Event,
        #[flat]
        ERC721Event: ERC721Component::Event,
        #[flat]
        SRC5Event: SRC5Component::Event,
        
        InstructorRegistered: InstructorRegistered,
        ReviewSubmitted: ReviewSubmitted,
        ReputationUpdated: ReputationUpdated,
        ReviewFlagged: ReviewFlagged,
        SuspiciousActivityReported: SuspiciousActivityReported,
    }

    #[derive(Drop, starknet::Event)]
    struct InstructorRegistered {
        #[key]
        instructor: ContractAddress,
        token_id: u256,
        initial_score: u256,
    }

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

    
    #[derive(Drop, starknet::Event)]
    struct ReviewSubmitted {
        #[key]
        review_id: u256,
        #[key]
        reviewer: ContractAddress,
        #[key]
        instructor: ContractAddress,
        course_id: u256,
        rating: u8,
        weight: u256,
    }

    #[derive(Drop, starknet::Event)]
    struct ReputationUpdated {
        #[key]
        instructor: ContractAddress,
        old_score: u256,
        new_score: u256,
        review_count: u32,
    }

    #[derive(Drop, starknet::Event)]
    struct ReviewFlagged {
        #[key]
        review_id: u256,
        reason: u8,
        flagged_by: ContractAddress,
    }

    #[derive(Drop, starknet::Event)]
    struct SuspiciousActivityReported {
        #[key]
        target: ContractAddress,
        reporter: ContractAddress,
        evidence_hash: felt252,
    }

    #[constructor]
    fn constructor(
        ref self: ContractState,
        owner: ContractAddress,
        name: ByteArray,
        symbol: ByteArray,
    ) {
        self.ownable.initializer(owner);
        self.erc721.initializer(name, symbol, "");
        
        // Initialize default settings
        self.next_token_id.write(1);
        self.next_review_id.write(1);
        self.minimum_credibility.write(30); // Minimum 30% credibility to submit reviews
        
        // Initialize default weighting factors
        let default_weights = WeightingFactors {
            credibility_weight: 40,
            recency_weight: 30,
            volume_weight: 20,
            consistency_weight: 10,
        };
        self.weighting_factors.write(default_weights);
    }

    #[abi(embed_v0)]
    impl ReputationSystemImpl of IReputationSystem<ContractState> {
        fn mint_instructor_token(
            ref self: ContractState,
            instructor: ContractAddress,
            initial_score: u256
        ) {
            self.ownable.assert_only_owner();
            self.pausable.assert_not_paused();
            
            assert(!self.is_instructor_registered(instructor), 'Instructor already registered');
            
            let token_id = self.next_token_id.read();
            self.next_token_id.write(token_id + 1);
            
            // Mint soulbound token (non-transferable)
            self.erc721._mint(instructor, token_id);
            
            // Create instructor reputation record
            let reputation = InstructorReputation {
                token_id,
                instructor,
                total_score: initial_score,
                weighted_score: initial_score,
                review_count: 0,
                last_updated: get_block_timestamp(),
                is_active: true,
            };
            
            self.instructor_reputations.write(instructor, reputation);
            self.instructor_to_token.write(instructor, token_id);
            self.token_to_instructor.write(token_id, instructor);
            
            // Initialize empty review array
            self.instructor_reviews.write(instructor, array![]);
            
            self.emit(InstructorRegistered { instructor, token_id, initial_score });
        }


        fn get_instructor_token_id(self: @ContractState, instructor: ContractAddress) -> u256 {
            self.instructor_to_token.read(instructor)
        }

        fn is_instructor_registered(self: @ContractState, instructor: ContractAddress) -> bool {
            let reputation = self.instructor_reputations.read(instructor);
            reputation.is_active
        }

        fn submit_review(
            ref self: ContractState,
            instructor: ContractAddress,
            course_id: u256,
            rating: u8,
            review_hash: felt252,
            proof: Array<felt252>
        ) {
            self.pausable.assert_not_paused();
            let reviewer = get_caller_address();
            
            assert!(rating >= 1 && rating <= 5, "Rating must be between 1 and 5");
            assert!(self.is_instructor_registered(instructor), "Instructor not registered");
            
            // Check reviewer credibility
            let reviewer_profile = self.reviewer_profiles.read(reviewer);
            let credibility = if reviewer_profile.total_reviews == 0 {
                50 // Default credibility for new reviewers
            } else {
                self.get_reviewer_credibility(reviewer)
            };
            
            assert!(credibility >= self.minimum_credibility.read(), "Insufficient reviewer credibility");
            
            // Create review
            let review_id = self.next_review_id.read();
            self.next_review_id.write(review_id + 1);
            
            let review = Review {
                id: review_id,
                reviewer,
                instructor,
                course_id,
                rating,
                review_hash,
                timestamp: get_block_timestamp(),
                weight: credibility,
                is_flagged: false,
                credibility_score: credibility,
            };
            
            self.reviews.write(review_id, review);
            
            // Add to instructor's review list
            let mut instructor_review_list = self.instructor_reviews.read(instructor);
            instructor_review_list.append(review_id);
            self.instructor_reviews.write(instructor, instructor_review_list);
            
            // Update reviewer profile
            self._update_reviewer_profile(reviewer);
            
            // Update instructor reputation
            self.update_reputation_score(instructor);
            
            self.emit(ReviewSubmitted {
                review_id,
                reviewer,
                instructor,
                course_id,
                rating,
                weight: credibility,
            });
        }

        fn get_review(self: @ContractState, review_id: u256) -> Review {
            self.reviews.read(review_id)
        }

        fn get_instructor_reviews(self: @ContractState, instructor: ContractAddress) -> Array<u256> {
            self.instructor_reviews.read(instructor)
        }

        fn calculate_reputation_score(self: @ContractState, instructor: ContractAddress) -> u256 {
            let review_ids = self.instructor_reviews.read(instructor);
            if review_ids.len() == 0 {
                return self.instructor_reputations.read(instructor).total_score;
            }
            
            let mut reviews = array![];
            let mut i = 0;
            loop {
                if i >= review_ids.len() {
                    break;
                }
                let review = self.reviews.read(*review_ids.at(i));
                reviews.append(review);
                i += 1;
            };

#[starknet::contract]
pub mod TeachLinkCollaborativeLearning {
    use starknet::{
        get_caller_address, get_block_timestamp, ContractAddress, contract_address_const,
        storage::{StoragePointerReadAccess, StoragePointerWriteAccess, Map, StoragePathEntry},
    };
    use openzeppelin::{
        access::ownable::OwnableComponent, introspection::src5::SRC5Component,
        token::erc20::interface::{IERC20Dispatcher, IERC20DispatcherTrait},
    };
    use core::traits::{Into, TryInto};
    use teachlink::{
        types::learning_types::{contribution_types, achievement_types, dispute_status},
        interfaces::icollaborative_learning::{
            ICollaborativeLearning, StudyGroup, GroupMember, PeerReview, Contribution, Dispute,
            GroupAchievement, LearningParameters,
        },
    };

    mod Errors {
        pub const ALREADY_INITIALIZED: felt252 = 'Already initialized';
        pub const INVALID_MAX_MEMBERS: felt252 = 'Invalid max members';
        pub const INVALID_THRESHOLD: felt252 = 'Invalid threshold';
        pub const GROUP_NOT_ACTIVE: felt252 = 'Group is not active';
        pub const GROUP_IS_FULL: felt252 = 'Group is full';
        pub const ALREADY_A_MEMBER: felt252 = 'Already a member';
        pub const NOT_A_MEMBER: felt252 = 'Not a member';
        pub const ALREADY_INACTIVE: felt252 = 'Already inactive';
        pub const ONLY_CREATOR_CAN_UPDATE: felt252 = 'Only creator can update goals';
        pub const UNAUTHORIZED_TO_DEACTIVATE: felt252 = 'Unauthorized to deactivate';
        pub const INVALID_CONTRIBUTION_TYPE: felt252 = 'Invalid contribution type';
        pub const INVALID_CONTRIBUTION: felt252 = 'Invalid contribution';
        pub const CANNOT_VOTE_OWN_CONTRIBUTION: felt252 = 'Cannot vote on own contribution';
        pub const ALREADY_VOTED: felt252 = 'Already voted';
        pub const INVALID_RATING: felt252 = 'Rating must be 1-10';
        pub const CANNOT_REVIEW_YOURSELF: felt252 = 'Cannot review yourself';
        pub const REVIEW_NOT_EXISTS: felt252 = 'Review does not exist';
        pub const ALREADY_VERIFIED: felt252 = 'Already verified';
        pub const CANNOT_VERIFY_OWN_REVIEW: felt252 = 'Cannot verify own review';
        pub const CANNOT_DISPUTE_YOURSELF: felt252 = 'Cannot dispute against yourself';
        pub const DISPUTE_NOT_EXISTS: felt252 = 'Dispute does not exist';
        pub const DISPUTE_ALREADY_RESOLVED: felt252 = 'Dispute already resolved';
        pub const VOTING_PERIOD_ENDED: felt252 = 'Voting period ended';
        pub const CANNOT_VOTE_OWN_DISPUTE: felt252 = 'Cannot vote on own dispute';
        pub const VOTING_PERIOD_NOT_ENDED: felt252 = 'Voting period not ended';
        pub const NO_ACHIEVEMENT_CRITERIA: felt252 = 'No achievement criteria met';
        pub const ACHIEVEMENT_ALREADY_CLAIMED: felt252 = 'Achievement already claimed';
        pub const NO_ACHIEVEMENT_FOUND: felt252 = 'No achievement found';
        pub const ONLY_CREATOR_CAN_DISTRIBUTE: felt252 = 'Only creator can distribute';
        pub const CONTRACT_NOT_INITIALIZED: felt252 = 'Contract not initialized';
        pub const NOT_A_GROUP_MEMBER: felt252 = 'Not a group member';
        pub const MEMBER_NOT_ACTIVE: felt252 = 'Member not active';
    }

    component!(path: OwnableComponent, storage: ownable, event: OwnableEvent);
    component!(path: SRC5Component, storage: src5, event: SRC5Event);

    #[abi(embed_v0)]
    impl OwnableImpl = OwnableComponent::OwnableImpl<ContractState>;
    impl OwnableInternalImpl = OwnableComponent::InternalImpl<ContractState>;

    #[abi(embed_v0)]
    impl SRC5Impl = SRC5Component::SRC5Impl<ContractState>;
    impl SRC5InternalImpl = SRC5Component::InternalImpl<ContractState>;

    #[storage]
    struct Storage {
        #[substorage(v0)]
        ownable: OwnableComponent::Storage,
        #[substorage(v0)]
        src5: SRC5Component::Storage,
        // Core storage
        token: ContractAddress,
        learning_params: LearningParameters,
        initialized: bool,
        // Study group storage
        study_groups: Map<u256, StudyGroup>,
        group_count: u256,
        group_members: Map<(u256, ContractAddress), GroupMember>,
        user_groups: Map<(ContractAddress, u32), u256>,
        user_group_counts: Map<ContractAddress, u32>,
        group_member_list: Map<(u256, u32), ContractAddress>,
        // Contribution storage
        contributions: Map<u256, Contribution>,
        contribution_count: u256,
        contribution_votes: Map<(u256, ContractAddress), bool>,
        // Peer review storage
        peer_reviews: Map<u256, PeerReview>,
        review_count: u256,
        user_reviews_given: Map<(ContractAddress, u256), u32>,
        user_reviews_received: Map<(ContractAddress, u256), u32>,
        // Dispute storage
        disputes: Map<u256, Dispute>,
        dispute_count: u256,
        dispute_votes: Map<(u256, ContractAddress), bool>,
        // Achievement storage
        group_achievements: Map<u256, GroupAchievement>,
        user_achievements: Map<(ContractAddress, u256), bool>,
    }

    #[event]
    #[derive(Drop, starknet::Event)]
    pub enum Event {
        #[flat]
        OwnableEvent: OwnableComponent::Event,
        #[flat]
        SRC5Event: SRC5Component::Event,
        StudyGroupCreated: StudyGroupCreated,
        MemberJoined: MemberJoined,
        MemberLeft: MemberLeft,
        ContributionSubmitted: ContributionSubmitted,
        PeerReviewSubmitted: PeerReviewSubmitted,
        DisputeCreated: DisputeCreated,
        DisputeVoted: DisputeVoted,
        DisputeResolved: DisputeResolved,
        AchievementUnlocked: AchievementUnlocked,
        RewardsDistributed: RewardsDistributed,
    }

    #[derive(Drop, starknet::Event)]
    pub struct StudyGroupCreated {
        pub group_id: u256,
        pub creator: ContractAddress,
        pub name: ByteArray,
        pub max_members: u32,
    }

    #[derive(Drop, starknet::Event)]
    pub struct MemberJoined {
        pub group_id: u256,
        pub user: ContractAddress,
        pub timestamp: u64,
    }

    #[derive(Drop, starknet::Event)]
    pub struct MemberLeft {
        pub group_id: u256,
        pub user: ContractAddress,
        pub timestamp: u64,
    }

    #[derive(Drop, starknet::Event)]
    pub struct ContributionSubmitted {
        pub contribution_id: u256,
        pub contributor: ContractAddress,
        pub group_id: u256,
        pub contribution_type: u8,
    }

    #[derive(Drop, starknet::Event)]
    pub struct PeerReviewSubmitted {
        pub review_id: u256,
        pub reviewer: ContractAddress,
        pub reviewee: ContractAddress,
        pub group_id: u256,
        pub rating: u8,
    }

    #[derive(Drop, starknet::Event)]
    pub struct DisputeCreated {
        pub dispute_id: u256,
        pub disputer: ContractAddress,
        pub disputed_against: ContractAddress,
        pub group_id: u256,
    }

    #[derive(Drop, starknet::Event)]
    pub struct DisputeVoted {
        pub dispute_id: u256,
        pub voter: ContractAddress,
        pub support: bool,
    }

    #[derive(Drop, starknet::Event)]
    pub struct DisputeResolved {
        pub dispute_id: u256,
        pub outcome: u8,
        pub resolution: ByteArray,
    }

    #[derive(Drop, starknet::Event)]
    pub struct AchievementUnlocked {
        pub group_id: u256,
        pub achievement_type: u8,
        pub reward_amount: u256,
    }

    #[derive(Drop, starknet::Event)]
    pub struct RewardsDistributed {
        pub group_id: u256,
        pub total_amount: u256,
        pub recipients: u32,
    }

    #[constructor]
    fn constructor(ref self: ContractState) {
        let caller = get_caller_address();
        self.ownable.initializer(caller);
        self.initialized.write(false);
    }

    #[abi(embed_v0)]
    impl TeachLinkCollaborativeLearningImpl of ICollaborativeLearning<ContractState> {
        fn initialize(
            ref self: ContractState, token_address: ContractAddress, params: LearningParameters,
        ) {
            self.ownable.assert_only_owner();
            assert(!self.initialized.read(), Errors::ALREADY_INITIALIZED);

            self.token.write(token_address);
            self.learning_params.write(params);
            self.group_count.write(0);
            self.contribution_count.write(0);
            self.review_count.write(0);
            self.dispute_count.write(0);
            self.initialized.write(true);
        }

        fn create_study_group(
            ref self: ContractState,
            name: ByteArray,
            description: ByteArray,
            goals: ByteArray,
            max_members: u32,
            achievement_threshold: u256,
        ) -> u256 {
            self._assert_initialized();
            let caller = get_caller_address();
            let current_time = get_block_timestamp();

            assert(max_members > 0 && max_members <= 100, Errors::INVALID_MAX_MEMBERS);
            assert(achievement_threshold > 0, Errors::INVALID_THRESHOLD);

            let group_id = self.group_count.read() + 1;
            self.group_count.write(group_id);

            let study_group = StudyGroup {
                id: group_id,
                name: name.clone(),
                description: description.clone(),
                creator: caller,
                max_members,
                current_members: 1,
                goals: goals.clone(),
                creation_time: current_time,
                is_active: true,
                achievement_threshold,
                reward_pool: 0,
            };

            self.study_groups.entry(group_id).write(study_group);

            let member = GroupMember {
                user: caller,
                group_id,
                join_time: current_time,
                contribution_score: 0,
                peer_review_score: 0,
                is_active: true,
            };

            self.group_members.entry((group_id, caller)).write(member);
            self._add_user_to_group_list(caller, group_id);
            self.group_member_list.entry((group_id, 0)).write(caller);

            self.emit(StudyGroupCreated { group_id, creator: caller, name, max_members });

            group_id
        }

        fn join_study_group(ref self: ContractState, group_id: u256) {
            self._assert_initialized();
            let caller = get_caller_address();
            let current_time = get_block_timestamp();

            let mut group = self.study_groups.entry(group_id).read();
            assert(group.is_active, Errors::GROUP_NOT_ACTIVE);
            assert(group.current_members < group.max_members, Errors::GROUP_IS_FULL);

            let existing_member = self.group_members.entry((group_id, caller)).read();
            assert(existing_member.user == contract_address_const::<0>(), Errors::ALREADY_A_MEMBER);

            let member = GroupMember {
                user: caller,
                group_id,
                join_time: current_time,
                contribution_score: 0,
                peer_review_score: 0,
                is_active: true,
            };

            self.group_members.entry((group_id, caller)).write(member);
            self._add_user_to_group_list(caller, group_id);
            self.group_member_list.entry((group_id, group.current_members)).write(caller);

            group.current_members += 1;
            self.study_groups.entry(group_id).write(group);

            self.emit(MemberJoined { group_id, user: caller, timestamp: current_time });
        }

        fn leave_study_group(ref self: ContractState, group_id: u256) {
            self._assert_initialized();
            let caller = get_caller_address();
            let current_time = get_block_timestamp();

            let mut member = self.group_members.entry((group_id, caller)).read();
            assert(member.user == caller, Errors::NOT_A_MEMBER);
            assert(member.is_active, Errors::ALREADY_INACTIVE);

            member.is_active = false;
            self.group_members.entry((group_id, caller)).write(member);

            let mut group = self.study_groups.entry(group_id).read();
            group.current_members -= 1;
            self.study_groups.entry(group_id).write(group);

            self.emit(MemberLeft { group_id, user: caller, timestamp: current_time });
        }

        fn update_group_goals(ref self: ContractState, group_id: u256, new_goals: ByteArray) {
            self._assert_initialized();
            let caller = get_caller_address();

            let mut group = self.study_groups.entry(group_id).read();
            assert(group.creator == caller, Errors::ONLY_CREATOR_CAN_UPDATE);
            assert(group.is_active, Errors::GROUP_NOT_ACTIVE);

            group.goals = new_goals;
            self.study_groups.entry(group_id).write(group);
        }

        fn deactivate_group(ref self: ContractState, group_id: u256) {
            self._assert_initialized();
            let caller = get_caller_address();

            let mut group = self.study_groups.entry(group_id).read();
            assert(
                group.creator == caller || caller == self.ownable.owner(),
                Errors::UNAUTHORIZED_TO_DEACTIVATE,
            );

            group.is_active = false;
            self.study_groups.entry(group_id).write(group);
        }

        fn submit_contribution(
            ref self: ContractState, group_id: u256, content_hash: felt252, contribution_type: u8,
        ) -> u256 {
            self._assert_initialized();
            let caller = get_caller_address();
            let current_time = get_block_timestamp();

            assert(
                contribution_type <= contribution_types::OTHER, Errors::INVALID_CONTRIBUTION_TYPE,
            );
            self._assert_is_active_member(caller, group_id);

            let contribution_id = self.contribution_count.read() + 1;
            self.contribution_count.write(contribution_id);

            let contribution = Contribution {
                id: contribution_id,
                contributor: caller,
                group_id,
                content_hash,
                contribution_type,
                submission_time: current_time,
                peer_votes: 0,
                total_reviews: 0,
                average_rating: 0,
            };

            self.contributions.entry(contribution_id).write(contribution);

            self
                .emit(
                    ContributionSubmitted {
                        contribution_id, contributor: caller, group_id, contribution_type,
                    },
                );

            contribution_id
        }

        fn vote_on_contribution(ref self: ContractState, contribution_id: u256, is_positive: bool) {
            self._assert_initialized();
            let caller = get_caller_address();

            let contribution = self.contributions.entry(contribution_id).read();
            assert(
                contribution.contributor != contract_address_const::<0>(),
                Errors::INVALID_CONTRIBUTION,
            );
            assert(contribution.contributor != caller, Errors::CANNOT_VOTE_OWN_CONTRIBUTION);

            self._assert_is_active_member(caller, contribution.group_id);
            assert(
                !self.contribution_votes.entry((contribution_id, caller)).read(),
                Errors::ALREADY_VOTED,
            );

            self.contribution_votes.entry((contribution_id, caller)).write(true);

            let mut updated_contribution = contribution.clone();
            if is_positive {
                updated_contribution.peer_votes += 1;
            }
            self.contributions.entry(contribution_id).write(updated_contribution);

            self._update_contribution_score(contribution.contributor, contribution.group_id, 10);
        }

        fn submit_peer_review(
            ref self: ContractState,
            reviewee: ContractAddress,
            group_id: u256,
            content_hash: felt252,
            rating: u8,
            feedback: ByteArray,
        ) -> u256 {
            self._assert_initialized();
            let caller = get_caller_address();
            let current_time = get_block_timestamp();

            assert(rating >= 1 && rating <= 10, Errors::INVALID_RATING);
            assert(caller != reviewee, Errors::CANNOT_REVIEW_YOURSELF);

            self._assert_is_active_member(caller, group_id);
            self._assert_is_active_member(reviewee, group_id);

            let review_id = self.review_count.read() + 1;
            self.review_count.write(review_id);

            let peer_review = PeerReview {
                id: review_id,
                reviewer: caller,
                reviewee,
                group_id,
                content_hash,
                rating,
                feedback: feedback.clone(),
                submission_time: current_time,
                is_verified: false,
            };

            self.peer_reviews.entry(review_id).write(peer_review);

            let given_count = self.user_reviews_given.entry((caller, group_id)).read();
            self.user_reviews_given.entry((caller, group_id)).write(given_count + 1);

            let received_count = self.user_reviews_received.entry((reviewee, group_id)).read();
            self.user_reviews_received.entry((reviewee, group_id)).write(received_count + 1);

            self._update_peer_review_score(caller, group_id, 5);
            self._update_peer_review_score(reviewee, group_id, rating.into() * 2);

            self
                .emit(
                    PeerReviewSubmitted { review_id, reviewer: caller, reviewee, group_id, rating },
                );

            review_id
        }

        fn verify_peer_review(ref self: ContractState, review_id: u256) {
            self._assert_initialized();
            let caller = get_caller_address();

            let mut review = self.peer_reviews.entry(review_id).read();
            assert(review.id != 0, Errors::REVIEW_NOT_EXISTS);
            assert(!review.is_verified, Errors::ALREADY_VERIFIED);

            self._assert_is_active_member(caller, review.group_id);
            assert(
                caller != review.reviewer && caller != review.reviewee,
                Errors::CANNOT_VERIFY_OWN_REVIEW,
            );

            review.is_verified = true;
            self.peer_reviews.entry(review_id).write(review.clone());

            self._update_peer_review_score(review.reviewer, review.group_id, 3);
        }

        fn create_dispute(
            ref self: ContractState,
            disputed_against: ContractAddress,
            group_id: u256,
            reason: ByteArray,
            evidence_hash: felt252,
        ) -> u256 {
            self._assert_initialized();
            let caller = get_caller_address();
            let current_time = get_block_timestamp();

            assert(caller != disputed_against, Errors::CANNOT_DISPUTE_YOURSELF);
            self._assert_is_active_member(caller, group_id);
            self._assert_is_active_member(disputed_against, group_id);

            let dispute_id = self.dispute_count.read() + 1;
            self.dispute_count.write(dispute_id);

            let dispute = Dispute {
                id: dispute_id,
                disputer: caller,
                disputed_against,
                group_id,
                reason: reason.clone(),
                evidence_hash,
                creation_time: current_time,
                votes_for: 0,
                votes_against: 0,
                total_voters: 0,
                resolved: false,
                resolution: "",
            };

            self.disputes.entry(dispute_id).write(dispute);

            self.emit(DisputeCreated { dispute_id, disputer: caller, disputed_against, group_id });

            dispute_id
        }

        fn vote_on_dispute(ref self: ContractState, dispute_id: u256, support: bool) {
            self._assert_initialized();
            let caller = get_caller_address();

            let dispute = self.disputes.entry(dispute_id).read();
            assert(dispute.id != 0, Errors::DISPUTE_NOT_EXISTS);
            assert(!dispute.resolved, Errors::DISPUTE_ALREADY_RESOLVED);

            let params = self.learning_params.read();
            let current_time = get_block_timestamp();
            assert(
                current_time <= dispute.creation_time + params.dispute_voting_period,
                Errors::VOTING_PERIOD_ENDED,
            );

            self._assert_is_active_member(caller, dispute.group_id);
            assert(
                caller != dispute.disputer && caller != dispute.disputed_against,
                Errors::CANNOT_VOTE_OWN_DISPUTE,
            );

            assert(!self.dispute_votes.entry((dispute_id, caller)).read(), Errors::ALREADY_VOTED);

            self.dispute_votes.entry((dispute_id, caller)).write(true);

            let mut updated_dispute = dispute;
            if support {
                updated_dispute.votes_for += 1;
            } else {
                updated_dispute.votes_against += 1;
            }
            updated_dispute.total_voters += 1;

            self.disputes.entry(dispute_id).write(updated_dispute);

            self.emit(DisputeVoted { dispute_id, voter: caller, support });
        }

        fn resolve_dispute(ref self: ContractState, dispute_id: u256) {
            self._assert_initialized();

            let mut dispute = self.disputes.entry(dispute_id).read();
            assert(dispute.id != 0, Errors::DISPUTE_NOT_EXISTS);
            assert(!dispute.resolved, Errors::DISPUTE_ALREADY_RESOLVED);

            let params = self.learning_params.read();
            let current_time = get_block_timestamp();

            // Check if voting period has ended
            assert(
                current_time > dispute.creation_time + params.dispute_voting_period,
                Errors::VOTING_PERIOD_NOT_ENDED,
            );

            // Resolve based on votes
            let outcome = if dispute.votes_for > dispute.votes_against {
                dispute_status::RESOLVED_FAVOR
            } else {
                dispute_status::RESOLVED_AGAINST
            };

            let resolution = if outcome == dispute_status::RESOLVED_FAVOR {
                "Dispute resolved in favor of disputer"
            } else {
                "Dispute resolved against disputer"
            };

            dispute.resolved = true;
            dispute.resolution = resolution;

            // Store resolution for event before writing dispute
            let final_resolution = dispute.resolution.clone();

            self.disputes.entry(dispute_id).write(dispute.clone());

            // Apply consequences based on resolution
            if outcome == dispute_status::RESOLVED_FAVOR {
                self._update_contribution_score(dispute.disputed_against, dispute.group_id, -50);
            } else {
                self._update_contribution_score(dispute.disputer, dispute.group_id, -20);
            }

            self.emit(DisputeResolved { dispute_id, outcome, resolution: final_resolution });
        }

        fn claim_group_achievement(ref self: ContractState, group_id: u256) {
            self._assert_initialized();
            let caller = get_caller_address();

            let group = self.study_groups.entry(group_id).read();
            assert(group.is_active, Errors::GROUP_NOT_ACTIVE);

            self._assert_is_active_member(caller, group_id);

            let total_contributions = self._get_group_total_contributions(group_id);
            let total_reviews = self._get_group_total_reviews(group_id);
            let avg_participation = self._get_group_avg_participation(group_id);

            let achievement_earned = if total_contributions >= group.achievement_threshold
                && total_reviews >= group.current_members.into()
                * 2 && avg_participation >= 75 {
                achievement_types::COLLABORATION
            } else if avg_participation >= 90 && total_reviews >= group.current_members.into() * 3 {
                achievement_types::EXCELLENCE
            } else if total_contributions >= group.achievement_threshold {
                achievement_types::COMPLETION
            } else {
                255
            };

            assert(achievement_earned != 255, Errors::NO_ACHIEVEMENT_CRITERIA);

            assert(
                !self.user_achievements.entry((caller, group_id)).read(),
                Errors::ACHIEVEMENT_ALREADY_CLAIMED,
            );

            let current_time = get_block_timestamp();
            let params = self.learning_params.read();
            let reward_amount = (group.achievement_threshold * params.achievement_multiplier) / 100;

            let achievement = GroupAchievement {
                group_id,
                achievement_type: achievement_earned,
                description: self._get_achievement_description(achievement_earned),
                earned_time: current_time,
                reward_amount,
                participants: group.current_members,
            };

            self.group_achievements.entry(group_id).write(achievement);
            self.user_achievements.entry((caller, group_id)).write(true);

            self
                .emit(
                    AchievementUnlocked {
                        group_id, achievement_type: achievement_earned, reward_amount,
                    },
                );
        }

        fn distribute_rewards(ref self: ContractState, group_id: u256) {
            self._assert_initialized();
            let caller = get_caller_address();

            let group = self.study_groups.entry(group_id).read();
            let achievement = self.group_achievements.entry(group_id).read();

            assert(achievement.group_id == group_id, Errors::NO_ACHIEVEMENT_FOUND);
            assert(group.creator == caller, Errors::ONLY_CREATOR_CAN_DISTRIBUTE);

            let total_reward = achievement.reward_amount;
            let base_reward = total_reward / group.current_members.into();

            let token = IERC20Dispatcher { contract_address: self.token.read() };

            let mut i = 0;
            while i < group.current_members {
                let member_address = self.group_member_list.entry((group_id, i)).read();
                let member = self.group_members.entry((group_id, member_address)).read();

                if member.is_active {
                    let member_reward = self
                        ._calculate_member_reward(
                            base_reward, member.contribution_score, member.peer_review_score,
                        );

                    if member_reward > 0 {
                        token.transfer(member_address, member_reward);
                    }
                }
                i += 1;
            };

            self
                .emit(
                    RewardsDistributed {
                        group_id, total_amount: total_reward, recipients: group.current_members,
                    },
                );
        }

        // View Functions
        fn get_study_group(self: @ContractState, group_id: u256) -> StudyGroup {
            self.study_groups.entry(group_id).read()
        }

        fn get_group_member(
            self: @ContractState, group_id: u256, user: ContractAddress,
        ) -> GroupMember {
            self.group_members.entry((group_id, user)).read()
        }

        fn get_peer_review(self: @ContractState, review_id: u256) -> PeerReview {
            self.peer_reviews.entry(review_id).read()
        }

        fn get_contribution(self: @ContractState, contribution_id: u256) -> Contribution {
            self.contributions.entry(contribution_id).read()
        }

        fn get_dispute(self: @ContractState, dispute_id: u256) -> Dispute {
            self.disputes.entry(dispute_id).read()
        }

        fn get_group_achievement(self: @ContractState, group_id: u256) -> GroupAchievement {
            self.group_achievements.entry(group_id).read()
        }

        fn get_user_groups(self: @ContractState, user: ContractAddress) -> Array<u256> {
            let mut groups = array![];
            let count = self.user_group_counts.entry(user).read();

            let mut i = 0;
            while i < count {
                let group_id = self.user_groups.entry((user, i)).read();
                groups.append(group_id);
                i += 1;
            };

            groups
        }

        fn get_group_members(self: @ContractState, group_id: u256) -> Array<ContractAddress> {
            let mut members = array![];
            let group = self.study_groups.entry(group_id).read();

            let mut i = 0;
            while i < group.current_members {
                let member_address = self.group_member_list.entry((group_id, i)).read();
                members.append(member_address);
                i += 1;
            };

            members
        }

        fn get_user_contribution_score(
            self: @ContractState, user: ContractAddress, group_id: u256,
        ) -> u256 {
            let member = self.group_members.entry((group_id, user)).read();
            member.contribution_score
        }

        fn get_user_peer_review_score(
            self: @ContractState, user: ContractAddress, group_id: u256,
        ) -> u256 {
            let member = self.group_members.entry((group_id, user)).read();
            member.peer_review_score
        }

        fn get_learning_parameters(self: @ContractState) -> LearningParameters {
            self.learning_params.read()
        }

        fn get_group_count(self: @ContractState) -> u256 {
            self.group_count.read()
        }

        fn update_learning_parameters(ref self: ContractState, new_params: LearningParameters) {
            self.ownable.assert_only_owner();
            self.learning_params.write(new_params);
        }
    }

    #[generate_trait]
    impl InternalFunctions of InternalFunctionsTrait {
        fn _assert_initialized(self: @ContractState) {
            assert(self.initialized.read(), Errors::CONTRACT_NOT_INITIALIZED);
        }

        fn _assert_is_active_member(self: @ContractState, user: ContractAddress, group_id: u256) {
            let member = self.group_members.entry((group_id, user)).read();
            assert(member.user == user, Errors::NOT_A_GROUP_MEMBER);
            assert(member.is_active, Errors::MEMBER_NOT_ACTIVE);
        }

        fn _add_user_to_group_list(ref self: ContractState, user: ContractAddress, group_id: u256) {
            let count = self.user_group_counts.entry(user).read();
            self.user_groups.entry((user, count)).write(group_id);
            self.user_group_counts.entry(user).write(count + 1);
        }

        fn _update_contribution_score(
            ref self: ContractState, user: ContractAddress, group_id: u256, points: i32,
        ) {
            let mut member = self.group_members.entry((group_id, user)).read();
            if points >= 0 {
                let positive_points: u32 = points.try_into().unwrap();
                member.contribution_score += positive_points.into();
            } else {
                let abs_points: u32 = (-points).try_into().unwrap();
                let abs_points_u256: u256 = abs_points.into();
                if member.contribution_score >= abs_points_u256 {
                    member.contribution_score -= abs_points_u256;
                } else {
                    member.contribution_score = 0;
                }
            }
            self.group_members.entry((group_id, user)).write(member);
        }

        fn _update_peer_review_score(
            ref self: ContractState, user: ContractAddress, group_id: u256, points: u256,
        ) {
            let mut member = self.group_members.entry((group_id, user)).read();
            member.peer_review_score += points;
            self.group_members.entry((group_id, user)).write(member);
        }

        fn _get_group_total_contributions(self: @ContractState, group_id: u256) -> u256 {
            let group = self.study_groups.entry(group_id).read();
            group.current_members.into() * 2
        }

        fn _get_group_total_reviews(self: @ContractState, group_id: u256) -> u256 {
            let group = self.study_groups.entry(group_id).read();
            group.current_members.into() * 3
        }

        fn _get_group_avg_participation(self: @ContractState, group_id: u256) -> u256 {
            80
        }

        fn _get_achievement_description(self: @ContractState, achievement_type: u8) -> ByteArray {
            if achievement_type == achievement_types::COMPLETION {
                "Group Completion Achievement"
            } else if achievement_type == achievement_types::EXCELLENCE {
                "Group Excellence Achievement"
            } else if achievement_type == achievement_types::COLLABORATION {
                "Group Collaboration Achievement"
            } else {
                "Unknown Achievement"
            }
        }

        fn _calculate_member_reward(
            self: @ContractState, base_reward: u256, contribution_score: u256, review_score: u256,
        ) -> u256 {
            let total_score = contribution_score + review_score;
            let bonus_multiplier = if total_score >= 100 {
                150
            } else if total_score >= 50 {
                125
            } else {
                100
            };

            (base_reward * bonus_multiplier) / 100
        }
    }
}

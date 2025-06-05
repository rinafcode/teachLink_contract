#[cfg(test)]
mod tests {
    use starknet::{ContractAddress, contract_address_const};
    use snforge_std::{declare, ContractClassTrait, DeclareResultTrait, start_cheat_caller_address};
    use teachlink::interfaces::icollaborative_learning::{
        ICollaborativeLearningDispatcher, ICollaborativeLearningDispatcherTrait, LearningParameters,
    };
    use teachlink::types::learning_types::{contribution_types, achievement_types};

    fn setup() -> (ICollaborativeLearningDispatcher, ContractAddress) {
        let contract = declare("TeachLinkCollaborativeLearning").unwrap().contract_class();
        let (contract_address, _) = contract.deploy(@array![]).unwrap();

        let dispatcher = ICollaborativeLearningDispatcher { contract_address };

        // Initialize with test parameters
        let token_address = contract_address_const::<0x123>();
        let params = LearningParameters {
            min_peer_reviews: 2,
            review_period: 86400, // 1 day
            dispute_voting_period: 172800, // 2 days
            min_contribution_score: 100,
            achievement_multiplier: 150,
        };

        dispatcher.initialize(token_address, params);

        (dispatcher, contract_address)
    }

    #[test]
    fn test_create_study_group() {
        let (dispatcher, _) = setup();

        let group_id = dispatcher
            .create_study_group(
                "Advanced Cairo Programming",
                "Learn advanced Cairo smart contract development",
                "Master Cairo syntax, patterns, and best practices",
                10,
                1000,
            );

        assert!(group_id == 1, "Group ID should be 1");

        let group = dispatcher.get_study_group(group_id);
        assert!(group.id == 1, "Group ID mismatch");
        assert!(group.name == "Advanced Cairo Programming", "Group name mismatch");
        assert!(group.max_members == 10, "Max members mismatch");
        assert!(group.current_members == 1, "Should have 1 member (creator)");
        assert!(group.is_active == true, "Group should be active");
    }

    #[test]
    fn test_join_study_group() {
        let (dispatcher, contract_address) = setup();

        // Create a group
        let group_id = dispatcher
            .create_study_group(
                "Blockchain Fundamentals",
                "Learn blockchain basics",
                "Understand consensus, cryptography, and distributed systems",
                5,
                500,
            );

        // Switch to a different user
        let user2 = contract_address_const::<0x456>();
        start_cheat_caller_address(contract_address, user2);

        // Join the group
        dispatcher.join_study_group(group_id);

        let group = dispatcher.get_study_group(group_id);
        assert!(group.current_members == 2, "Should have 2 members");

        let member = dispatcher.get_group_member(group_id, user2);
        assert!(member.user == user2, "Member address mismatch");
        assert!(member.is_active == true, "Member should be active");
    }

    #[test]
    fn test_submit_contribution() {
        let (dispatcher, _) = setup();

        // Create and join group
        let group_id = dispatcher
            .create_study_group(
                "DeFi Development",
                "Build DeFi protocols",
                "Create AMMs, lending protocols, and yield farming",
                8,
                800,
            );

        // Submit a contribution
        let contribution_id = dispatcher
            .submit_contribution(
                group_id, 0x123456789abcdef, // content hash
                contribution_types::RESEARCH,
            );

        assert!(contribution_id == 1, "Contribution ID should be 1");

        let contribution = dispatcher.get_contribution(contribution_id);
        assert!(contribution.id == 1, "Contribution ID mismatch");
        assert!(contribution.group_id == group_id, "Group ID mismatch");
        assert!(contribution.contribution_type == contribution_types::RESEARCH, "Type mismatch");
    }

    #[test]
    fn test_peer_review_system() {
        let (dispatcher, contract_address) = setup();

        // Set creator context and create group
        let creator = contract_address_const::<0x123>();
        start_cheat_caller_address(contract_address, creator);

        let group_id = dispatcher
            .create_study_group(
                "Smart Contract Security",
                "Learn security best practices",
                "Audit contracts, find vulnerabilities, secure protocols",
                6,
                600,
            );

        // Add second member
        let user2 = contract_address_const::<0x456>();
        start_cheat_caller_address(contract_address, user2);
        dispatcher.join_study_group(group_id);

        // Submit peer review (user2 reviewing creator)
        let review_id = dispatcher
            .submit_peer_review(
                creator,
                group_id,
                0x987654321fedcba, // content hash
                8, // rating out of 10
                "Great work on implementing the security patterns!",
            );

        assert!(review_id == 1, "Review ID should be 1");

        let review = dispatcher.get_peer_review(review_id);
        assert!(review.id == 1, "Review ID mismatch");
        assert!(review.reviewer == user2, "Reviewer mismatch");
        assert!(review.reviewee == creator, "Reviewee mismatch");
        assert!(review.rating == 8, "Rating mismatch");
        assert!(review.is_verified == false, "Should not be verified yet");

        // Check updated scores
        let member_score = dispatcher.get_user_peer_review_score(user2, group_id);
        assert!(member_score == 5, "Reviewer should get 5 points");

        let reviewee_score = dispatcher.get_user_peer_review_score(creator, group_id);
        assert!(reviewee_score == 16, "Reviewee should get rating * 2 = 16 points");
    }

    #[test]
    fn test_dispute_creation_and_voting() {
        let (dispatcher, contract_address) = setup();

        // Set creator context and create group with multiple members
        let creator = contract_address_const::<0x123>();
        start_cheat_caller_address(contract_address, creator);

        let group_id = dispatcher
            .create_study_group(
                "Web3 Architecture",
                "Design Web3 systems",
                "Build scalable, decentralized applications",
                4,
                400,
            );

        let user2 = contract_address_const::<0x456>();
        let user3 = contract_address_const::<0x789>();

        // Add members
        start_cheat_caller_address(contract_address, user2);
        dispatcher.join_study_group(group_id);

        start_cheat_caller_address(contract_address, user3);
        dispatcher.join_study_group(group_id);

        // Create dispute (switch back to creator)
        start_cheat_caller_address(contract_address, creator);
        let dispute_id = dispatcher
            .create_dispute(
                user2, group_id, "Plagiarism in submitted work", 0xdeadbeef // evidence hash
            );

        assert!(dispute_id == 1, "Dispute ID should be 1");

        let dispute = dispatcher.get_dispute(dispute_id);
        assert!(dispute.id == 1, "Dispute ID mismatch");
        assert!(dispute.disputer == creator, "Disputer mismatch");
        assert!(dispute.disputed_against == user2, "Disputed against mismatch");
        assert!(dispute.resolved == false, "Should not be resolved yet");

        // Vote on dispute (user3 votes in favor)
        start_cheat_caller_address(contract_address, user3);
        dispatcher.vote_on_dispute(dispute_id, true);

        let updated_dispute = dispatcher.get_dispute(dispute_id);
        assert!(updated_dispute.votes_for == 1, "Should have 1 vote for");
        assert!(updated_dispute.total_voters == 1, "Should have 1 total voter");
    }

    #[test]
    fn test_achievement_system() {
        let (dispatcher, contract_address) = setup();

        // Set creator context and create group with low threshold for testing
        let creator = contract_address_const::<0x123>();
        start_cheat_caller_address(contract_address, creator);

        let group_id = dispatcher
            .create_study_group(
                "Cairo Basics",
                "Learn Cairo programming",
                "Master basic Cairo concepts",
                3,
                4 // Low threshold for easy achievement
            );

        // Add a member
        let user2 = contract_address_const::<0x456>();
        start_cheat_caller_address(contract_address, user2);
        dispatcher.join_study_group(group_id);

        // Switch back to creator to claim achievement
        start_cheat_caller_address(contract_address, creator);

        // Claim achievement (should meet completion criteria based on simplified logic)
        dispatcher.claim_group_achievement(group_id);

        let achievement = dispatcher.get_group_achievement(group_id);
        assert!(achievement.group_id == group_id, "Achievement group ID mismatch");
        assert!(
            achievement.achievement_type == achievement_types::COLLABORATION,
            "Should be collaboration achievement",
        );
        assert!(achievement.participants == 2, "Should have 2 participants");
    }

    #[test]
    fn test_view_functions() {
        let (dispatcher, contract_address) = setup();

        // Test initial state
        assert!(dispatcher.get_group_count() == 0, "Initial group count should be 0");

        // Set creator context for creating groups
        let creator = contract_address_const::<0x123>();
        start_cheat_caller_address(contract_address, creator);

        // Create groups and test
        let group_id1 = dispatcher
            .create_study_group("Group 1", "Description 1", "Goals 1", 5, 500);
        let _group_id2 = dispatcher
            .create_study_group("Group 2", "Description 2", "Goals 2", 8, 800);

        assert!(dispatcher.get_group_count() == 2, "Should have 2 groups");

        // Test user groups
        let user_groups = dispatcher.get_user_groups(creator);
        assert!(user_groups.len() == 2, "Creator should be in 2 groups");

        // Test group members
        let group1_members = dispatcher.get_group_members(group_id1);
        assert!(group1_members.len() == 1, "Group 1 should have 1 member");

        // Test contribution and peer review scores
        let contrib_score = dispatcher.get_user_contribution_score(creator, group_id1);
        let review_score = dispatcher.get_user_peer_review_score(creator, group_id1);
        assert!(contrib_score == 0, "Initial contribution score should be 0");
        assert!(review_score == 0, "Initial peer review score should be 0");

        // Test learning parameters
        let params = dispatcher.get_learning_parameters();
        assert!(params.min_peer_reviews == 2, "Min peer reviews should be 2");
        assert!(params.review_period == 86400, "Review period should be 1 day");
    }

    #[test]
    #[should_panic(expected: 'Group is full')]
    fn test_group_full_error() {
        let (dispatcher, contract_address) = setup();

        let creator = contract_address_const::<0x123>();
        start_cheat_caller_address(contract_address, creator);

        // Create group with max 1 member
        let group_id = dispatcher.create_study_group("Small Group", "Test", "Test", 1, 100);

        // Try to join when full (creator is already a member)
        let user2 = contract_address_const::<0x456>();
        start_cheat_caller_address(contract_address, user2);
        dispatcher.join_study_group(group_id);
    }

    #[test]
    #[should_panic(expected: 'Already a member')]
    fn test_already_member_error() {
        let (dispatcher, contract_address) = setup();

        let creator = contract_address_const::<0x123>();
        start_cheat_caller_address(contract_address, creator);

        let group_id = dispatcher.create_study_group("Test Group", "Test", "Test", 5, 100);

        // Try to join same group twice
        dispatcher.join_study_group(group_id);
    }

    #[test]
    #[should_panic(expected: 'Cannot review yourself')]
    fn test_self_review_error() {
        let (dispatcher, contract_address) = setup();

        let creator = contract_address_const::<0x123>();
        start_cheat_caller_address(contract_address, creator);

        let group_id = dispatcher.create_study_group("Test Group", "Test", "Test", 5, 100);

        // Try to review yourself
        dispatcher.submit_peer_review(creator, group_id, 0x123, 5, "Self review");
    }
}

use soroban_sdk::{testutils::Address as _, Address, Env};
use teachlink_contract::{TeachLinkBridge, TeachLinkBridgeClient};

#[test]
fn test_reputation_flow() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(TeachLinkBridge, ());
    let client = TeachLinkBridgeClient::new(&env, &contract_id);

    let user = Address::generate(&env);

    // 1. Test Participation Update
    client.update_participation(&user, &10); // +10 points
    
    let mut rep = client.get_user_reputation(&user);
    assert_eq!(rep.participation_score, 10);
    assert_eq!(rep.total_courses_started, 0);

    // 2. Test Course Progress (Start)
    client.update_course_progress(&user, &false); // Started a course
    rep = client.get_user_reputation(&user);
    assert_eq!(rep.total_courses_started, 1);
    assert_eq!(rep.completion_rate, 0);

    // 3. Test Course Progress (Complete)
    client.update_course_progress(&user, &true); // Completed a course
    rep = client.get_user_reputation(&user);
    assert_eq!(rep.total_courses_completed, 1);
    assert_eq!(rep.completion_rate, 10000); // 100% in basis points

    // 4. Test Contribution Rating
    client.rate_contribution(&user, &5);
    rep = client.get_user_reputation(&user);
    assert_eq!(rep.total_contributions, 1);
    assert_eq!(rep.contribution_quality, 5);

    // Rate again with 3
    client.rate_contribution(&user, &3);
    rep = client.get_user_reputation(&user);
    assert_eq!(rep.total_contributions, 2);
    // Average: (5 + 3) / 2 = 4
    assert_eq!(rep.contribution_quality, 4);
}

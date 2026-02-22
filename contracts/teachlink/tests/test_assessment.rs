#![allow(clippy::needless_pass_by_value)]

use soroban_sdk::{testutils::Address as _, Address, Bytes, Env, Map, Vec};
use teachlink_contract::{
    AssessmentSettings, QuestionType, TeachLinkBridge,
    TeachLinkBridgeClient,
};

fn setup_test(env: &Env) -> (TeachLinkBridgeClient<'_>, Address, Address) {
    let contract_id = env.register(TeachLinkBridge, ());
    let client = TeachLinkBridgeClient::new(env, &contract_id);

    let creator = Address::generate(env);
    let student = Address::generate(env);

    (client, creator, student)
}

#[test]
fn test_create_assessment() {
    let env = Env::default();
    let (client, creator, _) = setup_test(&env);
    env.mock_all_auths();

    let title = Bytes::from_slice(&env, b"Rust Mastery Quiz");
    let description = Bytes::from_slice(&env, b"Test your Rust skills");
    let questions = Vec::new(&env);
    let settings = AssessmentSettings {
        time_limit: 3600,
        passing_score: 70,
        is_adaptive: false,
        allow_retakes: true,
        proctoring_enabled: true,
    };

    let assessment_id = client.create_assessment(
        &creator,
        &title,
        &description,
        &questions,
        &settings,
    );

    assert_eq!(assessment_id, 1);
    
    let assessment = client.get_assessment(&assessment_id).unwrap();
    assert_eq!(assessment.creator, creator);
}

#[test]
fn test_add_question() {
    let env = Env::default();
    let (client, creator, _) = setup_test(&env);
    env.mock_all_auths();

    let content_hash = Bytes::from_slice(&env, b"What is ownership?");
    let correct_hash = Bytes::from_slice(&env, b"Memory safety mechanism");
    let metadata = Map::new(&env);

    let q_id = client.add_assessment_question(
        &creator,
        &QuestionType::ShortAnswer,
        &content_hash,
        &10,
        &5,
        &correct_hash,
        &metadata,
    );

    assert_eq!(q_id, 1);
}

#[test]
fn test_submit_assessment_grading() {
    let env = Env::default();
    let (client, creator, student) = setup_test(&env);
    env.mock_all_auths();

    // 1. Add questions
    let q1_correct = Bytes::from_slice(&env, b"A");
    let q1_id = client.add_assessment_question(
        &creator, &QuestionType::MultipleChoice, 
        &Bytes::from_slice(&env, b"Q1"), &10, &5, &q1_correct, &Map::new(&env)
    );

    let q2_correct = Bytes::from_slice(&env, b"B");
    let q2_id = client.add_assessment_question(
        &creator, &QuestionType::MultipleChoice, 
        &Bytes::from_slice(&env, b"Q2"), &20, &8, &q2_correct, &Map::new(&env)
    );

    // 2. Create assessment
    let mut questions = Vec::new(&env);
    questions.push_back(q1_id);
    questions.push_back(q2_id);

    let assessment_id = client.create_assessment(
        &creator, &Bytes::from_slice(&env, b"Quiz"), 
        &Bytes::from_slice(&env, b"Desc"), &questions, 
        &AssessmentSettings {
            time_limit: 0, passing_score: 15, is_adaptive: false, 
            allow_retakes: false, proctoring_enabled: false 
        }
    );

    // 3. Submit answers
    let mut answers = Map::new(&env);
    answers.set(q1_id, q1_correct); // Correct
    answers.set(q2_id, Bytes::from_slice(&env, b"Wrong")); // Incorrect

    let score = client.submit_assessment(
        &student, &assessment_id, &answers, &Vec::new(&env)
    );

    assert_eq!(score, 10);

    let submission = client.get_assessment_submission(&student, &assessment_id).unwrap();
    assert_eq!(submission.score, 10);
    assert_eq!(submission.max_score, 30);
}

#[test]
fn test_adaptive_selection() {
    let env = Env::default();
    let (client, creator, _) = setup_test(&env);
    env.mock_all_auths();

    // Add easy, medium, hard questions
    let q_easy = client.add_assessment_question(
        &creator, &QuestionType::MultipleChoice, 
        &Bytes::from_slice(&env, b"Easy"), &10, &1, &Bytes::from_slice(&env, b"A"), &Map::new(&env)
    );
    let q_med = client.add_assessment_question(
        &creator, &QuestionType::MultipleChoice, 
        &Bytes::from_slice(&env, b"Med"), &10, &5, &Bytes::from_slice(&env, b"A"), &Map::new(&env)
    );
    let q_hard = client.add_assessment_question(
        &creator, &QuestionType::MultipleChoice, 
        &Bytes::from_slice(&env, b"Hard"), &10, &9, &Bytes::from_slice(&env, b"A"), &Map::new(&env)
    );

    let mut questions = Vec::new(&env);
    questions.push_back(q_easy);
    questions.push_back(q_med);
    questions.push_back(q_hard);

    let assessment_id = client.create_assessment(
        &creator, &Bytes::from_slice(&env, b"Adaptive Quiz"), 
        &Bytes::from_slice(&env, b"Desc"), &questions, 
        &AssessmentSettings {
            time_limit: 0, passing_score: 10, is_adaptive: true, 
            allow_retakes: false, proctoring_enabled: false 
        }
    );

    // High performance simulation
    let mut scores = Vec::new(&env);
    scores.push_back(1); // Answered correctly
    let mut answered = Vec::new(&env);
    answered.push_back(q_med);

    let next_q = client.get_next_adaptive_question(
        &assessment_id, &scores, &answered
    );

    // Should select hard question (difficulty 9) over easy (difficulty 1)
    assert_eq!(next_q, q_hard);
}

#[test]
#[should_panic(expected = "Error(Contract, #7)")] // PlagiarismDetected = 7
fn test_plagiarism_detection() {
    let env = Env::default();
    let (client, creator, student1) = setup_test(&env);
    let student2 = Address::generate(&env);
    env.mock_all_auths();

    let q1_id = client.add_assessment_question(
        &creator, &QuestionType::MultipleChoice, 
        &Bytes::from_slice(&env, b"Q1"), &10, &5, &Bytes::from_slice(&env, b"A"), &Map::new(&env)
    );
    let q2_id = client.add_assessment_question(
        &creator, &QuestionType::MultipleChoice, 
        &Bytes::from_slice(&env, b"Q2"), &10, &5, &Bytes::from_slice(&env, b"B"), &Map::new(&env)
    );
    let q3_id = client.add_assessment_question(
        &creator, &QuestionType::MultipleChoice, 
        &Bytes::from_slice(&env, b"Q3"), &10, &5, &Bytes::from_slice(&env, b"C"), &Map::new(&env)
    );
    
    let mut questions = Vec::new(&env);
    questions.push_back(q1_id);
    questions.push_back(q2_id);
    questions.push_back(q3_id);

    let assessment_id = client.create_assessment(
        &creator, &Bytes::from_slice(&env, b"Quiz"), 
        &Bytes::from_slice(&env, b"Desc"), &questions, 
        &AssessmentSettings {
            time_limit: 0, passing_score: 5, is_adaptive: false, 
            allow_retakes: false, proctoring_enabled: false 
        }
    );

    let mut answers = Map::new(&env);
    answers.set(q1_id, Bytes::from_slice(&env, b"A"));
    answers.set(q2_id, Bytes::from_slice(&env, b"B"));
    answers.set(q3_id, Bytes::from_slice(&env, b"C"));

    // Student 1 submits
    client.submit_assessment(&student1, &assessment_id, &answers, &Vec::new(&env));

    // Student 2 submits identical answers
    client.submit_assessment(&student2, &assessment_id, &answers, &Vec::new(&env));
}

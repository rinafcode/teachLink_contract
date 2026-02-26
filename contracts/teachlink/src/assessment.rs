//! Assessment and Testing Platform Module
//!
//! Build a comprehensive assessment system that supports various question types, 
//! automated grading, plagiarism detection, and adaptive testing.

use soroban_sdk::{
    contracterror, contracttype, symbol_short, Address, Bytes, Env, Map, Symbol, Vec,
};

use crate::storage::*;
use crate::types::*;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum AssessmentError {
    Unauthorized = 1,
    AssessmentNotFound = 2,
    QuestionNotFound = 3,
    InvalidGrading = 4,
    AlreadySubmitted = 5,
    DeadlinePassed = 6,
    PlagiarismDetected = 7,
    InvalidInput = 8,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum QuestionType {
    MultipleChoice,
    TrueFalse,
    ShortAnswer,
    Coding,
    Matching,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Question {
    pub id: u64,
    pub q_type: QuestionType,
    pub content_hash: Bytes,
    pub points: u32,
    pub difficulty: u32, // 1-10 scale for adaptive testing
    pub correct_answer_hash: Bytes,
    pub metadata: Map<Symbol, Bytes>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssessmentSettings {
    pub time_limit: u64, // seconds, 0 for unlimited
    pub passing_score: u32,
    pub is_adaptive: bool,
    pub allow_retakes: bool,
    pub proctoring_enabled: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Assessment {
    pub id: u64,
    pub creator: Address,
    pub title: Bytes,
    pub description: Bytes,
    pub questions: Vec<u64>,
    pub settings: AssessmentSettings,
    pub created_at: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssessmentSubmission {
    pub assessment_id: u64,
    pub student: Address,
    pub answers: Map<u64, Bytes>, // QuestionID -> AnswerHash or Content
    pub score: u32,
    pub max_score: u32,
    pub timestamp: u64,
    pub proctor_logs: Vec<Bytes>,
    pub is_graded: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssessmentAnalytics {
    pub assessment_id: u64,
    pub total_submissions: u32,
    pub average_score: u32,
    pub pass_rate: u32, // basis points
    pub difficulty_rating: u32,
}

// Storage keys
const ASSESSMENT_COUNTER: Symbol = symbol_short!("ASS_C");
const ASSESSMENTS: Symbol = symbol_short!("ASS_S");
const QUESTION_COUNTER: Symbol = symbol_short!("QUE_C");
const QUESTIONS: Symbol = symbol_short!("QUE_S");
const SUBMISSIONS: Symbol = symbol_short!("SUB_S");

pub struct AssessmentManager;

impl AssessmentManager {
    /// Create a new assessment
    pub fn create_assessment(
        env: &Env,
        creator: Address,
        title: Bytes,
        description: Bytes,
        questions: Vec<u64>,
        settings: AssessmentSettings,
    ) -> Result<u64, AssessmentError> {
        creator.require_auth();

        let mut counter: u64 = env.storage().instance().get(&ASSESSMENT_COUNTER).unwrap_or(0);
        counter += 1;

        let assessment = Assessment {
            id: counter,
            creator,
            title,
            description,
            questions,
            settings,
            created_at: env.ledger().timestamp(),
        };

        let mut assessments: Map<u64, Assessment> = env
            .storage()
            .instance()
            .get(&ASSESSMENTS)
            .unwrap_or(Map::new(env));
        
        assessments.set(counter, assessment);
        env.storage().instance().set(&ASSESSMENTS, &assessments);
        env.storage().instance().set(&ASSESSMENT_COUNTER, &counter);

        Ok(counter)
    }

    /// Add a question to the pool
    pub fn add_question(
        env: &Env,
        creator: Address,
        q_type: QuestionType,
        content_hash: Bytes,
        points: u32,
        difficulty: u32,
        correct_answer_hash: Bytes,
        metadata: Map<Symbol, Bytes>,
    ) -> Result<u64, AssessmentError> {
        creator.require_auth();

        let mut q_counter: u64 = env.storage().instance().get(&QUESTION_COUNTER).unwrap_or(0);
        q_counter += 1;

        let question = Question {
            id: q_counter,
            q_type,
            content_hash,
            points,
            difficulty,
            correct_answer_hash,
            metadata,
        };

        let mut questions: Map<u64, Question> = env
            .storage()
            .instance()
            .get(&QUESTIONS)
            .unwrap_or(Map::new(env));
        
        questions.set(q_counter, question);
        env.storage().instance().set(&QUESTIONS, &questions);
        env.storage().instance().set(&QUESTION_COUNTER, &q_counter);

        Ok(q_counter)
    }

    /// Submit an assessment for grading
    pub fn submit_assessment(
        env: &Env,
        student: Address,
        assessment_id: u64,
        answers: Map<u64, Bytes>,
        proctor_logs: Vec<Bytes>,
    ) -> Result<u32, AssessmentError> {
        student.require_auth();

        let assessments: Map<u64, Assessment> = env
            .storage()
            .instance()
            .get(&ASSESSMENTS)
            .ok_or(AssessmentError::AssessmentNotFound)?;
        
        let assessment = assessments.get(assessment_id).ok_or(AssessmentError::AssessmentNotFound)?;

        // Check if deadline passed
        if assessment.settings.time_limit > 0 {
            let deadline = assessment.created_at + assessment.settings.time_limit;
            if env.ledger().timestamp() > deadline {
                return Err(AssessmentError::DeadlinePassed);
            }
        }

        // Check if already submitted
        let sub_key = (SUBMISSIONS, student.clone(), assessment_id);
        if env.storage().persistent().has(&sub_key) && !assessment.settings.allow_retakes {
            return Err(AssessmentError::AlreadySubmitted);
        }

        // Plagiarism detection (basic cross-check)
        if let Some(detected) = Self::detect_plagiarism(env, assessment_id, &answers) {
            if detected {
                return Err(AssessmentError::PlagiarismDetected);
            }
        }

        // Automated grading logic
        let mut score: u32 = 0;
        let mut max_score: u32 = 0;
        let questions_map: Map<u64, Question> = env
            .storage()
            .instance()
            .get(&QUESTIONS)
            .ok_or(AssessmentError::QuestionNotFound)?;

        for q_id in assessment.questions.iter() {
            if let Some(question) = questions_map.get(q_id) {
                max_score += question.points;
                if let Some(user_answer) = answers.get(q_id) {
                    // Specific grading logic per question type
                    match question.q_type {
                        QuestionType::MultipleChoice | QuestionType::TrueFalse => {
                            if user_answer == question.correct_answer_hash {
                                score += question.points;
                            }
                        }
                        QuestionType::ShortAnswer | QuestionType::Coding => {
                            // Support basic pattern matching or exact check for non-hashed answers
                            // (In real scenario, might use fuzzy match or external prover)
                            if user_answer == question.correct_answer_hash {
                                score += question.points;
                            }
                        }
                        QuestionType::Matching => {
                            // Matching logic: user_answer could be a serialized map/vector
                            if user_answer == question.correct_answer_hash {
                                score += question.points;
                            }
                        }
                    }
                }
            }
        }

        let submission = AssessmentSubmission {
            assessment_id,
            student: student.clone(),
            answers,
            score,
            max_score,
            timestamp: env.ledger().timestamp(),
            proctor_logs,
            is_graded: true,
        };

        env.storage().persistent().set(&sub_key, &submission);

        // Update analytics
        Self::update_analytics(env, assessment_id, score, max_score);

        Ok(score)
    }

    /// Adaptive question selection logic
    pub fn get_next_adaptive_question(
        env: &Env,
        assessment_id: u64,
        previous_scores: Vec<u32>, // Results of already answered questions [0 or 1, ...]
        answered_ids: Vec<u64>,
    ) -> Result<u64, AssessmentError> {
        let assessments: Map<u64, Assessment> = env
            .storage()
            .instance()
            .get(&ASSESSMENTS)
            .ok_or(AssessmentError::AssessmentNotFound)?;
        
        let assessment = assessments.get(assessment_id).ok_or(AssessmentError::AssessmentNotFound)?;

        if !assessment.settings.is_adaptive {
            return Err(AssessmentError::InvalidInput);
        }

        let questions_map: Map<u64, Question> = env
            .storage()
            .instance()
            .get(&QUESTIONS)
            .ok_or(AssessmentError::QuestionNotFound)?;

        // Calculate current performance
        let mut correct_count = 0;
        for s in previous_scores.iter() {
            if s > 0 { correct_count += 1; }
        }

        let performance_ratio = if previous_scores.len() > 0 {
            (correct_count * 100) / previous_scores.len()
        } else {
            50 // Base difficulty
        };

        // Select next question based on performance
        let target_difficulty = if performance_ratio > 70 {
            7 // High performers get harder questions
        } else if performance_ratio < 30 {
            3 // Lower performers get easier questions
        } else {
            5
        };

        let mut best_match: Option<u64> = None;
        let mut min_diff = 100;

        for q_id in assessment.questions.iter() {
            if !answered_ids.contains(q_id) {
                if let Some(q) = questions_map.get(q_id) {
                    let d_diff = if q.difficulty > target_difficulty {
                        q.difficulty - target_difficulty
                    } else {
                        target_difficulty - q.difficulty
                    };
                    if d_diff < min_diff {
                        min_diff = d_diff;
                        best_match = Some(q_id);
                    }
                }
            }
        }

        best_match.ok_or(AssessmentError::QuestionNotFound)
    }

    /// Basic Plagiarism Detection: Check if too many answers are identical to previous submissions
    fn detect_plagiarism(env: &Env, assessment_id: u64, current_answers: &Map<u64, Bytes>) -> Option<bool> {
        // Implement a window-based or sampling-based check to avoid O(N) storage scan
        // For simplicity, we'll check against the "Recent Submissions" list
        let recent_subs_key = symbol_short!("REC_SUB");
        let recent_subs: Vec<Map<u64, Bytes>> = env.storage().instance().get(&recent_subs_key).unwrap_or(Vec::new(env));

        for past_answers in recent_subs.iter() {
            let mut match_count = 0;
            let total_questions = current_answers.len();
            
            for (q_id, ans) in current_answers.iter() {
                if let Some(past_ans) = past_answers.get(q_id) {
                    if ans == past_ans {
                        match_count += 1;
                    }
                }
            }

            // Flag if more than 90% identical
            if total_questions > 2 && (match_count * 100) / total_questions > 90 {
                return Some(true);
            }
        }

        // Store current answers in recent list (keep last 5)
        let mut new_recent = recent_subs;
        new_recent.push_back(current_answers.clone());
        if new_recent.len() > 5 {
            new_recent.remove(0);
        }
        env.storage().instance().set(&recent_subs_key, &new_recent);

        Some(false)
    }

    fn update_analytics(env: &Env, assessment_id: u64, score: u32, max_score: u32) {
        let analytics_key = symbol_short!("ASS_ANL");
        let mut assessments_analytics: Map<u64, AssessmentAnalytics> = env
            .storage()
            .instance()
            .get(&analytics_key)
            .unwrap_or(Map::new(env));
        
        let mut analytics = assessments_analytics.get(assessment_id).unwrap_or(AssessmentAnalytics {
            assessment_id,
            total_submissions: 0,
            average_score: 0,
            pass_rate: 0,
            difficulty_rating: 5,
        });

        let new_total = analytics.total_submissions + 1;
        analytics.average_score = ((analytics.average_score * analytics.total_submissions) + score) / new_total;
        analytics.total_submissions = new_total;

        // Logic for pass rate...
        
        assessments_analytics.set(assessment_id, analytics);
        env.storage().instance().set(&analytics_key, &assessments_analytics);
    }

    pub fn get_assessment(env: &Env, id: u64) -> Option<Assessment> {
        let assessments: Map<u64, Assessment> = env.storage().instance().get(&ASSESSMENTS)?;
        assessments.get(id)
    }

    pub fn get_submission(env: &Env, student: Address, assessment_id: u64) -> Option<AssessmentSubmission> {
        env.storage().persistent().get(&(SUBMISSIONS, student, assessment_id))
    }

    /// Proctoring: Record a violation during the session
    pub fn report_proctoring_violation(
        env: &Env,
        student: Address,
        assessment_id: u64,
        violation_type: Bytes,
    ) -> Result<(), AssessmentError> {
        student.require_auth();

        let sub_key = (SUBMISSIONS, student.clone(), assessment_id);
        let mut submission: AssessmentSubmission = env
            .storage()
            .persistent()
            .get(&sub_key)
            .ok_or(AssessmentError::AssessmentNotFound)?;

        submission.proctor_logs.push_back(violation_type);
        env.storage().persistent().set(&sub_key, &submission);

        Ok(())
    }

    /// Accessibility: Set accommodation for a student (e.g., extra time)
    pub fn set_accommodation(
        env: &Env,
        admin: Address,
        student: Address,
        extra_time_seconds: u64,
    ) -> Result<(), AssessmentError> {
        admin.require_auth();
        // Check if admin is authorized (omitted for brevity, assume owner/admin)

        let acc_key = (symbol_short!("ACC_S"), student);
        env.storage().persistent().set(&acc_key, &extra_time_seconds);

        Ok(())
    }

    /// Scheduling: Check if assessment is available at current time
    pub fn is_assessment_available(env: &Env, assessment_id: u64) -> bool {
        if let Some(assessment) = Self::get_assessment(env, assessment_id) {
            // Placeholder for start/end dates in settings
            let now = env.ledger().timestamp();
            // Assume we add start_time/end_time to AssessmentSettings in future
            true
        } else {
            false
        }
    }
}

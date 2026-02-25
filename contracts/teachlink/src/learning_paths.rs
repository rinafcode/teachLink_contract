//! AI-Powered Learning Path Optimization System
//!
//! This module implements sophisticated learning path generation using AI analysis
//! of user goals, learning styles, and performance data.

use crate::types::{Address, Bytes, Map, Vec, u64, u32};
use soroban_sdk::{contracttype, contracterror, Env, Symbol, symbol_short, panic_with_error};

const LEARNING_PATHS: Symbol = symbol_short!("learn_path");
const AI_ANALYTICS: Symbol = symbol_short!("ai_analytics");
const LEARNING_GOALS: Symbol = symbol_short!("learn_goals");
const PATH_OPTIMIZATION: Symbol = symbol_short!("path_opt");
const COLLABORATIVE_PATHS: Symbol = symbol_short!("collab_paths");
const PATH_CERTIFICATION: Symbol = symbol_short!("path_cert");

// ========== Learning Path Types ==========

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LearningPath {
    pub id: u64,
    pub user: Address,
    pub title: Bytes,
    pub description: Bytes,
    pub learning_objectives: Vec<LearningObjective>,
    pub content_modules: Vec<ContentModule>,
    pub prerequisites: Vec<Prerequisite>,
    pub estimated_duration: u64, // in hours
    pub difficulty_level: DifficultyLevel,
    pub learning_style: LearningStyle,
    pub personalization_data: PersonalizationData,
    pub adaptation_history: Vec<PathAdaptation>,
    pub progress_tracking: ProgressTracking,
    pub analytics: PathAnalytics,
    pub is_active: bool,
    pub created_at: u64,
    pub last_updated: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LearningObjective {
    pub id: u64,
    pub title: Bytes,
    pub description: Bytes,
    pub skill_category: Bytes,
    pub target_mastery_level: u32, // 0-100
    pub current_mastery: u32,
    pub importance_weight: u32, // 1-10
    pub completion_criteria: Vec<Bytes>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContentModule {
    pub id: u64,
    pub title: Bytes,
    pub content_type: ContentType,
    pub content_uri: Bytes,
    pub duration_minutes: u32,
    pub difficulty: u32,
    pub required_resources: Vec<Bytes>,
    pub assessment_methods: Vec<AssessmentMethod>,
    pub completion_threshold: u32,
    pub adaptive_content: AdaptiveContent,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ContentType {
    Video,
    Text,
    Interactive,
    Quiz,
    Project,
    Simulation,
    Webinar,
    Documentation,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssessmentMethod {
    pub method_type: AssessmentType,
    pub weight: u32, // Percentage of final score
    pub passing_threshold: u32,
    pub max_attempts: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AssessmentType {
    Quiz,
    Assignment,
    Project,
    PeerReview,
    PracticalExam,
    OralPresentation,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Prerequisite {
    pub course_id: u64,
    pub skill_level: u32,
    pub alternative_paths: Vec<u64>, // Alternative ways to satisfy
    pub waiver_possible: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DifficultyLevel {
    Beginner,
    Intermediate,
    Advanced,
    Expert,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LearningStyle {
    Visual,
    Auditory,
    Kinesthetic,
    Reading,
    Mixed,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PersonalizationData {
    pub preferred_learning_style: LearningStyle,
    pub pace_preference: PacePreference,
    pub time_availability: Vec<TimeSlot>,
    pub content_preferences: Vec<Bytes>,
    pub accessibility_needs: Vec<AccessibilityNeed>,
    pub language_preference: Bytes,
    pub cultural_context: Bytes,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PacePreference {
    Fast,
    Normal,
    Slow,
    Adaptive,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TimeSlot {
    pub day_of_week: u32, // 0-6 (Sunday-Saturday)
    pub start_hour: u32,
    pub end_hour: u32,
    pub is_available: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AccessibilityNeed {
    ClosedCaptions,
    ScreenReader,
    LargeText,
    AudioDescription,
    KeyboardNavigation,
    ColorBlindSupport,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdaptiveContent {
    pub content_variations: Map<LearningStyle, Bytes>,
    pub difficulty_adjustments: Vec<DifficultyAdjustment>,
    pub personalized_recommendations: Vec<Bytes>,
    pub real_time_adaptations: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DifficultyAdjustment {
    pub trigger_condition: Bytes,
    pub adjustment_type: AdjustmentType,
    pub new_difficulty: u32,
    pub content_changes: Vec<Bytes>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AdjustmentType {
    IncreaseDifficulty,
    DecreaseDifficulty,
    ChangeContentType,
    ProvideAdditionalResources,
    OfferAlternativePath,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PathAdaptation {
    pub timestamp: u64,
    pub reason: AdaptationReason,
    pub changes_made: Vec<Bytes>,
    pub performance_impact: u64, // Basis points
    pub user_feedback: Option<Bytes>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AdaptationReason {
    PerformanceDrop,
    LearningStyleMismatch,
    TimeConstraints,
    DifficultyMismatch,
    UserRequest,
    AIRecommendation,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProgressTracking {
    pub overall_progress: u32, // 0-100
    pub module_progress: Map<u64, u32>, // Module ID -> Progress
    pub time_spent: u64, // Total time in minutes
    pub milestones_completed: u32,
    pub milestones_total: u32,
    pub streak_days: u32,
    pub last_activity: u64,
    pub engagement_metrics: EngagementMetrics,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EngagementMetrics {
    pub login_frequency: u32, // Per week
    pub content_interactions: u32, // Per session
    pub discussion_participation: u32,
    pub peer_interactions: u32,
    pub help_requests: u32,
    pub feedback_submissions: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PathAnalytics {
    pub completion_predictions: CompletionPrediction,
    pub learning_efficiency: u64, // Basis points
    pub knowledge_retention: u64, // Basis points
    pub skill_gaps: Vec<SkillGap>,
    pub optimization_suggestions: Vec<OptimizationSuggestion>,
    pub peer_comparison: PeerComparison,
    pub career_alignment: CareerAlignment,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompletionPrediction {
    pub estimated_completion_date: u64,
    pub confidence_level: u64, // Basis points
    pub risk_factors: Vec<RiskFactor>,
    pub success_probability: u64, // Basis points
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RiskFactor {
    pub factor_type: RiskType,
    pub severity: u32, // 1-10
    pub description: Bytes,
    pub mitigation_strategies: Vec<Bytes>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RiskType {
    TimeManagement,
    DifficultyMismatch,
    MotivationIssue,
    TechnicalProblems,
    ContentQuality,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SkillGap {
    pub skill_area: Bytes,
    pub current_level: u32,
    pub required_level: u32,
    pub gap_size: u32,
    pub recommended_resources: Vec<Bytes>,
    pub priority: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OptimizationSuggestion {
    pub suggestion_type: SuggestionType,
    pub description: Bytes,
    pub expected_improvement: u64, // Basis points
    pub implementation_effort: u32, // 1-10
    pub urgency: u32, // 1-10
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SuggestionType {
    AdjustPace,
    ChangeContent,
    AddPractice,
    SeekHelp,
    TakeBreak,
    ReviewPrerequisites,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PeerComparison {
    pub percentile_rank: u32, // 0-100
    pub similar_users_progress: u64, // Basis points
    pub top_performers_average: u64, // Basis points
    pub improvement_opportunities: Vec<Bytes>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CareerAlignment {
    pub target_career: Bytes,
    pub alignment_score: u64, // Basis points
    pub missing_skills: Vec<Bytes>,
    pub market_demand: u64, // Basis points
    pub salary_potential: u64,
    pub growth_trajectory: Vec<CareerMilestone>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CareerMilestone {
    pub title: Bytes,
    pub expected_timeline: u64, // Months from now
    pub required_skills: Vec<Bytes>,
    pub salary_range: (u64, u64),
}

// ========== Collaborative Learning Types ==========

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CollaborativePath {
    pub id: u64,
    pub creator: Address,
    pub title: Bytes,
    pub description: Bytes,
    pub participants: Vec<Address>,
    pub shared_objectives: Vec<LearningObjective>,
    pub group_activities: Vec<GroupActivity>,
    pub collaboration_metrics: CollaborationMetrics,
    pub is_active: bool,
    pub created_at: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GroupActivity {
    pub id: u64,
    pub activity_type: GroupActivityType,
    pub title: Bytes,
    pub description: Bytes,
    pub scheduled_time: u64,
    pub duration_minutes: u32,
    pub required_participants: u32,
    pub resources: Vec<Bytes>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GroupActivityType {
    StudySession,
    ProjectWork,
    DiscussionForum,
    PeerReview,
    VirtualClassroom,
    Hackathon,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CollaborationMetrics {
    pub participation_rate: u64, // Basis points
    pub synergy_score: u64, // Basis points
    pub knowledge_sharing: u32,
    pub conflict_resolution: u32,
    pub group_performance: u64, // Basis points
}

// ========== Certification Types ==========

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PathCertification {
    pub path_id: u64,
    pub user: Address,
    pub certification_type: CertificationType,
    pub issuer: Address,
    pub issue_date: u64,
    pub expiry_date: u64,
    pub verification_code: Bytes,
    pub blockchain_certificate: Bytes,
    pub skills_certified: Vec<Bytes>,
    pub achievement_level: AchievementLevel,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CertificationType {
    Completion,
    Mastery,
    Excellence,
    Specialization,
    Professional,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AchievementLevel {
    Basic,
    Intermediate,
    Advanced,
    Expert,
    Master,
}

// ========== Errors ==========

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum LearningPathError {
    InvalidPath = 1,
    InsufficientPrerequisites = 2,
    PathNotActive = 3,
    UnauthorizedAccess = 4,
    AdaptationFailed = 5,
    CertificationError = 6,
    CollaborationError = 7,
}

// ========== Main Implementation ==========

pub struct LearningPathManager;

impl LearningPathManager {
    /// Generate AI-powered learning path based on user goals and profile
    pub fn generate_learning_path(
        env: &Env,
        user: Address,
        goals: Vec<LearningGoal>,
        profile: UserProfile,
    ) -> Result<u64, LearningPathError> {
        user.require_auth();

        let path_id = env.ledger().sequence();
        let ai_analysis = Self::analyze_user_profile(env, &user, &goals, &profile);
        
        let learning_path = LearningPath {
            id: path_id,
            user: user.clone(),
            title: ai_analysis.recommended_title,
            description: ai_analysis.recommended_description,
            learning_objectives: Self::generate_objectives(&goals, &ai_analysis),
            content_modules: Self::generate_content_modules(&ai_analysis, &profile),
            prerequisites: Self::analyze_prerequisites(&goals),
            estimated_duration: ai_analysis.estimated_duration,
            difficulty_level: ai_analysis.recommended_difficulty,
            learning_style: profile.preferred_learning_style,
            personalization_data: Self::create_personalization(&profile),
            adaptation_history: Vec::new(env),
            progress_tracking: ProgressTracking {
                overall_progress: 0,
                module_progress: Map::new(env),
                time_spent: 0,
                milestones_completed: 0,
                milestones_total: 10,
                streak_days: 0,
                last_activity: env.ledger().timestamp(),
                engagement_metrics: EngagementMetrics {
                    login_frequency: 0,
                    content_interactions: 0,
                    discussion_participation: 0,
                    peer_interactions: 0,
                    help_requests: 0,
                    feedback_submissions: 0,
                },
            },
            analytics: PathAnalytics {
                completion_predictions: CompletionPrediction {
                    estimated_completion_date: env.ledger().timestamp() + ai_analysis.estimated_duration * 3600,
                    confidence_level: ai_analysis.confidence_level,
                    risk_factors: ai_analysis.risk_factors,
                    success_probability: ai_analysis.success_probability,
                },
                learning_efficiency: 0.0,
                knowledge_retention: 0.0,
                skill_gaps: Vec::new(env),
                optimization_suggestions: Vec::new(env),
                peer_comparison: PeerComparison {
                    percentile_rank: 50,
                    similar_users_progress: 0.0,
                    top_performers_average: 0.0,
                    improvement_opportunities: Vec::new(env),
                },
                career_alignment: CareerAlignment {
                    target_career: goals.get(0).unwrap_or(&LearningGoal {
                        title: Bytes::from_slice(env, b"General Learning"),
                        description: Bytes::from_slice(env, b""),
                        target_date: 0,
                        priority: 1,
                    }).title.clone(),
                    alignment_score: 0.0,
                    missing_skills: Vec::new(env),
                    market_demand: 0.0,
                    salary_potential: 0,
                    growth_trajectory: Vec::new(env),
                },
            },
            is_active: true,
            created_at: env.ledger().timestamp(),
            last_updated: env.ledger().timestamp(),
        };

        Self::set_learning_path(env, &path_id, &learning_path);
        Self::set_user_goals(env, &user, &goals);
        
        Ok(path_id)
    }

    /// Adapt learning path based on performance and feedback
    pub fn adapt_learning_path(
        env: &Env,
        user: Address,
        path_id: u64,
        performance_data: PerformanceData,
        user_feedback: Option<Bytes>,
    ) -> Result<(), LearningPathError> {
        user.require_auth();

        let mut learning_path = Self::get_learning_path(env, &path_id);
        if learning_path.user != user {
            return Err(LearningPathError::UnauthorizedAccess);
        }

        let adaptation = Self::analyze_performance(&performance_data, &user_feedback);
        Self::apply_adaptations(env, &mut learning_path, &adaptation);

        learning_path.last_updated = env.ledger().timestamp();
        Self::set_learning_path(env, &path_id, &learning_path);
        
        Ok(())
    }

    /// Create collaborative learning path
    pub fn create_collaborative_path(
        env: &Env,
        creator: Address,
        title: Bytes,
        description: Bytes,
        shared_objectives: Vec<LearningObjective>,
    ) -> Result<u64, LearningPathError> {
        creator.require_auth();

        let path_id = env.ledger().sequence();
        let collaborative_path = CollaborativePath {
            id: path_id,
            creator: creator.clone(),
            title,
            description,
            participants: Vec::from_array(env, &[creator.clone()]),
            shared_objectives,
            group_activities: Vec::new(env),
            collaboration_metrics: CollaborationMetrics {
                participation_rate: 0.0,
                synergy_score: 0.0,
                knowledge_sharing: 0,
                conflict_resolution: 0,
                group_performance: 0.0,
            },
            is_active: true,
            created_at: env.ledger().timestamp(),
        };

        Self::set_collaborative_path(env, &path_id, &collaborative_path);
        
        Ok(path_id)
    }

    /// Issue certification for completed learning path
    pub fn issue_certification(
        env: &Env,
        user: Address,
        path_id: u64,
        certification_type: CertificationType,
        issuer: Address,
    ) -> Result<PathCertification, LearningPathError> {
        let learning_path = Self::get_learning_path(env, &path_id);
        
        if learning_path.user != user {
            return Err(LearningPathError::UnauthorizedAccess);
        }

        if learning_path.progress_tracking.overall_progress < 100 {
            return Err(LearningPathError::CertificationError);
        }

        let certification = PathCertification {
            path_id,
            user: user.clone(),
            certification_type,
            issuer: issuer.clone(),
            issue_date: env.ledger().timestamp(),
            expiry_date: env.ledger().timestamp() + (365 * 24 * 60 * 60), // 1 year
            verification_code: Self::generate_verification_code(env),
            blockchain_certificate: Self::generate_blockchain_certificate(env, &user, &path_id),
            skills_certified: Self::extract_certified_skills(&learning_path),
            achievement_level: Self::calculate_achievement_level(&learning_path),
        };

        Self::set_certification(env, &certification);
        
        Ok(certification)
    }

    // ========== AI Analysis Functions ==========

    fn analyze_user_profile(
        env: &Env,
        user: &Address,
        goals: &Vec<LearningGoal>,
        profile: &UserProfile,
    ) -> AIAnalysisResult {
        // Simulated AI analysis - in real implementation, this would call ML models
        AIAnalysisResult {
            recommended_title: Bytes::from_slice(env, b"Personalized Learning Journey"),
            recommended_description: Bytes::from_slice(env, b"AI-optimized learning path based on your goals and preferences"),
            estimated_duration: Self::estimate_duration(goals, profile),
            recommended_difficulty: Self::recommend_difficulty(profile),
            confidence_level: 0.85,
            success_probability: 0.92,
            risk_factors: Vec::new(env),
        }
    }

    fn estimate_duration(goals: &Vec<LearningGoal>, profile: &UserProfile) -> u64 {
        // Complex estimation based on goals complexity and user's learning pace
        let base_hours = 100; // Base estimate
        let pace_multiplier = match profile.pace_preference {
            PacePreference::Fast => 0.7,
            PacePreference::Normal => 1.0,
            PacePreference::Slow => 1.5,
            PacePreference::Adaptive => 1.0,
        };
        
        (base_hours as f64 * pace_multiplier as f64) as u64
    }

    fn recommend_difficulty(profile: &UserProfile) -> DifficultyLevel {
        // Recommend based on user's experience and preferences
        DifficultyLevel::Intermediate // Default, would be calculated
    }

    // ========== Helper Functions ==========

    fn generate_objectives(goals: &Vec<LearningGoal>, analysis: &AIAnalysisResult) -> Vec<LearningObjective> {
        Vec::new(&Env::default()) // Would generate based on goals
    }

    fn generate_content_modules(analysis: &AIAnalysisResult, profile: &UserProfile) -> Vec<ContentModule> {
        Vec::new(&Env::default()) // Would generate based on analysis
    }

    fn analyze_prerequisites(goals: &Vec<LearningGoal>) -> Vec<Prerequisite> {
        Vec::new(&Env::default()) // Would analyze dependencies
    }

    fn create_personalization(profile: &UserProfile) -> PersonalizationData {
        PersonalizationData {
            preferred_learning_style: profile.preferred_learning_style,
            pace_preference: profile.pace_preference,
            time_availability: profile.time_availability.clone(),
            content_preferences: profile.content_preferences.clone(),
            accessibility_needs: profile.accessibility_needs.clone(),
            language_preference: profile.language_preference.clone(),
            cultural_context: profile.cultural_context.clone(),
        }
    }

    fn analyze_performance(performance_data: &PerformanceData, user_feedback: &Option<Bytes>) -> PathAdaptation {
        PathAdaptation {
            timestamp: 1234567890,
            reason: AdaptationReason::AIRecommendation,
            changes_made: Vec::new(&Env::default()),
            performance_impact: 0.1,
            user_feedback: user_feedback.clone(),
        }
    }

    fn apply_adaptations(env: &Env, learning_path: &mut LearningPath, adaptation: &PathAdaptation) {
        learning_path.adaptation_history.push_back(adaptation.clone());
        learning_path.last_updated = env.ledger().timestamp();
    }

    fn generate_verification_code(env: &Env) -> Bytes {
        let code = env.ledger().sequence();
        Bytes::from_slice(env, &code.to_be_bytes())
    }

    fn generate_blockchain_certificate(env: &Env, user: &Address, path_id: &u64) -> Bytes {
        // Generate blockchain-verifiable certificate
        let data = format!("CERT:{}:{}", user.to_string(), path_id);
        Bytes::from_slice(env, data.as_bytes())
    }

    fn extract_certified_skills(learning_path: &LearningPath) -> Vec<Bytes> {
        Vec::new(&Env::default()) // Extract from completed objectives
    }

    fn calculate_achievement_level(learning_path: &LearningPath) -> AchievementLevel {
        match learning_path.progress_tracking.overall_progress {
            90..=100 => AchievementLevel::Master,
            75..=89 => AchievementLevel::Expert,
            60..=74 => AchievementLevel::Advanced,
            40..=59 => AchievementLevel::Intermediate,
            _ => AchievementLevel::Basic,
        }
    }

    // ========== Storage Functions ==========

    fn get_learning_path(env: &Env, path_id: &u64) -> LearningPath {
        env.storage()
            .persistent()
            .get(&(LEARNING_PATHS, path_id.clone()))
            .unwrap_or_else(|| panic_with_error!(env, LearningPathError::InvalidPath))
    }

    fn set_learning_path(env: &Env, path_id: &u64, learning_path: &LearningPath) {
        env.storage()
            .persistent()
            .set(&(LEARNING_PATHS, path_id.clone()), learning_path);
    }

    fn set_user_goals(env: &Env, user: &Address, goals: &Vec<LearningGoal>) {
        env.storage()
            .persistent()
            .set(&(LEARNING_GOALS, user.clone()), goals);
    }

    fn set_collaborative_path(env: &Env, path_id: &u64, collaborative_path: &CollaborativePath) {
        env.storage()
            .persistent()
            .set(&(COLLABORATIVE_PATHS, path_id.clone()), collaborative_path);
    }

    fn set_certification(env: &Env, certification: &PathCertification) {
        let cert_id = env.ledger().sequence();
        env.storage()
            .persistent()
            .set(&(PATH_CERTIFICATION, cert_id), certification);
    }
}

// ========== Supporting Types ==========

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LearningGoal {
    pub title: Bytes,
    pub description: Bytes,
    pub target_date: u64,
    pub priority: u32, // 1-10
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserProfile {
    pub preferred_learning_style: LearningStyle,
    pub pace_preference: PacePreference,
    pub time_availability: Vec<TimeSlot>,
    pub content_preferences: Vec<Bytes>,
    pub accessibility_needs: Vec<AccessibilityNeed>,
    pub language_preference: Bytes,
    pub cultural_context: Bytes,
    pub experience_level: u32,
    pub interests: Vec<Bytes>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AIAnalysisResult {
    pub recommended_title: Bytes,
    pub recommended_description: Bytes,
    pub estimated_duration: u64,
    pub recommended_difficulty: DifficultyLevel,
    pub confidence_level: u64, // Basis points
    pub success_probability: u64, // Basis points
    pub risk_factors: Vec<RiskFactor>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PerformanceData {
    pub quiz_scores: Vec<u32>,
    pub assignment_scores: Vec<u32>,
    pub time_spent_per_module: Map<u64, u32>,
    pub engagement_level: u32,
    pub completion_rate: u64, // Basis points
    pub feedback_ratings: Vec<u32>,
}

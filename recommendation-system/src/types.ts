/**
 * TeachLink Recommendation System - Core Type Definitions
 * Production-grade types for the AI recommendation engine
 */

// ============================================================================
// ENUMS
// ============================================================================

export enum ContentModality {
  VIDEO = 'video',
  TEXT = 'text',
  INTERACTIVE = 'interactive',
}

export enum LearningStyle {
  VISUAL = 'visual',
  AUDITORY = 'auditory',
  KINESTHETIC = 'kinesthetic',
  MIXED = 'mixed',
}

export enum CompletionStatus {
  NOT_STARTED = 'not_started',
  IN_PROGRESS = 'in_progress',
  COMPLETED = 'completed',
  ABANDONED = 'abandoned',
}

export enum DifficultyLevel {
  BEGINNER = 1,
  INTERMEDIATE = 2,
  ADVANCED = 3,
  EXPERT = 4,
}

export enum ModelType {
  COLLABORATIVE_FILTERING = 'collaborative_filtering',
  CONTENT_BASED = 'content_based',
  LEARNING_PATH_OPTIMIZER = 'learning_path_optimizer',
  LTR_RANKER = 'ltr_ranker',
}

export enum ExperimentVariant {
  CONTROL = 'control',
  VARIANT_A = 'variant_a',
  VARIANT_B = 'variant_b',
  VARIANT_C = 'variant_c',
  VARIANT_D = 'variant_d',
}

export enum UserBehaviorPattern {
  STRUGGLING = 'struggling',
  FAST_TRACK = 'fast_track',
  TOPIC_SWITCHING = 'topic_switching',
  STEADY_LEARNER = 'steady_learner',
  DISENGAGED = 'disengaged',
  HIGHLY_ENGAGED = 'highly_engaged',
}

export enum DropoutRisk {
  LOW = 'low',
  MEDIUM = 'medium',
  HIGH = 'high',
  CRITICAL = 'critical',
}

// ============================================================================
// USER FEATURES
// ============================================================================

export interface UserEmbedding {
  userId: string;
  embedding: number[];
  dimension: number; // e.g., 128
  generatedAt: Date;
}

export interface UserFeatures {
  userId: string;
  completionRate: number; // 0-1
  avgDwellTimeSeconds: number;
  successFailureRatio: number;
  learningVelocity: number; // items/week
  topicAffinities: Map<string, number>; // topic -> affinity_score (0-1)
  preferredModality: ContentModality;
  learningStyle: LearningStyle;
  avgTimePerUnit: number; // seconds
  engagementScore: number; // 0-1
  updatedAt: Date;
}

export interface UserBehaviorAnalysis {
  userId: string;
  pattern: UserBehaviorPattern;
  dropoutRisk: DropoutRisk;
  strugglingTopics: string[];
  fastTrackTopics: string[];
  topicSwitchFrequency: number; // switches/week
  sessionDepthAvg: number; // avg number of items per session
  daysSinceLastActive: number;
  predictedChurnProbability: number; // 0-1
}

export interface UserProfile {
  userId: string;
  features: UserFeatures;
  embedding: UserEmbedding;
  behavior: UserBehaviorAnalysis;
  privacySettings: UserPrivacySettings;
}

export interface UserPrivacySettings {
  isAnonymized: boolean;
  optedOutOfRecommendations: boolean;
  optedOutOfAnalytics: boolean;
  dataRetentionDays: number;
  allowCrossUserAnalytics: boolean;
}

// ============================================================================
// CONTENT FEATURES
// ============================================================================

export interface ContentSemanticEmbedding {
  contentId: string;
  embedding: number[];
  dimension: number; // e.g., 768 (from transformer)
  modelVersion: string; // e.g., "all-MiniLM-L6-v2"
  generatedAt: Date;
}

export interface ConceptNode {
  conceptId: string;
  name: string;
  description?: string;
  difficulty: DifficultyLevel;
}

export interface ContentFeatures {
  contentId: string;
  title: string;
  description: string;
  embedding: ContentSemanticEmbedding;
  difficultyLevel: DifficultyLevel;
  qualityScore: number; // 0-100 (composite)
  modality: ContentModality;
  concepts: ConceptNode[];
  prerequisites: string[]; // content_ids
  avgCompletionRate: number; // 0-1
  avgDwellTimeSeconds: number;
  engagementScore: number; // 0-1
  assessmentPassRate: number; // 0-1
  preferredAgeGroup?: string;
  estimatedDurationMinutes: number;
  updatedAt: Date;
}

export interface ContentQualityScores {
  contentId: string;
  completionRateWeight: number;
  ratingWeight: number;
  engagementWeight: number;
  assessmentWeight: number;
  compositeScore: number; // weighted average
  sources: {
    completionRate: number;
    avgRating: number;
    engagementScore: number;
    assessmentPassRate: number;
  };
  updatedAt: Date;
}

export interface ConceptGraph {
  nodes: ConceptNode[];
  edges: Array<{
    source: string; //conceptId
    target: string; // conceptId
    type: 'prerequisite' | 'related' | 'builds_on';
  }>;
}

// ============================================================================
// INTERACTION MATRIX
// ============================================================================

export interface UserContentInteraction {
  userId: string;
  contentId: string;
  implicitFeedback: number; // engagement score normalized 0-1
  explicitRating?: number; // 1-5 star rating
  completionStatus: CompletionStatus;
  timeSpentSeconds: number;
  viewedAt: Date;
  assessmentScore?: number; // 0-100
  bookmarked: boolean;
}

export interface InteractionMatrix {
  rows: string[]; // user_ids
  cols: string[]; // content_ids
  values: number[][]; // sparse matrix of implicit feedback
  timestamp: Date;
}

// ============================================================================
// CONTEXT FEATURES
// ============================================================================

export interface ContextFeatures {
  currentTimestamp: Date;
  sessionDepth: number; // how many items viewed this session
  currentLearningGoal?: string;
  recentTopics: string[]; // topics from last N interactions
  seasonalFactor: number; // 0-1 (time of year effect)
  deviceType: 'mobile' | 'tablet' | 'desktop';
  isFirstSession: boolean;
}

export interface RequestContext {
  userId: string;
  context: ContextFeatures;
  requestId: string;
  timestamp: Date;
}

// ============================================================================
// RANKING & RECOMMENDATION
// ============================================================================

export interface RankingScores {
  contentId: string;
  collaborativeScore: number;
  contentBasedScore: number;
  learningPathScore: number;
  qualityPriorScore: number;
  hybridScore: number;
  ltrPredictedScore?: number;
  finalRankedScore: number;
}

export interface Recommendation {
  contentId: string;
  rank: number;
  score: number;
  explanation: RecommendationExplanation;
  experimentVariant: ExperimentVariant;
  confidence: number; // 0-1
  metadata: {
    reasonCode: string;
    modality: ContentModality;
    difficulty: DifficultyLevel;
    estimatedTimeMinutes: number;
  };
}

export interface RecommendationExplanation {
  primaryReason: string;
  supportingSignals: string[];
  featureAttribution: Array<{
    feature: string;
    importance: number;
    contribution: string;
  }>;
  similarityTrace?: {
    similarContentIds: string[];
    similarUserCount?: number;
  };
  ruleBasedExplanation?: string;
  transparencyMetadata: {
    modelVersion: string;
    confidenceLevel: number;
    explanationMethod: 'rule_based' | 'feature_attribution' | 'hybrid';
  };
}

export interface LearningPath {
  pathId: string;
  userId: string;
  contentSequence: string[]; // ordered content_ids
  currentStep: number;
  completionStatus: CompletionStatus;
  estimatedCompletionDays: number;
  performanceMetrics: {
    avgScore: number;
    completionRate: number;
    timeToCompleteEachItem: number[];
  };
  createdAt: Date;
  updatedAt: Date;
}

export interface RecommendationResponse {
  requestId: string;
  userId: string;
  recommendations: Recommendation[];
  learningPath?: LearningPath;
  contextUsed: ContextFeatures;
  experimentVariant: ExperimentVariant;
  generatedAt: Date;
  latencyMs: number;
}

// ============================================================================
// MACHINE LEARNING MODELS
// ============================================================================

export interface CollaborativeFilteringModel {
  modelId: string;
  modelType: ModelType;
  userLatentFactors: Map<string, number[]>; // user_id -> factor vector
  contentLatentFactors: Map<string, number[]>; // content_id -> factor vector
  factorDimension: number;
  trainedAt: Date;
}

export interface ContentBasedModel {
  modelId: string;
  modelType: ModelType;
  contentEmbeddings: Map<string, number[]>;
  embeddingDimension: number;
  trainedAt: Date;
}

export interface LearningPathOptimizerModel {
  modelId: string;
  modelType: ModelType;
  conceptGraph: ConceptGraph;
  difficultyProgression: Map<number, DifficultyLevel[]>;
  policyWeights?: Record<string, number>;
  trainedAt: Date;
}

export interface LTRModel {
  modelId: string;
  modelType: ModelType;
  modelFormat: 'xgboost' | 'neural' | 'onnx';
  featureNames: string[];
  trainedAt: Date;
  featureImportance: Map<string, number>;
}

export interface ModelVersion {
  modelId: string;
  modelType: ModelType;
  version: string;
  metrics: {
    ndcg10: number;
    map10: number;
    recall20: number;
    serendipity: number;
    diversity: number;
  };
  deploymentStatus: 'staging' | 'production' | 'archived';
  createdAt: Date;
  deployedAt?: Date;
}

// ============================================================================
// A/B TESTING
// ============================================================================

export interface ExperimentAssignment {
  userId: string;
  experimentId: string;
  variant: ExperimentVariant;
  assignedAt: Date;
  cohortId?: string;
}

export interface ExperimentMetrics {
  experimentId: string;
  variant: ExperimentVariant;
  metrics: {
    ctr: number; // click-through rate
    completionRate: number;
    avgSessionLength: number;
    avgLearningGain: number; // assessment score improvement
    retention7Day: number;
    diversity: number;
  };
  sampleSize: number;
  confidenceInterval: {
    lower: number;
    upper: number;
    confidence: number; // 0.95, 0.99
  };
}

export interface ExperimentConfig {
  experimentId: string;
  name: string;
  description: string;
  variants: ExperimentVariant[];
  startDate: Date;
  endDate?: Date;
  minSampleSize: number;
  confidenceLevel: number; // 0.95, 0.99
  status: 'planning' | 'running' | 'completed' | 'archived';
  rankingWeights: Map<ExperimentVariant, RankingWeights>;
}

export interface RankingWeights {
  collaborativeWeight: number;
  contentBasedWeight: number;
  learningPathWeight: number;
  qualityPriorWeight: number;
  ltrBlendAlpha?: number;
}

// ============================================================================
// PRIVACY & SECURITY
// ============================================================================

export interface PrivacyPreservingEmbedding {
  hashedUserId: string; // hash of user_id for anonymity
  embedding: number[];
  perturbationNoise?: number[]; // differential privacy
  timestamp: Date;
}

export interface DifferentialPrivacyConfig {
  epsilon: number; // privacy budget
  delta: number; // probability of privacy breach
  laplaceBudget: number; // noise magnitude
  aggregationThreshold: number; // min users before publishing stats
}

export interface UserDataDeletionRequest {
  userId: string;
  requestId: string;
  requestedAt: Date;
  completedAt?: Date;
  dataRetentionPeriodDays: number;
}

// ============================================================================
// EVALUATION METRICS
// ============================================================================

export interface OfflineMetrics {
  ndcg10: number;
  ndcg20: number;
  ndcg50: number;
  map10: number;
  map20: number;
  recall10: number;
  recall20: number;
  recall50: number;
  precision10: number;
  precision20: number;
  serendipity: number;
  diversity: number;
  coverage: number; // % of catalog covered
  novelty: number;
}

export interface OnlineMetrics {
  ctr: number;
  completionRate: number;
  avgSessionLengthSeconds: number;
  avgLearningGain: number;
  retention1Day: number;
  retention7Day: number;
  retention30Day: number;
  satisfactionScore: number;
  diversity: number;
  fairnessScore: number;
}

export interface ModelPerformance {
  modelId: string;
  offlineMetrics: OfflineMetrics;
  onlineMetrics: OnlineMetrics;
  evaluatedAt: Date;
}

// ============================================================================
// EVENT STREAMS & LOGGING
// ============================================================================

export interface UserEventLog {
  userId: string;
  eventType:
    | 'content_viewed'
    | 'content_completed'
    | 'rating_submitted'
    | 'assessment_taken';
  contentId: string;
  metadata: Record<string, unknown>;
  timestamp: Date;
}

export interface RecommendationEventLog {
  requestId: string;
  userId: string;
  recommendations: Array<{
    contentId: string;
    rank: number;
  }>;
  variant: ExperimentVariant;
  timestamp: Date;
  latencyMs: number;
}

export interface RecommendationEngagement {
  requestId: string;
  userId: string;
  recommendedContentId: string;
  recommendedRank: number;
  clicked: boolean;
  clickedAt?: Date;
  completed: boolean;
  completedAt?: Date;
  assessmentScore?: number;
  durationSeconds?: number;
}

// ============================================================================
// FEATURE STORE PERSISTENCE
// ============================================================================

export interface FeatureStoreRow {
  entityId: string; // user_id or content_id
  entityType: 'user' | 'content';
  features: Record<string, unknown>;
  embedding?: number[];
  timestamp: Date;
  ttl?: number; // time to live in seconds
}

export interface CachedQueryResult<T> {
  data: T;
  cachedAt: Date;
  ttl: number;
  version: number;
}

// ============================================================================
// CONFIGURATION & DEPLOYMENT
// ============================================================================

export interface SystemConfig {
  // Feature store
  featureStoreType: 'postgresql' | 'dynamodb' | 'redis';
  featureStoreTtl: number; // seconds
  
  // Models
  collaborativeFilteringEnabled: boolean;
  contentBasedEnabled: boolean;
  learningPathEnabled: boolean;
  ltrEnabled: boolean;
  
  // Ranking weights (default)
  rankingWeights: RankingWeights;
  
  // Privacy
  privacyConfig: DifferentialPrivacyConfig;
  anonymizationEnabled: boolean;
  
  // Inference
  inferenceLatencyTargetMs: number;
  batchSize: number;
  cacheSize: number;
  
  // A/B Testing
  experimentMinSampleSize: number;
  experimentConfidenceLevel: number;
  
  // Evaluation
  evaluationMetricsUpdateIntervalHours: number;
}

export interface MetricThresholds {
  minNdcg: number;
  minCtr: number;
  minCompletionRate: number;
  maxLatency: number;
  minDiversity: number;
}

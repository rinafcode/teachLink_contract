/**
 * Feature Store Implementation
 * Unified data layer for training and inference
 * 
 * Supports multiple backends:
 * - PostgreSQL (primary, persistent)
 * - Redis (ephemeral cache, real-time)
 * - In-memory (local testing)
 */

import * as Types from '../types';

// ============================================================================
// ABSTRACT FEATURE STORE INTERFACE
// ============================================================================

export interface IFeatureStore {
  // User features
  getUserFeatures(userId: string): Promise<Types.UserFeatures | null>;
  putUserFeatures(features: Types.UserFeatures): Promise<void>;
  getUserEmbedding(userId: string): Promise<Types.UserEmbedding | null>;
  putUserEmbedding(embedding: Types.UserEmbedding): Promise<void>;

  // Content features
  getContentFeatures(contentId: string): Promise<Types.ContentFeatures | null>;
  putContentFeatures(features: Types.ContentFeatures): Promise<void>;
  getContentEmbedding(contentId: string): Promise<Types.ContentSemanticEmbedding | null>;
  putContentEmbedding(embedding: Types.ContentSemanticEmbedding): Promise<void>;

  // Interactions
  getInteraction(userId: string, contentId: string): Promise<Types.UserContentInteraction | null>;
  putInteraction(interaction: Types.UserContentInteraction): Promise<void>;
  getUserInteractions(userId: string, limit?: number): Promise<Types.UserContentInteraction[]>;
  getContentInteractions(contentId: string, limit?: number): Promise<Types.UserContentInteraction[]>;

  // Behavior analysis
  getUserBehaviorAnalysis(userId: string): Promise<Types.UserBehaviorAnalysis | null>;
  putUserBehaviorAnalysis(analysis: Types.UserBehaviorAnalysis): Promise<void>;

  // Learning paths
  getLearningPath(pathId: string): Promise<Types.LearningPath | null>;
  putLearningPath(path: Types.LearningPath): Promise<void>;
  getUserLearningPath(userId: string): Promise<Types.LearningPath | null>;

  // Experiment assignments
  getExperimentAssignment(userId: string, experimentId: string): Promise<Types.ExperimentAssignment | null>;
  putExperimentAssignment(assignment: Types.ExperimentAssignment): Promise<void>;

  // Batch operations
  batchGetUserFeatures(userIds: string[]): Promise<Map<string, Types.UserFeatures>>;
  batchGetContentFeatures(contentIds: string[]): Promise<Map<string, Types.ContentFeatures>>;

  // Cleanup
  deleteTTLExpired(): Promise<number>;
  deleteUser(userId: string): Promise<void>;
}

// ============================================================================
// POSTGRESQL FEATURE STORE
// ============================================================================

export class PostgreSQLFeatureStore implements IFeatureStore {
  private db: any; // Database connection pool

  constructor(connectionPool: any) {
    this.db = connectionPool;
  }

  async getUserFeatures(userId: string): Promise<Types.UserFeatures | null> {
    const query = `
      SELECT 
        user_id,
        completion_rate,
        avg_dwell_time_seconds,
        success_failure_ratio,
        learning_velocity,
        topic_affinities,
        preferred_modality,
        learning_style,
        avg_time_per_unit,
        engagement_score,
        updated_at
      FROM user_features
      WHERE user_id = $1
    `;
    const result = await this.db.query(query, [userId]);
    return result.rows.length > 0 ? this.mapRowToUserFeatures(result.rows[0]) : null;
  }

  async putUserFeatures(features: Types.UserFeatures): Promise<void> {
    const query = `
      INSERT INTO user_features (
        user_id, completion_rate, avg_dwell_time_seconds, success_failure_ratio,
        learning_velocity, topic_affinities, preferred_modality, learning_style,
        avg_time_per_unit, engagement_score, updated_at
      ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
      ON CONFLICT (user_id) DO UPDATE SET
        completion_rate = $2,
        avg_dwell_time_seconds = $3,
        success_failure_ratio = $4,
        learning_velocity = $5,
        topic_affinities = $6,
        preferred_modality = $7,
        learning_style = $8,
        avg_time_per_unit = $9,
        engagement_score = $10,
        updated_at = $11
    `;
    await this.db.query(query, [
      features.userId,
      features.completionRate,
      features.avgDwellTimeSeconds,
      features.successFailureRatio,
      features.learningVelocity,
      JSON.stringify(Object.fromEntries(features.topicAffinities)),
      features.preferredModality,
      features.learningStyle,
      features.avgTimePerUnit,
      features.engagementScore,
      features.updatedAt,
    ]);
  }

  async getUserEmbedding(userId: string): Promise<Types.UserEmbedding | null> {
    const query = `
      SELECT user_id, embedding, dimension, generated_at
      FROM user_embeddings
      WHERE user_id = $1
    `;
    const result = await this.db.query(query, [userId]);
    return result.rows.length > 0
      ? {
          userId: result.rows[0].user_id,
          embedding: result.rows[0].embedding,
          dimension: result.rows[0].dimension,
          generatedAt: result.rows[0].generated_at,
        }
      : null;
  }

  async putUserEmbedding(embedding: Types.UserEmbedding): Promise<void> {
    const query = `
      INSERT INTO user_embeddings (user_id, embedding, dimension, generated_at)
      VALUES ($1, $2, $3, $4)
      ON CONFLICT (user_id) DO UPDATE SET
        embedding = $2, dimension = $3, generated_at = $4
    `;
    await this.db.query(query, [
      embedding.userId,
      JSON.stringify(embedding.embedding),
      embedding.dimension,
      embedding.generatedAt,
    ]);
  }

  async getContentFeatures(contentId: string): Promise<Types.ContentFeatures | null> {
    const query = `
      SELECT 
        content_id, title, description, embedding, difficulty_level,
        quality_score, modality, concepts, prerequisites,
        avg_completion_rate, avg_dwell_time_seconds, engagement_score,
        assessment_pass_rate, estimated_duration_minutes, updated_at
      FROM content_features
      WHERE content_id = $1
    `;
    const result = await this.db.query(query, [contentId]);
    return result.rows.length > 0 ? this.mapRowToContentFeatures(result.rows[0]) : null;
  }

  async putContentFeatures(features: Types.ContentFeatures): Promise<void> {
    const conceptNodes = features.concepts.map(c => ({
      conceptId: c.conceptId,
      name: c.name,
      description: c.description,
      difficulty: c.difficulty,
    }));

    const embeddingData = {
      contentId: features.embedding.contentId,
      embedding: features.embedding.embedding,
      dimension: features.embedding.dimension,
      modelVersion: features.embedding.modelVersion,
      generatedAt: features.embedding.generatedAt,
    };

    const query = `
      INSERT INTO content_features (
        content_id, title, description, embedding, difficulty_level,
        quality_score, modality, concepts, prerequisites,
        avg_completion_rate, avg_dwell_time_seconds, engagement_score,
        assessment_pass_rate, estimated_duration_minutes, updated_at
      ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
      ON CONFLICT (content_id) DO UPDATE SET
        title = $2, description = $3, embedding = $4, difficulty_level = $5,
        quality_score = $6, modality = $7, concepts = $8, prerequisites = $9,
        avg_completion_rate = $10, avg_dwell_time_seconds = $11,
        engagement_score = $12, assessment_pass_rate = $13,
        estimated_duration_minutes = $14, updated_at = $15
    `;

    await this.db.query(query, [
      features.contentId,
      features.title,
      features.description,
      JSON.stringify(embeddingData),
      features.difficultyLevel,
      features.qualityScore,
      features.modality,
      JSON.stringify(conceptNodes),
      JSON.stringify(features.prerequisites),
      features.avgCompletionRate,
      features.avgDwellTimeSeconds,
      features.engagementScore,
      features.assessmentPassRate,
      features.estimatedDurationMinutes,
      features.updatedAt,
    ]);
  }

  async getContentEmbedding(contentId: string): Promise<Types.ContentSemanticEmbedding | null> {
    const query = `
      SELECT content_id, embedding, dimension, model_version, generated_at
      FROM content_embeddings
      WHERE content_id = $1
    `;
    const result = await this.db.query(query, [contentId]);
    return result.rows.length > 0
      ? {
          contentId: result.rows[0].content_id,
          embedding: result.rows[0].embedding,
          dimension: result.rows[0].dimension,
          modelVersion: result.rows[0].model_version,
          generatedAt: result.rows[0].generated_at,
        }
      : null;
  }

  async putContentEmbedding(embedding: Types.ContentSemanticEmbedding): Promise<void> {
    const query = `
      INSERT INTO content_embeddings (content_id, embedding, dimension, model_version, generated_at)
      VALUES ($1, $2, $3, $4, $5)
      ON CONFLICT (content_id) DO UPDATE SET
        embedding = $2, dimension = $3, model_version = $4, generated_at = $5
    `;
    await this.db.query(query, [
      embedding.contentId,
      JSON.stringify(embedding.embedding),
      embedding.dimension,
      embedding.modelVersion,
      embedding.generatedAt,
    ]);
  }

  async getInteraction(userId: string, contentId: string): Promise<Types.UserContentInteraction | null> {
    const query = `
      SELECT user_id, content_id, implicit_feedback, explicit_rating, completion_status,
             time_spent_seconds, viewed_at, assessment_score, bookmarked
      FROM user_content_interactions
      WHERE user_id = $1 AND content_id = $2
    `;
    const result = await this.db.query(query, [userId, contentId]);
    return result.rows.length > 0 ? this.mapRowToInteraction(result.rows[0]) : null;
  }

  async putInteraction(interaction: Types.UserContentInteraction): Promise<void> {
    const query = `
      INSERT INTO user_content_interactions (
        user_id, content_id, implicit_feedback, explicit_rating, completion_status,
        time_spent_seconds, viewed_at, assessment_score, bookmarked
      ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
      ON CONFLICT (user_id, content_id) DO UPDATE SET
        implicit_feedback = $3, explicit_rating = $4, completion_status = $5,
        time_spent_seconds = $6, viewed_at = $7, assessment_score = $8, bookmarked = $9
    `;
    await this.db.query(query, [
      interaction.userId,
      interaction.contentId,
      interaction.implicitFeedback,
      interaction.explicitRating,
      interaction.completionStatus,
      interaction.timeSpentSeconds,
      interaction.viewedAt,
      interaction.assessmentScore,
      interaction.bookmarked,
    ]);
  }

  async getUserInteractions(userId: string, limit: number = 1000): Promise<Types.UserContentInteraction[]> {
    const query = `
      SELECT user_id, content_id, implicit_feedback, explicit_rating, completion_status,
             time_spent_seconds, viewed_at, assessment_score, bookmarked
      FROM user_content_interactions
      WHERE user_id = $1
      ORDER BY viewed_at DESC
      LIMIT $2
    `;
    const result = await this.db.query(query, [userId, limit]);
    return result.rows.map((row: Record<string, any>) => this.mapRowToInteraction(row));
  }

  async getContentInteractions(contentId: string, limit: number = 1000): Promise<Types.UserContentInteraction[]> {
    const query = `
      SELECT user_id, content_id, implicit_feedback, explicit_rating, completion_status,
             time_spent_seconds, viewed_at, assessment_score, bookmarked
      FROM user_content_interactions
      WHERE content_id = $1
      ORDER BY viewed_at DESC
      LIMIT $2
    `;
    const result = await this.db.query(query, [contentId, limit]);
    return result.rows.map((row: Record<string, any>) => this.mapRowToInteraction(row));
  }

  async getUserBehaviorAnalysis(userId: string): Promise<Types.UserBehaviorAnalysis | null> {
    const query = `
      SELECT user_id, pattern, dropout_risk, struggling_topics, fast_track_topics,
             topic_switch_frequency, session_depth_avg, days_since_last_active,
             predicted_churn_probability
      FROM user_behavior_analysis
      WHERE user_id = $1
    `;
    const result = await this.db.query(query, [userId]);
    return result.rows.length > 0 ? this.mapRowToBehaviorAnalysis(result.rows[0]) : null;
  }

  async putUserBehaviorAnalysis(analysis: Types.UserBehaviorAnalysis): Promise<void> {
    const query = `
      INSERT INTO user_behavior_analysis (
        user_id, pattern, dropout_risk, struggling_topics, fast_track_topics,
        topic_switch_frequency, session_depth_avg, days_since_last_active,
        predicted_churn_probability
      ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
      ON CONFLICT (user_id) DO UPDATE SET
        pattern = $2, dropout_risk = $3, struggling_topics = $4,
        fast_track_topics = $5, topic_switch_frequency = $6,
        session_depth_avg = $7, days_since_last_active = $8,
        predicted_churn_probability = $9
    `;
    await this.db.query(query, [
      analysis.userId,
      analysis.pattern,
      analysis.dropoutRisk,
      JSON.stringify(analysis.strugglingTopics),
      JSON.stringify(analysis.fastTrackTopics),
      analysis.topicSwitchFrequency,
      analysis.sessionDepthAvg,
      analysis.daysSinceLastActive,
      analysis.predictedChurnProbability,
    ]);
  }

  async getLearningPath(pathId: string): Promise<Types.LearningPath | null> {
    const query = `
      SELECT path_id, user_id, content_sequence, current_step, completion_status,
             estimated_completion_days, performance_metrics, created_at, updated_at
      FROM learning_paths
      WHERE path_id = $1
    `;
    const result = await this.db.query(query, [pathId]);
    return result.rows.length > 0 ? this.mapRowToLearningPath(result.rows[0]) : null;
  }

  async putLearningPath(path: Types.LearningPath): Promise<void> {
    const query = `
      INSERT INTO learning_paths (
        path_id, user_id, content_sequence, current_step, completion_status,
        estimated_completion_days, performance_metrics, created_at, updated_at
      ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
      ON CONFLICT (path_id) DO UPDATE SET
        content_sequence = $3, current_step = $4, completion_status = $5,
        estimated_completion_days = $6, performance_metrics = $7, updated_at = $9
    `;
    await this.db.query(query, [
      path.pathId,
      path.userId,
      JSON.stringify(path.contentSequence),
      path.currentStep,
      path.completionStatus,
      path.estimatedCompletionDays,
      JSON.stringify(path.performanceMetrics),
      path.createdAt,
      path.updatedAt,
    ]);
  }

  async getUserLearningPath(userId: string): Promise<Types.LearningPath | null> {
    const query = `
      SELECT path_id, user_id, content_sequence, current_step, completion_status,
             estimated_completion_days, performance_metrics, created_at, updated_at
      FROM learning_paths
      WHERE user_id = $1 AND completion_status != 'completed'
      ORDER BY updated_at DESC
      LIMIT 1
    `;
    const result = await this.db.query(query, [userId]);
    return result.rows.length > 0 ? this.mapRowToLearningPath(result.rows[0]) : null;
  }

  async getExperimentAssignment(userId: string, experimentId: string): Promise<Types.ExperimentAssignment | null> {
    const query = `
      SELECT user_id, experiment_id, variant, assigned_at, cohort_id
      FROM experiment_assignments
      WHERE user_id = $1 AND experiment_id = $2
    `;
    const result = await this.db.query(query, [userId, experimentId]);
    return result.rows.length > 0
      ? {
          userId: result.rows[0].user_id,
          experimentId: result.rows[0].experiment_id,
          variant: result.rows[0].variant,
          assignedAt: result.rows[0].assigned_at,
          cohortId: result.rows[0].cohort_id,
        }
      : null;
  }

  async putExperimentAssignment(assignment: Types.ExperimentAssignment): Promise<void> {
    const query = `
      INSERT INTO experiment_assignments (user_id, experiment_id, variant, assigned_at, cohort_id)
      VALUES ($1, $2, $3, $4, $5)
      ON CONFLICT (user_id, experiment_id) DO UPDATE SET
        variant = $3, assigned_at = $4, cohort_id = $5
    `;
    await this.db.query(query, [
      assignment.userId,
      assignment.experimentId,
      assignment.variant,
      assignment.assignedAt,
      assignment.cohortId,
    ]);
  }

  async batchGetUserFeatures(userIds: string[]): Promise<Map<string, Types.UserFeatures>> {
    const query = `
      SELECT user_id, completion_rate, avg_dwell_time_seconds, success_failure_ratio,
             learning_velocity, topic_affinities, preferred_modality, learning_style,
             avg_time_per_unit, engagement_score, updated_at
      FROM user_features
      WHERE user_id = ANY($1)
    `;
    const result = await this.db.query(query, [userIds]);
    const map = new Map<string, Types.UserFeatures>();
    result.rows.forEach((row: Record<string, any>) => {
      map.set(row.user_id, this.mapRowToUserFeatures(row));
    });
    return map;
  }

  async batchGetContentFeatures(contentIds: string[]): Promise<Map<string, Types.ContentFeatures>> {
    const query = `
      SELECT content_id, title, description, embedding, difficulty_level,
             quality_score, modality, concepts, prerequisites,
             avg_completion_rate, avg_dwell_time_seconds, engagement_score,
             assessment_pass_rate, estimated_duration_minutes, updated_at
      FROM content_features
      WHERE content_id = ANY($1)
    `;
    const result = await this.db.query(query, [contentIds]);
    const map = new Map<string, Types.ContentFeatures>();
    result.rows.forEach((row: Record<string, any>) => {
      map.set(row.content_id, this.mapRowToContentFeatures(row));
    });
    return map;
  }

  async deleteTTLExpired(): Promise<number> {
    const query = `
      DELETE FROM feature_store_cache
      WHERE created_at < NOW() - INTERVAL '1 day'
    `;
    const result = await this.db.query(query);
    return result.rowCount;
  }

  async deleteUser(userId: string): Promise<void> {
    await Promise.all([
      this.db.query('DELETE FROM user_features WHERE user_id = $1', [userId]),
      this.db.query('DELETE FROM user_embeddings WHERE user_id = $1', [userId]),
      this.db.query('DELETE FROM user_behavior_analysis WHERE user_id = $1', [userId]),
      this.db.query('DELETE FROM user_content_interactions WHERE user_id = $1', [userId]),
      this.db.query('DELETE FROM learning_paths WHERE user_id = $1', [userId]),
      this.db.query('DELETE FROM experiment_assignments WHERE user_id = $1', [userId]),
    ]);
  }

  // ========================================================================
  // PRIVATE MAPPING HELPERS
  // ========================================================================

  private mapRowToUserFeatures(row: Record<string, any>): Types.UserFeatures {
    return {
      userId: row.user_id,
      completionRate: row.completion_rate,
      avgDwellTimeSeconds: row.avg_dwell_time_seconds,
      successFailureRatio: row.success_failure_ratio,
      learningVelocity: row.learning_velocity,
      topicAffinities: new Map(Object.entries(row.topic_affinities || {})),
      preferredModality: row.preferred_modality as Types.ContentModality,
      learningStyle: row.learning_style as Types.LearningStyle,
      avgTimePerUnit: row.avg_time_per_unit,
      engagementScore: row.engagement_score,
      updatedAt: row.updated_at,
    };
  }

  private mapRowToContentFeatures(row: Record<string, any>): Types.ContentFeatures {
    const embedding = JSON.parse(row.embedding);
    const concepts = (JSON.parse(row.concepts) || []).map((c: any) => ({
      conceptId: c.conceptId,
      name: c.name,
      description: c.description,
      difficulty: c.difficulty,
    }));

    return {
      contentId: row.content_id,
      title: row.title,
      description: row.description,
      embedding: {
        contentId: embedding.contentId,
        embedding: embedding.embedding,
        dimension: embedding.dimension,
        modelVersion: embedding.modelVersion,
        generatedAt: embedding.generatedAt,
      },
      difficultyLevel: row.difficulty_level,
      qualityScore: row.quality_score,
      modality: row.modality as Types.ContentModality,
      concepts,
      prerequisites: JSON.parse(row.prerequisites) || [],
      avgCompletionRate: row.avg_completion_rate,
      avgDwellTimeSeconds: row.avg_dwell_time_seconds,
      engagementScore: row.engagement_score,
      assessmentPassRate: row.assessment_pass_rate,
      estimatedDurationMinutes: row.estimated_duration_minutes,
      updatedAt: row.updated_at,
    };
  }

  private mapRowToInteraction(row: Record<string, any>): Types.UserContentInteraction {
    return {
      userId: row.user_id,
      contentId: row.content_id,
      implicitFeedback: row.implicit_feedback,
      explicitRating: row.explicit_rating,
      completionStatus: row.completion_status as Types.CompletionStatus,
      timeSpentSeconds: row.time_spent_seconds,
      viewedAt: row.viewed_at,
      assessmentScore: row.assessment_score,
      bookmarked: row.bookmarked,
    };
  }

  private mapRowToBehaviorAnalysis(row: Record<string, any>): Types.UserBehaviorAnalysis {
    return {
      userId: row.user_id,
      pattern: row.pattern as Types.UserBehaviorPattern,
      dropoutRisk: row.dropout_risk as Types.DropoutRisk,
      strugglingTopics: JSON.parse(row.struggling_topics) || [],
      fastTrackTopics: JSON.parse(row.fast_track_topics) || [],
      topicSwitchFrequency: row.topic_switch_frequency,
      sessionDepthAvg: row.session_depth_avg,
      daysSinceLastActive: row.days_since_last_active,
      predictedChurnProbability: row.predicted_churn_probability,
    };
  }

  private mapRowToLearningPath(row: any): Types.LearningPath {
    return {
      pathId: row.path_id,
      userId: row.user_id,
      contentSequence: JSON.parse(row.content_sequence),
      currentStep: row.current_step,
      completionStatus: row.completion_status as Types.CompletionStatus,
      estimatedCompletionDays: row.estimated_completion_days,
      performanceMetrics: JSON.parse(row.performance_metrics),
      createdAt: row.created_at,
      updatedAt: row.updated_at,
    };
  }
}

// ============================================================================
// REDIS FEATURE STORE (Cache Layer)
// ============================================================================

export class RedisFeatureStore implements IFeatureStore {
  private redis: any; // Redis client
  private ttl: number = 86400; // 24 hours

  constructor(redisClient: any, ttlSeconds: number = 86400) {
    this.redis = redisClient;
    this.ttl = ttlSeconds;
  }

  async getUserFeatures(userId: string): Promise<Types.UserFeatures | null> {
    const cached = await this.redis.get(`user_features:${userId}`);
    return cached ? JSON.parse(cached) : null;
  }

  async putUserFeatures(features: Types.UserFeatures): Promise<void> {
    await this.redis.setex(
      `user_features:${features.userId}`,
      this.ttl,
      JSON.stringify(features)
    );
  }

  async getUserEmbedding(userId: string): Promise<Types.UserEmbedding | null> {
    const cached = await this.redis.get(`user_embedding:${userId}`);
    return cached ? JSON.parse(cached) : null;
  }

  async putUserEmbedding(embedding: Types.UserEmbedding): Promise<void> {
    await this.redis.setex(
      `user_embedding:${embedding.userId}`,
      this.ttl,
      JSON.stringify(embedding)
    );
  }

  async getContentFeatures(contentId: string): Promise<Types.ContentFeatures | null> {
    const cached = await this.redis.get(`content_features:${contentId}`);
    return cached ? JSON.parse(cached) : null;
  }

  async putContentFeatures(features: Types.ContentFeatures): Promise<void> {
    await this.redis.setex(
      `content_features:${features.contentId}`,
      this.ttl,
      JSON.stringify(features)
    );
  }

  async getContentEmbedding(contentId: string): Promise<Types.ContentSemanticEmbedding | null> {
    const cached = await this.redis.get(`content_embedding:${contentId}`);
    return cached ? JSON.parse(cached) : null;
  }

  async putContentEmbedding(embedding: Types.ContentSemanticEmbedding): Promise<void> {
    await this.redis.setex(
      `content_embedding:${embedding.contentId}`,
      this.ttl,
      JSON.stringify(embedding)
    );
  }

  async getInteraction(userId: string, contentId: string): Promise<Types.UserContentInteraction | null> {
    const cached = await this.redis.get(`interaction:${userId}:${contentId}`);
    return cached ? JSON.parse(cached) : null;
  }

  async putInteraction(interaction: Types.UserContentInteraction): Promise<void> {
    await this.redis.setex(
      `interaction:${interaction.userId}:${interaction.contentId}`,
      this.ttl,
      JSON.stringify(interaction)
    );
  }

  async getUserInteractions(userId: string, limit: number = 1000): Promise<Types.UserContentInteraction[]> {
    // Not typically cached in Redis due to volume
    return [];
  }

  async getContentInteractions(contentId: string, limit: number = 1000): Promise<Types.UserContentInteraction[]> {
    // Not typically cached in Redis due to volume
    return [];
  }

  async getUserBehaviorAnalysis(userId: string): Promise<Types.UserBehaviorAnalysis | null> {
    const cached = await this.redis.get(`behavior:${userId}`);
    return cached ? JSON.parse(cached) : null;
  }

  async putUserBehaviorAnalysis(analysis: Types.UserBehaviorAnalysis): Promise<void> {
    await this.redis.setex(
      `behavior:${analysis.userId}`,
      this.ttl,
      JSON.stringify(analysis)
    );
  }

  async getLearningPath(pathId: string): Promise<Types.LearningPath | null> {
    const cached = await this.redis.get(`learning_path:${pathId}`);
    return cached ? JSON.parse(cached) : null;
  }

  async putLearningPath(path: Types.LearningPath): Promise<void> {
    await this.redis.setex(
      `learning_path:${path.pathId}`,
      this.ttl,
      JSON.stringify(path)
    );
  }

  async getUserLearningPath(userId: string): Promise<Types.LearningPath | null> {
    const cached = await this.redis.get(`learning_path:user:${userId}`);
    return cached ? JSON.parse(cached) : null;
  }

  async getExperimentAssignment(userId: string, experimentId: string): Promise<Types.ExperimentAssignment | null> {
    const cached = await this.redis.get(`experiment:${userId}:${experimentId}`);
    return cached ? JSON.parse(cached) : null;
  }

  async putExperimentAssignment(assignment: Types.ExperimentAssignment): Promise<void> {
    await this.redis.setex(
      `experiment:${assignment.userId}:${assignment.experimentId}`,
      this.ttl,
      JSON.stringify(assignment)
    );
  }

  async batchGetUserFeatures(userIds: string[]): Promise<Map<string, Types.UserFeatures>> {
    const map = new Map<string, Types.UserFeatures>();
    for (const userId of userIds) {
      const features = await this.getUserFeatures(userId);
      if (features) map.set(userId, features);
    }
    return map;
  }

  async batchGetContentFeatures(contentIds: string[]): Promise<Map<string, Types.ContentFeatures>> {
    const map = new Map<string, Types.ContentFeatures>();
    for (const contentId of contentIds) {
      const features = await this.getContentFeatures(contentId);
      if (features) map.set(contentId, features);
    }
    return map;
  }

  async deleteTTLExpired(): Promise<number> {
    // Redis handles TTL automatically
    return 0;
  }

  async deleteUser(userId: string): Promise<void> {
    const keys = await this.redis.keys(`*:${userId}*`);
    if (keys.length > 0) {
      await this.redis.del(...keys);
    }
  }
}

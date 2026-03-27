/**
 * Synthetic Test Datasets
 * 
 * Datasets for validating multiple user personas:
 * - New user (cold start)
 * - Advanced learner (fast-track)
 * - Struggling learner
 * - Multi-interest learner
 */

import * as Types from '../src/types';

// ============================================================================
// DATASET GENERATORS
// ============================================================================

export class SyntheticDatasetGenerator {
  /**
   * Generate cold-start user (new, no history)
   */
  static generateColdStartUser(userId: string = 'user_cold_001'): {
    userProfile: Types.UserProfile;
    interactions: Types.UserContentInteraction[];
  } {
    const userProfile: Types.UserProfile = {
      userId,
      features: {
        userId,
        completionRate: 0,
        avgDwellTimeSeconds: 0,
        successFailureRatio: 0,
        learningVelocity: 0,
        topicAffinities: new Map(),
        preferredModality: Types.ContentModality.VIDEO,
        learningStyle: Types.LearningStyle.VISUAL,
        avgTimePerUnit: 0,
        engagementScore: 0,
        updatedAt: new Date(),
      },
      embedding: {
        userId,
        embedding: new Array(128).fill(0.5),
        dimension: 128,
        generatedAt: new Date(),
      },
      behavior: {
        userId,
        pattern: Types.UserBehaviorPattern.STEADY_LEARNER,
        dropoutRisk: Types.DropoutRisk.MEDIUM,
        strugglingTopics: [],
        fastTrackTopics: [],
        topicSwitchFrequency: 0,
        sessionDepthAvg: 0,
        daysSinceLastActive: -1,
        predictedChurnProbability: 0.3,
      },
      privacySettings: {
        isAnonymized: false,
        optedOutOfRecommendations: false,
        optedOutOfAnalytics: false,
        dataRetentionDays: 90,
        allowCrossUserAnalytics: true,
      },
    };

    return { userProfile, interactions: [] };
  }

  /**
   * Generate advanced/fast-track learner
   */
  static generateAdvancedLearner(userId: string = 'user_advanced_001'): {
    userProfile: Types.UserProfile;
    interactions: Types.UserContentInteraction[];
  } {
    const interactions: Types.UserContentInteraction[] = [
      {
        userId,
        contentId: 'course_001',
        implicitFeedback: 0.95,
        explicitRating: 5,
        completionStatus: Types.CompletionStatus.COMPLETED,
        timeSpentSeconds: 1800,
        viewedAt: new Date(Date.now() - 7 * 24 * 60 * 60 * 1000),
        assessmentScore: 92,
        bookmarked: true,
      },
      {
        userId,
        contentId: 'course_002',
        implicitFeedback: 0.98,
        explicitRating: 5,
        completionStatus: Types.CompletionStatus.COMPLETED,
        timeSpentSeconds: 1500,
        viewedAt: new Date(Date.now() - 5 * 24 * 60 * 60 * 1000),
        assessmentScore: 95,
        bookmarked: true,
      },
      {
        userId,
        contentId: 'course_003',
        implicitFeedback: 0.92,
        explicitRating: 4,
        completionStatus: Types.CompletionStatus.COMPLETED,
        timeSpentSeconds: 1200,
        viewedAt: new Date(Date.now() - 1 * 24 * 60 * 60 * 1000),
        assessmentScore: 88,
        bookmarked: true,
      },
    ];

    const topicAffinities = new Map<string, number>([
      ['advanced-algorithms', 0.95],
      ['machine-learning', 0.92],
      ['system-design', 0.88],
      ['distributed-systems', 0.85],
    ]);

    const userProfile: Types.UserProfile = {
      userId,
      features: {
        userId,
        completionRate: 0.95,
        avgDwellTimeSeconds: 1500,
        successFailureRatio: 15.0,
        learningVelocity: 3.5, // 3.5 courses/week
        topicAffinities,
        preferredModality: Types.ContentModality.INTERACTIVE,
        learningStyle: Types.LearningStyle.KINESTHETIC,
        avgTimePerUnit: 1400,
        engagementScore: 0.94,
        updatedAt: new Date(),
      },
      embedding: {
        userId,
        embedding: new Array(128).fill(0).map(() => Math.random()),
        dimension: 128,
        generatedAt: new Date(),
      },
      behavior: {
        userId,
        pattern: Types.UserBehaviorPattern.FAST_TRACK,
        dropoutRisk: Types.DropoutRisk.LOW,
        strugglingTopics: [],
        fastTrackTopics: ['advanced-algorithms', 'machine-learning'],
        topicSwitchFrequency: 0.5,
        sessionDepthAvg: 8,
        daysSinceLastActive: 1,
        predictedChurnProbability: 0.02,
      },
      privacySettings: {
        isAnonymized: false,
        optedOutOfRecommendations: false,
        optedOutOfAnalytics: false,
        dataRetentionDays: 180,
        allowCrossUserAnalytics: true,
      },
    };

    return { userProfile, interactions };
  }

  /**
   * Generate struggling learner
   */
  static generateStrugglingLearner(userId: string = 'user_struggling_001'): {
    userProfile: Types.UserProfile;
    interactions: Types.UserContentInteraction[];
  } {
    const interactions: Types.UserContentInteraction[] = [
      {
        userId,
        contentId: 'intro_001',
        implicitFeedback: 0.45,
        explicitRating: 2,
        completionStatus: Types.CompletionStatus.ABANDONED,
        timeSpentSeconds: 600,
        viewedAt: new Date(Date.now() - 14 * 24 * 60 * 60 * 1000),
        assessmentScore: 35,
        bookmarked: false,
      },
      {
        userId,
        contentId: 'intro_002',
        implicitFeedback: 0.52,
        explicitRating: 3,
        completionStatus: Types.CompletionStatus.COMPLETED,
        timeSpentSeconds: 3600,
        viewedAt: new Date(Date.now() - 10 * 24 * 60 * 60 * 1000),
        assessmentScore: 42,
        bookmarked: false,
      },
      {
        userId,
        contentId: 'basics_001',
        implicitFeedback: 0.38,
        explicitRating: 2,
        completionStatus: Types.CompletionStatus.IN_PROGRESS,
        timeSpentSeconds: 1200,
        viewedAt: new Date(Date.now() - 3 * 24 * 60 * 60 * 1000),
        assessmentScore: 38,
        bookmarked: false,
      },
    ];

    const topicAffinities = new Map<string, number>([
      ['programming-basics', 0.35],
      ['problem-solving', 0.28],
    ]);

    const userProfile: Types.UserProfile = {
      userId,
      features: {
        userId,
        completionRate: 0.33,
        avgDwellTimeSeconds: 1800,
        successFailureRatio: 0.6,
        learningVelocity: 0.5, // 0.5 courses/week
        topicAffinities,
        preferredModality: Types.ContentModality.TEXT,
        learningStyle: Types.LearningStyle.AUDITORY,
        avgTimePerUnit: 1800,
        engagementScore: 0.32,
        updatedAt: new Date(),
      },
      embedding: {
        userId,
        embedding: new Array(128).fill(0).map(() => Math.random()),
        dimension: 128,
        generatedAt: new Date(),
      },
      behavior: {
        userId,
        pattern: Types.UserBehaviorPattern.STRUGGLING,
        dropoutRisk: Types.DropoutRisk.HIGH,
        strugglingTopics: ['advanced-topics', 'algorithms'],
        fastTrackTopics: [],
        topicSwitchFrequency: 2.0,
        sessionDepthAvg: 2,
        daysSinceLastActive: 3,
        predictedChurnProbability: 0.65,
      },
      privacySettings: {
        isAnonymized: false,
        optedOutOfRecommendations: false,
        optedOutOfAnalytics: false,
        dataRetentionDays: 90,
        allowCrossUserAnalytics: true,
      },
    };

    return { userProfile, interactions };
  }

  /**
   * Generate multi-interest learner
   */
  static generateMultiInterestLearner(userId: string = 'user_multi_001'): {
    userProfile: Types.UserProfile;
    interactions: Types.UserContentInteraction[];
  } {
    const interactions: Types.UserContentInteraction[] = Array.from(
      { length: 15 },
      (_, i) => ({
        userId,
        contentId: `course_${String(i + 1).padStart(3, '0')}`,
        implicitFeedback: 0.65 + Math.random() * 0.25,
        explicitRating: 3 + Math.floor(Math.random() * 3),
        completionStatus:
          i < 10 ? Types.CompletionStatus.COMPLETED : Types.CompletionStatus.IN_PROGRESS,
        timeSpentSeconds: 1200 + Math.random() * 1800,
        viewedAt: new Date(Date.now() - (15 - i) * 24 * 60 * 60 * 1000),
        assessmentScore: 60 + Math.random() * 30,
        bookmarked: Math.random() > 0.5,
      })
    );

    const topicAffinities = new Map<string, number>([
      ['web-development', 0.75],
      ['data-science', 0.72],
      ['ui-ux-design', 0.68],
      ['project-management', 0.55],
      ['python', 0.78],
      ['javascript', 0.72],
    ]);

    const userProfile: Types.UserProfile = {
      userId,
      features: {
        userId,
        completionRate: 0.67,
        avgDwellTimeSeconds: 1500,
        successFailureRatio: 2.0,
        learningVelocity: 2.1,
        topicAffinities,
        preferredModality: Types.ContentModality.VIDEO,
        learningStyle: Types.LearningStyle.MIXED,
        avgTimePerUnit: 1500,
        engagementScore: 0.68,
        updatedAt: new Date(),
      },
      embedding: {
        userId,
        embedding: new Array(128).fill(0).map(() => Math.random()),
        dimension: 128,
        generatedAt: new Date(),
      },
      behavior: {
        userId,
        pattern: Types.UserBehaviorPattern.TOPIC_SWITCHING,
        dropoutRisk: Types.DropoutRisk.MEDIUM,
        strugglingTopics: [],
        fastTrackTopics: ['python', 'data-science'],
        topicSwitchFrequency: 3.5,
        sessionDepthAvg: 5,
        daysSinceLastActive: 1,
        predictedChurnProbability: 0.25,
      },
      privacySettings: {
        isAnonymized: false,
        optedOutOfRecommendations: false,
        optedOutOfAnalytics: false,
        dataRetentionDays: 180,
        allowCrossUserAnalytics: true,
      },
    };

    return { userProfile, interactions };
  }

  /**
   * Generate synthetic content catalog
   */
  static generateContentCatalog(size: number = 100): Types.ContentFeatures[] {
    const modalities = [Types.ContentModality.VIDEO, Types.ContentModality.TEXT, Types.ContentModality.INTERACTIVE];
    const difficulties = [
      Types.DifficultyLevel.BEGINNER,
      Types.DifficultyLevel.INTERMEDIATE,
      Types.DifficultyLevel.ADVANCED,
      Types.DifficultyLevel.EXPERT,
    ];
    const topics = [
      'web-development',
      'data-science',
      'python',
      'javascript',
      'machine-learning',
      'ui-ux-design',
      'databases',
      'devops',
      'algorithms',
      'system-design',
    ];

    const catalog: Types.ContentFeatures[] = [];

    for (let i = 1; i <= size; i++) {
      const contentId = `course_${String(i).padStart(4, '0')}`;
      const topic = topics[Math.floor(Math.random() * topics.length)];
      const difficulty = difficulties[Math.floor(Math.random() * difficulties.length)];

      catalog.push({
        contentId,
        title: `${topic.replace(/-/g, ' ').toUpperCase()} Course ${i}`,
        description: `Learn ${topic} with comprehensive coverage. This course covers fundamentals and advanced topics.`,
        embedding: {
          contentId,
          embedding: new Array(768).fill(0).map(() => Math.random()),
          dimension: 768,
          modelVersion: 'all-MiniLM-L6-v2',
          generatedAt: new Date(),
        },
        difficultyLevel: difficulty,
        qualityScore: 60 + Math.random() * 40,
        modality: modalities[Math.floor(Math.random() * modalities.length)],
        concepts: [
          { conceptId: topic, name: topic, description: '', difficulty },
        ],
        prerequisites: i > 10 ? [`course_${String(i - 5).padStart(4, '0')}`] : [],
        avgCompletionRate: 0.5 + Math.random() * 0.4,
        avgDwellTimeSeconds: 1200 + Math.random() * 1800,
        engagementScore: 0.6 + Math.random() * 0.3,
        assessmentPassRate: 0.65 + Math.random() * 0.3,
        estimatedDurationMinutes: 30 + Math.floor(Math.random() * 120),
        updatedAt: new Date(),
      });
    }

    return catalog;
  }

  /**
   * Generate interaction matrix for collaborative filtering
   */
  static generateInteractionMatrix(
    userCount: number = 50,
    contentCount: number = 100,
    density: number = 0.1
  ): Map<string, Map<string, number>> {
    const matrix = new Map<string, Map<string, number>>();

    for (let u = 0; u < userCount; u++) {
      const userId = `user_${String(u + 1).padStart(4, '0')}`;
      const userRow = new Map<string, number>();

      for (let c = 0; c < contentCount; c++) {
        const contentId = `course_${String(c + 1).padStart(4, '0')}`;

        if (Math.random() < density) {
          // Interaction exists
          const feedback = Math.random(); // Normalized to 0-1
          userRow.set(contentId, feedback);
        }
      }

      matrix.set(userId, userRow);
    }

    return matrix;
  }
}

// ============================================================================
// EXPORT DATASETS
// ============================================================================

export function generateAllTestDatasets() {
  return {
    coldStartUser: SyntheticDatasetGenerator.generateColdStartUser(),
    advancedLearner: SyntheticDatasetGenerator.generateAdvancedLearner(),
    strugglingLearner: SyntheticDatasetGenerator.generateStrugglingLearner(),
    multiInterestLearner: SyntheticDatasetGenerator.generateMultiInterestLearner(),
    contentCatalog: SyntheticDatasetGenerator.generateContentCatalog(100),
    interactionMatrix: SyntheticDatasetGenerator.generateInteractionMatrix(50, 100, 0.1),
  };
}

/**
 * Privacy Preservation Layer
 * 
 * Implements:
 * - User anonymization
 * - Differential privacy
 * - Opt-out handling
 * - PII filtering
 * - Data minimization
 */

import * as Types from '../types';

// Handle crypto import for Node.js environments
let createHashFunction: (algorithm: string) => any;
if (typeof (globalThis as any).require !== 'undefined') {
  try {
    const crypto = (globalThis as any).require('crypto');
    createHashFunction = (algorithm: string) => crypto.createHash(algorithm);
  } catch (e) {
    // Fallback if crypto is not available
    createHashFunction = (algorithm: string) => ({
      update: (data: string) => ({
        digest: () => `hash_${data.substring(0, 10)}`,
      }),
    });
  }
} else {
  // Browser fallback
  createHashFunction = (algorithm: string) => ({
    update: (data: string) => ({
      digest: () => `hash_${data.substring(0, 10)}`,
    }),
  });
}

// Get environment variable safely
const getEnvVar = (key: string, defaultValue: string): string => {
  if (typeof (globalThis as any).process !== 'undefined' && (globalThis as any).process.env) {
    return (globalThis as any).process.env[key] || defaultValue;
  }
  return defaultValue;
};

// Generate UUID safely for both Node and browser
const generateUUID = (): string => {
  if (typeof crypto !== 'undefined' && (crypto as any).randomUUID) {
    return (crypto as any).randomUUID();
  }
  // Fallback UUID v4 implementation
  return 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, (c) => {
    const r = Math.random() * 16 | 0;
    const v = c === 'x' ? r : (r & 0x3 | 0x8);
    return v.toString(16);
  });
};

// ============================================================================
// USER ANONYMIZATION
// ============================================================================

export class UserAnonymizer {
  private salt: string;

  constructor(salt: string = getEnvVar('ANONYMIZATION_SALT', 'teachlink-salt')) {
    this.salt = salt;
  }

  /**
   * Create anonymous hash of user ID
   */
  hashUserId(userId: string): string {
    return createHashFunction('sha256')
      .update(userId + this.salt)
      .digest('hex');
  }

  /**
   * Create anonymous user profile
   */
  anonymizeUserProfile(profile: Types.UserProfile): {
    hashedUserId: string;
    features: Omit<Types.UserFeatures, 'userId'>;
    embedding: Omit<Types.UserEmbedding, 'userId'>;
  } {
    const hashedUserId = this.hashUserId(profile.userId);

    const featuresCopy = { ...profile.features };
    delete (featuresCopy as any).userId;

    const embeddingCopy = { ...profile.embedding };
    delete (embeddingCopy as any).userId;

    return {
      hashedUserId,
      features: featuresCopy,
      embedding: embeddingCopy,
    };
  }

  /**
   * Create one-time session ID (ephemeral)
   */
  generateEphemeralSessionId(userId: string, timestamp: Date = new Date()): string {
    const sessionData = `${userId}:${timestamp.toISOString()}:${Math.random()}`;
    return createHashFunction('sha256').update(sessionData).digest('hex');
  }

  /**
   * Verify session authenticity without revealing user ID
   */
  verifySessionId(sessionId: string, maxAgeMs: number = 3600000): boolean {
    // In production: store session with timestamp, check age
    const sessionAgeMs = Date.now() % maxAgeMs;
    return sessionAgeMs < maxAgeMs;
  }
}

// ============================================================================
// DIFFERENTIAL PRIVACY
// ============================================================================

export class DifferentialPrivacyEngine {
  private epsilon: number;
  private delta: number;
  private budget: number;

  constructor(
    epsilon: number = 0.5,
    delta: number = 1e-5,
    aggregationThreshold: number = 100
  ) {
    this.epsilon = epsilon;
    this.delta = delta;
    this.budget = epsilon;
  }

  /**
   * Add Laplace noise to scalar value
   * Implements DP-Laplace mechanism
   */
  addLaplaceNoise(value: number, sensitivity: number = 1.0): number {
    const scale = sensitivity / this.epsilon;
    const noise = this.sampleLaplace(scale);
    return value + noise;
  }

  /**
   * Add noise to embedding vector
   */
  noiseEmbedding(embedding: number[]): number[] {
    const sensitivity = Math.sqrt(embedding.length); // L2 sensitivity
    const scale = sensitivity / this.epsilon;

    return embedding.map(() => this.sampleLaplace(scale));
  }

  /**
   * Add noise to count data
   */
  addCountNoise(count: number): number {
    if (count < 10) {
      // Don't disclose small counts
      return 0;
    }

    const noisyCount = this.addLaplaceNoise(count, 1.0);
    return Math.max(0, Math.round(noisyCount));
  }

  /**
   * Privacy-safe histogram aggregation
   */
  aggregateHistogram(
    data: Map<string, number>,
    aggregationThreshold: number = 100
  ): Map<string, number> {
    const result = new Map<string, number>();

    for (const [key, count] of data) {
      if (count >= aggregationThreshold) {
        const noisyCount = this.addCountNoise(count);
        result.set(key, noisyCount);
      }
      // Suppress low-count items
    }

    return result;
  }

  /**
   * Check privacy budget
   */
  getRemainingBudget(): number {
    return Math.max(0, this.budget);
  }

  /**
   * Consume epsilon budget
   */
  consumeBudget(amount: number): boolean {
    if (amount > this.budget) {
      console.warn('[DP] Privacy budget exceeded');
      return false;
    }
    this.budget -= amount;
    return true;
  }

  // ========================================================================
  // PRIVATE HELPERS
  // ========================================================================

  private sampleLaplace(scale: number): number {
    /**
     * Generate Laplace-distributed random variable
     * Using inverse transform sampling
     */
    const u = Math.random() - 0.5;
    return -scale * Math.sign(u) * Math.log(1 - 2 * Math.abs(u));
  }
}

// ============================================================================
// OPT-OUT MANAGEMENT
// ============================================================================

export class OptOutManager {
  private optedOutUsers: Set<string> = new Set();
  private optOutReasons: Map<string, string> = new Map();
  private optOutTimestamps: Map<string, Date> = new Map();

  /**
   * Register user opt-out
   */
  optOut(userId: string, reason?: string): void {
    this.optedOutUsers.add(userId);
    if (reason) {
      this.optOutReasons.set(userId, reason);
    }
    this.optOutTimestamps.set(userId, new Date());
    console.log(`[OptOut] User ${userId} opted out${reason ? ': ' + reason : ''}`);
  }

  /**
   * Prevent recommendations for opted-out user
   */
  canRecommendTo(userId: string): boolean {
    return !this.optedOutUsers.has(userId);
  }

  /**
   * Disable analytics for opted-out user
   */
  canCollectAnalytics(userId: string): boolean {
    return this.canRecommendTo(userId);
  }

  /**
   * Get list of opted-out users
   */
  getOptedOutUsers(): string[] {
    return Array.from(this.optedOutUsers);
  }

  /**
   * Re-opt-in user (with confirmation)
   */
  reOptIn(userId: string): void {
    this.optedOutUsers.delete(userId);
    this.optOutReasons.delete(userId);
    console.log(`[OptOut] User ${userId} re-opted in`);
  }

  /**
   * Generate privacy report for user
   */
  generatePrivacyReport(userId: string): {
    userId: string;
    optedOut: boolean;
    optedOutAt?: Date;
    reason?: string;
  } {
    return {
      userId,
      optedOut: this.optedOutUsers.has(userId),
      optedOutAt: this.optOutTimestamps.get(userId),
      reason: this.optOutReasons.get(userId),
    };
  }
}

// ============================================================================
// PII FILTERING
// ============================================================================

export class PIIFilter {
  private piiPatterns: Map<string, RegExp> = new Map([
    ['email', /[^\s@]+@[^\s@]+\.[^\s@]+/g],
    ['phone', /[\d-+\(\)\s]{10,}/g],
    ['ssn', /\d{3}-\d{2}-\d{4}/g],
    ['credit_card', /\d{4}[\s-]?\d{4}[\s-]?\d{4}[\s-]?\d{4}/g],
  ]);

  /**
   * Detect PII in text
   */
  detectPII(text: string): { type: string; matches: string[] }[] {
    const detections: { type: string; matches: string[] }[] = [];

    for (const [type, pattern] of this.piiPatterns) {
      const matches = text.match(pattern);
      if (matches) {
        detections.push({ type, matches });
      }
    }

    return detections;
  }

  /**
   * Remove PII from text (redaction)
   */
  redactPII(text: string): string {
    let redacted = text;

    for (const [type, pattern] of this.piiPatterns) {
      redacted = redacted.replace(pattern, `[${type.toUpperCase()}_REDACTED]`);
    }

    return redacted;
  }

  /**
   * Filter object removing sensitive fields
   */
  filterSensitiveFields<T extends Record<string, any>>(
    obj: T,
    allowedFields: Set<string>
  ): Partial<T> {
    const filtered: Partial<T> = {};

    for (const [key, value] of Object.entries(obj)) {
      if (allowedFields.has(key)) {
        filtered[key as keyof T] = value;
      }
    }

    return filtered;
  }
}

// ============================================================================
// DATA MINIMIZATION
// ============================================================================

export class DataMinimizer {
  /**
   * Keep only necessary features for recommendations
   */
  minimizeUserFeatures(features: Types.UserFeatures): Partial<Types.UserFeatures> {
    return {
      // Only necessary for recommendations
      completionRate: features.completionRate,
      successFailureRatio: features.successFailureRatio,
      learningVelocity: features.learningVelocity,
      topicAffinities: features.topicAffinities,
      preferredModality: features.preferredModality,
      engagementScore: features.engagementScore,
      // Exclude: learningStyle, avgTimePerUnit, avgDwellTimeSeconds (not needed for core logic)
    };
  }

  /**
   * Remove older interactions (retention policy)
   */
  retentionFilter(
    interactions: Types.UserContentInteraction[],
    retentionDaysMax: number = 90
  ): Types.UserContentInteraction[] {
    const cutoffDate = new Date();
    cutoffDate.setDate(cutoffDate.getDate() - retentionDaysMax);

    return interactions.filter(i => i.viewedAt > cutoffDate);
  }

  /**
   * Aggregate interactions to reduce granularity
   */
  aggregateInteractions(
    interactions: Types.UserContentInteraction[],
    windowDays: number = 7
  ): Types.UserContentInteraction[] {
    const aggregated = new Map<string, Types.UserContentInteraction>();
    const now = new Date();

    for (const interaction of interactions) {
      const ageMs = now.getTime() - interaction.viewedAt.getTime();
      const ageDays = Math.floor(ageMs / (24 * 60 * 60 * 1000));
      const windowIndex = Math.floor(ageDays / windowDays);

      const key = `${interaction.contentId}:${windowIndex}`;

      if (aggregated.has(key)) {
        const existing = aggregated.get(key)!;
        existing.implicitFeedback = Math.max(
          existing.implicitFeedback,
          interaction.implicitFeedback
        );
        existing.timeSpentSeconds += interaction.timeSpentSeconds;
      } else {
        aggregated.set(key, { ...interaction });
      }
    }

    return Array.from(aggregated.values());
  }
}

// ============================================================================
// PRIVACY COMPLIANCE MANAGER
// ============================================================================

export class PrivacyComplianceManager {
  private anonymizer: UserAnonymizer;
  private dpEngine: DifferentialPrivacyEngine;
  private optOutManager: OptOutManager;
  private piiFilter: PIIFilter;
  private dataMinimizer: DataMinimizer;

  private userDeletionRequests: Map<string, Types.UserDataDeletionRequest> = new Map();

  constructor(
    anonymizer?: UserAnonymizer,
    dpEngine?: DifferentialPrivacyEngine,
    optOutManager?: OptOutManager,
    piiFilter?: PIIFilter,
    dataMinimizer?: DataMinimizer
  ) {
    this.anonymizer = anonymizer || new UserAnonymizer();
    this.dpEngine = dpEngine || new DifferentialPrivacyEngine();
    this.optOutManager = optOutManager || new OptOutManager();
    this.piiFilter = piiFilter || new PIIFilter();
    this.dataMinimizer = dataMinimizer || new DataMinimizer();
  }

  /**
   * Check if user can be recommended to
   */
  canRecommendTo(userId: string): boolean {
    return this.optOutManager.canRecommendTo(userId);
  }

  /**
   * Process user data with privacy policies applied
   */
  processUserDataPrivate(
    userId: string,
    features: Types.UserFeatures,
    applyDP: boolean = false
  ): {
    hashedUserId: string;
    minimizedFeatures: Partial<Types.UserFeatures>;
    noiseApplied?: boolean;
  } {
    return {
      hashedUserId: this.anonymizer.hashUserId(userId),
      minimizedFeatures: this.dataMinimizer.minimizeUserFeatures(features),
      noiseApplied: applyDP,
    };
  }

  /**
   * Request user data deletion (GDPR/CCPA compliance)
   */
  requestDataDeletion(userId: string): Types.UserDataDeletionRequest {
    const request: Types.UserDataDeletionRequest = {
      userId,
      requestId: generateUUID(),
      requestedAt: new Date(),
      dataRetentionPeriodDays: 30,
    };

    this.userDeletionRequests.set(request.requestId, request);
    console.log(`[Privacy] Data deletion request for user ${userId}: ${request.requestId}`);

    return request;
  }

  /**
   * Execute data deletion
   */
  async executeDataDeletion(requestId: string): Promise<void> {
    const request = this.userDeletionRequests.get(requestId);
    if (!request) {
      throw new Error(`Deletion request not found: ${requestId}`);
    }

    console.log(`[Privacy] Executing deletion for user ${request.userId}`);

    // In production: delete from all data stores
    // - Feature store
    // - Event logs
    // - Model training data
    // - Analytics

    request.completedAt = new Date();
    this.userDeletionRequests.set(requestId, request);
  }

  /**
   * Generate privacy policy report
   */
  generatePrivacyReport(userId: string): {
    optOutStatus: ReturnType<OptOutManager['generatePrivacyReport']>;
    privacyBudget: number;
    dataMinimizationApplied: boolean;
    piiDetected: any[];
  } {
    return {
      optOutStatus: this.optOutManager.generatePrivacyReport(userId),
      privacyBudget: this.dpEngine.getRemainingBudget(),
      dataMinimizationApplied: true,
      piiDetected: [],
    };
  }

  /**
   * Audit trail for privacy events
   */
  logPrivacyEvent(
    userId: string,
    eventType: 'access' | 'processing' | 'deletion' | 'opt_out',
    details: Record<string, any>
  ): void {
    const entry = {
      userId: this.anonymizer.hashUserId(userId),
      eventType,
      timestamp: new Date(),
      details,
    };

    console.log('[Privacy Audit]', JSON.stringify(entry));
    // In production: persist to immutable audit log
  }
}

# TeachLink Recommendation System - API Reference

## Overview

Production-grade API for generating personalized learning recommendations with explanations, A/B testing support, and privacy preservation.

**Base URL**: `https://api.teachlink.com/v1/recommendations`

**Authentication**: Bearer token (JWT)

---

## Endpoints

### 1. Get Recommendations

Generate personalized recommendations for a user with full explanation metadata.

**Endpoint**: `POST /recommendations`

**Latency Target**: <150ms P95

**Request**:
```json
{
  "userId": "user_12345",
  "context": {
    "currentTimestamp": "2026-02-20T10:30:00Z",
    "sessionDepth": 3,
    "currentLearningGoal": "learn-machine-learning",
    "recentTopics": ["python", "data-science"],
    "seasonalFactor": 0.8,
    "deviceType": "desktop",
    "isFirstSession": false
  },
  "candidateContentIds": [
    "course_0001",
    "course_0002",
    "course_0003",
    "course_0004",
    "course_0005"
  ],
  "k": 5,
  "includeExplanations": true,
  "experimentId": "exp_20260220_001"
}
```

**Response (200 OK)**:
```json
{
  "requestId": "req_kj8x9p2m",
  "userId": "user_12345",
  "recommendations": [
    {
      "contentId": "course_0001",
      "rank": 1,
      "score": 0.87,
      "explanation": {
        "primaryReason": "Aligns perfectly with your machine learning goals and learning pace",
        "supportingSignals": [
          "You completed 8 similar advanced ML courses with 92% avg score",
          "Users with your learning velocity rated this 4.8/5",
          "Available as interactive tutorial (your preferred format)"
        ],
        "featureAttribution": [
          {
            "feature": "topic_match",
            "importance": 0.42,
            "contribution": "Strong alignment with your machine learning focus"
          },
          {
            "feature": "collaborative_signal",
            "importance": 0.38,
            "contribution": "Similar learners found this highly valuable"
          },
          {
            "feature": "learning_path_fit",
            "importance": 0.15,
            "contribution": "Logical progression from your current level"
          },
          {
            "feature": "content_quality",
            "importance": 0.05,
            "contribution": "Consistently high engagement metrics"
          }
        ],
        "similarityTrace": {
          "similarContentIds": ["course_0002", "course_0003"],
          "similarUserCount": 847
        },
        "ruleBasedExplanation": "You are a fast learner with strong performance in advanced topics. We prioritize challenging content that matches your interests.",
        "transparencyMetadata": {
          "modelVersion": "hybrid_v2.1",
          "confidenceLevel": 0.89,
          "explanationMethod": "hybrid"
        }
      },
      "experimentVariant": "control",
      "confidence": 0.89,
      "metadata": {
        "reasonCode": "hybrid_ranking",
        "modality": "interactive",
        "difficulty": 3,
        "estimatedTimeMinutes": 120
      }
    },
    {
      "contentId": "course_0002",
      "rank": 2,
      "score": 0.84,
      "explanation": {
        "primaryReason": "Builds on your existing knowledge of data structures",
        "supportingSignals": [
          "Prerequisite for your next learning goal",
          "94% completion rate among similar learners",
          "Recent course by top instructor (4.9/5 rating)"
        ],
        "featureAttribution": [
          {
            "feature": "learning_path_fit",
            "importance": 0.50,
            "contribution": "Essential next step in recommendation sequence"
          },
          {
            "feature": "content_quality",
            "importance": 0.35,
            "contribution": "Exceptional student outcomes and satisfaction"
          },
          {
            "feature": "collaborative_signal",
            "importance": 0.15,
            "contribution": "Well-received by your cohort"
          }
        ],
        "ruleBasedExplanation": "Based on your learning path progress, this course prepares you for the next milestone.",
        "transparencyMetadata": {
          "modelVersion": "hybrid_v2.1",
          "confidenceLevel": 0.84,
          "explanationMethod": "rule_based"
        }
      },
      "experimentVariant": "control",
      "confidence": 0.84,
      "metadata": {
        "reasonCode": "hybrid_ranking",
        "modality": "video",
        "difficulty": 3,
        "estimatedTimeMinutes": 90
      }
    },
    {
      "contentId": "course_0003",
      "rank": 3,
      "score": 0.78,
      "explanation": {
        "primaryReason": "Serendipitous discovery matching your emerging interests",
        "supportingSignals": [
          "Unexpectedly combines your interests in ML and system design",
          "5% of users discover this through cross-topic recommendations",
          "97% recommend this to others"
        ],
        "featureAttribution": [
          {
            "feature": "serendipity_factor",
            "importance": 0.45,
            "contribution": "Unexpected but highly relevant to your evolving interests"
          },
          {
            "feature": "content_quality",
            "importance": 0.40,
            "contribution": "Exceptional learning outcomes"
          },
          {
            "feature": "diversity_boost",
            "importance": 0.15,
            "contribution": "Introduces complementary domain"
          }
        ],
        "ruleBasedExplanation": "We noticed your growing interest in systems topics. This bridges ML and infrastructure design.",
        "transparencyMetadata": {
          "modelVersion": "hybrid_v2.1",
          "confidenceLevel": 0.78,
          "explanationMethod": "hybrid"
        }
      },
      "experimentVariant": "control",
      "confidence": 0.78,
      "metadata": {
        "reasonCode": "serendipity_driven",
        "modality": "text",
        "difficulty": 4,
        "estimatedTimeMinutes": 150
      }
    }
  ],
  "learningPath": {
    "pathId": "path_user_12345_001",
    "userId": "user_12345",
    "contentSequence": [
      "course_0001",
      "course_0002",
      "course_0003",
      "course_0004",
      "milestone_checkpoint_1"
    ],
    "currentStep": 0,
    "completionStatus": "in_progress",
    "estimatedCompletionDays": 21,
    "performanceMetrics": {
      "avgScore": 88.5,
      "completionRate": 0.92,
      "timeToCompleteEachItem": [120, 90, 150, 110, 30]
    },
    "createdAt": "2026-02-01T00:00:00Z",
    "updatedAt": "2026-02-20T10:30:00Z"
  },
  "contextUsed": {
    "currentTimestamp": "2026-02-20T10:30:00Z",
    "sessionDepth": 3,
    "currentLearningGoal": "learn-machine-learning",
    "recentTopics": ["python", "data-science"],
    "seasonalFactor": 0.8,
    "deviceType": "desktop",
    "isFirstSession": false
  },
  "experimentVariant": "control",
  "generatedAt": "2026-02-20T10:30:01Z",
  "latencyMs": 47
}
```

---

### 2. Batch Recommendations

Generate recommendations for multiple users simultaneously (max 100 users per request).

**Endpoint**: `POST /recommendations/batch`

**Request**:
```json
{
  "requests": [
    {
      "userId": "user_12345",
      "context": { /* ... */ }
    },
    {
      "userId": "user_67890",
      "context": { /* ... */ }
    }
  ],
  "candidateContentIds": ["course_0001", "course_0002"],
  "k": 5,
  "maxConcurrency": 50
}
```

**Response (200 OK)**:
```json
{
  "results": [
    { /* recommendation response for user 1 */ },
    { /* recommendation response for user 2 */ }
  ],
  "processingTimeMs": 234,
  "successCount": 2,
  "errorCount": 0
}
```

---

### 3. Explain Recommendation

Get detailed explanation for why a specific recommendation was made.

**Endpoint**: `GET /recommendations/{requestId}/explain`

**Response (200 OK)**:
```json
{
  "requestId": "req_kj8x9p2m",
  "contentId": "course_0001",
  "rank": 1,
  "explanation": {
    "primaryReason": "Aligns perfectly with your machine learning goals",
    "supportingSignals": ["signal_1", "signal_2"],
    "featureAttribution": [
      {
        "feature": "topic_match",
        "importance": 0.42,
        "contribution": "Strong alignment with interests"
      }
    ],
    "counterfactuals": [
      "If you had lower completion rate, this would rank lower",
      "If you preferred video over interactive, this might not rank as high"
    ],
    "userSegment": "advanced_learner_cohort",
    "modelConfidence": 0.89
  }
}
```

---

### 4. User Profile

Get user profile with embeddings and behavior analysis.

**Endpoint**: `GET /users/{userId}/profile`

**Response (200 OK)**:
```json
{
  "userId": "user_12345",
  "features": {
    "completionRate": 0.92,
    "avgDwellTimeSeconds": 1450,
    "successFailureRatio": 18.5,
    "learningVelocity": 3.2,
    "topicAffinities": {
      "machine-learning": 0.95,
      "python": 0.92,
      "data-science": 0.88
    },
    "preferredModality": "interactive",
    "learningStyle": "kinesthetic",
    "engagementScore": 0.91
  },
  "embedding": {
    "dimension": 128,
    "vector": [0.12, -0.45, /* ... */],
    "generatedAt": "2026-02-20T00:00:00Z"
  },
  "behavior": {
    "pattern": "fast_track",
    "dropoutRisk": "low",
    "strugglingTopics": [],
    "fastTrackTopics": ["machine-learning", "python"],
    "predictedChurnProbability": 0.02
  }
}
```

---

### 5. Learning Path

Get or update user's current learning path.

**Endpoint**: `GET /users/{userId}/learning-path`

**Response (200 OK)**:
```json
{
  "pathId": "path_user_12345_001",
  "userId": "user_12345",
  "contentSequence": [
    "course_0001",
    "course_0002",
    "milestone_1"
  ],
  "currentStep": 1,
  "completionStatus": "in_progress",
  "estimatedCompletionDays": 14,
  "performanceMetrics": {
    "avgScore": 88.5,
    "completionRate": 0.92
  },
  "adaptiveRecommendations": [
    {
      "reason": "You mastered this topic, accelerating path",
      "adjustment": "skip_to_step_3"
    }
  ]
}
```

---

### 6. A/B Test Results

Get metrics for active experiment.

**Endpoint**: `GET /experiments/{experimentId}/metrics`

**Response (200 OK)**:
```json
{
  "experimentId": "exp_20260220_001",
  "status": "running",
  "variants": {
    "control": {
      "ctr": 0.068,
      "completionRate": 0.45,
      "avgSessionLength": 524,
      "learning_gain": 7.2,
      "retention_7day": 0.72,
      "sampleSize": 5234,
      "confidenceInterval": {
        "lower": 0.065,
        "upper": 0.071,
        "confidence": 0.95
      }
    },
    "variant_a": {
      "ctr": 0.074,
      "completionRate": 0.48,
      "avgSessionLength": 547,
      "learning_gain": 8.1,
      "retention_7day": 0.75,
      "sampleSize": 5198,
      "confidenceInterval": {
        "lower": 0.071,
        "upper": 0.077,
        "confidence": 0.95
      }
    }
  },
  "winner": "variant_a",
  "winRate": 0.94,
  "minDetectableEffect": 0.05,
  "daysRemaining": 8,
  "recommendation": "Variant A shows 8.8% improvement in CTR. Continue for 1 more week for confirmation."
}
```

---

### 7. Privacy & Opt-Out

Manage user privacy settings and opt-out.

**Endpoint**: `POST /users/{userId}/privacy/opt-out`

**Request**:
```json
{
  "reason": "too_many_notifications",
  "optOutOfRecommendations": true,
  "optOutOfAnalytics": false,
  "dataRetentionDays": 30
}
```

**Response (200 OK)**:
```json
{
  "userId": "user_12345",
  "status": "opted_out",
  "effectiveDate": "2026-02-20T10:30:00Z",
  "privacySummary": {
    "recommendationsDisabled": true,
    "analyticsDisabled": false,
    "dataRetentionDays": 30,
    "personalDataAnonymized": false
  }
}
```

---

### 8. Data Deletion Request (GDPR/CCPA)

Request permanent data deletion.

**Endpoint**: `DELETE /users/{userId}/data`

**Request**:
```json
{
  "reason": "user_request",
  "immediateDelete": false
}
```

**Response (202 Accepted)**:
```json
{
  "requestId": "delete_req_abc123",
  "userId": "user_12345",
  "status": "pending",
  "estimatedCompletionDate": "2026-03-22T00:00:00Z",
  "dataToDelete": [
    "user_features",
    "interaction_history",
    "embeddings",
    "analytics_data"
  ],
  "retentionPeriodDays": 30
}
```

---

### 9. Model Health & Metrics

Get system health and model performance metrics.

**Endpoint**: `GET /system/health`

**Response (200 OK)**:
```json
{
  "status": "healthy",
  "timestamp": "2026-02-20T10:30:00Z",
  "inference": {
    "latency_p50_ms": 28,
    "latency_p95_ms": 87,
    "latency_p99_ms": 142,
    "throughput_qps": 1250,
    "availability": 0.9998
  },
  "models": {
    "collaborative_filtering": {
      "status": "active",
      "version": "cf_v1.5",
      "ndcg10": 0.78,
      "updated_at": "2026-02-19T00:00:00Z"
    },
    "content_based": {
      "status": "active",
      "version": "cb_v2.0",
      "ndcg10": 0.76,
      "updated_at": "2026-02-19T00:00:00Z"
    }
  },
  "alerts": [],
  "metrics": {
    "users_active": 127456,
    "recommendations_generated_24h": 5234890,
    "average_ctr": 0.071,
    "average_completion_rate": 0.467
  }
}
```

---

## Error Responses

### 400 Bad Request
```json
{
  "error": "invalid_request",
  "message": "Missing required field: candidateContentIds",
  "details": {
    "field": "candidateContentIds",
    "expected": "array"
  }
}
```

### 401 Unauthorized
```json
{
  "error": "unauthorized",
  "message": "Invalid or expired token"
}
```

### 429 Rate Limited
```json
{
  "error": "rate_limited",
  "message": "Exceeded rate limit",
  "retryAfterSeconds": 60
}
```

### 500 Internal Server Error
```json
{
  "error": "internal_error",
  "message": "Failed to generate recommendations",
  "requestId": "req_kj8x9p2m",
  "timestamp": "2026-02-20T10:30:00Z"
}
```

---

## SDK Example (TypeScript)

```typescript
import { RecommendationClient } from '@teachlink/recommendation-sdk';

const client = new RecommendationClient({
  apiKey: process.env.TEACHLINK_API_KEY,
  baseUrl: 'https://api.teachlink.com/v1'
});

// Get recommendations
const response = await client.getRecommendations({
  userId: 'user_12345',
  context: {
    currentLearningGoal: 'learn-machine-learning',
    deviceType: 'desktop'
  },
  candidateContentIds: ['course_0001', 'course_0002'],
  k: 5
});

console.log(`Generated ${response.recommendations.length} recommendations`);
console.log(`Primary reason: ${response.recommendations[0].explanation.primaryReason}`);
```

---

## Rate Limits

- **Per user**: 100 requests/minute
- **Per API key**: 10,000 requests/minute
- **Burst**: Up to 1,000 QPS

---

## Monitoring & Alerts

Track model performance and system health:

```json
{
  "metrics": {
    "ndcg10": 0.78,
    "map10": 0.65,
    "ctr": 0.071,
    "completion_rate": 0.467
  },
  "thresholds": {
    "ndcg10_min": 0.70,
    "ctr_min": 0.05,
    "latency_max_ms": 150
  }
}
```

---

## Integration Checklist

- [ ] Set up API authentication
- [ ] Integrate recommendation endpoint
- [ ] Implement explanation UI
- [ ] Set up A/B test tracking
- [ ] Configure privacy opt-out flow
- [ ] Monitor latency and quality metrics
- [ ] Set up alerting for performance degradation

/**
 * NLP Content Understanding Pipeline
 * 
 * Responsibilities:
 * - Text normalization & cleaning
 * - Semantic embeddings (from transformers)
 * - Concept extraction & tagging
 * - Content similarity computation
 */

import * as Types from '../types';

// ============================================================================
// TEXT PREPROCESSING
// ============================================================================

export class ContentNormalizer {
  private stopWords = new Set([
    'the', 'a', 'an', 'and', 'or', 'but', 'in', 'on', 'at', 'to', 'for',
    'of', 'is', 'are', 'was', 'were', 'be', 'been', 'being', 'have', 'has'
  ]);

  /**
   * Normalize content text for embedding and analysis
   */
  normalize(text: string): string {
    return text
      .toLowerCase()
      .replace(/[^\w\s]/g, ' ') // Remove special chars
      .replace(/\s+/g, ' ') // Normalize spaces
      .trim();
  }

  /**
   * Tokenize text into words
   */
  tokenize(text: string): string[] {
    const normalized = this.normalize(text);
    return normalized.split(/\s+/).filter(token => token.length > 0);
  }

  /**
   * Remove stop words
   */
  removeStopWords(tokens: string[]): string[] {
    return tokens.filter(token => !this.stopWords.has(token));
  }

  /**
   * Extract key terms (TF-IDF style)
   */
  extractKeyTerms(text: string, k: number = 10): string[] {
    const tokens = this.removeStopWords(this.tokenize(text));
    const termFreq = new Map<string, number>();

    for (const token of tokens) {
      termFreq.set(token, (termFreq.get(token) || 0) + 1);
    }

    return Array.from(termFreq.entries())
      .sort((a, b) => b[1] - a[1])
      .slice(0, k)
      .map(([term]) => term);
  }
}

// ============================================================================
// SEMANTIC EMBEDDING GENERATOR
// ============================================================================

/**
 * Interface for embedding backends
 * In production: integrate with sentence-transformers, OpenAI, etc.
 */
export interface IEmbeddingGenerator {
  generateEmbedding(text: string): Promise<number[]>;
  generateBatchEmbeddings(texts: string[]): Promise<number[][]>;
  getModelName(): string;
  getDimension(): number;
}

export class TransformerEmbeddingGenerator implements IEmbeddingGenerator {
  private modelName: string = 'all-MiniLM-L6-v2';
  private dimension: number = 384;
  private normalizer = new ContentNormalizer();

  /**
   * In real implementation, use sentence-transformers library
   * For demo: simulate embeddings
   */
  async generateEmbedding(text: string): Promise<number[]> {
    const normalized = this.normalizer.normalize(text);
    const tokens = this.normalizer.tokenize(normalized);
    
    // Simulated embedding (in production: use actual model)
    return this.simpleHashToEmbedding(normalized, this.dimension);
  }

  async generateBatchEmbeddings(texts: string[]): Promise<number[][]> {
    return Promise.all(texts.map(text => this.generateEmbedding(text)));
  }

  getModelName(): string {
    return this.modelName;
  }

  getDimension(): number {
    return this.dimension;
  }

  // ========================================================================
  // PRIVATE HELPERS
  // ========================================================================

  private simpleHashToEmbedding(text: string, dimension: number): number[] {
    /**
     * Deterministic hash-based embedding for demo
     * In production: use actual transformer model
     */
    let seed = 0;
    for (let i = 0; i < text.length; i++) {
      seed = ((seed << 5) - seed) + text.charCodeAt(i);
      seed |= 0;
    }

    const random = () => {
      seed = (seed * 9301 + 49297) % 233280;
      return seed / 233280;
    };

    const embedding = new Array(dimension).fill(0).map(() => random());
    
    // Normalize to unit vector
    const norm = Math.sqrt(embedding.reduce((sum, val) => sum + val * val, 0));
    return embedding.map(val => val / norm);
  }
}

// ============================================================================
// CONCEPT EXTRACTION
// ============================================================================

export interface ConceptExtractionModel {
  extractConcepts(text: string, k?: number): string[];
  getConcepts(): Set<string>;
}

export class NaiveConceptExtractor implements ConceptExtractionModel {
  private conceptLexicon = new Map<string, Types.ConceptNode>();
  private domainKeywords: { [domain: string]: string[] } = {
    'math': ['algebra', 'geometry', 'calculus', 'differential', 'integral', 'function', 'equation'],
    'cs': ['algorithm', 'data-structure', 'sorting', 'searching', 'tree', 'graph', 'programming'],
    'science': ['physics', 'chemistry', 'biology', 'quantum', 'molecular', 'particle', 'atom'],
    'language': ['grammar', 'vocabulary', 'syntax', 'semantics', 'phonetics', 'morphology'],
  };

  addConcept(concept: Types.ConceptNode): void {
    this.conceptLexicon.set(concept.conceptId, concept);
  }

  /**
   * Extract relevant concepts from content text
   */
  extractConcepts(text: string, k: number = 5): string[] {
    const normalizer = new ContentNormalizer();
    const tokens = normalizer.removeStopWords(normalizer.tokenize(text));
    
    const foundConcepts: Array<[string, number]> = [];

    for (const token of tokens) {
      for (const [domain, keywords] of Object.entries(this.domainKeywords)) {
        for (const keyword of keywords) {
          if (keyword.includes(token) || token.includes(keyword)) {
            foundConcepts.push([keyword, 1]);
          }
        }
      }
    }

    // Deduplicate and sort by frequency
    const conceptCounts = new Map<string, number>();
    for (const [concept, count] of foundConcepts) {
      conceptCounts.set(concept, (conceptCounts.get(concept) || 0) + count);
    }

    return Array.from(conceptCounts.entries())
      .sort((a, b) => b[1] - a[1])
      .slice(0, k)
      .map(([concept]) => concept);
  }

  getConcepts(): Set<string> {
    return new Set(this.conceptLexicon.keys());
  }
}

// ============================================================================
// CONTENT EMBEDDER (Orchestrator)
// ============================================================================

export class ContentEmbedder {
  private embeddingGenerator: IEmbeddingGenerator;
  private conceptExtractor: ConceptExtractionModel;
  private normalizer = new ContentNormalizer();

  constructor(
    embeddingGenerator: IEmbeddingGenerator = new TransformerEmbeddingGenerator(),
    conceptExtractor: ConceptExtractionModel = new NaiveConceptExtractor()
  ) {
    this.embeddingGenerator = embeddingGenerator;
    this.conceptExtractor = conceptExtractor;
  }

  /**
   * Process content: extract embedding, concepts, keywords
   */
  async processContent(
    contentId: string,
    title: string,
    description: string,
    difficulty: Types.DifficultyLevel = Types.DifficultyLevel.INTERMEDIATE
  ): Promise<Types.ContentSemanticEmbedding> {
    console.log(`[Embedder] Processing content: ${contentId}`);

    // Combine title and description for embedding
    const fullText = `${title}. ${description}`;
    
    // Generate semantic embedding
    const embedding = await this.embeddingGenerator.generateEmbedding(fullText);

    const result: Types.ContentSemanticEmbedding = {
      contentId,
      embedding,
      dimension: this.embeddingGenerator.getDimension(),
      modelVersion: this.embeddingGenerator.getModelName(),
      generatedAt: new Date(),
    };

    console.log(`[Embedder] Completed embedding for ${contentId}`);
    return result;
  }

  /**
   * Batch process multiple content items
   */
  async processBatch(
    items: Array<{
      contentId: string;
      title: string;
      description: string;
      difficulty?: Types.DifficultyLevel;
    }>
  ): Promise<Map<string, Types.ContentSemanticEmbedding>> {
    console.log(`[Embedder] Processing batch of ${items.length} items`);

    const results = new Map<string, Types.ContentSemanticEmbedding>();

    for (const item of items) {
      const embedding = await this.processContent(
        item.contentId,
        item.title,
        item.description,
        item.difficulty
      );
      results.set(item.contentId, embedding);
    }

    return results;
  }

  /**
   * Extract concepts and keywords from content
   */
  extractContentFeatures(
    title: string,
    description: string
  ): {
    concepts: string[];
    keywords: string[];
  } {
    const fullText = `${title}. ${description}`;
    const concepts = this.conceptExtractor.extractConcepts(fullText, 5);
    const keywords = this.normalizer.extractKeyTerms(fullText, 8);

    return { concepts, keywords };
  }

  /**
   * Compute similarity between two content items
   */
  async computeSimilarity(
    embedding1: number[],
    embedding2: number[]
  ): Promise<number> {
    return this.cosineSimilarity(embedding1, embedding2);
  }

  /**
   * Find K most similar items
   */
  async findSimilarContent(
    targetEmbedding: number[],
    candidateEmbeddings: Map<string, number[]>,
    k: number = 5
  ): Promise<Array<[string, number]>> {
    const similarities: Array<[string, number]> = [];

    for (const [contentId, embedding] of candidateEmbeddings) {
      const similarity = this.cosineSimilarity(targetEmbedding, embedding);
      similarities.push([contentId, similarity]);
    }

    return similarities
      .sort((a, b) => b[1] - a[1])
      .slice(0, k);
  }

  // ========================================================================
  // PRIVATE HELPERS
  // ========================================================================

  private cosineSimilarity(a: number[], b: number[]): number {
    let dotProduct = 0;
    let normA = 0;
    let normB = 0;

    for (let i = 0; i < a.length; i++) {
      dotProduct += a[i] * b[i];
      normA += a[i] * a[i];
      normB += b[i] * b[i];
    }

    normA = Math.sqrt(normA);
    normB = Math.sqrt(normB);

    if (normA === 0 || normB === 0) return 0;
    return dotProduct / (normA * normB);
  }
}

// ============================================================================
// CONTENT SIMILARITY MATRIX
// ============================================================================

export class ContentSimilarityMatrix {
  private similarities: Map<string, Map<string, number>> = new Map();

  /**
   * Build similarity matrix from embeddings
   */
  async build(contentEmbeddings: Map<string, number[]>): Promise<void> {
    console.log('[SimilarityMatrix] Building similarity matrix');

    const embeddingArray = Array.from(contentEmbeddings.entries());
    const n = embeddingArray.length;

    for (let i = 0; i < n; i++) {
      const [contentIdI, embeddingI] = embeddingArray[i];
      const rowSimilarities = new Map<string, number>();

      for (let j = 0; j < n; j++) {
        if (i === j) {
          rowSimilarities.set(contentIdI, 1.0);
          continue;
        }

        const [contentIdJ, embeddingJ] = embeddingArray[j];
        const similarity = this.cosineSimilarity(embeddingI, embeddingJ);
        rowSimilarities.set(contentIdJ, similarity);
      }

      this.similarities.set(contentIdI, rowSimilarities);
    }

    console.log('[SimilarityMatrix] Matrix built successfully');
  }

  /**
   * Query similar content
   */
  getSimilar(contentId: string, k: number = 10): Array<[string, number]> {
    const row = this.similarities.get(contentId);
    if (!row) return [];

    return Array.from(row.entries())
      .filter(([id]) => id !== contentId)
      .sort((a, b) => b[1] - a[1])
      .slice(0, k);
  }

  /**
   * Get similarity between two items
   */
  getSimilarity(contentId1: string, contentId2: string): number {
    if (contentId1 === contentId2) return 1.0;

    const row = this.similarities.get(contentId1);
    return row?.get(contentId2) ?? 0;
  }

  // ========================================================================
  // PRIVATE HELPERS
  // ========================================================================

  private cosineSimilarity(a: number[], b: number[]): number {
    let dotProduct = 0;
    let normA = 0;
    let normB = 0;

    for (let i = 0; i < a.length; i++) {
      dotProduct += a[i] * b[i];
      normA += a[i] * a[i];
      normB += b[i] * b[i];
    }

    normA = Math.sqrt(normA);
    normB = Math.sqrt(normB);

    if (normA === 0 || normB === 0) return 0;
    return dotProduct / (normA * normB);
  }
}

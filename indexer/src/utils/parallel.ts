/**
 * Parallel processing utilities with concurrency control.
 *
 * Provides batched parallel execution to avoid overwhelming external services
 * or database connections while still achieving significant speedup over
 * sequential processing.
 */

export interface ParallelBatchOptions {
  /** Maximum number of concurrent tasks (default: 5) */
  concurrency?: number;
  /** Whether to continue processing remaining items if one fails (default: false) */
  continueOnError?: boolean;
}

export interface ParallelBatchResult<T> {
  results: T[];
  errors: Array<{ index: number; error: Error }>;
  /** Total wall-clock duration in milliseconds */
  durationMs: number;
}

/**
 * Execute an array of tasks in parallel batches with concurrency control.
 *
 * Unlike `Promise.all` which runs everything at once, this limits the number
 * of concurrent inflight operations to avoid resource exhaustion.
 *
 * @param items - Items to process
 * @param fn - Async function to apply to each item
 * @param options - Concurrency and error-handling options
 * @returns Results in the same order as the input items
 *
 * @example
 * ```ts
 * const results = await parallelBatch(
 *   ledgerNumbers,
 *   (ledger) => fetchLedger(ledger),
 *   { concurrency: 10 },
 * );
 * ```
 */
export async function parallelBatch<T, R>(
  items: T[],
  fn: (item: T, index: number) => Promise<R>,
  options: ParallelBatchOptions = {},
): Promise<ParallelBatchResult<R>> {
  const { concurrency = 5, continueOnError = false } = options;
  const start = Date.now();
  const results: R[] = new Array(items.length);
  const errors: Array<{ index: number; error: Error }> = [];

  // Process in batches of `concurrency` size
  for (let i = 0; i < items.length; i += concurrency) {
    const batch = items.slice(i, i + concurrency);
    const batchPromises = batch.map(async (item, batchIdx) => {
      const globalIdx = i + batchIdx;
      try {
        results[globalIdx] = await fn(item, globalIdx);
      } catch (err) {
        const error = err instanceof Error ? err : new Error(String(err));
        errors.push({ index: globalIdx, error });
        if (!continueOnError) {
          throw error;
        }
      }
    });

    if (continueOnError) {
      await Promise.allSettled(batchPromises);
    } else {
      await Promise.all(batchPromises);
    }
  }

  return {
    results,
    errors,
    durationMs: Date.now() - start,
  };
}

/**
 * Run multiple independent async operations in parallel and return all results.
 *
 * This is a typed convenience wrapper around `Promise.all` that also captures
 * timing information for performance measurement.
 *
 * @example
 * ```ts
 * const { results, durationMs } = await parallelAll(
 *   () => checkDatabase(),
 *   () => checkHorizon(),
 *   () => checkIndexerState(),
 * );
 * const [dbStatus, horizonStatus, indexerStatus] = results;
 * ```
 */
export async function parallelAll<T extends readonly (() => Promise<any>)[]>(
  ...fns: T
): Promise<{
  results: { [K in keyof T]: Awaited<ReturnType<T[K]>> };
  durationMs: number;
}> {
  const start = Date.now();
  const results = (await Promise.all(fns.map((fn) => fn()))) as {
    [K in keyof T]: Awaited<ReturnType<T[K]>>;
  };
  return { results, durationMs: Date.now() - start };
}

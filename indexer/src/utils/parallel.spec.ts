import { parallelBatch, parallelAll } from './parallel';

describe('parallelBatch', () => {
  it('should process all items and return results in order', async () => {
    const items = [1, 2, 3, 4, 5];
    const result = await parallelBatch(
      items,
      async (n) => n * 2,
      { concurrency: 2 },
    );

    expect(result.results).toEqual([2, 4, 6, 8, 10]);
    expect(result.errors).toEqual([]);
    expect(result.durationMs).toBeGreaterThanOrEqual(0);
  });

  it('should respect concurrency limit', async () => {
    let maxConcurrent = 0;
    let currentConcurrent = 0;

    const items = Array.from({ length: 10 }, (_, i) => i);
    await parallelBatch(
      items,
      async () => {
        currentConcurrent++;
        maxConcurrent = Math.max(maxConcurrent, currentConcurrent);
        await new Promise((r) => setTimeout(r, 50));
        currentConcurrent--;
      },
      { concurrency: 3 },
    );

    expect(maxConcurrent).toBeLessThanOrEqual(3);
  });

  it('should stop on error when continueOnError is false', async () => {
    const processed: number[] = [];
    const items = [1, 2, 3, 4, 5];

    await expect(
      parallelBatch(
        items,
        async (n) => {
          if (n === 3) throw new Error('fail');
          processed.push(n);
          return n;
        },
        { concurrency: 1, continueOnError: false },
      ),
    ).rejects.toThrow('fail');
  });

  it('should continue on error when continueOnError is true', async () => {
    const items = [1, 2, 3, 4, 5];
    const result = await parallelBatch(
      items,
      async (n) => {
        if (n === 3) throw new Error('fail');
        return n * 2;
      },
      { concurrency: 2, continueOnError: true },
    );

    expect(result.errors).toHaveLength(1);
    expect(result.errors[0].index).toBe(2);
    expect(result.results[0]).toBe(2);
    expect(result.results[1]).toBe(4);
    expect(result.results[3]).toBe(8);
    expect(result.results[4]).toBe(10);
  });

  it('should handle empty input', async () => {
    const result = await parallelBatch([], async () => 1, { concurrency: 5 });
    expect(result.results).toEqual([]);
    expect(result.errors).toEqual([]);
  });

  it('should achieve speedup over sequential execution', async () => {
    const items = Array.from({ length: 6 }, (_, i) => i);

    // Sequential timing estimate: 6 * 50ms = 300ms
    // Parallel (concurrency 3): 2 batches * 50ms = ~100ms
    const result = await parallelBatch(
      items,
      async () => {
        await new Promise((r) => setTimeout(r, 50));
        return true;
      },
      { concurrency: 3 },
    );

    // Should be significantly faster than sequential (~300ms)
    expect(result.durationMs).toBeLessThan(250);
    expect(result.results).toHaveLength(6);
  });
});

describe('parallelAll', () => {
  it('should run independent operations in parallel', async () => {
    const { results, durationMs } = await parallelAll(
      () => Promise.resolve('a'),
      () => Promise.resolve(42),
      () => Promise.resolve(true),
    );

    expect(results).toEqual(['a', 42, true]);
    expect(durationMs).toBeGreaterThanOrEqual(0);
  });

  it('should achieve speedup over sequential execution', async () => {
    const delay = (ms: number) => new Promise((r) => setTimeout(r, ms));

    const { durationMs } = await parallelAll(
      () => delay(50).then(() => 'a'),
      () => delay(50).then(() => 'b'),
      () => delay(50).then(() => 'c'),
    );

    // Sequential would be ~150ms, parallel should be ~50ms
    expect(durationMs).toBeLessThan(120);
  });

  it('should propagate errors', async () => {
    await expect(
      parallelAll(
        () => Promise.resolve(1),
        () => Promise.reject(new Error('fail')),
      ),
    ).rejects.toThrow('fail');
  });
});

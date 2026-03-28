import { AsyncLocalStorage } from 'async_hooks';

export const asyncLocalStorage = new AsyncLocalStorage<Map<string, any>>();

export function runWithCorrelationId<T>(correlationId: string, callback: () => T): T {
  return asyncLocalStorage.run(new Map([['correlationId', correlationId]]), callback);
}

export function getCorrelationId(): string | undefined {
  const store = asyncLocalStorage.getStore();
  return store ? store.get('correlationId') : undefined;
}
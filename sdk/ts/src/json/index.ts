/**
 * JSON utilities.
 */

import { InfraError, Result, ok, err } from '../errors.js';

/**
 * Safely parse JSON.
 */
export function parseJson<T = unknown>(json: string): Result<T> {
  try {
    return ok(JSON.parse(json) as T);
  } catch (e) {
    return err(InfraError.validation(`Invalid JSON: ${e}`));
  }
}

/**
 * Safely stringify to JSON.
 */
export function stringifyJson(value: unknown, pretty = false): Result<string> {
  try {
    const result = pretty
      ? JSON.stringify(value, null, 2)
      : JSON.stringify(value);
    return ok(result);
  } catch (e) {
    return err(InfraError.validation(`Failed to stringify: ${e}`));
  }
}

/**
 * Get a value at a path in a JSON object.
 */
export function getPath(obj: unknown, path: string): unknown {
  const parts = path.split('.');
  let current: unknown = obj;

  for (const part of parts) {
    if (current === null || current === undefined) {
      return undefined;
    }

    if (typeof current === 'object') {
      current = (current as Record<string, unknown>)[part];
    } else {
      return undefined;
    }
  }

  return current;
}

/**
 * Set a value at a path in a JSON object.
 */
export function setPath(obj: Record<string, unknown>, path: string, value: unknown): void {
  const parts = path.split('.');
  let current: Record<string, unknown> = obj;

  for (let i = 0; i < parts.length - 1; i++) {
    const part = parts[i];
    if (!(part in current) || typeof current[part] !== 'object') {
      current[part] = {};
    }
    current = current[part] as Record<string, unknown>;
  }

  current[parts[parts.length - 1]] = value;
}

/**
 * Deep clone an object.
 */
export function deepClone<T>(obj: T): T {
  return JSON.parse(JSON.stringify(obj));
}

/**
 * Deep merge objects.
 */
export function deepMerge<T extends Record<string, unknown>>(
  target: T,
  ...sources: Partial<T>[]
): T {
  const result = { ...target };

  for (const source of sources) {
    for (const key in source) {
      const sourceValue = source[key];
      const targetValue = result[key];

      if (
        sourceValue !== null &&
        typeof sourceValue === 'object' &&
        !Array.isArray(sourceValue) &&
        targetValue !== null &&
        typeof targetValue === 'object' &&
        !Array.isArray(targetValue)
      ) {
        result[key] = deepMerge(
          targetValue as Record<string, unknown>,
          sourceValue as Record<string, unknown>
        ) as T[Extract<keyof T, string>];
      } else if (sourceValue !== undefined) {
        result[key] = sourceValue as T[Extract<keyof T, string>];
      }
    }
  }

  return result;
}

/**
 * Compare two JSON values for equality.
 */
export function jsonEquals(a: unknown, b: unknown): boolean {
  if (a === b) return true;
  if (a === null || b === null) return false;
  if (typeof a !== typeof b) return false;

  if (Array.isArray(a) && Array.isArray(b)) {
    if (a.length !== b.length) return false;
    return a.every((val, i) => jsonEquals(val, b[i]));
  }

  if (typeof a === 'object' && typeof b === 'object') {
    const aKeys = Object.keys(a);
    const bKeys = Object.keys(b);

    if (aKeys.length !== bKeys.length) return false;

    return aKeys.every((key) =>
      jsonEquals(
        (a as Record<string, unknown>)[key],
        (b as Record<string, unknown>)[key]
      )
    );
  }

  return false;
}

/**
 * Compute the difference between two JSON objects.
 */
export function jsonDiff(
  before: Record<string, unknown>,
  after: Record<string, unknown>
): Record<string, { before?: unknown; after?: unknown }> {
  const diff: Record<string, { before?: unknown; after?: unknown }> = {};
  const allKeys = new Set([...Object.keys(before), ...Object.keys(after)]);

  for (const key of allKeys) {
    const beforeValue = before[key];
    const afterValue = after[key];

    if (!jsonEquals(beforeValue, afterValue)) {
      diff[key] = { before: beforeValue, after: afterValue };
    }
  }

  return diff;
}

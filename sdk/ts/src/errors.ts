/**
 * Error types for the infrastructure SDK.
 */

/**
 * Error context for detailed error information.
 */
export interface ErrorContext {
  /** Source file where the error occurred */
  sourceFile?: string;
  /** Line number */
  line?: number;
  /** Additional context data */
  data?: Record<string, unknown>;
}

/**
 * Base error class for infrastructure errors.
 */
export class InfraError extends Error {
  /** Error code */
  readonly code: string;
  /** Error context */
  readonly context?: ErrorContext;
  /** Whether this error is retryable */
  readonly retryable: boolean;

  constructor(
    code: string,
    message: string,
    options?: {
      context?: ErrorContext;
      retryable?: boolean;
      cause?: Error;
    }
  ) {
    super(message, { cause: options?.cause });
    this.name = 'InfraError';
    this.code = code;
    this.context = options?.context;
    this.retryable = options?.retryable ?? false;
  }

  /**
   * Create a configuration error.
   */
  static config(message: string, key?: string): InfraError {
    return new InfraError('CONFIG_ERROR', message, {
      context: key ? { data: { key } } : undefined,
    });
  }

  /**
   * Create a crypto error.
   */
  static crypto(message: string, operation?: string): InfraError {
    return new InfraError('CRYPTO_ERROR', message, {
      context: operation ? { data: { operation } } : undefined,
    });
  }

  /**
   * Create a validation error.
   */
  static validation(message: string, field?: string): InfraError {
    return new InfraError('VALIDATION_ERROR', message, {
      context: field ? { data: { field } } : undefined,
    });
  }

  /**
   * Create a not found error.
   */
  static notFound(resource: string, id?: string): InfraError {
    const message = id ? `${resource} not found: ${id}` : `${resource} not found`;
    return new InfraError('NOT_FOUND', message, {
      context: { data: { resource, id } },
    });
  }

  /**
   * Create an external service error.
   */
  static external(service: string, message: string, retryable = true): InfraError {
    return new InfraError('EXTERNAL_ERROR', message, {
      context: { data: { service } },
      retryable,
    });
  }
}

/**
 * Result type for operations that can fail.
 */
export type Result<T, E = InfraError> =
  | { ok: true; value: T }
  | { ok: false; error: E };

/**
 * Create a successful result.
 */
export function ok<T>(value: T): Result<T, never> {
  return { ok: true, value };
}

/**
 * Create a failed result.
 */
export function err<E>(error: E): Result<never, E> {
  return { ok: false, error };
}

/**
 * Unwrap a result, throwing if it's an error.
 */
export function unwrap<T>(result: Result<T>): T {
  if (result.ok) {
    return result.value;
  }
  throw result.error;
}

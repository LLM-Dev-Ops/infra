/**
 * ID generation utilities.
 */

/**
 * ID type options.
 */
export type IdType = 'uuid-v4' | 'uuid-v7' | 'ulid' | 'nanoid' | 'snowflake';

/**
 * ID generator interface.
 */
export interface IdGenerator {
  generate(): string;
}

/**
 * Generate a UUID v4 (random).
 */
export function uuidV4(): string {
  return crypto.randomUUID();
}

/**
 * Generate a UUID v7 (time-ordered).
 */
export function uuidV7(): string {
  const now = Date.now();
  const bytes = new Uint8Array(16);

  // Fill with random bytes
  crypto.getRandomValues(bytes);

  // Set timestamp (first 48 bits)
  bytes[0] = (now / 0x10000000000) & 0xff;
  bytes[1] = (now / 0x100000000) & 0xff;
  bytes[2] = (now / 0x1000000) & 0xff;
  bytes[3] = (now / 0x10000) & 0xff;
  bytes[4] = (now / 0x100) & 0xff;
  bytes[5] = now & 0xff;

  // Set version (7) and variant (10)
  bytes[6] = (bytes[6] & 0x0f) | 0x70;
  bytes[8] = (bytes[8] & 0x3f) | 0x80;

  return formatUuid(bytes);
}

/**
 * Format bytes as UUID string.
 */
function formatUuid(bytes: Uint8Array): string {
  const hex = Array.from(bytes)
    .map((b) => b.toString(16).padStart(2, '0'))
    .join('');

  return `${hex.slice(0, 8)}-${hex.slice(8, 12)}-${hex.slice(12, 16)}-${hex.slice(16, 20)}-${hex.slice(20)}`;
}

/**
 * ULID alphabet (Crockford's Base32).
 */
const ULID_ALPHABET = '0123456789ABCDEFGHJKMNPQRSTVWXYZ';

/**
 * Generate a ULID (Universally Unique Lexicographically Sortable Identifier).
 */
export function ulid(): string {
  const now = Date.now();
  let result = '';

  // Encode timestamp (10 characters)
  let timestamp = now;
  for (let i = 9; i >= 0; i--) {
    result = ULID_ALPHABET[timestamp % 32] + result;
    timestamp = Math.floor(timestamp / 32);
  }

  // Encode random part (16 characters)
  const randomBytes = new Uint8Array(10);
  crypto.getRandomValues(randomBytes);

  for (let i = 0; i < 16; i++) {
    const byteIndex = Math.floor(i * 0.625);
    const bitOffset = (i * 5) % 8;
    let value: number;

    if (bitOffset <= 3) {
      value = (randomBytes[byteIndex] >> (3 - bitOffset)) & 0x1f;
    } else {
      value =
        ((randomBytes[byteIndex] << (bitOffset - 3)) |
          (randomBytes[byteIndex + 1] >> (11 - bitOffset))) &
        0x1f;
    }

    result += ULID_ALPHABET[value];
  }

  return result;
}

/**
 * NanoID default alphabet.
 */
const NANOID_ALPHABET =
  'useandom-26T198340PX75pxJACKVERYMINDBUSHWOLF_GQZbfghjklqvwyzrict';

/**
 * Generate a NanoID.
 */
export function nanoid(size = 21, alphabet = NANOID_ALPHABET): string {
  const bytes = new Uint8Array(size);
  crypto.getRandomValues(bytes);

  let result = '';
  const mask = (2 << (Math.log2(alphabet.length - 1) | 0)) - 1;
  const step = Math.ceil((1.6 * mask * size) / alphabet.length);

  let i = 0;
  while (result.length < size) {
    const randomBytes = new Uint8Array(step);
    crypto.getRandomValues(randomBytes);

    for (const byte of randomBytes) {
      const index = byte & mask;
      if (index < alphabet.length) {
        result += alphabet[index];
        if (result.length === size) break;
      }
    }
  }

  return result;
}

/**
 * Snowflake ID generator.
 */
export class SnowflakeGenerator implements IdGenerator {
  private sequence = 0;
  private lastTimestamp = -1;
  private readonly machineId: number;
  private readonly epoch: number;

  constructor(machineId = 0, epoch = 1609459200000) {
    this.machineId = machineId & 0x3ff; // 10 bits
    this.epoch = epoch;
  }

  generate(): string {
    let timestamp = Date.now() - this.epoch;

    if (timestamp === this.lastTimestamp) {
      this.sequence = (this.sequence + 1) & 0xfff; // 12 bits
      if (this.sequence === 0) {
        // Wait for next millisecond
        while (Date.now() - this.epoch <= this.lastTimestamp) {
          // spin
        }
        timestamp = Date.now() - this.epoch;
      }
    } else {
      this.sequence = 0;
    }

    this.lastTimestamp = timestamp;

    // 41 bits timestamp + 10 bits machine ID + 12 bits sequence
    const id =
      BigInt(timestamp) << 22n |
      BigInt(this.machineId) << 12n |
      BigInt(this.sequence);

    return id.toString();
  }
}

/**
 * Generate an ID of the specified type.
 */
export function generateId(type: IdType = 'uuid-v4'): string {
  switch (type) {
    case 'uuid-v4':
      return uuidV4();
    case 'uuid-v7':
      return uuidV7();
    case 'ulid':
      return ulid();
    case 'nanoid':
      return nanoid();
    case 'snowflake':
      return new SnowflakeGenerator().generate();
    default:
      throw new Error(`Unknown ID type: ${type}`);
  }
}

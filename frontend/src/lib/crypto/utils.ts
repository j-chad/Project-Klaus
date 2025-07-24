export const NONCE_BYTES = 12; // AES-GCM nonce size is 12 bytes

/** Combines a nonce and a symmetric key into a single Uint8Array.
 *
 * This is useful for hybrid encryption schemes where the nonce needs to be
 * included with the key for decryption.
 *
 * @param nonce - A cryptographic nonce of size NONCE_BYTES.
 * @param key - An ArrayBuffer representing the symmetric key.
 */
export function combineNonceAndKey(nonce: Uint8Array, key: ArrayBuffer): Uint8Array {
	const combined = new Uint8Array(NONCE_BYTES + key.byteLength);
	combined.set(nonce, 0);
	combined.set(new Uint8Array(key), NONCE_BYTES);
	return combined;
}

/** Splits a combined ArrayBuffer into its nonce and key components.
 *
 * This is the inverse of `combineNonceAndKey`, allowing you to retrieve
 * the original nonce and key from a combined ArrayBuffer.
 *
 * @param combined - The combined ArrayBuffer containing the nonce and key.
 */
export function splitNonceAndKey(combined: ArrayBuffer): { nonce: ArrayBuffer; key: ArrayBuffer } {
	const nonce = combined.slice(0, NONCE_BYTES);
	const key = combined.slice(NONCE_BYTES);
	return { nonce, key };
}

/**
 * Returns a cryptographically secure random integer between min and max, inclusive.
 *
 * Adapted from https://stackoverflow.com/questions/18230217/
 *
 * @param {number} min - the lowest integer in the desired range (inclusive)
 * @param {number} max - the highest integer in the desired range (inclusive)
 * @returns {number} Random number
 */
export function secureRandomInt(min: number, max: number): number {
	const RAND_MAX_RANGE_SIZE = 2 ** 53;
	const buffer = new Uint32Array(2 ** 10);

	if (!(Number.isSafeInteger(min) && Number.isSafeInteger(max))) {
		throw Error('min and max must be safe integers');
	}

	if (min > max) {
		throw Error('min must be less than or equal to max');
	}

	const rangeSize = max - min + 1;
	if (rangeSize > RAND_MAX_RANGE_SIZE) {
		throw Error('(max - min) must be <= Number.MAX_SAFE_INTEGER');
	}

	const rejectionThreshold = RAND_MAX_RANGE_SIZE - (RAND_MAX_RANGE_SIZE % rangeSize);
	let offset = buffer.length;
	let result: number;
	do {
		if (offset + 1 >= buffer.length) {
			crypto.getRandomValues(buffer);
			offset = 0;
		}

		const n1 = buffer[offset++];
		const n2 = buffer[offset++];
		if (n1 === undefined || n2 === undefined) {
			throw Error('failed to generate random numbers');
		}

		result = (n1 & 0x1f_ffff) * 0x1_0000_0000 + n2;
	} while (result >= rejectionThreshold);
	return min + (result % rangeSize);
}

/**
 * Securely permutes the order of an array using the Fisher-Yates shuffle algorithm and a CSRNG.
 *
 * Shuffling is done in place, and the input array is modified directly.
 *
 * @template T - The type of the elements in the array.
 * @param {T[]} array - An array of elements to be shuffled.
 * @return {T[]} The same array with its elements shuffled in place.
 */
export function securePermute<T>(array: T[]): T[] {
	// Fisher-Yates shuffle algorithm
	const n = array.length;
	for (let i = n - 1; i > 0; i--) {
		const j = secureRandomInt(0, i);
		[array[i], array[j]] = [array[j], array[i]] as [T, T];
	}

	return array;
}

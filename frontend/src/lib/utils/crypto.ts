const RSA_KEY_PARAMS: RsaHashedKeyGenParams = {
	name: 'RSA-OAEP',
	modulusLength: 4096,
	publicExponent: new Uint8Array([0x01, 0x00, 0x01]), // 65537,
	hash: 'SHA-512'
};

export async function generatePublicationKeypair(): Promise<CryptoKeyPair> {
	return crypto.subtle.generateKey(RSA_KEY_PARAMS, false, []);
}

export async function encryptPublicationMessage(
	message: string,
	publicKeys: CryptoKey[]
): Promise<ArrayBuffer> {
	const permutedKeys = securePermutePublicKeys(publicKeys);

	let messageBuffer = new TextEncoder().encode(message).buffer as ArrayBuffer; // technically is an ArrayBufferLike but will be replaced immediately with an ArrayBuffer
	for (const key of permutedKeys) {
		messageBuffer = await crypto.subtle.encrypt({ name: 'RSA-OAEP' }, key, messageBuffer);
	}

	return messageBuffer;
}

export async function decryptPublicationMessagesRound(
	messages: ArrayBuffer[],
	privateKey: CryptoKey
): Promise<ArrayBuffer[]> {
	const decryptedMessages: ArrayBuffer[] = [];

	for (const message of messages) {
		let decryptedBuffer: ArrayBuffer;
		try {
			decryptedBuffer = await crypto.subtle.decrypt({ name: 'RSA-OAEP' }, privateKey, message);
		} catch (error) {
			if (error instanceof DOMException && error.name === 'InvalidAccessError') {
				// incorrect key - skip this message
				continue;
			}

			throw error;
		}

		decryptedMessages.push(decryptedBuffer);
	}

	return decryptedMessages;
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

/** Generates a symmetric key for AES-GCM encryption.
 *
 * This is used for hybrid encryption, where the symmetric key is used to encrypt the actual message,
 */
export async function generateSymmetricKey(): Promise<CryptoKey> {
	return crypto.subtle.generateKey(
		{
			name: 'AES-GCM',
			length: 256
		},
		false,
		['encrypt', 'decrypt']
	);
}

/**
 * Securely permutes the order of public keys using the Fisher-Yates shuffle algorithm and a CSRNG.
 *
 * @param {CryptoKey[]} keys - An array of CryptoKey objects representing public keys.
 * @return {CryptoKey[]} A new array of CryptoKey objects with the order of keys shuffled.
 */
function securePermutePublicKeys(keys: CryptoKey[]): CryptoKey[] {
	// Fisher-Yates shuffle algorithm
	const result = keys.slice();

	const n = keys.length;
	for (let i = n - 1; i > 0; i--) {
		const j = secureRandomInt(0, i);
		[result[i], result[j]] = [result[j], result[i]] as [CryptoKey, CryptoKey];
	}

	return result;
}

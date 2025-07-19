const NONCE_BYTES = 12; // AES-GCM nonce size is 12 bytes

export async function generatePublicationKeypair(): Promise<CryptoKeyPair> {
	return crypto.subtle.generateKey(
		{
			name: 'RSA-OAEP',
			modulusLength: 4096,
			publicExponent: new Uint8Array([0x01, 0x00, 0x01]), // 65537,
			hash: 'SHA-512'
		} satisfies RsaHashedKeyGenParams,
		false,
		['encrypt', 'decrypt']
	);
}

export async function encryptPublicationMessage(
	message: string,
	publicKeys: CryptoKey[]
): Promise<Uint8Array> {
	const permutedKeys = securePermutePublicKeys(publicKeys);

	let messageBuffer: Uint8Array = new TextEncoder().encode(message);
	const nonceBuffer = new Uint8Array(12); // AES-GCM nonce size is 12 bytes / 96 bits

	for (const rsaKey of permutedKeys) {
		messageBuffer = await hybridEncrypt(messageBuffer, rsaKey, nonceBuffer);
	}

	return messageBuffer;
}

export async function decryptPublicationMessagesRound(
	messages: Uint8Array[],
	privateKey: CryptoKey
): Promise<ArrayBuffer[]> {
	const decryptionTasks = messages.map((encryptedMessage) =>
		hybridDecrypt(encryptedMessage, privateKey)
	);
	return (await Promise.all(decryptionTasks)).filter((result) => result !== null);
}

/** Generates a 256-bit symmetric key for AES-GCM encryption.
 *
 * This is used for hybrid encryption, where the symmetric key is used to encrypt the actual message,
 */
async function generateSymmetricKey(): Promise<CryptoKey> {
	return crypto.subtle.generateKey(
		{
			name: 'AES-GCM',
			length: 256
		} satisfies AesKeyGenParams,
		true,
		['encrypt']
	);
}

/** Hybrid encryption function that combines RSA and AES-GCM encryption.
 *
 * To allow for large messages, the message is first encrypted with a randomly generated AES key,
 * which is then encrypted with the provided RSA public key.
 *
 * @param {Uint8Array} message - The message to encrypt.
 * @param {CryptoKey} rsaKey - The RSA public key to use for encrypting the AES key.
 * @param {Uint8Array} nonceBuffer - A buffer to hold the nonce (initialization vector) for AES-GCM encryption. This should be 12 bytes long.
 */
async function hybridEncrypt(
	message: Uint8Array,
	rsaKey: CryptoKey,
	nonceBuffer: Uint8Array
): Promise<Uint8Array> {
	if (nonceBuffer.length !== NONCE_BYTES) {
		throw new Error(`nonceBuffer must be exactly ${NONCE_BYTES} bytes long for AES-GCM`);
	}

	const aesKey = await generateSymmetricKey();
	const iv = crypto.getRandomValues(nonceBuffer);

	const cipherText = await crypto.subtle.encrypt(
		{
			name: 'AES-GCM',
			iv
		} satisfies AesGcmParams,
		aesKey,
		message
	);

	const rawAesKey = await crypto.subtle.exportKey('raw', aesKey);
	const combinedKeyData = combineNonceAndKey(iv, rawAesKey);

	const encryptedKeyMaterial = await crypto.subtle.encrypt(
		{ name: 'RSA-OAEP' },
		rsaKey,
		combinedKeyData
	);

	const encryptedMessage = new Uint8Array(encryptedKeyMaterial.byteLength + cipherText.byteLength);
	encryptedMessage.set(new Uint8Array(encryptedKeyMaterial), 0);
	encryptedMessage.set(new Uint8Array(cipherText), encryptedKeyMaterial.byteLength);

	return encryptedMessage;
}

async function hybridDecrypt(
	message: Uint8Array,
	rsaPrivateKey: CryptoKey
): Promise<ArrayBuffer | null> {
	const modulusBits = (rsaPrivateKey.algorithm as RsaHashedKeyGenParams).modulusLength;
	const encryptedMessageByteLength = modulusBits / 8;

	const encryptedKeyMaterial = message.slice(0, encryptedMessageByteLength);
	const cipherText = message.slice(encryptedMessageByteLength);

	let decryptedKeyMaterial: ArrayBuffer;
	try {
		decryptedKeyMaterial = await crypto.subtle.decrypt(
			{ name: 'RSA-OAEP' },
			rsaPrivateKey,
			encryptedKeyMaterial
		);
	} catch (error) {
		if (error instanceof DOMException && error.name === 'InvalidAccessError') {
			// incorrect key
			return null;
		}

		throw error;
	}

	const { nonce, key: rawKey } = splitNonceAndKey(decryptedKeyMaterial);
	const key = await crypto.subtle.importKey('raw', rawKey, 'AES-GCM', false, ['decrypt']);

	return await crypto.subtle.decrypt(
		{
			name: 'AES-GCM',
			iv: nonce
		} satisfies AesGcmParams,
		key,
		cipherText
	);
}

function combineNonceAndKey(nonce: Uint8Array, key: ArrayBuffer): Uint8Array {
	const combined = new Uint8Array(NONCE_BYTES + key.byteLength);
	combined.set(nonce, 0);
	combined.set(new Uint8Array(key), NONCE_BYTES);
	return combined;
}

function splitNonceAndKey(combined: ArrayBuffer): { nonce: ArrayBuffer; key: ArrayBuffer } {
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
function secureRandomInt(min: number, max: number): number {
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

import { combineNonceAndKey, NONCE_BYTES, securePermute, splitNonceAndKey } from './utils';

export async function generatePublicationKeypair(): Promise<CryptoKeyPair> {
	return crypto.subtle.generateKey(
		{
			name: 'RSA-OAEP',
			modulusLength: 4096,
			publicExponent: new Uint8Array([0x01, 0x00, 0x01]), // 65537,
			hash: 'SHA-512'
		} satisfies RsaHashedKeyGenParams,
		true,
		['encrypt', 'decrypt']
	);
}

export async function encryptPublicationMessage(
	message: string,
	publicKeys: CryptoKey[]
): Promise<Uint8Array> {
	const permutedKeys = securePermute(publicKeys);

	let messageBuffer: Uint8Array = new TextEncoder().encode(message);
	const nonceBuffer = new Uint8Array(NONCE_BYTES);

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

export async function exportPublicKey(key: CryptoKey): Promise<string> {
	if (key.type !== 'public') {
		throw new Error('Key must be a public key to export');
	}

	const exportedKey = await crypto.subtle.exportKey('spki', key);
	return btoa(String.fromCharCode(...new Uint8Array(exportedKey)));
}

/**
 * Calculates the SHA-256 fingerprint of a CryptoKey.
 *
 * This function exports the key as a raw binary format, computes its SHA-256 hash,
 * and returns the hash as a hexadecimal string.
 *
 * @param {CryptoKey} key - The CryptoKey to calculate the fingerprint for. It must be public and extractable.
 * @returns {Promise<string>} A promise that resolves to the SHA-256 fingerprint in hexadecimal format.
 */
export async function calculateKeyFingerprint(key: CryptoKey): Promise<string> {
	if (key.type !== 'public') {
		throw new Error('Key must be a public key to calculate its fingerprint');
	}

	const exportedKey = await crypto.subtle.exportKey('spki', key);
	const hashBuffer = await crypto.subtle.digest('SHA-256', exportedKey);
	const hashArray = Array.from(new Uint8Array(hashBuffer), (b) => {
		return b.toString(16).padStart(2, '0');
	});

	return hashArray.join('');
}

export async function decryptAuthenticationChallenge(
	challenge: string,
	privateKey: CryptoKey
): Promise<string> {
	const binaryChallenge = atob(challenge);
	const challengeBuffer = new Uint8Array(binaryChallenge.length);
	for (let i = 0; i < binaryChallenge.length; i++) {
		challengeBuffer[i] = binaryChallenge.charCodeAt(i);
	}

	const decryptedBuffer = await crypto.subtle.decrypt(
		{ name: 'RSA-OAEP' } satisfies RsaOaepParams,
		privateKey,
		challengeBuffer
	);

	return new TextDecoder().decode(decryptedBuffer);
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
		['encrypt', 'decrypt']
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

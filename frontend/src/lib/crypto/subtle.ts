import { combineNonceAndKey, NONCE_BYTES, securePermute, splitNonceAndKey } from './utils';

/**
 * Generates a new RSA key pair for publication encryption & decryption.
 *
 * These keys will be used to encrypt and decrypt messages in the publication system
 * as well as authentication challenges.
 */
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

/** Constructs a layered encryption message for publication.
 *
 * @param message - The message to encrypt.
 * @param publicKeys - An array of public keys to encrypt the message with.
 */
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

/** Attempts to decrypt all messages in a round of publication messages.
 *
 * This function will try to decrypt each message using the provided private key.
 * If a message cannot be decrypted (e.g., due to an incorrect key), it will be skipped.
 *
 * It is possible that for a given round no messages can be decrypted. Like-wise, it is possible that
 * multiple or all messages can be decrypted.
 *
 * @param messages - An array of encrypted messages to decrypt.
 * @param privateKey - The private key to use for decryption.
 */
export async function decryptPublicationMessagesRound(
	messages: Uint8Array[],
	privateKey: CryptoKey
): Promise<ArrayBuffer[]> {
	const decryptionTasks = messages.map((encryptedMessage) =>
		hybridDecrypt(encryptedMessage, privateKey)
	);
	return (await Promise.all(decryptionTasks)).filter((result) => result !== null);
}

/** Exports a public key to a base64-encoded DER format.
 *
 * @param key - The CryptoKey to export. It must be a public key.
 */
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

/** Decrypts an authentication challenge using a private RSA key.
 *
 * This function takes a base64-encoded challenge string, decodes it,
 * and decrypts it using the provided RSA private key.
 *
 * @param challenge - The base64-encoded challenge string to decrypt.
 * @param privateKey - The RSA private key to use for decryption.
 */
export async function decryptAuthenticationChallenge(
	challenge: string,
	privateKey: CryptoKey
): Promise<string> {
	const decryptedBuffer = await crypto.subtle.decrypt(
		{ name: 'RSA-OAEP' } satisfies RsaOaepParams,
		privateKey,
		decodeBase64(challenge)
	);

	return new TextDecoder().decode(decryptedBuffer);
}

/** Generates a 256-bit symmetric key for AES-GCM encryption.
 *
 * This is used for hybrid encryption, where the symmetric key is used to encrypt the actual message,
 * and the symmetric key itself is encrypted with an RSA public key.
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

/**
 * Hybrid decryption function that combines RSA and AES-GCM decryption.
 *
 * This function takes an encrypted message, extracts the encrypted AES key,
 * decrypts it using the provided RSA private key, and then decrypts the message
 * using the decrypted AES key.
 *
 * @param {Uint8Array} message - The encrypted message to decrypt.
 * @param {CryptoKey} rsaPrivateKey - The RSA private key to use for decrypting the AES key.
 */
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

/**
 * Decodes a base64-encoded string into a Uint8Array.
 *
 * @param {string} base64 - The base64-encoded string to decode.
 * @returns {Uint8Array} The decoded binary data as a Uint8Array.
 */
function decodeBase64(base64: string): Uint8Array {
	const binaryString = atob(base64);
	const byteArray = new Uint8Array(binaryString.length);
	for (let i = 0; i < binaryString.length; i++) {
		byteArray[i] = binaryString.charCodeAt(i);
	}
	return byteArray;
}

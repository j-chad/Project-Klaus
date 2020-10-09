import crypto from "crypto";

export function makeKey(length) {
	let result = new Array(length).fill("");
	const characters = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789';
	const charactersLength = characters.length;

	result = result.map(()=>characters.charAt(crypto.randomInt(charactersLength)))
	return result.join("");
}
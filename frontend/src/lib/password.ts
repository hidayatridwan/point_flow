const UPPER = 'ABCDEFGHJKLMNPQRSTUVWXYZ';
const LOWER = 'abcdefghijkmnpqrstuvwxyz';
const DIGITS = '23456789';
const ALL = UPPER + LOWER + DIGITS;

function randomIndex(max: number): number {
	const array = new Uint32Array(1);
	crypto.getRandomValues(array);
	return array[0] % max;
}

/**
 * Generate a random password with at least one uppercase letter, one
 * lowercase letter, and one digit. Ambiguous characters (I, l, O, o, 0, 1)
 * are excluded so the password is easy to read back to a member.
 */
export function generateStrongPassword(length = 8): string {
	const size = Math.max(length, 3);
	const chars = [
		UPPER[randomIndex(UPPER.length)],
		LOWER[randomIndex(LOWER.length)],
		DIGITS[randomIndex(DIGITS.length)]
	];
	for (let i = chars.length; i < size; i += 1) {
		chars.push(ALL[randomIndex(ALL.length)]);
	}
	for (let i = chars.length - 1; i > 0; i -= 1) {
		const j = randomIndex(i + 1);
		[chars[i], chars[j]] = [chars[j], chars[i]];
	}
	return chars.join('');
}

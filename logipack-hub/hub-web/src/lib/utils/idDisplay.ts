export function compactId(value: string, visibleChars = 8): string {
	const normalized = value.trim();
	if (normalized.length <= visibleChars) {
		return normalized;
	}
	return `${normalized.slice(0, visibleChars)}...`;
}

export function isIdColumn(column: string): boolean {
	const normalized = column.trim().toLowerCase();
	return normalized === "id" || normalized.endsWith("_id");
}

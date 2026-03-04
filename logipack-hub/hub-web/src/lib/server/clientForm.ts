export type ClientFormValues = {
	name: string;
	email: string | null;
	phone: string | null;
};

export type ClientFieldErrors = {
	name?: string;
	email?: string;
	phone?: string;
};

export function parseClientFormData(formData: FormData): ClientFormValues {
	const email = String(formData.get("email") ?? "").trim().toLowerCase();
	const phone = String(formData.get("phone") ?? "").trim();

	return {
		name: String(formData.get("name") ?? "").trim(),
		email: email || null,
		phone: phone || null,
	};
}

export function validateClientForm(values: ClientFormValues): ClientFieldErrors {
	const fieldErrors: ClientFieldErrors = {};
	const emailPattern = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
	const phonePattern = /^\+\d{6,15}$/;

	if (!values.name) {
		fieldErrors.name = "client.form.name_required";
	}
	if (values.email && !emailPattern.test(values.email)) {
		fieldErrors.email = "client.form.email_invalid";
	}
	if (values.phone && !phonePattern.test(values.phone)) {
		fieldErrors.phone = "client.form.phone_invalid";
	}

	return fieldErrors;
}

export function hasClientFormErrors(fieldErrors: ClientFieldErrors): boolean {
	return Object.values(fieldErrors).some((value) => Boolean(value));
}

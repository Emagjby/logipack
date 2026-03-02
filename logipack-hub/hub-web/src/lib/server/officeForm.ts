export type OfficeFormValues = {
	name: string;
	city: string;
	address: string;
};

export type OfficeFieldErrors = {
	name?: string;
	city?: string;
	address?: string;
};

export function parseOfficeFormData(formData: FormData): OfficeFormValues {
	return {
		name: String(formData.get("name") ?? "").trim(),
		city: String(formData.get("city") ?? "").trim(),
		address: String(formData.get("address") ?? "").trim(),
	};
}

export function validateOfficeForm(
	values: OfficeFormValues,
): OfficeFieldErrors {
	const fieldErrors: OfficeFieldErrors = {};

	if (!values.name) {
		fieldErrors.name = "office.form.name_required";
	}
	if (!values.city) {
		fieldErrors.city = "office.form.city_required";
	}
	if (!values.address) {
		fieldErrors.address = "office.form.address_required";
	}

	return fieldErrors;
}

export function hasOfficeFormErrors(fieldErrors: OfficeFieldErrors): boolean {
	return Object.values(fieldErrors).some((value) => Boolean(value));
}

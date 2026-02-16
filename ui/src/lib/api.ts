import type { RaucStatus, RaucBundleInfo } from '$lib/types/rauc';

export async function fetchStatus(): Promise<RaucStatus> {
	const response = await fetch('/api/status');
	if (!response.ok) {
		const error = await response.text();
		throw new Error(error);
	}
	return response.json();
}

export async function uploadBundle(file: File): Promise<string> {
	const formData = new FormData();
	formData.append('file', file);

	const response = await fetch('/api/upload', {
		method: 'POST',
		body: formData
	});

	const result = await response.text();
	if (!response.ok) {
		throw new Error(result);
	}
	return result;
}

export async function fetchBundleInfo(): Promise<RaucBundleInfo> {
	const response = await fetch('/api/bundle-info');
	if (!response.ok) {
		const error = await response.text();
		throw new Error(error);
	}
	return response.json();
}

export async function* installBundle(): AsyncGenerator<string, void, unknown> {
	const response = await fetch('/api/install');
	if (!response.body) {
		throw new Error('No response body');
	}

	const reader = response.body.getReader();
	const decoder = new TextDecoder();

	while (true) {
		const { done, value } = await reader.read();
		if (done) break;

		const text = decoder.decode(value, { stream: true });
		yield text;
	}
}

export async function rebootSystem(): Promise<string> {
	const response = await fetch('/api/reboot', { method: 'POST' });
	return response.text();
}

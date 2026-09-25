import type { RaucStatus, RaucBundleInfo, UpdateCheck } from '$lib/types/rauc';

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

/** `null` when no UPDATE_MANIFEST_URL is configured. */
export async function checkForUpdate(): Promise<UpdateCheck | null> {
	const response = await fetch('/api/update-check', { cache: 'no-cache' });
	if (response.status === 404) return null;
	if (!response.ok) {
		throw new Error(await response.text());
	}
	return response.json();
}

/** Installs the uploaded bundle, or with `url` a bundle streamed from there. */
export async function* installBundle(url?: string): AsyncGenerator<string, void, unknown> {
	const response = url
		? await fetch('/api/install-url', {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({ url })
			})
		: await fetch('/api/install');
	if (!response.ok) {
		throw new Error(await response.text());
	}
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

<script lang="ts">
	interface Props {
		onUploadSuccess?: () => void;
	}

	let { onUploadSuccess }: Props = $props();

	let selectedFile = $state<File | null>(null);
	let uploading = $state(false);
	let uploadProgress = $state(0);
	let saving = $state(false);
	let uploadResult = $state<{ type: 'success' | 'error'; message: string } | null>(null);

	async function handleFileChange(event: Event) {
		const input = event.target as HTMLInputElement;
		const file = input.files?.[0] || null;

		if (!file) {
			return;
		}

		selectedFile = file;
		uploadResult = null;

		// Auto-upload on file selection
		await handleUpload(file);
	}

	async function handleUpload(file: File) {
		uploading = true;
		uploadProgress = 0;
		saving = false;
		uploadResult = null;

		try {
			const formData = new FormData();
			formData.append('file', file);

			const xhr = new XMLHttpRequest();

			// Track upload progress
			xhr.upload.addEventListener('progress', (event) => {
				if (event.lengthComputable) {
					uploadProgress = Math.round((event.loaded / event.total) * 100);
					if (uploadProgress === 100) {
						saving = true;
					}
				}
			});

			// Handle completion
			const uploadPromise = new Promise<string>((resolve, reject) => {
				xhr.addEventListener('load', () => {
					if (xhr.status >= 200 && xhr.status < 300) {
						resolve(xhr.responseText);
					} else {
						reject(new Error(xhr.responseText || 'Upload failed'));
					}
				});

				xhr.addEventListener('error', () => {
					reject(new Error('Network error during upload'));
				});

				xhr.addEventListener('abort', () => {
					reject(new Error('Upload aborted'));
				});
			});

			xhr.open('POST', '/api/upload');
			xhr.send(formData);

			const message = await uploadPromise;
			uploadResult = { type: 'success', message };
			if (onUploadSuccess) {
				setTimeout(() => onUploadSuccess(), 500);
			}
		} catch (err) {
			uploadResult = {
				type: 'error',
				message: err instanceof Error ? err.message : 'Upload failed'
			};
		} finally {
			uploading = false;
			saving = false;
			uploadProgress = 0;
		}
	}
</script>

<div class="bg-card overflow-hidden shadow sm:rounded-lg">
	<div class="border-subtle border-b px-4 py-5 sm:px-6">
		<h2 class="text-primary text-lg leading-6 font-medium">Upload Update Bundle</h2>
		<p class="text-secondary mt-1 text-sm">Select and upload an update bundle</p>
	</div>

	<div class="px-4 py-5 sm:p-6">
		<div class="flex flex-col gap-4">
			<div class="flex flex-wrap items-center gap-4">
				<label class="relative inline-flex cursor-pointer">
					<span
						class="border-subtle bg-card text-primary bg-hover inline-flex items-center rounded-md border px-4 py-2 text-sm font-medium shadow-sm"
						class:opacity-50={uploading}
						class:cursor-not-allowed={uploading}
					>
						{uploading ? 'Uploading...' : 'Choose File'}
					</span>
					<input
						type="file"
						class="sr-only"
						accept=".raucb"
						onchange={handleFileChange}
						disabled={uploading}
					/>
				</label>

				{#if selectedFile && !uploading}
					<span class="text-secondary text-sm">
						{selectedFile.name}
					</span>
				{/if}
			</div>

			{#if uploading}
				<div class="space-y-2">
					<div class="flex items-center justify-between text-sm">
						<span class="text-primary font-medium">
							{saving ? 'Saving...' : `Uploading ${uploadProgress}%`}
						</span>
						<span class="text-secondary">{selectedFile?.name}</span>
					</div>
					<div class="bg-subtle h-2.5 w-full rounded-full">
						<div
							class="h-2.5 rounded-full transition-all duration-300"
							style="width: {uploadProgress}%; background-color: var(--primary-color)"
						></div>
					</div>
				</div>
			{/if}

			{#if uploadResult}
				<div
					class="rounded-md p-4"
					class:bg-success={uploadResult.type === 'success'}
					class:bg-error={uploadResult.type === 'error'}
				>
					<div class="flex">
						<div class="ml-3">
							<p
								class="text-sm font-medium"
								class:text-success={uploadResult.type === 'success'}
								class:text-error={uploadResult.type === 'error'}
							>
								{uploadResult.message}
							</p>
						</div>
					</div>
				</div>
			{/if}
		</div>
	</div>
</div>

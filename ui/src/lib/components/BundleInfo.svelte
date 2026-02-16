<script lang="ts">
	import type { RaucBundleInfo } from '$lib/types/rauc';
	import { fetchBundleInfo } from '$lib/api';

	interface Props {
		onInstall?: () => void;
		onUploadDifferent?: () => void;
	}

	let { onInstall, onUploadDifferent }: Props = $props();

	let bundleInfo = $state<RaucBundleInfo | null>(null);
	let loading = $state(false);
	let error = $state<string | null>(null);

	async function loadBundleInfo() {
		loading = true;
		error = null;
		try {
			bundleInfo = await fetchBundleInfo();
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to load bundle info';
		} finally {
			loading = false;
		}
	}

	// Load bundle info on mount
	$effect(() => {
		loadBundleInfo();
	});
</script>

<div class="bg-card overflow-hidden shadow sm:rounded-lg">
	<div class="border-subtle border-b px-4 py-5 sm:px-6">
		<h2 class="text-primary text-lg leading-6 font-medium">Bundle Information</h2>
		<p class="text-secondary mt-1 text-sm">Details about the uploaded update bundle</p>
	</div>

	<div class="px-4 py-5 sm:p-6">
		{#if loading}
			<div class="flex items-center justify-center py-12">
				<div
					class="h-8 w-8 animate-spin rounded-full border-b-2"
					style="border-color: var(--primary-color)"
				></div>
			</div>
		{:else if error}
			<div class="bg-error rounded-md p-4">
				<div class="flex">
					<div class="ml-3">
						<h3 class="text-error text-sm font-medium">Error loading bundle info</h3>
						<div class="text-error mt-2 text-sm">{error}</div>
					</div>
				</div>
			</div>
		{:else if bundleInfo}
			<dl class="grid grid-cols-1 gap-5 sm:grid-cols-2 lg:grid-cols-4">
				<div class="bg-subtle overflow-hidden rounded-lg px-4 py-5 sm:p-6">
					<dt class="text-secondary truncate text-sm font-medium">Compatible</dt>
					<dd class="text-primary mt-1 text-lg font-semibold">{bundleInfo.compatible}</dd>
				</div>
				<div class="bg-subtle overflow-hidden rounded-lg px-4 py-5 sm:p-6">
					<dt class="text-secondary truncate text-sm font-medium">Version</dt>
					<dd class="text-primary mt-1 text-lg font-semibold">{bundleInfo.version}</dd>
				</div>
				<div class="bg-subtle overflow-hidden rounded-lg px-4 py-5 sm:p-6">
					<dt class="text-secondary truncate text-sm font-medium">Build</dt>
					<dd class="text-primary mt-1 text-lg font-semibold">{bundleInfo.build}</dd>
				</div>
				<div class="bg-subtle overflow-hidden rounded-lg px-4 py-5 sm:p-6">
					<dt class="text-secondary truncate text-sm font-medium">Format</dt>
					<dd class="text-primary mt-1 text-lg font-semibold">{bundleInfo.format}</dd>
				</div>
			</dl>

			{#if bundleInfo.description || bundleInfo.hash}
				<dl class="divide-muted border-subtle mt-5 divide-y border-t">
					{#if bundleInfo.description}
						<div class="py-4">
							<dt class="text-secondary text-sm font-medium">Description</dt>
							<dd class="text-primary mt-1 text-sm">{bundleInfo.description}</dd>
						</div>
					{/if}
					{#if bundleInfo.hash}
						<div class="py-4">
							<dt class="text-secondary text-sm font-medium">Hash</dt>
							<dd class="text-primary mt-1 font-mono text-xs break-all">{bundleInfo.hash}</dd>
						</div>
					{/if}
				</dl>
			{/if}

			{#if bundleInfo.images && bundleInfo.images.length > 0}
				<h3 class="text-primary mb-4 text-xl font-semibold">Images</h3>

				{#each bundleInfo.images as imageObj}
					{@const imageName = Object.keys(imageObj)[0]}
					{@const image = imageObj[imageName]}

					<div
						class="bg-subtle mb-3 rounded-lg border-l-4 p-4"
						style="border-color: var(--primary-color)"
					>
						<h3 class="text-primary mb-3 text-lg font-semibold">{imageName}</h3>

						<div class="grid gap-2 text-sm md:grid-cols-2 lg:grid-cols-3">
							{#if image.filename}
								<div>
									<strong class="text-secondary">Filename:</strong>
									<span class="text-primary">{image.filename}</span>
								</div>
							{/if}
							{#if image.type}
								<div>
									<strong class="text-secondary">Type:</strong>
									<span class="text-primary">{image.type}</span>
								</div>
							{/if}
							{#if image.size}
								<div>
									<strong class="text-secondary">Size:</strong>
									<span class="text-primary">{(image.size / 1024 / 1024).toFixed(2)} MB</span>
								</div>
							{/if}
							{#if image.checksum}
								<div class="md:col-span-3">
									<strong class="text-secondary">Checksum:</strong>
									<code class="text-primary text-xs break-all">{image.checksum}</code>
								</div>
							{/if}
						</div>
					</div>
				{/each}
			{/if}

			<div class="mt-6 flex gap-3">
				<button
					onclick={onInstall}
					class="inline-flex items-center rounded-md px-4 py-2 text-sm font-semibold text-white shadow-sm hover:opacity-90"
					style="background-color: var(--primary-color)"
				>
					Install Bundle
				</button>
				<button
					onclick={onUploadDifferent}
					class="border-subtle bg-card text-primary bg-hover inline-flex items-center rounded-md border px-4 py-2 text-sm font-medium shadow-sm"
				>
					Upload Different Image
				</button>
			</div>
		{/if}
	</div>
</div>

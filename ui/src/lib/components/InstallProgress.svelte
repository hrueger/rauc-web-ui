<script lang="ts">
	import { installBundle, rebootSystem } from '$lib/api';
	import { onMount } from 'svelte';

	let installOutput = $state('');
	let installing = $state(false);
	let installSuccess = $state(false);
	let rebooting = $state(false);
	let showRebootOverlay = $state(false);
	let reconnectionAttempts = $state(0);
	let installOutputElement: HTMLPreElement | null = null;

	onMount(() => {
		setTimeout(() => {
			startInstall();
		}, 200);
	});

	function scrollToBottom() {
		if (installOutputElement) {
			installOutputElement.scrollTop = installOutputElement.scrollHeight;
		}
	}

	async function startInstall() {
		installing = true;
		installSuccess = false;
		installOutput = 'Starting installation...\n';

		try {
			for await (const chunk of installBundle()) {
				installOutput += chunk;
				if (chunk.includes('[DONE]')) {
					installSuccess = true;
				}
				scrollToBottom();
			}
		} catch (err) {
			installOutput += `\n[ERROR] ${err instanceof Error ? err.message : 'Installation failed'}\n`;
			scrollToBottom();
		} finally {
			installing = false;
		}
	}

	async function checkConnection(): Promise<boolean> {
		try {
			const response = await fetch('/api/status', {
				method: 'GET',
				cache: 'no-cache'
			});
			return response.ok;
		} catch {
			return false;
		}
	}

	async function waitForReconnection() {
		reconnectionAttempts = 0;

		// Wait a few seconds before starting to poll (give the system time to reboot)
		await new Promise((resolve) => setTimeout(resolve, 10000));

		// Poll every 2 seconds
		const pollInterval = 2000;
		const maxAttempts = 60; // Try for 2 minutes

		for (let i = 0; i < maxAttempts; i++) {
			reconnectionAttempts = i + 1;
			const isConnected = await checkConnection();

			if (isConnected) {
				// Connection restored, reload the page
				window.location.reload();
				return;
			}

			await new Promise((resolve) => setTimeout(resolve, pollInterval));
		}

		// If we get here, reconnection failed
		alert('Could not reconnect to the system. Please check manually and refresh the page.');
	}

	async function handleReboot() {
		if (!confirm('Are you sure you want to reboot now?')) {
			return;
		}

		rebooting = true;
		showRebootOverlay = true;
		installOutput += '\nInitiating reboot...\n';

		try {
			const result = await rebootSystem();
			installOutput += result + '\n';
			installOutput += 'System is rebooting. This page will become unavailable.\n';
		} catch (err) {
			installOutput += 'Reboot command sent (connection lost as expected)\n';
		}

		scrollToBottom();

		// Start the reconnection process
		await waitForReconnection();
	}
</script>

<div class="bg-card overflow-hidden shadow sm:rounded-lg">
	<div class="border-subtle border-b px-4 py-5 sm:px-6">
		<h2 class="text-primary text-lg leading-6 font-medium">Installation Progress</h2>
		<p class="text-secondary mt-1 text-sm">Real-time installation status and output</p>
	</div>

	<div class="px-4 py-5 sm:p-6">
		<pre
			bind:this={installOutputElement}
			class="max-h-96 overflow-y-auto rounded-lg p-4 font-mono text-sm"
			style="background-color: color-mix(in srgb, var(--foreground-color) 95%, transparent); color: var(--background-color)">{installOutput}</pre>

		{#if installSuccess && !rebooting}
			<div class="mt-4">
				<button
					onclick={handleReboot}
					class="inline-flex items-center rounded-md px-4 py-2 text-sm font-semibold text-white shadow-sm hover:opacity-90"
					style="background-color: var(--primary-color)"
				>
					Reboot Now
				</button>
			</div>
		{/if}
	</div>
</div>

{#if showRebootOverlay}
	<div
		class="fixed inset-0 z-50 flex items-center justify-center"
		style="background-color: rgba(0, 0, 0, 0.75)"
	>
		<div class="text-center">
			<div class="mb-6 flex justify-center">
				<div
					class="h-16 w-16 animate-spin rounded-full border-t-4 border-b-4"
					style="border-color: var(--primary-color)"
				></div>
			</div>
			<h2 class="mb-2 text-2xl font-bold" style="color: var(--background-color)">
				System Rebooting
			</h2>
			<p class="text-lg" style="color: rgba(var(--background-color), 0.8)">
				Please wait while the system restarts...
			</p>
			{#if reconnectionAttempts > 0}
				<p class="mt-4 text-sm" style="color: rgba(var(--background-color), 0.6)">
					Reconnection attempt {reconnectionAttempts}...
				</p>
			{/if}
		</div>
	</div>
{/if}

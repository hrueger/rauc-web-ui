<script lang="ts">
	// Shown only when the server has an UPDATE_MANIFEST_URL: the newest bundle
	// for this system, installed straight from its URL.
	import type { UpdateCheck } from '$lib/types/rauc';
	import { checkForUpdate } from '$lib/api';

	interface Props {
		onInstall: (url: string) => void;
	}

	let { onInstall }: Props = $props();

	let check = $state<UpdateCheck | null>(null);
	let configured = $state(true);
	let loading = $state(false);
	let error = $state<string | null>(null);

	async function load() {
		loading = true;
		error = null;
		try {
			check = await checkForUpdate();
			configured = check !== null;
		} catch (err) {
			error = err instanceof Error ? err.message : 'Update check failed';
		} finally {
			loading = false;
		}
	}

	$effect(() => {
		load();
	});

	function install() {
		if (!check?.latest) return;
		if (!confirm(`Install version ${check.latest.version} from the update server now?`)) return;
		onInstall(check.latest.url);
	}
</script>

{#if configured}
	<div class="bg-card overflow-hidden shadow sm:rounded-lg">
		<div class="border-subtle flex items-center justify-between border-b px-4 py-5 sm:px-6">
			<div>
				<h2 class="text-primary text-lg leading-6 font-medium">Online Update</h2>
				<p class="text-secondary mt-1 text-sm">The newest bundle from the update server</p>
			</div>
			<button
				onclick={load}
				disabled={loading}
				class="text-primary rounded-md px-3 py-2 text-sm font-semibold shadow-sm ring-1 ring-inset ring-gray-300 hover:opacity-80 disabled:opacity-50"
			>
				Check again
			</button>
		</div>

		<div class="px-4 py-5 sm:p-6">
			{#if loading && !check}
				<p class="text-secondary text-sm">Checking...</p>
			{:else if error}
				<div class="bg-error text-error rounded-md p-4 text-sm">{error}</div>
			{:else if check}
				<dl class="grid grid-cols-1 gap-5 sm:grid-cols-3">
					<div class="bg-subtle rounded-lg px-4 py-5">
						<dt class="text-secondary text-sm font-medium">Installed</dt>
						<dd class="text-primary mt-1 text-lg font-semibold">
							{check.bootedVersion ?? 'unknown'}{check.bootedSlot ? ` (slot ${check.bootedSlot})` : ''}
						</dd>
					</div>
					<div class="bg-subtle rounded-lg px-4 py-5">
						<dt class="text-secondary text-sm font-medium">Newest</dt>
						<dd class="text-primary mt-1 text-lg font-semibold">
							{check.latest?.version ?? 'none for this system'}
						</dd>
					</div>
					<div class="bg-subtle rounded-lg px-4 py-5">
						<dt class="text-secondary text-sm font-medium">Compatible</dt>
						<dd class="text-primary mt-1 truncate text-lg font-semibold">{check.compatible}</dd>
					</div>
				</dl>
				{#if check.latest?.notes && check.available}
					<pre class="text-secondary mt-4 text-sm whitespace-pre-wrap">{check.latest.notes}</pre>
				{/if}
				<div class="mt-4">
					{#if check.available}
						<button
							onclick={install}
							class="inline-flex items-center rounded-md px-4 py-2 text-sm font-semibold text-white shadow-sm hover:opacity-90"
							style="background-color: var(--primary-color)"
						>
							Install {check.latest?.version}
						</button>
					{:else}
						<p class="text-secondary text-sm">This system is up to date.</p>
					{/if}
				</div>
			{/if}
		</div>
	</div>
{/if}

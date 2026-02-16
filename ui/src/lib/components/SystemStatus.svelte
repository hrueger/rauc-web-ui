<script lang="ts">
	import type { RaucStatus } from '$lib/types/rauc';
	import { fetchStatus } from '$lib/api';

	let status = $state<RaucStatus | null>(null);
	let loading = $state(false);
	let error = $state<string | null>(null);

	async function loadStatus() {
		loading = true;
		error = null;
		try {
			status = await fetchStatus();
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to load status';
		} finally {
			loading = false;
		}
	}

	// Load status on mount
	$effect(() => {
		loadStatus();
	});
</script>

<div class="bg-card overflow-hidden shadow sm:rounded-lg">
	<div class="border-subtle border-b px-4 py-5 sm:px-6">
		<div class="flex items-center justify-between">
			<h2 class="text-primary text-lg leading-6 font-medium">System Status</h2>
			<button
				onclick={loadStatus}
				disabled={loading}
				class="inline-flex items-center rounded-md px-3 py-2 text-sm font-semibold text-white shadow-sm hover:opacity-90 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 disabled:cursor-not-allowed disabled:opacity-50"
				style="background-color: var(--primary-color); outline-color: var(--primary-color)"
			>
				{loading ? 'Loading...' : 'Refresh'}
			</button>
		</div>
	</div>

	<div class="px-4 py-5 sm:p-6">
		{#if loading && !status}
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
						<h3 class="text-error text-sm font-medium">Error loading status</h3>
						<div class="text-error mt-2 text-sm">{error}</div>
					</div>
				</div>
			</div>
		{:else if status}
			<dl class="grid grid-cols-1 gap-5 sm:grid-cols-2 lg:grid-cols-4">
				<div class="bg-subtle overflow-hidden rounded-lg px-4 py-5 sm:p-6">
					<dt class="text-secondary truncate text-sm font-medium">Compatible</dt>
					<dd class="text-primary mt-1 text-xl font-semibold tracking-tight">
						{status.compatible}
					</dd>
				</div>
				<div class="bg-subtle overflow-hidden rounded-lg px-4 py-5 sm:p-6">
					<dt class="text-secondary truncate text-sm font-medium">Booted Slot</dt>
					<dd class="text-primary mt-1 text-xl font-semibold tracking-tight">{status.booted}</dd>
				</div>
				<div class="bg-subtle overflow-hidden rounded-lg px-4 py-5 sm:p-6">
					<dt class="text-secondary truncate text-sm font-medium">Boot Primary</dt>
					<dd class="text-primary mt-1 text-xl font-semibold tracking-tight">
						{status.boot_primary}
					</dd>
				</div>
				{#if status.variant}
					<div class="bg-subtle overflow-hidden rounded-lg px-4 py-5 sm:p-6">
						<dt class="text-secondary truncate text-sm font-medium">Variant</dt>
						<dd class="text-primary mt-1 text-xl font-semibold tracking-tight">
							{status.variant}
						</dd>
					</div>
				{/if}
			</dl>

			<div class="mt-8">
				<h3 class="text-primary text-base leading-6 font-semibold">Slots</h3>
				<div class="mt-4 space-y-4">
					{#each status.slots as slotObj}
						{@const slotName = Object.keys(slotObj)[0]}
						{@const slot = slotObj[slotName]}
						{@const isBooted = slot.state === 'booted'}

						<div class="border-subtle bg-card overflow-hidden rounded-lg border">
							<div class="border-subtle border-b px-4 py-3">
								<div class="flex items-center justify-between">
									<h4 class="text-primary text-base font-medium">{slotName}</h4>
									<div class="flex items-center gap-2">
										<span
											class="inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-medium"
											class:badge-success={isBooted}
											class:badge-inactive={!isBooted}
										>
											{slot.state}
										</span>
										{#if slot.boot_status}
											<span
												class="inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-medium"
												class:badge-success={slot.boot_status === 'good'}
												class:badge-danger={slot.boot_status !== 'good'}
											>
												{slot.boot_status}
											</span>
										{/if}
									</div>
								</div>
							</div>
							<dl class="grid grid-cols-1 gap-x-4 gap-y-4 px-4 py-4 sm:grid-cols-2">
								<div>
									<dt class="text-secondary text-sm font-medium">Device</dt>
									<dd class="text-primary mt-1 text-sm">{slot.device}</dd>
								</div>
								<div>
									<dt class="text-secondary text-sm font-medium">Type</dt>
									<dd class="text-primary mt-1 text-sm">{slot.type}</dd>
								</div>
								<div>
									<dt class="text-secondary text-sm font-medium">Bootname</dt>
									<dd class="text-primary mt-1 text-sm">{slot.bootname}</dd>
								</div>
								<div>
									<dt class="text-secondary text-sm font-medium">Class</dt>
									<dd class="text-primary mt-1 text-sm">{slot.class}</dd>
								</div>
								{#if slot.mountpoint}
									<div class="sm:col-span-2">
										<dt class="text-secondary text-sm font-medium">Mountpoint</dt>
										<dd class="text-primary mt-1 text-sm">{slot.mountpoint}</dd>
									</div>
								{/if}
							</dl>
						</div>
					{/each}
				</div>
			</div>
		{/if}
	</div>
</div>

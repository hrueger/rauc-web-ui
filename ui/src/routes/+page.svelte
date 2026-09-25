<script lang="ts">
	import { SystemStatus, BundleUpload, BundleInfo, InstallProgress, OnlineUpdate } from '$lib/components';

	type View = 'upload' | 'bundle-info' | 'install';

	let currentView = $state<View>('upload');
	/** Set when the install streams a bundle from the update manifest. */
	let installUrl = $state<string | undefined>(undefined);

	function handleUploadSuccess() {
		currentView = 'bundle-info';
	}

	function handleUploadDifferent() {
		currentView = 'upload';
	}

	function handleInstall() {
		installUrl = undefined;
		currentView = 'install';
	}

	function handleInstallFromUrl(url: string) {
		installUrl = url;
		currentView = 'install';
	}
</script>

<div class="min-h-screen py-8">
	<div class="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
		<div class="mb-8 text-center">
			<h1 class="text-primary text-3xl font-bold tracking-tight sm:text-4xl">
				{document.title}
			</h1>
			<p class="text-secondary mt-2 text-sm">Manage system updates and bundle installations</p>
		</div>

		<div class="space-y-6">
			{#if currentView !== 'install'}
				<OnlineUpdate onInstall={handleInstallFromUrl} />
			{/if}
			{#if currentView === 'upload'}
				<BundleUpload onUploadSuccess={handleUploadSuccess} />
			{:else if currentView === 'bundle-info'}
				<BundleInfo onInstall={handleInstall} onUploadDifferent={handleUploadDifferent} />
			{:else if currentView === 'install'}
				<InstallProgress url={installUrl} />
			{/if}
			<SystemStatus />
		</div>
	</div>
</div>

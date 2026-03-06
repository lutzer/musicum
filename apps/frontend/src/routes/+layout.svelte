<script lang="ts">
	import '$lib/styles/globals.css';
	import Sidebar from '$lib/components/Sidebar.svelte';
	import Header from '$lib/components/Header.svelte';

	let { children } = $props();

	let searchQuery = $state('');

	function handleSearch(query: string) {
		searchQuery = query;
	}
</script>

<div class="app">
	<Header bind:searchQuery onsearch={handleSearch} />

	<div class="app-body">
		<Sidebar />

		<main class="main-content">
			{@render children()}
		</main>
	</div>
</div>

<style>
	.app {
		display: flex;
		flex-direction: column;
		height: 100vh;
		overflow: hidden;
	}

	.app-body {
		display: flex;
		flex: 1;
		overflow: hidden;
	}

	.main-content {
		flex: 1;
		overflow-y: auto;
		padding: var(--space-md);
	}
</style>

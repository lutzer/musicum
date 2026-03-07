<script lang="ts">
	import { onMount } from 'svelte';
	import '$lib/styles/globals.css';
	import Header from '$lib/components/Header.svelte';
	import { authStore } from '$lib/stores/auth.svelte';

	let { children } = $props();

	let searchQuery = $state('');

	function handleSearch(query: string) {
		searchQuery = query;
	}

	onMount(() => {
		authStore.initialize();
	});
</script>

<div class="app">
	<Header bind:searchQuery onsearch={handleSearch} />

	<main class="main-content">
		{@render children()}
	</main>
</div>

<style>
	.app {
		display: flex;
		flex-direction: column;
		height: 100vh;
		overflow: hidden;
	}

	.main-content {
		flex: 1;
		overflow-y: auto;
		padding: var(--space-md);
	}
</style>

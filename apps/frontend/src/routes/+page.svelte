<script lang="ts">
	import { onMount } from 'svelte';
	import { listCollections } from '$lib/api';
	import type { CollectionResponse } from '$lib/types';
	import CollectionCard from '$lib/components/CollectionCard.svelte';

	let collections = $state<CollectionResponse[]>([]);
	let loading = $state(true);
	let error = $state<string | null>(null);

	onMount(async () => {
		try {
			const response = await listCollections();
			collections = response.items;
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load collections';
		} finally {
			loading = false;
		}
	});
</script>

<div class="page">
	<div class="page-header">
		<h1 class="page-title">&gt; Collections</h1>
		<span class="page-count">[{collections.length}]</span>
	</div>

	{#if loading}
		<div class="loading">Loading...</div>
	{:else if error}
		<div class="error">Error: {error}</div>
	{:else if collections.length === 0}
		<div class="empty">
			<p>No collections found.</p>
			<p class="empty-hint">Create your first collection to get started.</p>
		</div>
	{:else}
		<div class="grid">
			{#each collections as collection (collection.id)}
				<CollectionCard {collection} />
			{/each}
		</div>
	{/if}
</div>

<style>
	.page-header {
		display: flex;
		align-items: baseline;
		gap: var(--space-sm);
		margin-bottom: var(--space-lg);
	}

	.page-title {
		font-weight: normal;
	}

	.grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
		gap: var(--space-md);
	}

	.empty-hint {
		margin-top: var(--space-sm);
	}
</style>

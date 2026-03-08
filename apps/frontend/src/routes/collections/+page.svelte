<script lang="ts">
	import { onMount } from 'svelte';
	import { listCollections, listTracks } from '$lib/api';
	import type { CollectionResponse } from '$lib/types';
	import CollectionCard from '$lib/components/CollectionCard.svelte';
	import { authStore } from '$lib/stores/auth.svelte';

	let collections = $state<CollectionResponse[]>([]);
	let loading = $state(true);
	let error = $state<string | null>(null);

	let page = 0;

	onMount(async () => {
		try {
			collections = (await listCollections()).items;
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load collections';
		} finally {
			loading = false;
		}
	});
</script>

<div>
	<div class="page-header">
		<h1 class="page-title">Collections</h1>
		<span class="page-count">[{collections.length}]</span>
		{#if authStore.isAuthenticated}
			<a href="/create_collection" class="page-action">[+ new]</a>
		{/if}
	</div>

	{#if loading}
		<div class="loading">Loading...</div>
	{:else if error}
		<div class="error">Error: {error}</div>
	{:else if collections.length === 0}
		<div class="empty">
			<p>No collections found.</p>
			{#if authStore.isAuthenticated}
				<p class="empty-hint">
					<a href="/create_collection">[+ Create your first collection]</a>
				</p>
			{/if}
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

	.page-toggle {
		margin-left: var(--space-md);
	}

	.page-action {
		margin-left: auto;
	}

	.grid {
		display: flex;
		gap: var(--space-md);
		flex-wrap: wrap;
	}

	.empty-hint {
		margin-top: var(--space-sm);
	}
</style>

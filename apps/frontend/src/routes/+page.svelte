<script lang="ts">
	import { onMount } from 'svelte';
	import { listCollections, listTracks } from '$lib/api';
	import type { CollectionResponse, TrackResponse } from '$lib/types';
	import CollectionCard from '$lib/components/CollectionCard.svelte';
	import { authStore } from '$lib/stores/auth.svelte';
	import TrackRow from '$lib/components/TrackRow.svelte';

	let collections = $state<CollectionResponse[]>([]);
	let tracks = $state<TrackResponse[]>([]);
	let loading = $state(true);
	let error = $state<string | null>(null);

	let page = 0;

	onMount(async () => {
		try {
			collections = (await listCollections()).items;
			tracks = (await listTracks()).items;
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load collections';
		} finally {
			loading = false;
		}
	});
</script>

<div class="page">
	<div class="collections">
		<a href="/collections">
			<div class="page-header">
				<h1 class="page-title">Collections</h1>
				<span class="page-count">[{collections.length}]</span>
			</div>
		</a>

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
	<div class="tracks">
		
		<a href="/tracks">
			<div class="page-header">
				<h1 class="page-title">Tracks</h1>
				<span class="page-count">[{tracks.length}]</span>
			</div>
		</a>

		{#if loading}
			<div class="loading">Loading...</div>
		{:else if error}
			<div class="error">Error: {error}</div>
		{:else if tracks.length === 0}
			<div class="empty">
				<p>No tracks found.</p>
				<p class="empty-hint">
					<a href="/tracks/new">[+ Upload your first track]</a>
				</p>
			</div>
		{:else}
			<div class="list">
				{#each tracks as track (track.id)}
					<TrackRow {track} />
				{/each}
			</div>
		{/if}
	</div>
</div>


<style>
	.page {
		display: flex;
		justify-content: center;
		align-items: center;
		flex-direction: column;
		height: calc(100% - var(--header-height) - var(--space-lg));
		margin-top: calc(var(--header-height) + var(--space-lg));
		box-sizing: border-box;
	}
	.tracks, .collections {
		padding: var(--space-lg) 0;
		width: 100%;
	}

	.page-header {
		display: flex;
		align-items: baseline;
		gap: var(--space-sm);
		margin-bottom: var(--space-lg);
	}

	.page-title {
		font-weight: normal;
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

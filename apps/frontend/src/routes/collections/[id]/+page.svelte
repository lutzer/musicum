<script lang="ts">
	import { page } from '$app/stores';
	import { getCollection } from '$lib/api/collections';
	import { authStore } from '$lib/stores/auth.svelte';
	import { UserRole, type CollectionDetailResponse } from '$lib/types';

	let canEdit = $derived(
		authStore.user?.id === collection?.user_id || authStore.user?.role === UserRole.ADMIN
	);

	let collection = $state<CollectionDetailResponse | null>(null);
	let error = $state('');
	let loading = $state(true);

	$effect(() => {
		const id = $page.params.id;
		if (id) {
			loadCollection(parseInt(id, 10));
		}
	});

	async function loadCollection(id: number) {
		loading = true;
		error = '';
		try {
			collection = await getCollection(id);
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to load collection';
		} finally {
			loading = false;
		}
	}

	function formatDate(dateString: string): string {
		const date = new Date(dateString);
		return date.toLocaleDateString('en-US', {
			year: 'numeric',
			month: 'short',
			day: 'numeric'
		});
	}

	function formatDuration(seconds: number | null): string {
		if (seconds === null) return '--:--';
		const mins = Math.floor(seconds / 60);
		const secs = Math.floor(seconds % 60);
		return `${mins}:${secs.toString().padStart(2, '0')}`;
	}
</script>

<div class="collection-page">
	{#if loading}
		<p>Loading...</p>
	{:else if error}
		<p class="error">{error}</p>
		<p><a href="/collections">[&lt; back to collections]</a></p>
	{:else if collection}
		<header class="collection-header">
			<h1>{collection.name}</h1>
			<span class="badge" class:private={!collection.is_public}>
				{collection.is_public ? 'public' : 'private'}
			</span>
		</header>

		<div class="collection-meta">
			<span>[#] {collection.tracks.length} track{collection.tracks.length !== 1 ? 's' : ''}</span>
			<span>{formatDate(collection.created_at)}</span>
		</div>

		{#if collection.description}
			<p class="collection-description">{collection.description}</p>
		{/if}

		{#if collection.tracks.length > 0}
			<div class="tracks-section">
				<h2>Tracks</h2>
				<ol class="tracks-list">
					{#each collection.tracks.toSorted((a, b) => a.position - b.position) as collectionTrack}
						<li class="track-item">
							<span class="track-position">{collectionTrack.position}.</span>
							<a href="/tracks/{collectionTrack.track.slug}" class="track-link">
								{collectionTrack.track.title}
							</a>
							<span class="track-duration"
								>{formatDuration(collectionTrack.track.duration_seconds)}</span
							>
						</li>
					{/each}
				</ol>
			</div>
		{:else}
			<p class="no-tracks">No tracks in this collection yet.</p>
		{/if}

		<div class="collection-actions">
			<a href="/collections">[&lt; back]</a>
			{#if canEdit}
				<a href="/collections/{collection.id}/edit">[edit]</a>
			{/if}
		</div>
	{/if}
</div>

<style>
	.collection-page {
		width: 100%;
		max-width: 600px;
	}

	.collection-header {
		display: flex;
		align-items: center;
		gap: var(--space-md);
		margin-bottom: var(--space-md);
	}

	.collection-header h1 {
		margin: 0;
	}

	.badge {
		font-size: 0.9em;
	}

	.badge.private {
		opacity: 0.7;
	}

	.collection-meta {
		display: flex;
		flex-wrap: wrap;
		gap: var(--space-md);
		margin-bottom: var(--space-md);
		opacity: 0.8;
	}

	.collection-description {
		margin-bottom: var(--space-md);
	}

	.tracks-section {
		margin-bottom: var(--space-md);
	}

	.tracks-section h2 {
		margin-bottom: var(--space-sm);
	}

	.tracks-list {
		list-style: none;
		padding: 0;
		margin: 0;
		display: flex;
		flex-direction: column;
		gap: var(--space-xs);
	}

	.track-item {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		padding: var(--space-sm);
		border: 1px solid;
	}

	.track-position {
		opacity: 0.7;
		min-width: 2em;
	}

	.track-link {
		flex: 1;
	}

	.track-duration {
		opacity: 0.7;
	}

	.no-tracks {
		opacity: 0.7;
		margin-bottom: var(--space-md);
	}

	.collection-actions {
		margin-top: var(--space-lg);
	}

	.error {
		color: inherit;
	}
</style>

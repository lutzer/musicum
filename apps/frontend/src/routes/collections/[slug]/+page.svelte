<script lang="ts">
	import { page } from '$app/stores';
	import { getCollectionBySlug } from '$lib/api/collections';
	import { authStore } from '$lib/stores/auth.svelte';
	import { UserRole, type CollectionDetailResponse } from '$lib/types';
	import TrackRow from '$lib/components/TrackRow.svelte';

	let collection = $state<CollectionDetailResponse | null>(null);
	let error = $state('');
	let loading = $state(true);

	let canEdit = $derived(
		authStore.user?.id === collection?.user_id || authStore.user?.role === UserRole.ADMIN
	);

	$effect(() => {
		const slug = $page.params.slug;
		if (slug) {
			loadCollection(slug);
		}
	});

	async function loadCollection(slug: string) {
		loading = true;
		error = '';
		try {
			collection = await getCollectionBySlug(slug);
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

<div class="collection-page center">
	{#if loading}
		<p>Loading...</p>
	{:else if error}
		<p class="error">{error}</p>
		<p><a href="/">[&lt; back to collections]</a></p>
	{:else if collection}
		<header class="collection-header">
			<h1>{collection.title}</h1>
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
				<ul>
					{#each collection.tracks.toSorted((a, b) => a.position - b.position) as collectionTrack}
						<li>
							<TrackRow track={collectionTrack.track} />
						</li>
					{/each}
				</ul>
			</div>
		{:else}
			<p class="no-tracks">No tracks in this collection yet.</p>
		{/if}

		<div class="collection-actions">
			<a href="/">[&lt; back]</a>
			{#if canEdit}
				<a href="/collection/{collection.slug}/edit">[edit]</a>
			{/if}
		</div>
	{/if}
</div>

<style>
	.collection-page {
		width: 100%;
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

<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { listTracks } from '$lib/api/tracks';
	import type { TrackResponse } from '$lib/types';
	import TrackRow from '$lib/components/TrackRow.svelte';
	import TagFilter from '$lib/components/TagFilter.svelte';
	import { authStore } from '$lib/stores/auth.svelte';

	let tracks = $state<TrackResponse[]>([]);
	let loading = $state(true);
	let error = $state<string | null>(null);
	let selectedTag = $state<string | null>(null);

	let allTags = $derived(() => {
		const tagsSet = new Set<string>();
		for (const track of tracks) {
			if (track.tags) {
				for (const tag of track.tags.split(',')) {
					const trimmed = tag.trim();
					if (trimmed) tagsSet.add(trimmed);
				}
			}
		}
		return Array.from(tagsSet).sort();
	});

	let filteredTracks = $derived(() => {
		const tag = selectedTag;
		if (!tag) return tracks;
		return tracks.filter((t) => t.tags && t.tags.toLowerCase().includes(tag.toLowerCase()));
	});

	onMount(async () => {
		if (!authStore.isAuthenticated) {
			goto('/login');
			return;
		}

		try {
			const userId = authStore.user?.id;
			const response = await listTracks(userId ? { user_id: userId } : undefined);
			tracks = response.items;
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load tracks';
		} finally {
			loading = false;
		}
	});
</script>

<div class="page">
	<div class="page-header">
		<h1 class="page-title">My Tracks</h1>
		<span class="page-count">[{filteredTracks().length}]</span>
		<a href="/tracks/new" class="page-action">[+ new]</a>
	</div>

	{#if allTags().length > 0}
		<div class="filter-section">
			<TagFilter tags={allTags()} bind:selectedTag />
		</div>
	{/if}

	{#if loading}
		<div class="loading">Loading...</div>
	{:else if error}
		<div class="error">Error: {error}</div>
	{:else if filteredTracks().length === 0}
		<div class="empty">
			<p>No tracks found.</p>
			<p class="empty-hint">
				<a href="/tracks/new">[+ Upload your first track]</a>
			</p>
		</div>
	{:else}
		<div class="list">
			{#each filteredTracks() as track (track.id)}
				<TrackRow {track} />
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

	.page-action {
		margin-left: auto;
	}

	.filter-section {
		margin-bottom: var(--space-md);
	}

	.list {
		display: flex;
		flex-direction: column;
	}

	.empty-hint {
		margin-top: var(--space-sm);
	}
</style>

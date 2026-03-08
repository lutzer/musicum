<script lang="ts">
	import { onMount } from 'svelte';
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
		try {
			// Show public tracks (no user_id filter)
			const response = await listTracks();
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
		<h1 class="page-title">Tracks</h1>
		<span class="page-count">[{filteredTracks().length}]</span>
		<a href="/" class="page-toggle">[view collections]</a>
		{#if authStore.isAuthenticated}
			<a href="/create_track" class="page-action">[+ new]</a>
		{/if}
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
			{#if authStore.isAuthenticated}
				<p class="empty-hint">
					<a href="/create_track">[+ Upload your first track]</a>
				</p>
			{/if}
		</div>
	{:else}
		<ul>
			{#each filteredTracks() as track (track.id)}
				<li>
					<TrackRow {track} />
				</li>
			{/each}
		</ul>
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

	.filter-section {
		margin-bottom: var(--space-md);
	}

	.empty-hint {
		margin-top: var(--space-sm);
	}
</style>

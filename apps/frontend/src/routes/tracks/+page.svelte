<script lang="ts">
	import { onMount } from 'svelte';
	import { listTracks } from '$lib/api/tracks';
	import type { TrackResponse } from '$lib/types';
	import TrackCard from '$lib/components/TrackCard.svelte';
	import { authStore } from '$lib/stores/auth.svelte';

	let tracks = $state<TrackResponse[]>([]);
	let loading = $state(true);
	let error = $state<string | null>(null);

	onMount(async () => {
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
		<h1 class="page-title">Tracks</h1>
		<span class="page-count">[{tracks.length}]</span>
		<a href="/tracks/new" class="page-action">[+ new]</a>
	</div>

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
		<div class="grid">
			{#each tracks as track (track.id)}
				<TrackCard {track} />
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

	.grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
		gap: var(--space-md);
	}

	.empty-hint {
		margin-top: var(--space-sm);
	}
</style>

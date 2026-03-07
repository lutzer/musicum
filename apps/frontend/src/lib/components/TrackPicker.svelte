<script lang="ts">
	import { listTracks } from '$lib/api/tracks';
	import type { TrackResponse } from '$lib/types';

	interface Props {
		excludeTrackIds: number[];
		onSelect: (track: TrackResponse) => void;
		onClose: () => void;
	}

	let { excludeTrackIds, onSelect, onClose }: Props = $props();

	let tracks = $state<TrackResponse[]>([]);
	let loading = $state(true);
	let error = $state('');
	let searchQuery = $state('');

	$effect(() => {
		loadTracks();
	});

	async function loadTracks() {
		loading = true;
		error = '';
		try {
			const response = await listTracks();
			tracks = response.items;
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to load tracks';
		} finally {
			loading = false;
		}
	}

	function formatDuration(seconds: number | null): string {
		if (seconds === null) return '--:--';
		const mins = Math.floor(seconds / 60);
		const secs = Math.floor(seconds % 60);
		return `${mins}:${secs.toString().padStart(2, '0')}`;
	}

	let filteredTracks = $derived.by(() => {
		const available = tracks.filter((t) => !excludeTrackIds.includes(t.id));
		if (!searchQuery.trim()) {
			return available;
		}
		const query = searchQuery.toLowerCase();
		return available.filter((t) => t.title.toLowerCase().includes(query));
	});
</script>

<div class="track-picker">
	<div class="picker-header">
		<h4>Add Track</h4>
		<button type="button" class="close-btn" onclick={onClose}>[x]</button>
	</div>

	{#if error}
		<p class="error">{error}</p>
	{/if}

	<div class="search-row">
		<input
			type="text"
			class="search-input"
			placeholder="Search tracks..."
			bind:value={searchQuery}
		/>
	</div>

	{#if loading}
		<p class="loading">Loading tracks...</p>
	{:else if filteredTracks.length === 0}
		<p class="no-tracks">
			{#if tracks.length === 0}
				No tracks available.
			{:else if excludeTrackIds.length === tracks.length}
				All tracks are already in the collection.
			{:else}
				No tracks match your search.
			{/if}
		</p>
	{:else}
		<ul class="tracks-list">
			{#each filteredTracks as track (track.id)}
				<li class="track-item">
					<button type="button" class="track-btn" onclick={() => onSelect(track)}>
						<span class="track-title">{track.title}</span>
						<span class="track-duration">{formatDuration(track.duration_seconds)}</span>
						<span class="add-icon">[+]</span>
					</button>
				</li>
			{/each}
		</ul>
	{/if}
</div>

<style>
	.track-picker {
		display: flex;
		flex-direction: column;
		gap: var(--space-sm);
		padding: var(--space-md);
		border: 1px solid;
	}

	.picker-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
	}

	.picker-header h4 {
		margin: 0;
	}

	.close-btn {
		font: inherit;
		border: none;
		background: none;
		padding: 0;
		cursor: pointer;
	}

	.error {
		padding: var(--space-sm);
		border: 1px solid;
	}

	.search-row {
		display: flex;
	}

	.search-input {
		flex: 1;
		padding: var(--space-sm);
		font: inherit;
		border: 1px solid;
		background: none;
	}

	.loading,
	.no-tracks {
		opacity: 0.7;
	}

	.tracks-list {
		list-style: none;
		padding: 0;
		margin: 0;
		display: flex;
		flex-direction: column;
		gap: var(--space-xs);
		max-height: 200px;
		overflow-y: auto;
	}

	.track-item {
		display: flex;
	}

	.track-btn {
		flex: 1;
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		padding: var(--space-sm);
		font: inherit;
		border: 1px solid;
		background: none;
		cursor: pointer;
		text-align: left;
	}

	.track-btn:hover {
		opacity: 0.8;
	}

	.track-title {
		flex: 1;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.track-duration {
		opacity: 0.7;
	}

	.add-icon {
		opacity: 0.7;
	}
</style>

<script lang="ts">
	import {
		listCollectionTracks,
		addTrackToCollection,
		removeTrackFromCollection,
		reorderTracks
	} from '$lib/api/collections';
	import type { CollectionTrackResponse, TrackResponse } from '$lib/types';
	import CollectionTrackItem from './CollectionTrackItem.svelte';
	import TrackPicker from './TrackPicker.svelte';

	interface Props {
		collectionId: number;
	}

	let { collectionId }: Props = $props();

	let tracks = $state<CollectionTrackResponse[]>([]);
	let loading = $state(true);
	let error = $state('');
	let showPicker = $state(false);

	let draggedIndex = $state<number | null>(null);

	$effect(() => {
		loadTracks();
	});

	async function loadTracks() {
		loading = true;
		error = '';
		try {
			tracks = await listCollectionTracks(collectionId);
			tracks = tracks.sort((a, b) => a.position - b.position);
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to load tracks';
		} finally {
			loading = false;
		}
	}

	async function handleAddTrack(track: TrackResponse) {
		error = '';
		try {
			await addTrackToCollection(collectionId, {
				track_id: track.id,
				position: tracks.length + 1
			});
			await loadTracks();
			showPicker = false;
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to add track';
		}
	}

	async function handleRemoveTrack(trackId: number) {
		error = '';
		try {
			await removeTrackFromCollection(collectionId, trackId);
			await loadTracks();
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to remove track';
		}
	}

	function handleDragStart(event: DragEvent, index: number) {
		draggedIndex = index;
		if (event.dataTransfer) {
			event.dataTransfer.effectAllowed = 'move';
		}
	}

	function handleDragOver(event: DragEvent, index: number) {
		event.preventDefault();
		if (draggedIndex === null || draggedIndex === index) return;

		const reordered = [...tracks];
		const [dragged] = reordered.splice(draggedIndex, 1);
		reordered.splice(index, 0, dragged);
		tracks = reordered;
		draggedIndex = index;
	}

	async function handleDragEnd() {
		if (draggedIndex === null) return;

		const newOrder = tracks.map((t) => t.track_id);
		draggedIndex = null;

		try {
			await reorderTracks(collectionId, { track_ids: newOrder });
			await loadTracks();
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to reorder tracks';
			await loadTracks();
		}
	}

	let excludeTrackIds = $derived(tracks.map((t) => t.track_id));
</script>

<div class="collection-track-manager">
	<h3>Tracks</h3>

	{#if error}
		<p class="error">{error}</p>
	{/if}

	{#if loading}
		<p>Loading tracks...</p>
	{:else}
		<div class="tracks-list">
			{#each tracks as collectionTrack, index (collectionTrack.id)}
				<div
					role="listitem"
					ondragstart={(e) => handleDragStart(e, index)}
					ondragover={(e) => handleDragOver(e, index)}
					ondragend={handleDragEnd}
				>
					<CollectionTrackItem {collectionTrack} onRemove={handleRemoveTrack} />
				</div>
			{/each}
		</div>

		{#if tracks.length === 0}
			<p class="no-tracks">No tracks in this collection.</p>
		{/if}
	{/if}

	{#if showPicker}
		<TrackPicker {excludeTrackIds} onSelect={handleAddTrack} onClose={() => (showPicker = false)} />
	{:else}
		<button type="button" class="add-track-btn" onclick={() => (showPicker = true)}>
			[+ add track]
		</button>
	{/if}
</div>

<style>
	.collection-track-manager {
		display: flex;
		flex-direction: column;
		gap: var(--space-md);
	}

	h3 {
		margin: 0;
	}

	.error {
		color: inherit;
		padding: var(--space-sm);
		border: 1px solid;
	}

	.tracks-list {
		display: flex;
		flex-direction: column;
		gap: var(--space-sm);
	}

	.no-tracks {
		opacity: 0.7;
	}

	.add-track-btn {
		padding: var(--space-sm);
		font: inherit;
		border: 1px solid;
		background: none;
		cursor: pointer;
	}
</style>

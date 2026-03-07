<script lang="ts">
	import type { CollectionTrackResponse } from '$lib/types';

	interface Props {
		collectionTrack: CollectionTrackResponse;
		onRemove: (trackId: number) => void;
		draggable?: boolean;
	}

	let { collectionTrack, onRemove, draggable = true }: Props = $props();

	function formatDuration(seconds: number | null): string {
		if (seconds === null) return '--:--';
		const mins = Math.floor(seconds / 60);
		const secs = Math.floor(seconds % 60);
		return `${mins}:${secs.toString().padStart(2, '0')}`;
	}
</script>

<div class="collection-track-item" {draggable}>
	{#if draggable}
		<span class="drag-handle">[=] </span>
	{/if}

	<span class="track-position">{collectionTrack.position}.</span>

	<div class="track-info">
		<span class="track-title">{collectionTrack.track.title}</span>
	</div>

	<span class="track-duration">{formatDuration(collectionTrack.track.duration_seconds)}</span>

	<button type="button" class="remove-btn" onclick={() => onRemove(collectionTrack.track_id)}>
		[x]
	</button>
</div>

<style>
	.collection-track-item {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		padding: var(--space-sm);
		border: 1px solid;
	}

	.drag-handle {
		cursor: grab;
		user-select: none;
	}

	.track-position {
		opacity: 0.7;
		min-width: 2em;
	}

	.track-info {
		flex: 1;
		min-width: 0;
	}

	.track-title {
		display: block;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.track-duration {
		opacity: 0.7;
	}

	.remove-btn {
		font: inherit;
		border: none;
		background: none;
		padding: 0;
		cursor: pointer;
		opacity: 0.7;
	}

	.remove-btn:hover {
		opacity: 1;
	}
</style>

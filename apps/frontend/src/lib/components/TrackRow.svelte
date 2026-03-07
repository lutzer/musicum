<script lang="ts">
	import type { TrackResponse } from '$lib/types';

	interface Props {
		track: TrackResponse;
	}

	let { track }: Props = $props();

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

<a href="/tracks/{track.slug}" class="row">
	<span class="row-icon">[~]</span>
	<span class="row-title">{track.title}</span>
	<span class="row-duration">{formatDuration(track.duration_seconds)}</span>
	<span class="row-date">{formatDate(track.created_at)}</span>
	{#if track.is_public}
		<span class="row-badge">public</span>
	{/if}
</a>

<style>
	.row {
		display: flex;
		align-items: center;
		gap: var(--space-md);
		padding: var(--space-sm) 0;
		border-bottom: 1px solid;
		color: inherit;
	}

	.row-icon {
		flex-shrink: 0;
	}

	.row-title {
		flex: 1;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.row-duration {
		flex-shrink: 0;
		width: 50px;
		text-align: right;
	}

	.row-date {
		flex-shrink: 0;
		width: 100px;
	}

	.row-badge {
		flex-shrink: 0;
		width: 50px;
		text-align: right;
	}
</style>

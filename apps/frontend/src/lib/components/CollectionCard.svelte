<script lang="ts">
	import type { CollectionResponse } from '$lib/types';

	interface Props {
		collection: CollectionResponse;
	}

	let { collection }: Props = $props();

	function formatDate(dateString: string): string {
		const date = new Date(dateString);
		return date.toLocaleDateString('en-US', {
			year: 'numeric',
			month: 'short',
			day: 'numeric'
		});
	}
</script>

<a href="/collection/{collection.slug}" class="card">
	<div class="card-header">
		<span class="card-icon">[*]</span>
		<h3 class="card-title">{collection.title}</h3>
	</div>

	{#if collection.description}
		<p class="card-description">{collection.description}</p>
	{/if}

	<div class="card-footer">
		<span class="card-date">{formatDate(collection.created_at)}</span>
		{#if collection.is_public}
			<span class="card-badge">public</span>
		{:else}
			<span class="card-badge private">private</span>
		{/if}
	</div>
</a>

<style>
	.card {
		display: block;
		border: 1px solid;
		padding: var(--space-md);
		color: inherit;
		width: 33%;
		min-width: 250px;
		flex-grow: 1;
	}

	.card-header {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		margin-bottom: var(--space-sm);
	}

	.card-icon {
		flex-shrink: 0;
	}

	.card-title {
		font-size: inherit;
		font-weight: normal;
		white-space: nowrap;
		overflow: hidden;
	}

	.card-description {
		margin-bottom: var(--space-md);
		display: -webkit-box;
		-webkit-line-clamp: 2;
		line-clamp: 2;
		-webkit-box-orient: vertical;
		overflow: hidden;
	}

	.card-footer {
		display: flex;
		justify-content: space-between;
		align-items: center;
	}
</style>

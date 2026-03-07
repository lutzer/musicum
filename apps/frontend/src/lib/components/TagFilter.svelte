<script lang="ts">
	interface Props {
		tags: string[];
		selectedTag: string | null;
		onselect?: (tag: string | null) => void;
	}

	let { tags, selectedTag = $bindable(null), onselect }: Props = $props();

	function handleSelect(tag: string | null) {
		selectedTag = tag;
		onselect?.(tag);
	}
</script>

<div class="tag-filter">
	<button
		type="button"
		class="tag-button"
		class:selected={selectedTag === null}
		onclick={() => handleSelect(null)}
	>
		[All]
	</button>
	{#each tags as tag}
		<button
			type="button"
			class="tag-button"
			class:selected={selectedTag === tag}
			onclick={() => handleSelect(tag)}
		>
			[{tag}]
		</button>
	{/each}
</div>

<style>
	.tag-filter {
		display: flex;
		flex-wrap: wrap;
		gap: var(--space-sm);
	}

	.tag-button {
		font: inherit;
		background: none;
		border: none;
		cursor: pointer;
		padding: 0;
	}

	.tag-button.selected {
		text-decoration: underline;
	}
</style>

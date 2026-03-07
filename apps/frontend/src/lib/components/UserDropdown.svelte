<script lang="ts">
	import { authStore } from '$lib/stores/auth.svelte';

	let isOpen = $state(false);

	function toggle() {
		isOpen = !isOpen;
	}

	function close() {
		isOpen = false;
	}

	function handleLogout() {
		authStore.clearUser();
		close();
	}

	function handleClickOutside(event: MouseEvent) {
		const target = event.target as HTMLElement;
		if (!target.closest('.dropdown')) {
			close();
		}
	}

	$effect(() => {
		if (isOpen) {
			document.addEventListener('click', handleClickOutside);
			return () => {
				document.removeEventListener('click', handleClickOutside);
			};
		}
	});
</script>

<div class="dropdown">
	<button type="button" class="dropdown-trigger" onclick={toggle}>
		{authStore.user?.username}
	</button>

	{#if isOpen}
		<div class="dropdown-menu">
			<a href="/user/collections" class="dropdown-item" onclick={close}> [C] My Collections </a>
			<a href="/user/tracks" class="dropdown-item" onclick={close}> [T] My Tracks </a>
			<div class="dropdown-divider"></div>
			<button type="button" class="dropdown-item" onclick={handleLogout}> [X] Logout </button>
		</div>
	{/if}
</div>

<style>
	.dropdown {
		position: relative;
	}

	.dropdown-trigger {
		font: inherit;
		background: none;
		border: none;
		cursor: pointer;
		padding: 0;
	}

	.dropdown-menu {
		position: absolute;
		top: 100%;
		right: 0;
		z-index: 100;
		min-width: 160px;
		margin-top: var(--space-xs);
		padding: var(--space-xs) 0;
		background: var(--color-bg, #fff);
		border: 1px solid;
	}

	.dropdown-item {
		display: block;
		width: 100%;
		padding: var(--space-xs) var(--space-sm);
		font: inherit;
		background: none;
		border: none;
		cursor: pointer;
		text-align: left;
		color: inherit;
	}

	.dropdown-divider {
		margin: var(--space-xs) 0;
		border-top: 1px solid;
	}
</style>

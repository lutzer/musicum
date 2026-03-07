<script lang="ts">
	import { onMount } from 'svelte';
	import { listUsers } from '$lib/api/users';
	import type { UserResponse } from '$lib/types';

	let users = $state<UserResponse[]>([]);
	let loading = $state(true);
	let error = $state<string | null>(null);
	let total = $state(0);
	let page = $state(1);
	let pageSize = $state(20);

	onMount(() => {
		loadUsers();
	});

	async function loadUsers() {
		loading = true;
		error = null;
		try {
			const response = await listUsers({ page, page_size: pageSize });
			users = response.items;
			total = response.total;
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load users';
		} finally {
			loading = false;
		}
	}

	function formatDate(dateString: string): string {
		const date = new Date(dateString);
		return date.toLocaleDateString('en-US', {
			year: 'numeric',
			month: 'short',
			day: 'numeric'
		});
	}

	function goToPage(newPage: number) {
		page = newPage;
		loadUsers();
	}

	const totalPages = $derived(Math.ceil(total / pageSize));
</script>

<div class="page">
	<div class="page-header">
		<h1 class="page-title">Users</h1>
		<span class="page-count">[{total}]</span>
	</div>

	{#if loading}
		<div class="loading">Loading...</div>
	{:else if error}
		<div class="error">Error: {error}</div>
	{:else if users.length === 0}
		<div class="empty">
			<p>No users found.</p>
		</div>
	{:else}
		<div class="users-list">
			{#each users as user (user.id)}
				<div class="user-item">
					<span class="user-name">{user.username}</span>
					<span class="user-email">{user.email}</span>
					<span class="user-role">[{user.role}]</span>
					<span class="user-date">{formatDate(user.created_at)}</span>
				</div>
			{/each}
		</div>

		{#if totalPages > 1}
			<div class="pagination">
				<button type="button" disabled={page <= 1} onclick={() => goToPage(page - 1)}>
					[&lt; prev]
				</button>
				<span class="page-info">page {page} of {totalPages}</span>
				<button type="button" disabled={page >= totalPages} onclick={() => goToPage(page + 1)}>
					[next &gt;]
				</button>
			</div>
		{/if}
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

	.users-list {
		display: flex;
		flex-direction: column;
		gap: var(--space-sm);
	}

	.user-item {
		display: flex;
		flex-wrap: wrap;
		gap: var(--space-md);
		padding: var(--space-sm);
		border: 1px solid;
	}

	.user-name {
		font-weight: bold;
	}

	.user-email {
		opacity: 0.8;
	}

	.user-role {
		opacity: 0.7;
	}

	.user-date {
		margin-left: auto;
		opacity: 0.6;
	}

	.pagination {
		display: flex;
		align-items: center;
		gap: var(--space-md);
		margin-top: var(--space-lg);
	}

	.pagination button {
		font: inherit;
		border: none;
		background: none;
		padding: 0;
		cursor: pointer;
	}

	.pagination button:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.page-info {
		opacity: 0.8;
	}
</style>

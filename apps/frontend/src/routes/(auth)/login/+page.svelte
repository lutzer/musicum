<script lang="ts">
	import { goto } from '$app/navigation';
	import { login, getCurrentUser } from '$lib/api/auth';
	import { authStore } from '$lib/stores/auth.svelte';
	import FormField from '$lib/components/forms/FormField.svelte';
	import FormMessage from '$lib/components/forms/FormMessage.svelte';

	let email = $state('');
	let password = $state('');
	let error = $state('');
	let isSubmitting = $state(false);

	async function handleSubmit(event: Event) {
		event.preventDefault();
		error = '';
		isSubmitting = true;

		try {
			await login({ email, password });
			const user = await getCurrentUser();
			authStore.setUser(user);
			goto('/');
		} catch (err) {
			error = err instanceof Error ? err.message : 'Login failed';
		} finally {
			isSubmitting = false;
		}
	}
</script>

<div class="login-page">
	<h1>Login</h1>

	<form onsubmit={handleSubmit} class="login-form">
		<FormMessage type="error" message={error} />

		<FormField label="Email" type="email" name="email" bind:value={email} required />

		<FormField label="Password" type="password" name="password" bind:value={password} required />

		<button type="submit" class="submit-button" disabled={isSubmitting}>
			{isSubmitting ? '[logging in...]' : '[login]'}
		</button>

		<p class="form-footer">
			Don't have an account? <a href="/register">[register]</a>
		</p>
	</form>
</div>

<style>
	.login-page {
		width: 100%;
		max-width: 400px;
	}

	h1 {
		margin-bottom: var(--space-lg);
	}

	.login-form {
		display: flex;
		flex-direction: column;
		gap: var(--space-md);
	}

	.submit-button {
		padding: var(--space-sm);
		font: inherit;
		border: 1px solid;
		background: none;
		cursor: pointer;
	}

	.submit-button:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.form-footer {
		text-align: center;
	}
</style>

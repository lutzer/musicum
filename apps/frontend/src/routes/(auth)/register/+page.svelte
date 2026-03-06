<script lang="ts">
	import { goto } from '$app/navigation';
	import { register } from '$lib/api/auth';
	import FormField from '$lib/components/forms/FormField.svelte';
	import FormMessage from '$lib/components/forms/FormMessage.svelte';

	let username = $state('');
	let email = $state('');
	let password = $state('');
	let error = $state('');
	let success = $state('');
	let isSubmitting = $state(false);

	async function handleSubmit(event: Event) {
		event.preventDefault();
		error = '';
		success = '';
		isSubmitting = true;

		try {
			await register({ username, email, password });
			success = 'Registration successful! Redirecting to login...';
			setTimeout(() => {
				goto('/login');
			}, 2000);
		} catch (err) {
			error = err instanceof Error ? err.message : 'Registration failed';
		} finally {
			isSubmitting = false;
		}
	}
</script>

<div class="register-page">
	<h1>Register</h1>

	<form onsubmit={handleSubmit} class="register-form">
		<FormMessage type="error" message={error} />
		<FormMessage type="success" message={success} />

		<FormField label="Username" type="text" name="username" bind:value={username} required />

		<FormField label="Email" type="email" name="email" bind:value={email} required />

		<FormField label="Password" type="password" name="password" bind:value={password} required />

		<button type="submit" class="submit-button" disabled={isSubmitting}>
			{isSubmitting ? '[registering...]' : '[register]'}
		</button>

		<p class="form-footer">
			Already have an account? <a href="/login">[login]</a>
		</p>
	</form>
</div>

<style>
	.register-page {
		width: 100%;
		max-width: 400px;
	}

	h1 {
		margin-bottom: var(--space-lg);
	}

	.register-form {
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

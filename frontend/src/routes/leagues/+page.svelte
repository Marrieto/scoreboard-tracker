<script lang="ts">
	import { onMount } from 'svelte';
	import { getLeagues, createLeague, updateLeague, closeLeague, type League } from '$lib/api';
	import { getAuth } from '$lib/stores/auth.svelte';
	import { loadLeagues } from '$lib/stores/league.svelte';

	const auth = getAuth();
	let leagues = $state<League[]>([]);
	let loading = $state(true);
	let error = $state('');

	// Create form state
	let showCreate = $state(false);
	let newId = $state('');
	let newName = $state('');
	let newDescription = $state('');
	let creating = $state(false);

	// Edit state
	let editingId = $state<string | null>(null);
	let editName = $state('');
	let editDescription = $state('');
	let saving = $state(false);

	onMount(async () => {
		await fetchLeagues();
	});

	async function fetchLeagues() {
		loading = true;
		try {
			leagues = await getLeagues();
		} catch (e: any) {
			error = e.message;
		} finally {
			loading = false;
		}
	}

	async function handleCreate() {
		if (!newId || !newName) return;
		creating = true;
		error = '';
		try {
			await createLeague({
				id: newId.toLowerCase().replace(/[^a-z0-9-]/g, '-'),
				name: newName,
				description: newDescription,
			});
			showCreate = false;
			newId = '';
			newName = '';
			newDescription = '';
			await fetchLeagues();
			await loadLeagues(); // Refresh global store
		} catch (e: any) {
			error = e.message;
		} finally {
			creating = false;
		}
	}

	function startEdit(league: League) {
		editingId = league.id;
		editName = league.name;
		editDescription = league.description;
	}

	async function handleSaveEdit() {
		if (!editingId || !editName) return;
		saving = true;
		try {
			await updateLeague(editingId, { name: editName, description: editDescription });
			editingId = null;
			await fetchLeagues();
			await loadLeagues();
		} catch (e: any) {
			error = e.message;
		} finally {
			saving = false;
		}
	}

	async function handleClose(id: string) {
		if (!confirm('Close this league? New matches can no longer be assigned to it.')) return;
		try {
			await closeLeague(id);
			await fetchLeagues();
			await loadLeagues();
		} catch (e: any) {
			error = e.message;
		}
	}

	function canManage(league: League): boolean {
		if (auth.info.role === 'admin') return true;
		return auth.info.user_id === league.created_by;
	}
</script>

<div class="page-header">
	<div>
		<h1 class="page-title">LEAGUES</h1>
		<p class="page-subtitle">Seasons, tournaments, and bragging rights.</p>
	</div>
	{#if auth.info.authenticated}
		<button class="btn btn-primary" onclick={() => showCreate = !showCreate}>
			{showCreate ? 'Cancel' : '+ New League'}
		</button>
	{/if}
</div>

{#if error}
	<div class="error-banner">{error}</div>
{/if}

<!-- Create form -->
{#if showCreate}
	<form class="create-form card" onsubmit={(e) => { e.preventDefault(); handleCreate(); }}>
		<div class="form-row">
			<div class="form-field">
				<label class="field-label">ID (slug)</label>
				<input
					type="text"
					bind:value={newId}
					class="input"
					placeholder="spring-2026"
					required
				/>
			</div>
			<div class="form-field">
				<label class="field-label">Name</label>
				<input
					type="text"
					bind:value={newName}
					class="input"
					placeholder="Spring 2026 Season"
					required
				/>
			</div>
		</div>
		<div class="form-field">
			<label class="field-label">Description <span class="optional">(optional)</span></label>
			<input
				type="text"
				bind:value={newDescription}
				class="input"
				placeholder="Casual games, March through May"
			/>
		</div>
		<button type="submit" class="btn btn-primary" disabled={creating || !newId || !newName}>
			{creating ? 'Creating...' : 'Create League'}
		</button>
	</form>
{/if}

{#if loading}
	<div class="loading-state">
		<span class="loading-spinner"></span>
		<span>Loading leagues...</span>
	</div>
{:else if leagues.length === 0}
	<div class="empty-state card">
		<span class="empty-icon">🏅</span>
		<h2>No leagues yet</h2>
		<p>Create a league to start tracking seasons!</p>
	</div>
{:else}
	<div class="league-list">
		{#each leagues as league}
			<div class="league-card card" class:closed={league.status === 'closed'}>
				{#if editingId === league.id}
					<div class="edit-form">
						<input bind:value={editName} class="input" placeholder="League name" />
						<input bind:value={editDescription} class="input" placeholder="Description" />
						<div class="edit-actions">
							<button class="btn btn-primary btn-sm" disabled={saving || !editName} onclick={handleSaveEdit}>
								{saving ? 'Saving...' : 'Save'}
							</button>
							<button class="btn btn-ghost btn-sm" onclick={() => editingId = null}>Cancel</button>
						</div>
					</div>
				{:else}
					<div class="league-header">
						<div>
							<div class="league-name">
								{league.name}
								<span class="league-status" class:active={league.status === 'active'} class:status-closed={league.status === 'closed'}>
									{league.status}
								</span>
							</div>
							{#if league.description}
								<p class="league-desc">{league.description}</p>
							{/if}
							<div class="league-meta">
								<span>Created {new Date(league.created_at).toLocaleDateString()}</span>
								{#if league.closed_at}
									<span>Closed {new Date(league.closed_at).toLocaleDateString()}</span>
								{/if}
							</div>
						</div>
						{#if canManage(league)}
							<div class="league-actions">
								<button class="btn btn-ghost btn-sm" onclick={() => startEdit(league)}>Edit</button>
								{#if league.status === 'active'}
									<button class="btn btn-ghost btn-sm close-btn" onclick={() => handleClose(league.id)}>Close</button>
								{/if}
							</div>
						{/if}
					</div>
				{/if}
			</div>
		{/each}
	</div>
{/if}

<style>
	.page-header {
		display: flex;
		align-items: flex-start;
		justify-content: space-between;
		margin-bottom: 2rem;
		gap: 1rem;
		flex-wrap: wrap;
	}
	.page-title {
		font-family: var(--font-display);
		font-size: 3rem;
		font-weight: 900;
		letter-spacing: 0.03em;
		line-height: 1;
	}
	.page-subtitle {
		color: var(--text-muted);
		font-size: 0.95rem;
		margin-top: 0.4rem;
	}

	.create-form {
		padding: 1.25rem;
		margin-bottom: 1.5rem;
		display: flex;
		flex-direction: column;
		gap: 0.75rem;
		border-color: rgba(180, 247, 74, 0.15);
	}
	.form-row {
		display: flex;
		gap: 0.75rem;
	}
	.form-field {
		flex: 1;
		display: flex;
		flex-direction: column;
		gap: 0.3rem;
	}
	.field-label {
		font-size: 0.78rem;
		font-weight: 700;
		color: var(--text-secondary);
		text-transform: uppercase;
		letter-spacing: 0.04em;
	}
	.optional {
		font-weight: 400;
		color: var(--text-muted);
		text-transform: none;
		letter-spacing: 0;
	}
	.input {
		padding: 0.5rem 0.7rem;
		background: var(--bg-surface);
		border: 1px solid rgba(255, 255, 255, 0.1);
		border-radius: var(--radius-sm);
		color: var(--text-primary);
		font-family: var(--font-body);
		font-size: 0.85rem;
	}
	.input:focus {
		outline: none;
		border-color: var(--neon-green);
	}

	.league-list {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
	}
	.league-card {
		padding: 1rem 1.25rem;
	}
	.league-card.closed {
		opacity: 0.6;
	}
	.league-header {
		display: flex;
		justify-content: space-between;
		align-items: flex-start;
		gap: 1rem;
	}
	.league-name {
		font-weight: 700;
		font-size: 1.05rem;
		display: flex;
		align-items: center;
		gap: 0.5rem;
	}
	.league-status {
		font-family: var(--font-mono);
		font-size: 0.65rem;
		font-weight: 700;
		padding: 0.1rem 0.4rem;
		border-radius: 100px;
		text-transform: uppercase;
		letter-spacing: 0.04em;
	}
	.league-status.active {
		background: rgba(180, 247, 74, 0.12);
		color: var(--neon-green);
	}
	.league-status.status-closed {
		background: rgba(255, 255, 255, 0.06);
		color: var(--text-muted);
	}
	.league-desc {
		color: var(--text-secondary);
		font-size: 0.85rem;
		margin-top: 0.25rem;
	}
	.league-meta {
		display: flex;
		gap: 1rem;
		margin-top: 0.35rem;
		font-size: 0.75rem;
		color: var(--text-muted);
		font-family: var(--font-mono);
	}
	.league-actions {
		display: flex;
		gap: 0.4rem;
		flex-shrink: 0;
	}
	.close-btn {
		color: var(--neon-red) !important;
	}

	.edit-form {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
	}
	.edit-actions {
		display: flex;
		gap: 0.4rem;
	}

	.error-banner {
		background: rgba(255, 59, 92, 0.12);
		border: 1px solid rgba(255, 59, 92, 0.25);
		color: var(--neon-red);
		padding: 0.65rem 1rem;
		border-radius: var(--radius-sm);
		font-size: 0.85rem;
		margin-bottom: 1rem;
	}

	.loading-state {
		display: flex; align-items: center; gap: 0.75rem;
		color: var(--text-secondary); padding: 3rem 0; justify-content: center;
	}
	.loading-spinner {
		width: 18px; height: 18px;
		border: 2px solid var(--text-muted);
		border-top-color: var(--neon-green);
		border-radius: 50%;
		animation: spin 0.7s linear infinite;
	}
	@keyframes spin { to { transform: rotate(360deg); } }
	.empty-state {
		text-align: center;
		padding: 3rem 2rem;
	}
	.empty-icon { font-size: 3rem; display: block; margin-bottom: 1rem; }
	.empty-state h2 {
		font-family: var(--font-display);
		font-size: 1.5rem;
		margin-bottom: 0.5rem;
	}
	.empty-state p { color: var(--text-secondary); }

	@media (max-width: 640px) {
		.form-row { flex-direction: column; }
		.league-header { flex-direction: column; }
	}
</style>

<script lang="ts">
	import { onMount } from 'svelte';
	import { getPlayers, createPlayer, deletePlayer, type Player } from '$lib/api';

	let players = $state<Player[]>([]);
	let loading = $state(true);
	let error = $state('');
	let showForm = $state(false);

	// New player form
	let newId = $state('');
	let newName = $state('');
	let newNickname = $state('');
	let newEmoji = $state('🏓');
	let formError = $state('');
	let submitting = $state(false);

	const emojiOptions = ['🏓', '🔥', '💀', '👑', '🎯', '💪', '🦅', '🐍', '🎪', '🌟', '🏆', '⚡'];

	onMount(async () => {
		await loadPlayers();
	});

	async function loadPlayers() {
		try {
			players = await getPlayers();
		} catch (e: any) {
			error = e.message;
		} finally {
			loading = false;
		}
	}

	async function handleCreate() {
		if (!newId || !newName) return;
		submitting = true;
		formError = '';
		try {
			await createPlayer({
				id: newId.toLowerCase().replace(/[^a-z0-9-]/g, ''),
				name: newName,
				nickname: newNickname,
				avatar_emoji: newEmoji,
			});
			showForm = false;
			newId = '';
			newName = '';
			newNickname = '';
			newEmoji = '🏓';
			await loadPlayers();
		} catch (e: any) {
			formError = e.message;
		} finally {
			submitting = false;
		}
	}

	async function handleDelete(id: string, name: string) {
		if (!confirm(`Delete ${name}? This can't be undone.`)) return;
		try {
			await deletePlayer(id);
			await loadPlayers();
		} catch (e: any) {
			error = e.message;
		}
	}
</script>

<div class="page-header">
	<div>
		<h1 class="page-title">PLAYERS</h1>
		<p class="page-subtitle">The warriors of the court.</p>
	</div>
	<button class="btn btn-primary" onclick={() => showForm = !showForm}>
		{showForm ? 'Cancel' : '+ Add Player'}
	</button>
</div>

{#if showForm}
	<form class="add-form card animate-in" onsubmit={(e) => { e.preventDefault(); handleCreate(); }}>
		{#if formError}
			<div class="error-banner">{formError}</div>
		{/if}
		<div class="form-row">
			<div class="form-field">
				<label for="new-id">ID (slug)</label>
				<input id="new-id" class="input" bind:value={newId} placeholder="e.g. martin" required />
			</div>
			<div class="form-field">
				<label for="new-name">Name</label>
				<input id="new-name" class="input" bind:value={newName} placeholder="e.g. Martin" required />
			</div>
		</div>
		<div class="form-row">
			<div class="form-field">
				<label for="new-nick">Nickname <span class="optional">(optional)</span></label>
				<input id="new-nick" class="input" bind:value={newNickname} placeholder="e.g. The Dinkmaster" />
			</div>
			<div class="form-field">
				<span class="form-label-text">Avatar</span>
				<div class="emoji-picker">
					{#each emojiOptions as emoji}
						<button
							type="button"
							class="emoji-btn"
							class:selected={newEmoji === emoji}
							onclick={() => newEmoji = emoji}
						>{emoji}</button>
					{/each}
				</div>
			</div>
		</div>
		<button type="submit" class="btn btn-primary" disabled={submitting || !newId || !newName}>
			{submitting ? 'Creating...' : 'Create Player'}
		</button>
	</form>
{/if}

{#if loading}
	<div class="loading-state">
		<span class="loading-spinner"></span>
		<span>Loading players...</span>
	</div>
{:else if error}
	<div class="error-state card"><p>{error}</p></div>
{:else}
	<div class="player-grid">
		{#each players as player, i}
			<a
				href="/players/{player.id}"
				class="player-card card animate-in"
				style="animation-delay: {i * 60}ms"
			>
				<div class="player-avatar">{player.avatar_emoji}</div>
				<div class="player-details">
					<h3 class="player-name">{player.name}</h3>
					{#if player.nickname}
						<p class="player-nickname">"{player.nickname}"</p>
					{/if}
					<p class="player-id">@{player.id}</p>
				</div>
				<button
					class="delete-btn"
					title="Delete player"
					onclick={(e) => { e.preventDefault(); handleDelete(player.id, player.name); }}
				>×</button>
			</a>
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
	.page-subtitle { color: var(--text-muted); font-size: 0.95rem; margin-top: 0.4rem; }

	.add-form {
		margin-bottom: 2rem;
		display: flex;
		flex-direction: column;
		gap: 1rem;
	}
	.form-row {
		display: flex;
		gap: 1rem;
	}
	.form-field {
		flex: 1;
		display: flex;
		flex-direction: column;
		gap: 0.35rem;
	}
	.form-field label {
		font-size: 0.8rem;
		font-weight: 600;
		color: var(--text-secondary);
		text-transform: uppercase;
		letter-spacing: 0.05em;
	}
	.optional {
		text-transform: none;
		letter-spacing: 0;
		color: var(--text-muted);
	}
	.input {
		width: 100%;
		padding: 0.6rem 0.85rem;
		background: var(--bg-surface);
		border: 1px solid rgba(255, 255, 255, 0.1);
		border-radius: var(--radius-sm);
		color: var(--text-primary);
		font-family: var(--font-body);
		font-size: 0.9rem;
	}
	.input:focus {
		outline: none;
		border-color: var(--neon-green);
	}

	.emoji-picker {
		display: flex;
		flex-wrap: wrap;
		gap: 0.3rem;
	}
	.emoji-btn {
		width: 36px;
		height: 36px;
		border-radius: var(--radius-sm);
		background: var(--bg-surface);
		border: 1px solid rgba(255, 255, 255, 0.08);
		font-size: 1.1rem;
		display: flex;
		align-items: center;
		justify-content: center;
		transition: all 0.15s;
	}
	.emoji-btn:hover { border-color: rgba(255, 255, 255, 0.2); }
	.emoji-btn.selected {
		border-color: var(--neon-green);
		background: var(--neon-green-dim);
		box-shadow: 0 0 10px rgba(180, 247, 74, 0.2);
	}

	.error-banner {
		background: rgba(255, 59, 92, 0.12);
		border: 1px solid rgba(255, 59, 92, 0.25);
		color: var(--neon-red);
		padding: 0.65rem 1rem;
		border-radius: var(--radius-sm);
		font-size: 0.85rem;
	}

	.player-grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(260px, 1fr));
		gap: 0.75rem;
	}
	.player-card {
		display: flex;
		align-items: center;
		gap: 1rem;
		padding: 1rem 1.25rem;
		position: relative;
		color: var(--text-primary);
	}
	.player-avatar {
		font-size: 2.5rem;
		flex-shrink: 0;
	}
	.player-details { flex: 1; min-width: 0; }
	.player-name {
		font-weight: 700;
		font-size: 1.1rem;
	}
	.player-nickname {
		color: var(--text-secondary);
		font-size: 0.8rem;
		font-style: italic;
		margin-top: 0.15rem;
	}
	.player-id {
		color: var(--text-muted);
		font-family: var(--font-mono);
		font-size: 0.75rem;
		margin-top: 0.15rem;
	}

	.delete-btn {
		position: absolute;
		top: 0.5rem;
		right: 0.5rem;
		width: 24px;
		height: 24px;
		border-radius: 50%;
		background: transparent;
		color: var(--text-muted);
		font-size: 1rem;
		display: flex;
		align-items: center;
		justify-content: center;
		opacity: 0;
		transition: all 0.15s;
	}
	.player-card:hover .delete-btn { opacity: 1; }
	.delete-btn:hover {
		background: rgba(255, 59, 92, 0.15);
		color: var(--neon-red);
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

	@media (max-width: 640px) {
		.form-row { flex-direction: column; }
	}
</style>

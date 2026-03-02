<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { getAuth } from '$lib/stores/auth.svelte';
	import {
		getUsers, updateUserRole, linkPlayer as linkPlayerApi,
		getPlayers, getLeagues, closeLeague,
		type User, type Player, type League,
	} from '$lib/api';
	import { loadLeagues } from '$lib/stores/league.svelte';

	const auth = getAuth();
	let users = $state<User[]>([]);
	let players = $state<Player[]>([]);
	let leagues = $state<League[]>([]);
	let loading = $state(true);
	let error = $state('');

	// Link player state
	let linkingOid = $state<string | null>(null);
	let linkPlayerId = $state('');

	onMount(async () => {
		// Redirect non-admins
		if (!auth.loading && (!auth.info.authenticated || auth.info.role !== 'admin')) {
			goto('/');
			return;
		}

		try {
			[users, players, leagues] = await Promise.all([
				getUsers(),
				getPlayers(),
				getLeagues(),
			]);
		} catch (e: any) {
			error = e.message;
		} finally {
			loading = false;
		}
	});

	async function toggleRole(user: User) {
		const newRole = user.role === 'admin' ? 'user' : 'admin';
		if (!confirm(`Change ${user.name} from ${user.role} to ${newRole}?`)) return;
		try {
			await updateUserRole(user.oid, newRole);
			users = await getUsers();
		} catch (e: any) {
			alert('Failed: ' + e.message);
		}
	}

	function startLinkPlayer(oid: string, currentPlayerId: string | null) {
		linkingOid = oid;
		linkPlayerId = currentPlayerId ?? '';
	}

	async function handleLinkPlayer() {
		if (!linkingOid) return;
		try {
			await linkPlayerApi(linkingOid, linkPlayerId || null);
			users = await getUsers();
			linkingOid = null;
		} catch (e: any) {
			alert('Failed: ' + e.message);
		}
	}

	async function handleCloseLeague(id: string) {
		if (!confirm('Close this league?')) return;
		try {
			await closeLeague(id);
			leagues = await getLeagues();
			await loadLeagues();
		} catch (e: any) {
			alert('Failed: ' + e.message);
		}
	}
</script>

<div class="page-header">
	<h1 class="page-title">ADMIN PANEL</h1>
	<p class="page-subtitle">With great power comes great responsibility.</p>
</div>

{#if loading}
	<div class="loading-state">
		<span class="loading-spinner"></span>
		<span>Loading...</span>
	</div>
{:else if error}
	<div class="error-state card"><p>{error}</p></div>
{:else}
	<!-- Users Section -->
	<section class="admin-section">
		<h2 class="section-title">👥 Users ({users.length})</h2>
		<div class="user-list">
			{#each users as user}
				<div class="user-row card">
					<div class="user-info">
						<div class="user-name">{user.name}</div>
						<div class="user-email">{user.email}</div>
						<div class="user-meta">
							<span class="role-badge" class:admin={user.role === 'admin'}>
								{user.role}
							</span>
							{#if user.player_id}
								<span class="player-link">
									Linked: {players.find(p => p.id === user.player_id)?.name ?? user.player_id}
								</span>
							{:else}
								<span class="no-link">No player linked</span>
							{/if}
						</div>
					</div>
					<div class="user-actions">
						<button class="btn btn-ghost btn-sm" onclick={() => toggleRole(user)}>
							{user.role === 'admin' ? 'Demote' : 'Promote'}
						</button>
						{#if linkingOid === user.oid}
							<div class="link-form">
								<select bind:value={linkPlayerId} class="select-sm">
									<option value="">None</option>
									{#each players as p}
										<option value={p.id}>{p.avatar_emoji} {p.name}</option>
									{/each}
								</select>
								<button class="btn btn-primary btn-sm" onclick={handleLinkPlayer}>Save</button>
								<button class="btn btn-ghost btn-sm" onclick={() => linkingOid = null}>X</button>
							</div>
						{:else}
							<button class="btn btn-ghost btn-sm" onclick={() => startLinkPlayer(user.oid, user.player_id)}>
								Link Player
							</button>
						{/if}
					</div>
				</div>
			{/each}
		</div>
	</section>

	<!-- Leagues Section -->
	<section class="admin-section">
		<h2 class="section-title">🏅 Leagues ({leagues.length})</h2>
		<div class="league-list">
			{#each leagues as league}
				<div class="league-row card" class:closed={league.status === 'closed'}>
					<div>
						<span class="league-name">{league.name}</span>
						<span class="status-tag" class:active={league.status === 'active'}>
							{league.status}
						</span>
					</div>
					{#if league.status === 'active'}
						<button class="btn btn-ghost btn-sm close-league-btn" onclick={() => handleCloseLeague(league.id)}>
							Close
						</button>
					{/if}
				</div>
			{/each}
		</div>
	</section>
{/if}

<style>
	.page-header {
		margin-bottom: 2rem;
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

	.admin-section {
		margin-bottom: 2.5rem;
	}
	.section-title {
		font-family: var(--font-display);
		font-size: 1.3rem;
		font-weight: 700;
		letter-spacing: 0.04em;
		color: var(--text-secondary);
		margin-bottom: 0.75rem;
	}

	.user-list, .league-list {
		display: flex;
		flex-direction: column;
		gap: 0.4rem;
	}
	.user-row {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: 0.85rem 1.1rem;
		gap: 1rem;
	}
	.user-name { font-weight: 700; }
	.user-email { font-size: 0.78rem; color: var(--text-muted); }
	.user-meta {
		display: flex;
		gap: 0.5rem;
		align-items: center;
		margin-top: 0.25rem;
	}
	.role-badge {
		font-family: var(--font-mono);
		font-size: 0.65rem;
		font-weight: 700;
		padding: 0.1rem 0.4rem;
		border-radius: 100px;
		text-transform: uppercase;
		background: rgba(255, 255, 255, 0.06);
		color: var(--text-muted);
	}
	.role-badge.admin {
		background: rgba(180, 247, 74, 0.12);
		color: var(--neon-green);
	}
	.player-link {
		font-size: 0.75rem;
		color: var(--text-secondary);
	}
	.no-link {
		font-size: 0.75rem;
		color: var(--text-muted);
		font-style: italic;
	}

	.user-actions {
		display: flex;
		align-items: center;
		gap: 0.4rem;
		flex-shrink: 0;
	}
	.link-form {
		display: flex;
		align-items: center;
		gap: 0.3rem;
	}
	.select-sm {
		padding: 0.25rem 0.4rem;
		background: var(--bg-surface);
		border: 1px solid rgba(255, 255, 255, 0.1);
		border-radius: var(--radius-sm);
		color: var(--text-primary);
		font-size: 0.78rem;
	}

	.league-row {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: 0.75rem 1.1rem;
	}
	.league-row.closed { opacity: 0.5; }
	.league-name { font-weight: 700; margin-right: 0.5rem; }
	.status-tag {
		font-family: var(--font-mono);
		font-size: 0.65rem;
		font-weight: 700;
		padding: 0.1rem 0.35rem;
		border-radius: 100px;
		text-transform: uppercase;
		background: rgba(255, 255, 255, 0.06);
		color: var(--text-muted);
	}
	.status-tag.active {
		background: rgba(180, 247, 74, 0.12);
		color: var(--neon-green);
	}
	.close-league-btn {
		color: var(--neon-red) !important;
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
		.user-row { flex-direction: column; align-items: flex-start; }
		.user-actions { margin-top: 0.5rem; }
	}
</style>

<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { getPlayers, createMatch, type Player } from '$lib/api';
	import { playVictory } from '$lib/sounds.svelte';

	let players = $state<Player[]>([]);
	let loading = $state(true);
	let submitting = $state(false);
	let error = $state('');
	let success = $state(false);

	// Form state
	let winner1 = $state('');
	let winner2 = $state('');
	let loser1 = $state('');
	let loser2 = $state('');
	let winnerScore = $state<string>('');
	let loserScore = $state<string>('');
	let comment = $state('');

	const trashTalkSuggestions = [
		"Destroyed them at the kitchen line",
		"They couldn't return a serve to save their lives",
		"Absolute clinic on the court",
		"Mercy rule should've been invoked",
		"They'll need therapy after this one",
		"Not even close, baby",
		"Someone call the ambulance",
		"The dink heard round the world",
	];

	let placeholderComment = $state('');

	onMount(async () => {
		placeholderComment = trashTalkSuggestions[Math.floor(Math.random() * trashTalkSuggestions.length)];
		try {
			players = await getPlayers();
		} catch (e: any) {
			error = e.message;
		} finally {
			loading = false;
		}
	});

	// Validation: all 4 players must be different
	const selectedPlayers = $derived([winner1, winner2, loser1, loser2].filter(Boolean));
	const hasDuplicates = $derived(new Set(selectedPlayers).size !== selectedPlayers.length);
	const allSelected = $derived(selectedPlayers.length === 4);
	const canSubmit = $derived(allSelected && !hasDuplicates && !submitting);

	// Auto-generated roast based on score differential
	const autoRoast = $derived(() => {
		const ws = parseInt(winnerScore);
		const ls = parseInt(loserScore);
		if (isNaN(ws) || isNaN(ls)) return '';
		const diff = ws - ls;
		if (diff >= 10) return "That's not a game, that's a crime scene.";
		if (diff >= 7) return "Absolutely demolished. No survivors.";
		if (diff >= 5) return "Dominant performance.";
		if (diff <= 1) return "Nail-biter! Could've gone either way.";
		if (diff <= 2) return "Close one! Respect.";
		return '';
	});

	async function handleSubmit() {
		if (!canSubmit) return;
		submitting = true;
		error = '';

		try {
			await createMatch({
				winner1_id: winner1,
				winner2_id: winner2,
				loser1_id: loser1,
				loser2_id: loser2,
				winner_score: winnerScore ? parseInt(winnerScore) : undefined,
				loser_score: loserScore ? parseInt(loserScore) : undefined,
				comment: comment || undefined,
			});
			success = true;
			// Confetti explosion!
			try {
				const confetti = (await import('canvas-confetti')).default;
				confetti({
					particleCount: 150,
					spread: 80,
					origin: { y: 0.6 },
					colors: ['#b4f74a', '#ff8534', '#4aabf7', '#ffffff'],
				});
			} catch { /* confetti not available */ }
			playVictory();
			setTimeout(() => goto('/'), 2000);
		} catch (e: any) {
			error = e.message;
		} finally {
			submitting = false;
		}
	}

	function playerLabel(p: Player): string {
		return `${p.avatar_emoji} ${p.name}`;
	}
</script>

<div class="page-header">
	<h1 class="page-title">RECORD MATCH</h1>
	<p class="page-subtitle">Who dominated? Who got wrecked?</p>
</div>

{#if success}
	<div class="success-card card">
		<div class="success-icon">🎉</div>
		<h2>Match Recorded!</h2>
		<p>Redirecting to leaderboard...</p>
	</div>
{:else if loading}
	<div class="loading-state">
		<span class="loading-spinner"></span>
		<span>Loading players...</span>
	</div>
{:else}
	<form class="match-form" onsubmit={(e) => { e.preventDefault(); handleSubmit(); }}>
		{#if error}
			<div class="error-banner">{error}</div>
		{/if}

		<div class="form-section winners-section">
			<h2 class="section-label winners-label">🏆 WINNERS</h2>
			<div class="player-selects">
				<select bind:value={winner1} class="select">
					<option value="">Select player 1...</option>
					{#each players as p}
						<option value={p.id}>{playerLabel(p)}</option>
					{/each}
				</select>
				<span class="and">&</span>
				<select bind:value={winner2} class="select">
					<option value="">Select player 2...</option>
					{#each players as p}
						<option value={p.id}>{playerLabel(p)}</option>
					{/each}
				</select>
			</div>
		</div>

		<div class="vs-divider">VS</div>

		<div class="form-section losers-section">
			<h2 class="section-label losers-label">💀 LOSERS</h2>
			<div class="player-selects">
				<select bind:value={loser1} class="select">
					<option value="">Select player 1...</option>
					{#each players as p}
						<option value={p.id}>{playerLabel(p)}</option>
					{/each}
				</select>
				<span class="and">&</span>
				<select bind:value={loser2} class="select">
					<option value="">Select player 2...</option>
					{#each players as p}
						<option value={p.id}>{playerLabel(p)}</option>
					{/each}
				</select>
			</div>
		</div>

		{#if hasDuplicates}
			<div class="error-banner">A player can't be on both teams (or listed twice)!</div>
		{/if}

		<div class="form-section score-section">
			<h2 class="section-label">📊 Score <span class="optional">(optional)</span></h2>
			<div class="score-inputs">
				<input
					type="number"
					bind:value={winnerScore}
					placeholder="Winner"
					class="input score-input"
					min="0"
					max="99"
				/>
				<span class="score-dash">-</span>
				<input
					type="number"
					bind:value={loserScore}
					placeholder="Loser"
					class="input score-input"
					min="0"
					max="99"
				/>
			</div>
			{#if autoRoast()}
				<p class="auto-roast">{autoRoast()}</p>
			{/if}
		</div>

		<div class="form-section">
			<h2 class="section-label">🗣️ Trash Talk <span class="optional">(optional)</span></h2>
			<textarea
				bind:value={comment}
				class="input textarea"
				placeholder={placeholderComment}
				rows="2"
			></textarea>
		</div>

		<button type="submit" class="btn btn-primary btn-lg submit-btn" disabled={!canSubmit}>
			{#if submitting}
				Recording...
			{:else}
				Record Match 🏓
			{/if}
		</button>
	</form>
{/if}

<style>
	.page-header { margin-bottom: 2rem; }
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

	.match-form {
		max-width: 600px;
		display: flex;
		flex-direction: column;
		gap: 1.5rem;
	}

	.form-section {
		background: var(--bg-card);
		border: 1px solid rgba(255, 255, 255, 0.06);
		border-radius: var(--radius);
		padding: 1.25rem;
	}

	.winners-section { border-color: rgba(180, 247, 74, 0.15); }
	.losers-section { border-color: rgba(255, 59, 92, 0.15); }

	.section-label {
		font-family: var(--font-display);
		font-size: 1.1rem;
		font-weight: 700;
		margin-bottom: 0.75rem;
		letter-spacing: 0.04em;
	}
	.winners-label { color: var(--neon-green); }
	.losers-label { color: var(--neon-red); }
	.optional {
		color: var(--text-muted);
		font-family: var(--font-body);
		font-size: 0.75rem;
		font-weight: 400;
		letter-spacing: 0;
	}

	.player-selects {
		display: flex;
		align-items: center;
		gap: 0.75rem;
	}
	.and {
		color: var(--text-muted);
		font-weight: 700;
		flex-shrink: 0;
	}

	.select, .input {
		width: 100%;
		padding: 0.6rem 0.85rem;
		background: var(--bg-surface);
		border: 1px solid rgba(255, 255, 255, 0.1);
		border-radius: var(--radius-sm);
		color: var(--text-primary);
		font-family: var(--font-body);
		font-size: 0.9rem;
		transition: border-color 0.15s;
	}
	.select:focus, .input:focus {
		outline: none;
		border-color: var(--neon-green);
	}
	.select option {
		background: var(--bg-surface);
	}

	.vs-divider {
		text-align: center;
		font-family: var(--font-display);
		font-size: 2rem;
		font-weight: 900;
		color: var(--text-muted);
		letter-spacing: 0.1em;
	}

	.score-inputs {
		display: flex;
		align-items: center;
		gap: 0.75rem;
	}
	.score-input {
		max-width: 100px;
		text-align: center;
		font-family: var(--font-mono);
		font-size: 1.2rem;
		font-weight: 700;
	}
	.score-dash {
		color: var(--text-muted);
		font-size: 1.5rem;
		font-weight: 700;
	}

	.auto-roast {
		margin-top: 0.5rem;
		font-size: 0.8rem;
		font-style: italic;
		color: var(--neon-orange);
	}

	.textarea {
		resize: vertical;
		min-height: 60px;
	}

	.submit-btn {
		width: 100%;
		justify-content: center;
		font-size: 1.1rem;
		padding: 0.85rem;
	}
	.submit-btn:disabled {
		opacity: 0.4;
		cursor: not-allowed;
	}

	:global(.btn-lg) {
		padding: 0.85rem 1.5rem;
		font-size: 1.1rem;
	}

	.error-banner {
		background: rgba(255, 59, 92, 0.12);
		border: 1px solid rgba(255, 59, 92, 0.25);
		color: var(--neon-red);
		padding: 0.65rem 1rem;
		border-radius: var(--radius-sm);
		font-size: 0.85rem;
	}

	.success-card {
		text-align: center;
		padding: 3rem 2rem;
	}
	.success-icon {
		font-size: 4rem;
		margin-bottom: 1rem;
	}
	.success-card h2 {
		font-family: var(--font-display);
		font-size: 2rem;
		color: var(--neon-green);
		margin-bottom: 0.5rem;
	}
	.success-card p {
		color: var(--text-secondary);
	}

	.loading-state {
		display: flex;
		align-items: center;
		gap: 0.75rem;
		color: var(--text-secondary);
		padding: 3rem 0;
		justify-content: center;
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
		.player-selects { flex-direction: column; }
		.and { display: none; }
	}
</style>

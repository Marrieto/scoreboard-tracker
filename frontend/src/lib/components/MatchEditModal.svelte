<script lang="ts">
	import type { MatchRecord, Player, League } from '$lib/api';

	let {
		match: m,
		players,
		leagues,
		onSave,
		onClose,
	}: {
		match: MatchRecord;
		players: Player[];
		leagues: League[];
		onSave: (updated: MatchRecord) => void;
		onClose: () => void;
	} = $props();

	// Editable fields initialized from the match
	let winner1 = $state(m.winner1_id);
	let winner2 = $state(m.winner2_id);
	let loser1 = $state(m.loser1_id);
	let loser2 = $state(m.loser2_id);
	let winnerScore = $state(m.winner_score?.toString() ?? '');
	let loserScore = $state(m.loser_score?.toString() ?? '');
	let comment = $state(m.comment);
	let leagueId = $state(m.league_id ?? '');
	let saving = $state(false);

	const selectedPlayers = $derived([winner1, winner2, loser1, loser2].filter(Boolean));
	const hasDuplicates = $derived(new Set(selectedPlayers).size !== selectedPlayers.length);
	const allSelected = $derived(selectedPlayers.length === 4);
	const canSave = $derived(allSelected && !hasDuplicates && !saving);

	function handleSave() {
		if (!canSave) return;
		saving = true;
		onSave({
			...m,
			winner1_id: winner1,
			winner2_id: winner2,
			loser1_id: loser1,
			loser2_id: loser2,
			winner_score: winnerScore ? parseInt(winnerScore) : null,
			loser_score: loserScore ? parseInt(loserScore) : null,
			comment,
			league_id: leagueId || null,
		});
	}

	function playerLabel(p: Player): string {
		return `${p.avatar_emoji} ${p.name}`;
	}
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="modal-backdrop" onclick={onClose} onkeydown={(e) => e.key === 'Escape' && onClose()}>
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div class="modal-content card" onclick={(e) => e.stopPropagation()}>
		<div class="modal-header">
			<h2 class="modal-title">Edit Match</h2>
			<button class="close-btn" onclick={onClose}>&times;</button>
		</div>

		<div class="modal-body">
			<!-- Winners -->
			<div class="edit-section">
				<label class="edit-label winners-label">🏆 Winners</label>
				<div class="edit-row">
					<select bind:value={winner1} class="select">
						{#each players as p}
							<option value={p.id}>{playerLabel(p)}</option>
						{/each}
					</select>
					<span class="and">&</span>
					<select bind:value={winner2} class="select">
						{#each players as p}
							<option value={p.id}>{playerLabel(p)}</option>
						{/each}
					</select>
				</div>
			</div>

			<!-- Losers -->
			<div class="edit-section">
				<label class="edit-label losers-label">💀 Losers</label>
				<div class="edit-row">
					<select bind:value={loser1} class="select">
						{#each players as p}
							<option value={p.id}>{playerLabel(p)}</option>
						{/each}
					</select>
					<span class="and">&</span>
					<select bind:value={loser2} class="select">
						{#each players as p}
							<option value={p.id}>{playerLabel(p)}</option>
						{/each}
					</select>
				</div>
			</div>

			{#if hasDuplicates}
				<div class="error-banner">A player can't be on both teams!</div>
			{/if}

			<!-- Scores -->
			<div class="edit-section">
				<label class="edit-label">📊 Score</label>
				<div class="edit-row score-row">
					<input type="number" bind:value={winnerScore} class="input score-input" placeholder="W" min="0" max="99" />
					<span class="score-dash">-</span>
					<input type="number" bind:value={loserScore} class="input score-input" placeholder="L" min="0" max="99" />
				</div>
			</div>

			<!-- Comment -->
			<div class="edit-section">
				<label class="edit-label">🗣️ Comment</label>
				<textarea bind:value={comment} class="input textarea" rows="2"></textarea>
			</div>

			<!-- League -->
			{#if leagues.length > 0}
				<div class="edit-section">
					<label class="edit-label">🏅 League</label>
					<select bind:value={leagueId} class="select">
						<option value="">No league</option>
						{#each leagues as league}
							<option value={league.id}>{league.name}</option>
						{/each}
					</select>
				</div>
			{/if}
		</div>

		<div class="modal-footer">
			<button class="btn btn-ghost" onclick={onClose}>Cancel</button>
			<button class="btn btn-primary" disabled={!canSave} onclick={handleSave}>
				{saving ? 'Saving...' : 'Save Changes'}
			</button>
		</div>
	</div>
</div>

<style>
	.modal-backdrop {
		position: fixed;
		inset: 0;
		background: rgba(0, 0, 0, 0.7);
		backdrop-filter: blur(4px);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 200;
		padding: 1rem;
	}
	.modal-content {
		width: 100%;
		max-width: 520px;
		max-height: 90vh;
		overflow-y: auto;
		padding: 0;
	}
	.modal-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 1.25rem 1.5rem;
		border-bottom: 1px solid rgba(255, 255, 255, 0.06);
	}
	.modal-title {
		font-family: var(--font-display);
		font-size: 1.3rem;
		font-weight: 700;
	}
	.close-btn {
		background: none;
		border: none;
		color: var(--text-muted);
		font-size: 1.5rem;
		cursor: pointer;
		padding: 0 0.25rem;
	}
	.close-btn:hover { color: var(--text-primary); }

	.modal-body {
		padding: 1.25rem 1.5rem;
		display: flex;
		flex-direction: column;
		gap: 1rem;
	}
	.modal-footer {
		display: flex;
		justify-content: flex-end;
		gap: 0.75rem;
		padding: 1rem 1.5rem;
		border-top: 1px solid rgba(255, 255, 255, 0.06);
	}

	.edit-section {
		display: flex;
		flex-direction: column;
		gap: 0.4rem;
	}
	.edit-label {
		font-family: var(--font-display);
		font-size: 0.85rem;
		font-weight: 700;
		letter-spacing: 0.04em;
	}
	.winners-label { color: var(--neon-green); }
	.losers-label { color: var(--neon-red); }

	.edit-row {
		display: flex;
		align-items: center;
		gap: 0.5rem;
	}
	.and {
		color: var(--text-muted);
		font-weight: 700;
		flex-shrink: 0;
	}

	.select, .input {
		width: 100%;
		padding: 0.5rem 0.7rem;
		background: var(--bg-surface);
		border: 1px solid rgba(255, 255, 255, 0.1);
		border-radius: var(--radius-sm);
		color: var(--text-primary);
		font-family: var(--font-body);
		font-size: 0.85rem;
	}
	.select:focus, .input:focus {
		outline: none;
		border-color: var(--neon-green);
	}
	.select option {
		background: var(--bg-surface);
	}

	.score-row {
		max-width: 250px;
	}
	.score-input {
		max-width: 80px;
		text-align: center;
		font-family: var(--font-mono);
		font-weight: 700;
	}
	.score-dash {
		color: var(--text-muted);
		font-size: 1.2rem;
		font-weight: 700;
	}
	.textarea {
		resize: vertical;
		min-height: 50px;
	}

	.error-banner {
		background: rgba(255, 59, 92, 0.12);
		border: 1px solid rgba(255, 59, 92, 0.25);
		color: var(--neon-red);
		padding: 0.5rem 0.75rem;
		border-radius: var(--radius-sm);
		font-size: 0.8rem;
	}

	@media (max-width: 640px) {
		.edit-row { flex-direction: column; }
		.and { display: none; }
	}
</style>

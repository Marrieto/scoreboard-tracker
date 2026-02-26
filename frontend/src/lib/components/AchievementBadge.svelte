<script lang="ts">
	import type { PlayerStats } from '$lib/api';

	let { stats }: { stats: PlayerStats } = $props();

	// Compute achievements from stats
	interface Achievement {
		icon: string;
		title: string;
		description: string;
	}

	const achievements = $derived<Achievement[]>(() => {
		const list: Achievement[] = [];

		if (stats.wins >= 1)
			list.push({ icon: '🩸', title: 'First Blood', description: 'Won your first match' });
		if (stats.wins >= 10)
			list.push({ icon: '🔟', title: 'Double Digits', description: '10+ wins' });
		if (stats.wins >= 25)
			list.push({ icon: '⚡', title: 'Veteran', description: '25+ wins' });
		if (stats.win_rate >= 0.8 && stats.total_games >= 5)
			list.push({ icon: '🛡️', title: 'Untouchable', description: '80%+ win rate (5+ games)' });
		if (stats.streak >= 5)
			list.push({ icon: '🔥', title: 'On Fire', description: '5+ win streak' });
		if (stats.streak >= 3)
			list.push({ icon: '🌊', title: 'Riding the Wave', description: '3+ win streak' });
		if (stats.best_partner && stats.best_partner.wins >= 5)
			list.push({ icon: '💕', title: 'Partnership Goals', description: '5+ wins with same partner' });
		if (stats.total_games >= 20)
			list.push({ icon: '🎮', title: 'Addicted', description: '20+ total games' });
		if (stats.total_games >= 50)
			list.push({ icon: '🏟️', title: 'Court Regular', description: '50+ total games' });
		if (stats.win_rate === 1.0 && stats.total_games >= 3)
			list.push({ icon: '💎', title: 'Perfect Record', description: 'Undefeated (3+ games)' });
		if (stats.win_rate < 0.2 && stats.total_games >= 5)
			list.push({ icon: '🪦', title: 'Beyond Help', description: '<20% win rate (5+ games)' });

		return list;
	});
</script>

{#if achievements().length > 0}
	<div class="achievements">
		<h3 class="achievements-title">ACHIEVEMENTS</h3>
		<div class="badge-grid">
			{#each achievements() as badge, i}
				<div class="achievement animate-in" style="animation-delay: {i * 50}ms" title={badge.description}>
					<span class="achievement-icon">{badge.icon}</span>
					<span class="achievement-name">{badge.title}</span>
				</div>
			{/each}
		</div>
	</div>
{/if}

<style>
	.achievements {
		margin-bottom: 1.5rem;
	}
	.achievements-title {
		font-family: var(--font-display);
		font-size: 1.3rem;
		font-weight: 700;
		letter-spacing: 0.04em;
		color: var(--text-secondary);
		margin-bottom: 0.75rem;
	}
	.badge-grid {
		display: flex;
		flex-wrap: wrap;
		gap: 0.4rem;
	}
	.achievement {
		display: inline-flex;
		align-items: center;
		gap: 0.35rem;
		padding: 0.35rem 0.75rem;
		background: rgba(255, 255, 255, 0.04);
		border: 1px solid rgba(255, 255, 255, 0.08);
		border-radius: 100px;
		font-size: 0.8rem;
		transition: all 0.2s;
		cursor: default;
	}
	.achievement:hover {
		background: rgba(255, 255, 255, 0.08);
		border-color: rgba(255, 255, 255, 0.15);
		transform: translateY(-1px);
	}
	.achievement-icon {
		font-size: 1rem;
	}
	.achievement-name {
		font-weight: 600;
		color: var(--text-secondary);
	}
</style>

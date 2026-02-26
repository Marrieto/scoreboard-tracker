<script lang="ts">
	let { streak }: { streak: number } = $props();

	const isWinning = $derived(streak > 0);
	const count = $derived(Math.abs(streak));
	const icon = $derived(isWinning ? '🔥' : '💀');
	const label = $derived(isWinning ? `${count}W streak` : `${count}L streak`);
</script>

{#if count >= 2}
	<span class="streak" class:winning={isWinning} class:losing={!isWinning} title={label}>
		<span class="streak-icon" class:fire={isWinning}>{icon}</span>
		<span class="streak-count">{count}</span>
	</span>
{/if}

<style>
	.streak {
		display: inline-flex;
		align-items: center;
		gap: 0.15rem;
		padding: 0.1rem 0.4rem;
		border-radius: 100px;
		font-size: 0.75rem;
		font-weight: 700;
		margin-left: 0.4rem;
	}
	.streak.winning {
		background: rgba(255, 133, 52, 0.15);
		color: var(--neon-orange);
	}
	.streak.losing {
		background: rgba(255, 59, 92, 0.12);
		color: var(--neon-red);
	}
	.streak-icon {
		font-size: 0.85rem;
	}
	.streak-icon.fire {
		animation: fire-flicker 0.6s ease-in-out infinite;
	}
	.streak-count {
		font-family: var(--font-mono);
	}
</style>

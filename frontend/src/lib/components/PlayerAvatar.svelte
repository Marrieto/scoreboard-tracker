<script lang="ts">
	let {
		emoji,
		size = 'md',
		rank,
	}: {
		emoji: string;
		size?: 'sm' | 'md' | 'lg' | 'xl';
		rank?: number;
	} = $props();

	const sizeClass = `avatar-${size}`;
	const isChampion = rank === 1;
</script>

<div class="avatar {sizeClass}" class:champion={isChampion}>
	<span class="emoji">{emoji}</span>
	{#if isChampion}
		<span class="crown">👑</span>
	{/if}
</div>

<style>
	.avatar {
		position: relative;
		display: inline-flex;
		align-items: center;
		justify-content: center;
		flex-shrink: 0;
	}
	.avatar-sm .emoji { font-size: 1.2rem; }
	.avatar-md .emoji { font-size: 2rem; }
	.avatar-lg .emoji { font-size: 3rem; }
	.avatar-xl .emoji { font-size: 4rem; }

	.champion {
		animation: pulse-glow 3s ease-in-out infinite;
	}
	.crown {
		position: absolute;
		top: -12px;
		left: 50%;
		transform: translateX(-50%);
		font-size: 1rem;
		animation: fire-flicker 2s ease-in-out infinite;
	}
	.avatar-lg .crown, .avatar-xl .crown {
		font-size: 1.4rem;
		top: -16px;
	}
</style>

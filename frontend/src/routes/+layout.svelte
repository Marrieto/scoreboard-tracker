<script lang="ts">
	import '../app.css';
	import { checkAuth, getAuth } from '$lib/stores/auth.svelte';
	import { onMount } from 'svelte';
	import { page } from '$app/stores';

	let { children } = $props();
	const auth = getAuth();
	let mobileNavOpen = $state(false);

	onMount(() => {
		checkAuth();
	});

	const navLinks = [
		{ href: '/', label: 'Leaderboard', icon: '🏆' },
		{ href: '/matches', label: 'Matches', icon: '🏓' },
		{ href: '/matches/new', label: 'Record', icon: '📝' },
		{ href: '/players', label: 'Players', icon: '👥' },
		{ href: '/hall-of-shame', label: 'Shame', icon: '💀' },
	];
</script>

<svelte:head>
	<title>Scoreboard Tracker</title>
</svelte:head>

<div class="app-shell">
	<nav class="topnav">
		<div class="topnav-inner container">
			<a href="/" class="logo">
				<span class="logo-icon">🥒</span>
				<span class="logo-text">SCOREBOARD</span>
			</a>

			<div class="nav-links" class:open={mobileNavOpen}>
				{#each navLinks as link}
					<a
						href={link.href}
						class="nav-link"
						class:active={$page.url.pathname === link.href}
						onclick={() => mobileNavOpen = false}
					>
						<span class="nav-icon">{link.icon}</span>
						<span>{link.label}</span>
					</a>
				{/each}
			</div>

			<div class="nav-right">
				{#if auth.info.authenticated}
					<span class="user-badge">{auth.info.name}</span>
				{:else if !auth.loading}
					<a href="/api/auth/login" class="btn btn-primary btn-sm">Sign In</a>
				{/if}
				<button class="hamburger" onclick={() => mobileNavOpen = !mobileNavOpen} aria-label="Toggle navigation">
					<span></span><span></span><span></span>
				</button>
			</div>
		</div>
	</nav>

	<main class="main-content container">
		{@render children()}
	</main>
</div>

<style>
	.app-shell {
		min-height: 100dvh;
		display: flex;
		flex-direction: column;
	}

	/* ── Top Navigation ─────────────────────────── */
	.topnav {
		background: rgba(10, 14, 20, 0.85);
		backdrop-filter: blur(12px);
		border-bottom: 1px solid rgba(255, 255, 255, 0.06);
		position: sticky;
		top: 0;
		z-index: 100;
	}
	.topnav-inner {
		display: flex;
		align-items: center;
		height: 60px;
		gap: 2rem;
	}
	.logo {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		flex-shrink: 0;
	}
	.logo-icon {
		font-size: 1.5rem;
	}
	.logo-text {
		font-family: var(--font-display);
		font-size: 1.6rem;
		font-weight: 900;
		letter-spacing: 0.04em;
		color: var(--neon-green);
		text-shadow: 0 0 20px rgba(180, 247, 74, 0.3);
	}

	.nav-links {
		display: flex;
		align-items: center;
		gap: 0.25rem;
		flex: 1;
	}
	.nav-link {
		display: flex;
		align-items: center;
		gap: 0.4rem;
		padding: 0.4rem 0.8rem;
		border-radius: var(--radius-sm);
		color: var(--text-secondary);
		font-size: 0.875rem;
		font-weight: 600;
		transition: all 0.15s;
	}
	.nav-link:hover {
		color: var(--text-primary);
		background: rgba(255, 255, 255, 0.05);
	}
	.nav-link.active {
		color: var(--neon-green);
		background: var(--neon-green-dim);
	}
	.nav-icon {
		font-size: 1rem;
	}

	.nav-right {
		display: flex;
		align-items: center;
		gap: 0.75rem;
		flex-shrink: 0;
	}
	.user-badge {
		font-size: 0.8rem;
		color: var(--text-secondary);
		padding: 0.3rem 0.8rem;
		border-radius: 100px;
		background: rgba(255, 255, 255, 0.05);
		border: 1px solid rgba(255, 255, 255, 0.08);
	}

	.hamburger {
		display: none;
		flex-direction: column;
		gap: 4px;
		background: none;
		padding: 4px;
	}
	.hamburger span {
		display: block;
		width: 20px;
		height: 2px;
		background: var(--text-secondary);
		border-radius: 1px;
		transition: 0.2s;
	}

	:global(.btn-sm) {
		padding: 0.4rem 0.85rem;
		font-size: 0.8rem;
	}

	/* ── Main content ────────────────────────────── */
	.main-content {
		flex: 1;
		padding-top: 2rem;
		padding-bottom: 3rem;
	}

	/* ── Mobile ───────────────────────────────────── */
	@media (max-width: 768px) {
		.nav-links {
			position: fixed;
			top: 60px;
			left: 0;
			right: 0;
			background: rgba(10, 14, 20, 0.97);
			backdrop-filter: blur(12px);
			flex-direction: column;
			padding: 1rem;
			gap: 0.25rem;
			border-bottom: 1px solid rgba(255, 255, 255, 0.06);
			transform: translateY(-100%);
			opacity: 0;
			pointer-events: none;
			transition: all 0.25s ease;
		}
		.nav-links.open {
			transform: translateY(0);
			opacity: 1;
			pointer-events: auto;
		}
		.nav-link {
			width: 100%;
			padding: 0.75rem 1rem;
		}
		.hamburger {
			display: flex;
		}
	}
</style>

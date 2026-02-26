// lib/api.ts — API client helper.
//
// Provides typed functions for calling the backend REST API.
// In development, Vite proxies /api to the Rust backend.
// In production, same origin — no proxy needed.

export interface Player {
	id: string;
	name: string;
	nickname: string;
	avatar_emoji: string;
}

export interface MatchRecord {
	id: string;
	winner1_id: string;
	winner2_id: string;
	loser1_id: string;
	loser2_id: string;
	winner_score: number | null;
	loser_score: number | null;
	comment: string;
	recorded_by: string;
	played_at: string;
}

export interface LeaderboardEntry {
	player_id: string;
	player_name: string;
	avatar_emoji: string;
	nickname: string;
	wins: number;
	losses: number;
	total_games: number;
	win_rate: number;
	streak: number;
}

export interface PlayerStats {
	player_id: string;
	player_name: string;
	avatar_emoji: string;
	nickname: string;
	wins: number;
	losses: number;
	total_games: number;
	win_rate: number;
	streak: number;
	best_partner: { partner_id: string; partner_name: string; wins: number; losses: number } | null;
	nemesis: {
		opponent_id: string;
		opponent_name: string;
		wins_against: number;
		losses_against: number;
	} | null;
	recent_matches: MatchRecord[];
}

export interface RivalryEntry {
	player1_id: string;
	player1_name: string;
	player2_id: string;
	player2_name: string;
	player1_wins: number;
	player2_wins: number;
}

export interface AuthInfo {
	authenticated: boolean;
	user_id?: string;
	name?: string;
	email?: string;
}

async function apiFetch<T>(path: string, options?: RequestInit): Promise<T> {
	const res = await fetch(path, {
		...options,
		headers: {
			'Content-Type': 'application/json',
			...options?.headers,
		},
	});
	if (!res.ok) {
		const body = await res.json().catch(() => ({ error: res.statusText }));
		throw new Error(body.error || `API error: ${res.status}`);
	}
	// Handle 204 No Content
	if (res.status === 204) return undefined as T;
	return res.json();
}

// Auth
export const getAuthInfo = () => apiFetch<AuthInfo>('/api/auth/me');
export const logout = () => apiFetch<void>('/api/auth/logout', { method: 'POST' });

// Players
export const getPlayers = () => apiFetch<Player[]>('/api/players');
export const createPlayer = (player: Omit<Player, never>) =>
	apiFetch<Player>('/api/players', { method: 'POST', body: JSON.stringify(player) });
export const updatePlayer = (id: string, data: Partial<Player>) =>
	apiFetch<Player>(`/api/players/${id}`, { method: 'PUT', body: JSON.stringify(data) });
export const deletePlayer = (id: string) =>
	apiFetch<void>(`/api/players/${id}`, { method: 'DELETE' });

// Matches
export const getMatches = (limit?: number) =>
	apiFetch<MatchRecord[]>(`/api/matches${limit ? `?limit=${limit}` : ''}`);
export const createMatch = (data: {
	winner1_id: string;
	winner2_id: string;
	loser1_id: string;
	loser2_id: string;
	winner_score?: number;
	loser_score?: number;
	comment?: string;
}) => apiFetch<MatchRecord>('/api/matches', { method: 'POST', body: JSON.stringify(data) });
export const deleteMatch = (id: string) =>
	apiFetch<void>(`/api/matches/${id}`, { method: 'DELETE' });

// Leaderboard & Stats
export const getLeaderboard = () => apiFetch<LeaderboardEntry[]>('/api/leaderboard');
export const getPlayerStats = (id: string) => apiFetch<PlayerStats>(`/api/players/${id}/stats`);
export const getRivalries = () => apiFetch<RivalryEntry[]>('/api/rivalries');

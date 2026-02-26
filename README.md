# Scoreboard Tracker

A fun internal web app for tracking pickleball match results, leaderboards, stats, and trash talk.

## Architecture

```
┌─────────────────────────────────────┐
│         Docker Container            │
│  ┌──────────────────────────────┐   │
│  │   Axum (Rust) HTTP Server    │   │
│  │  /api/*  → JSON REST API     │   │
│  │  /*      → SvelteKit SPA     │   │
│  └──────────┬───────────────────┘   │
│             │ HTTPS                 │
└─────────────┼───────────────────────┘
              ▼
   ┌─────────────────────┐
   │  Azure Table Storage │
   └─────────────────────┘
```

- **Backend**: Rust (Axum) — serves the REST API and static frontend files
- **Frontend**: SvelteKit (Svelte 5) — static SPA with retro-arcade aesthetic
- **Storage**: Azure Table Storage — cheap, serverless NoSQL for players & matches
- **Auth**: Microsoft Entra ID (Azure AD) via OIDC

## Features

- **Leaderboard** with win rates, streaks, crown/skull animations
- **Match recording** with confetti explosions and auto-generated roasts
- **Player profiles** with achievement badges, nemesis tracker, best partner stats
- **Hall of Shame** — worst stats, biggest blowouts, "The Pickle Jar"
- **Charts** — win rate trajectory over time
- **Sound effects** — victory fanfare, sad trombone (toggle-able)
- **Rivalries** — head-to-head records between all player pairs

## Quick Start (Development)

### Prerequisites

- Rust 1.75+ (`rustup`)
- Node.js 22+ (`node`, `npm`)
- An Azure Storage Account with Table Storage
- An Azure AD / Entra ID app registration (for auth)

### 1. Configure environment

```bash
cp .env.example .env
# Edit .env with your Azure credentials
```

### 2. Run the backend

```bash
cargo run
# Serves API on http://localhost:3000
```

### 3. Run the frontend (dev server)

```bash
cd frontend
npm install
npm run dev
# Opens http://localhost:5173, proxies /api to :3000
```

## Docker Deployment

### Build

```bash
docker build -t scoreboard .
```

### Run

```bash
docker run -p 3000:3000 --env-file .env scoreboard
```

The app is accessible at `http://localhost:3000`.

## API Endpoints

### Players
| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/players` | List all players |
| `POST` | `/api/players` | Create a player |
| `PUT` | `/api/players/:id` | Update a player |
| `DELETE` | `/api/players/:id` | Delete a player |

### Matches
| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/matches?limit=N` | List recent matches |
| `POST` | `/api/matches` | Record a new match |
| `DELETE` | `/api/matches/:id` | Delete a match |

### Leaderboard & Stats
| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/leaderboard` | Ranked player list with stats |
| `GET` | `/api/players/:id/stats` | Detailed player stats |
| `GET` | `/api/rivalries` | Head-to-head records |

### Auth
| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/auth/login` | Redirect to Microsoft login |
| `GET` | `/api/auth/callback` | OIDC callback |
| `GET` | `/api/auth/me` | Current user info |
| `POST` | `/api/auth/logout` | Clear session |

## Environment Variables

| Variable | Description | Example |
|----------|-------------|---------|
| `AZURE_STORAGE_ACCOUNT` | Storage account name | `myscoreboard` |
| `AZURE_STORAGE_ACCESS_KEY` | Storage account key | `abc123...` |
| `AZURE_TENANT_ID` | Entra ID tenant | `xxxxxxxx-xxxx-...` |
| `AZURE_CLIENT_ID` | App registration client ID | `xxxxxxxx-xxxx-...` |
| `AZURE_CLIENT_SECRET` | App registration secret | `secret...` |
| `APP_URL` | Public URL of the app | `https://scoreboard.example.com` |
| `SESSION_SECRET` | Secret for signing session JWTs | `random-secret-string` |
| `PORT` | Server port (default 3000) | `3000` |

## Project Structure

```
scoreboard/
├── src/                    # Rust backend
│   ├── main.rs             # Server entry point
│   ├── config.rs           # Env var configuration
│   ├── auth/               # OIDC login + JWT session middleware
│   ├── storage/            # Azure Table Storage client + CRUD
│   ├── routes/             # API route handlers
│   └── models/             # Player + MatchRecord domain types
├── frontend/               # SvelteKit SPA
│   └── src/
│       ├── routes/         # Pages (leaderboard, matches, players, hall-of-shame)
│       └── lib/            # Components, API client, stores, sounds
├── Dockerfile              # Multi-stage build
└── .env.example            # Environment variable template
```

# 🗄️ RouxFlow Database Schema

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│  FRONTEND (offline-first)     SUPABASE (cloud)          │
│  IndexedDB                    PostgreSQL                │
│                                                         │
│  users (local_only)     ←→    users (auth.users)        │
│  sessions               ←→    sessions                  │
│  solves                 ←→    solves                    │
│  pending_sync           →     (sync queue)              │
└─────────────────────────────────────────────────────────┘
```

---

## Tables

### `users` (Supabase only, via Auth)

Managed automatically by Supabase Auth (`auth.users` table).

```sql
-- Public profiles table for additional user info
CREATE TABLE public.profiles (
  id UUID PRIMARY KEY REFERENCES auth.users(id) ON DELETE CASCADE,
  username TEXT UNIQUE NOT NULL,
  created_at TIMESTAMPTZ DEFAULT NOW(),
  avatar_url TEXT
);

-- RLS
ALTER TABLE profiles ENABLE ROW LEVEL SECURITY;
CREATE POLICY "Public read" ON profiles FOR SELECT USING (true);
CREATE POLICY "Own update" ON profiles FOR UPDATE USING (auth.uid() = id);
```

---

### `sessions`

| Column | Local (IDB/SQLite) | Supabase | Description |
|---|---|---|---|
| `id` | TEXT PK | UUID PK | Unique identifier |
| `user_id` | TEXT | UUID FK → auth.users | Owner |
| `name` | TEXT | TEXT | Session name |
| `session_type` | TEXT | TEXT ('WCA', 'Free') | Type |
| `created_at` | INTEGER | TIMESTAMPTZ | Creation timestamp |
| `synced_at` | INTEGER | — | Last sync timestamp (local only) |

```sql
-- Local schema
CREATE TABLE sessions (
  id TEXT PRIMARY KEY,
  user_id TEXT,
  name TEXT NOT NULL,
  session_type TEXT NOT NULL CHECK (session_type IN ('WCA', 'Free')),
  created_at INTEGER NOT NULL,
  synced_at INTEGER
);

-- Supabase
CREATE TABLE sessions (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id UUID NOT NULL REFERENCES auth.users(id) ON DELETE CASCADE,
  name TEXT NOT NULL,
  session_type TEXT NOT NULL CHECK (session_type IN ('WCA', 'Free')),
  created_at TIMESTAMPTZ DEFAULT NOW()
);

-- RLS
ALTER TABLE sessions ENABLE ROW LEVEL SECURITY;
CREATE POLICY "Own data" ON sessions FOR ALL USING (auth.uid() = user_id);
```

---

### `solves`

| Column | Local (IDB/SQLite) | Supabase | Description |
|---|---|---|---|
| `id` | TEXT PK | UUID PK | Unique identifier |
| `session_id` | TEXT FK | UUID FK | Parent session |
| `user_id` | TEXT | UUID FK | Owner (denormalized) |
| `scramble` | TEXT | TEXT | WCA scramble sequence |
| `moves` | TEXT (JSON) | JSONB | Move array with timestamps |
| `time_ms` | INTEGER | INTEGER | Total time in milliseconds |
| `phases` | TEXT (JSON) | JSONB | Roux split times {fb, sb, cmll, lse} |
| `is_valid` | INTEGER | BOOLEAN | Valid solve flag |
| `created_at` | INTEGER | TIMESTAMPTZ | Solve timestamp |
| `signature` | TEXT | TEXT | HMAC anti-tampering signature |
| `synced_at` | INTEGER | — | Local sync timestamp |

```sql
-- Local schema
CREATE TABLE solves (
  id TEXT PRIMARY KEY,
  session_id TEXT NOT NULL REFERENCES sessions(id),
  user_id TEXT,
  scramble TEXT NOT NULL,
  moves TEXT NOT NULL,  -- JSON array
  time_ms INTEGER NOT NULL CHECK (time_ms > 500 AND time_ms < 600000),
  phases TEXT,          -- JSON {fb, sb, cmll, lse}
  is_valid INTEGER NOT NULL DEFAULT 1,
  created_at INTEGER NOT NULL,
  signature TEXT,
  synced_at INTEGER
);

-- Supabase
CREATE TABLE solves (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  session_id UUID NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
  user_id UUID NOT NULL REFERENCES auth.users(id) ON DELETE SET NULL,
  scramble TEXT NOT NULL,
  moves JSONB NOT NULL,
  time_ms INTEGER NOT NULL CHECK (time_ms > 500 AND time_ms < 600000),
  phases JSONB,
  is_valid BOOLEAN NOT NULL DEFAULT true,
  created_at TIMESTAMPTZ DEFAULT NOW(),
  signature TEXT
);

-- RLS
ALTER TABLE solves ENABLE ROW LEVEL SECURITY;
CREATE POLICY "Own data" ON solves FOR ALL USING (auth.uid() = user_id);
CREATE POLICY "Public WCA read" ON solves FOR SELECT USING (
  EXISTS (SELECT 1 FROM sessions WHERE sessions.id = solves.session_id AND sessions.session_type = 'WCA')
);

-- Rate limiting
CREATE POLICY "Rate limit" ON solves FOR INSERT WITH CHECK (
  NOT EXISTS (
    SELECT 1 FROM solves 
    WHERE user_id = auth.uid() 
    AND created_at > NOW() - INTERVAL '3 seconds'
  )
);
```

---

### `cubes` (Bluetooth Devices)

| Column | Local (IDB/SQLite) | Supabase | Description |
|---|---|---|---|
| `id` | TEXT PK | UUID PK | Unique ID / MAC address |
| `user_id` | TEXT | UUID FK | Owner |
| `name` | TEXT | TEXT | User-assigned cube name |
| `device_type` | TEXT | TEXT | e.g. 'moyu_ai', 'gan_v3', 'qiyi' |
| `mac_address` | TEXT | TEXT | Physical BLE address |
| `created_at` | INTEGER | TIMESTAMPTZ | Pair timestamp |
| `synced_at` | INTEGER | — | Local sync timestamp |

```sql
-- Local schema
CREATE TABLE cubes (
  id TEXT PRIMARY KEY,
  user_id TEXT,
  name TEXT NOT NULL,
  device_type TEXT NOT NULL,
  mac_address TEXT NOT NULL,
  created_at INTEGER NOT NULL,
  synced_at INTEGER
);

-- Supabase
CREATE TABLE cubes (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id UUID NOT NULL REFERENCES auth.users(id) ON DELETE CASCADE,
  name TEXT NOT NULL,
  device_type TEXT NOT NULL,
  mac_address TEXT NOT NULL,
  created_at TIMESTAMPTZ DEFAULT NOW()
);

-- RLS
ALTER TABLE cubes ENABLE ROW LEVEL SECURITY;
CREATE POLICY "Own data" ON cubes FOR ALL USING (auth.uid() = user_id);
```

---

### `leaderboard` (Supabase — Materialized View)

```sql
-- View for leaderboard rankings (computed, not a table)
CREATE VIEW leaderboard_ao5 AS
SELECT 
  p.username,
  s.user_id,
  AVG(s.time_ms) as ao5,
  MAX(s.created_at) as last_solve
FROM (
  SELECT *, ROW_NUMBER() OVER (PARTITION BY user_id ORDER BY created_at DESC) as rn
  FROM solves
  WHERE is_valid = true
) s
JOIN profiles p ON p.id = s.user_id
WHERE s.rn <= 5
GROUP BY s.user_id, p.username
HAVING COUNT(*) = 5
ORDER BY ao5;
```

---

## Sync Strategy

```
LOCAL → CLOUD
─────────────────────────────────────────────────────────
1. Solve recorded locally (synced_at = NULL)
2. User online / connected
3. Push to Supabase with HMAC signature
4. Supabase validates signature and inserts
5. Local: set synced_at = NOW()

CLOUD → LOCAL  
─────────────────────────────────────────────────────────
1. User signs in on a new device
2. Pull user solves from Supabase
3. Insert into local IndexedDB
```

---

## Supabase Setup

1. Create a project at [supabase.com](https://supabase.com)
2. SQL Editor → run the schema definitions above
3. Copy URL and anonymous key into `.env` (or CI secrets)

# 🗄️ RouxFlow Database Schema

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│  TAURI (offline)              SUPABASE (cloud)          │
│  SQLite                       PostgreSQL                │
│                                                         │
│  users (local_only)     ←→    users (auth.users)        │
│  sessions               ←→    sessions                  │
│  solves                 ←→    solves                    │
│  pending_sync           →     (sync queue)              │
└─────────────────────────────────────────────────────────┘
```

---

## Tables

### `users` (Supabase uniquement, via Auth)

Géré automatiquement par Supabase Auth. Table `auth.users`.

```sql
-- Table publique pour les infos additionnelles
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

| Colonne | SQLite | Supabase | Description |
|---------|--------|----------|-------------|
| `id` | TEXT PK | UUID PK | Identifiant unique |
| `user_id` | TEXT | UUID FK → auth.users | Propriétaire |
| `name` | TEXT | TEXT | Nom de la session |
| `session_type` | TEXT | TEXT ('WCA', 'Free') | Type |
| `created_at` | INTEGER | TIMESTAMPTZ | Date création |
| `synced_at` | INTEGER | — | Dernière sync (local only) |

```sql
-- SQLite
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

| Colonne | SQLite | Supabase | Description |
|---------|--------|----------|-------------|
| `id` | TEXT PK | UUID PK | Identifiant unique |
| `session_id` | TEXT FK | UUID FK | Session parent |
| `user_id` | TEXT | UUID FK | Propriétaire (dénormalisé) |
| `scramble` | TEXT | TEXT | Scramble WCA |
| `moves` | TEXT (JSON) | JSONB | Liste des moves |
| `time_ms` | INTEGER | INTEGER | Temps total en ms |
| `phases` | TEXT (JSON) | JSONB | Temps par phase Roux |
| `is_valid` | INTEGER | BOOLEAN | Solve valide |
| `created_at` | INTEGER | TIMESTAMPTZ | Date du solve |
| `signature` | TEXT | TEXT | HMAC signature |
| `synced_at` | INTEGER | — | Local only |

```sql
-- SQLite
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

### `leaderboard` (Supabase - vue matérialisée)

```sql
-- Vue pour le leaderboard (calculée, pas de table)
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
1. Solve créé localement (synced_at = NULL)
2. User se connecte / online
3. Push vers Supabase avec signature HMAC
4. Supabase valide et insère
5. Local: synced_at = NOW()

CLOUD → LOCAL  
─────────────────────────────────────────────────────────
1. User se connecte sur nouveau device
2. Pull tous les solves depuis Supabase
3. Insert dans SQLite local
```

---

## Setup Supabase

1. Créer projet sur [supabase.com](https://supabase.com)
2. SQL Editor → coller les scripts ci-dessus
3. Copier URL + anon key dans `.env`

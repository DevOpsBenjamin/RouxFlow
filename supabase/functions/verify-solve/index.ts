// Supabase Edge Function: verify-solve
// Verifies HMAC-SHA256 integrity of solve data before accepting sync.
//
// Environment variable required:
//   HMAC_SECRET — hex-encoded 32-byte key (same key embedded in WASM binary)
//
// Deploy: supabase functions deploy verify-solve
// Set secret: supabase secrets set HMAC_SECRET=<hex>

const HMAC_SECRET = Deno.env.get("HMAC_SECRET");

async function computeHmac(key: Uint8Array, message: string): Promise<string> {
  const cryptoKey = await crypto.subtle.importKey(
    "raw",
    key,
    { name: "HMAC", hash: "SHA-256" },
    false,
    ["sign"],
  );
  const sig = await crypto.subtle.sign("HMAC", cryptoKey, new TextEncoder().encode(message));
  return Array.from(new Uint8Array(sig))
    .map((b) => b.toString(16).padStart(2, "0"))
    .join("");
}

function hexToBytes(hex: string): Uint8Array {
  const bytes = new Uint8Array(hex.length / 2);
  for (let i = 0; i < hex.length; i += 2) {
    bytes[i / 2] = parseInt(hex.substring(i, i + 2), 16);
  }
  return bytes;
}

interface Solve {
  id: string;
  time: number;
  date: number;
  scramble?: string | null;
  moves: string[];
  penalty?: string | null;
  integrity?: string | null;
}

function canonicalMessage(solve: Solve): string {
  const scramble = solve.scramble ?? "";
  const moves = solve.moves.join(",");
  const penalty = solve.penalty ?? "";
  return `${solve.id}|${solve.time}|${solve.date}|${scramble}|${moves}|${penalty}`;
}

Deno.serve(async (req) => {
  if (req.method !== "POST") {
    return new Response(JSON.stringify({ error: "Method not allowed" }), {
      status: 405,
      headers: { "Content-Type": "application/json" },
    });
  }

  if (!HMAC_SECRET) {
    return new Response(JSON.stringify({ error: "Server misconfigured" }), {
      status: 500,
      headers: { "Content-Type": "application/json" },
    });
  }

  const key = hexToBytes(HMAC_SECRET);

  let solve: Solve;
  try {
    solve = await req.json();
  } catch {
    return new Response(JSON.stringify({ error: "Invalid JSON" }), {
      status: 400,
      headers: { "Content-Type": "application/json" },
    });
  }

  if (!solve.integrity) {
    return new Response(JSON.stringify({ error: "Missing integrity field" }), {
      status: 403,
      headers: { "Content-Type": "application/json" },
    });
  }

  const message = canonicalMessage(solve);
  const expected = await computeHmac(key, message);

  if (expected !== solve.integrity) {
    return new Response(JSON.stringify({ error: "Integrity check failed" }), {
      status: 403,
      headers: { "Content-Type": "application/json" },
    });
  }

  return new Response(JSON.stringify({ ok: true }), {
    status: 200,
    headers: { "Content-Type": "application/json" },
  });
});

/**
 * Seeds a running API with a demo Family so the app has something to show.
 *
 *   pnpm --filter web seed                       # against http://localhost:8080
 *   $env.API_URL = "http://localhost:9000"; pnpm --filter web seed
 *
 * Signs in as the demo Owner if they already exist, and only adds what is missing,
 * so running it twice is safe. Uses the public API only, like the app does.
 */

const API = process.env.API_URL ?? "http://localhost:8080";
const PASSWORD = "correct horse battery";
const OWNER = { email: "jonas@example.com", name: "Jonas" };
const VIEWER = { email: "mormor@example.com", name: "Mormor" };

type Json = Record<string, unknown>;

async function send(method: string, path: string, cookie: string | null, body?: Json) {
  const headers: Record<string, string> = {};
  if (cookie) headers.cookie = cookie;
  if (body) headers["content-type"] = "application/json";
  const response = await fetch(`${API}${path}`, {
    method,
    headers,
    ...(body ? { body: JSON.stringify(body) } : {}),
  });
  const text = await response.text();
  const json = (text ? JSON.parse(text) : {}) as Json;
  const setCookie = response.headers.get("set-cookie");
  return { status: response.status, json, cookie: setCookie ? (setCookie.split(";")[0] ?? null) : null };
}

async function must(method: string, path: string, cookie: string | null, body?: Json): Promise<Json> {
  const reply = await send(method, path, cookie, body);
  if (reply.status >= 300) throw new Error(`${method} ${path} → ${reply.status} ${JSON.stringify(reply.json)}`);
  return reply.json;
}

/** Signs up, or signs in if the email is taken. Returns the Session cookie. */
async function session(person: { email: string; name: string }): Promise<string> {
  const up = await send("POST", "/api/users", null, { ...person, password: PASSWORD });
  if (up.status === 201 && up.cookie) return up.cookie;
  const signIn = await send("POST", "/api/sessions", null, { email: person.email, password: PASSWORD });
  if (signIn.status !== 200 || !signIn.cookie) throw new Error(`cannot sign in ${person.email}`);
  return signIn.cookie;
}

const sek = (kr: number) => ({ minor: Math.round(kr * 100), currency: "SEK" });

function daysAgo(days: number): string {
  const date = new Date();
  date.setUTCDate(date.getUTCDate() - days);
  return date.toISOString().slice(0, 10);
}

async function main() {
  const owner = await session(OWNER);
  const me = await must("GET", "/api/me", owner);
  const families = me.families as { id: string; name: string }[];
  if (families.length > 0) {
    console.log(`Already seeded: ${families[0]?.name}. Sign in as ${OWNER.email} / ${PASSWORD}`);
    return;
  }

  const family = await must("POST", "/api/families", owner, { name: "The Kacs", currency: "SEK" });
  const familyId = family.id as string;

  const invite = await must("POST", `/api/families/${familyId}/invitations`, owner, {
    email: VIEWER.email,
    role: "viewer",
  });
  const secret = (invite.link as string).replace("/join/", "");
  const viewer = await session(VIEWER);
  await must("POST", `/api/invitations/${secret}/accept`, viewer, {});

  const elsa = await must("POST", `/api/families/${familyId}/kids`, owner, { name: "Elsa" });
  const leo = await must("POST", `/api/families/${familyId}/kids`, owner, { name: "Leo" });
  const wallet = (kid: Json) => `/api/families/${familyId}/wallets/${kid.wallet_id as string}`;

  // Elsa: Main, a Savings account with a goal, and one for fun stuff.
  const elsaWallet = wallet(elsa);
  let view = await must("POST", `${elsaWallet}/accounts`, owner, {
    name: "Savings",
    savings_goal: { name: "Lego set", target: sek(800) },
  });
  view = await must("POST", `${elsaWallet}/accounts`, owner, { name: "Fun stuff" });
  const accounts = view.accounts as { id: string; name: string }[];
  const id = (name: string) => accounts.find((a) => a.name === name)?.id ?? "";

  const deposits: [number, string, string, number][] = [
    [20, "allowance", "Pocket money week 45", 24],
    [500, "gift", "Birthday money from mormor", 21],
    [20, "allowance", "Pocket money week 46", 17],
    [50, "chore", "Washed the car", 14],
    [20, "allowance", "Pocket money week 47", 10],
    [30, "chore", "Raked the leaves", 6],
    [20, "allowance", "Pocket money week 48", 3],
  ];
  for (const [kr, category, memo, ago] of deposits) {
    await must("POST", `${elsaWallet}/deposits`, owner, { amount: sek(kr), category, memo, occurred_on: daysAgo(ago) });
  }
  await must("POST", `${elsaWallet}/transfers`, owner, {
    from: id("Main"),
    to: id("Savings"),
    amount: sek(400),
    memo: "Saving for the Lego set",
    occurred_on: daysAgo(20),
  });
  await must("POST", `${elsaWallet}/transfers`, owner, {
    from: id("Main"),
    to: id("Savings"),
    amount: sek(140),
    memo: "More for the Lego set",
    occurred_on: daysAgo(4),
  });
  await must("POST", `${elsaWallet}/transfers`, owner, {
    from: id("Main"),
    to: id("Fun stuff"),
    amount: sek(80),
    memo: "To Fun stuff",
    occurred_on: daysAgo(9),
  });
  await must("POST", `${elsaWallet}/withdrawals`, owner, {
    account_id: id("Fun stuff"),
    amount: sek(35),
    category: "purchase",
    memo: "Comic book",
    occurred_on: daysAgo(2),
  });

  // Leo: only a Main account.
  const leoWallet = wallet(leo);
  await must("POST", `${leoWallet}/deposits`, owner, {
    amount: sek(10),
    category: "allowance",
    memo: "Pocket money",
    occurred_on: daysAgo(7),
  });
  await must("POST", `${leoWallet}/deposits`, owner, {
    amount: sek(75),
    category: "gift",
    memo: "From farfar",
    occurred_on: daysAgo(5),
  });
  const leoView = await must("GET", leoWallet, owner);
  const leoMain = (leoView.accounts as { id: string }[])[0]?.id ?? "";
  await must("POST", `${leoWallet}/withdrawals`, owner, {
    account_id: leoMain,
    amount: sek(15),
    category: "donation",
    memo: "Charity box at school",
    occurred_on: daysAgo(1),
  });

  console.log("Seeded The Kacs with Elsa and Leo.");
  console.log(`  Owner:  ${OWNER.email} / ${PASSWORD}`);
  console.log(`  Viewer: ${VIEWER.email} / ${PASSWORD}`);
}

main().catch((error: unknown) => {
  console.error(error);
  process.exit(1);
});

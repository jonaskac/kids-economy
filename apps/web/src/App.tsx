import { Button } from "@kids-economy/design-system";

export function App() {
  return (
    <main className="mx-auto flex min-h-dvh max-w-md flex-col gap-6 px-4 py-8">
      <h1 className="text-3xl font-bold">Kids Economy</h1>
      <section aria-labelledby="family-empty" className="rounded-card border border-line p-6">
        <h2 id="family-empty" className="text-lg font-semibold">
          No kids yet
        </h2>
        <p className="mt-1 text-muted">Add a kid to open their Wallet and record their first Deposit.</p>
        <Button className="mt-4 w-full">Add a kid</Button>
      </section>
    </main>
  );
}

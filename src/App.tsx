import { useEffect, useState } from "react";
import { getTheme, quit, type ThemeName } from "./lib/tauri";

// Step 1 shell: theme + quit keys only. The dashboard layout lands in step 3
// (docs/plan.md).
export default function App() {
  const [theme, setTheme] = useState<ThemeName | null>(null);

  useEffect(() => {
    getTheme()
      .then(setTheme)
      .catch(() => setTheme("fizx"));
  }, []);

  useEffect(() => {
    document.documentElement.classList.toggle("theme-upleb", theme === "upleb");
  }, [theme]);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape" || e.key === "q" || e.key === "Q") void quit();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);

  return (
    <main className="flex h-full flex-col items-center justify-center gap-3">
      <h1 className="font-mono text-6xl font-bold text-mauve">ding</h1>
      <p className="font-mono text-sm text-muted">
        {theme ?? "…"} · dashboard comes next
      </p>
    </main>
  );
}

import { useEffect, useState } from "react";
import { getTheme, quit, type ThemeName } from "./lib/tauri";
import { useSnapshot, type Tone } from "./lib/data";

const toneClass: Record<Tone, string> = {
  ok: "text-ok",
  accent: "text-accent",
  auburn: "text-auburn",
  muted: "text-muted",
};

// Step 2: a plain readout of the live data, to check the back end end to
// end. The dashboard layout replaces it in step 3 (docs/plan.md).
export default function App() {
  const [theme, setTheme] = useState<ThemeName | null>(null);
  const snap = useSnapshot();

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

  if (!snap) {
    return (
      <main className="flex h-full items-center justify-center font-mono text-muted">
        ding · {theme ?? "…"}
      </main>
    );
  }

  const { weather, calendar, services } = snap;
  const days = Object.keys(calendar.by_date).length;
  const events = Object.values(calendar.by_date).reduce((n, v) => n + v.length, 0);

  return (
    <main className="grid h-full grid-cols-3 gap-8 p-10 font-mono text-sm">
      <section className="space-y-2">
        <h2 className="text-mauve">weather {weather.moon}</h2>
        {weather.cities.map((c, i) => (
          <div key={c.label} className="flex justify-between">
            <span className="text-muted">{c.label}</span>
            <span>
              {c.temp || "—"} · aqi {weather.aqi[i]?.text || "—"}
            </span>
          </div>
        ))}
        {weather.sun && (
          <div className="text-muted">
            sun {weather.sun[0]} – {weather.sun[1]}
          </div>
        )}
        {weather.forecast.map((f) => (
          <div key={f.day} className="flex justify-between">
            <span className="text-muted">{f.day}</span>
            <span>{f.hilo}</span>
          </div>
        ))}
      </section>

      <section className="space-y-2">
        <h2 className="text-mauve">calendar · {calendar.status}</h2>
        {calendar.calendars.map((c) => (
          <div key={c.name} className="flex justify-between">
            <span style={c.color ? { color: `rgb(${c.color.join(" ")})` } : undefined}>{c.name}</span>
            <span className={c.error ? "text-alert" : "text-muted"}>
              {c.error ? "error" : c.next ? (c.next.time ?? "all day") : "—"}
            </span>
          </div>
        ))}
        <div className="text-muted">
          {events} events over {days} days
        </div>
      </section>

      <section className="space-y-2">
        <h2 className="text-mauve">daemons</h2>
        {[...services.chain, ...services.net].map((r) => (
          <div key={r.caption} className="flex justify-between">
            <span className="text-muted">{r.caption}</span>
            <span className={toneClass[r.tone]}>{r.value}</span>
          </div>
        ))}
      </section>
    </main>
  );
}

import { useEffect, useState } from "react";
import { getIdentity, getTheme, quit, type Identity, type ThemeName } from "./lib/tauri";
import { useSnapshot } from "./lib/data";
import { readPalette, type Palette } from "./dash/palette";
import Dashboard from "./dash/Dashboard";

export default function App() {
  const [theme, setTheme] = useState<ThemeName | null>(null);
  const [pal, setPal] = useState<Palette | null>(null);
  const [ident, setIdent] = useState<Identity | null>(null);
  const snap = useSnapshot();

  // Text is measured to place it, so wait for the bundled clock face
  // before the first draw (at most a second).
  const [fontReady, setFontReady] = useState(false);
  useEffect(() => {
    const done = () => setFontReady(true);
    const timer = window.setTimeout(done, 1000);
    document.fonts.load('bold 40px "Udash Clock"').then(done, done);
    return () => window.clearTimeout(timer);
  }, []);

  useEffect(() => {
    getTheme()
      .catch((): ThemeName => "fizx")
      .then((t) => {
        // class first, so the palette reads the right tokens
        document.documentElement.classList.toggle("theme-upleb", t === "upleb");
        setTheme(t);
        setPal(readPalette());
      });
    getIdentity().then(setIdent).catch(() => setIdent(null));
  }, []);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape" || e.key === "q" || e.key === "Q") void quit();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);

  if (!theme || !pal || !fontReady) return null;
  return <Dashboard snap={snap} pal={pal} ident={ident} />;
}

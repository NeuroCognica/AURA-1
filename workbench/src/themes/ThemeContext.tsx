import React, { createContext, useContext, useEffect, useMemo, useState } from "react";
import { ARCHETYPES, ArchetypeId, ArchetypeTheme, DEFAULT_ARCHETYPE } from "./archetypes";
import { playThemeSound } from "./sound";

export interface ThemeContextValue {
  current: ArchetypeTheme;
  setArchetype: (id: ArchetypeId) => void;
}

const ThemeContext = createContext<ThemeContextValue>({
  current: DEFAULT_ARCHETYPE,
  setArchetype: () => {},
});

export const useTheme = () => useContext(ThemeContext);

export const ThemeProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const [currentId, setCurrentId] = useState<ArchetypeId>(DEFAULT_ARCHETYPE.id);

  const current = useMemo(() => ARCHETYPES[currentId], [currentId]);

  useEffect(() => {
    const root = document.documentElement;
    Object.entries(current.cssVars).forEach(([k, v]) => {
      root.style.setProperty(k, v);
    });
    root.style.setProperty("--app-font-family", current.fontFamily);

    // Sound stub
    if (current.sound?.enter) {
      playThemeSound(current.sound.enter);
    }
  }, [current]);

  const value = useMemo(
    () => ({ current, setArchetype: (id: ArchetypeId) => setCurrentId(id) }),
    [current]
  );

  return <ThemeContext.Provider value={value}>{children}</ThemeContext.Provider>;
};

export default ThemeContext;

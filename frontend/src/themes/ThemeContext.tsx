import React, { createContext, useContext, useEffect, useMemo, useState } from "react";
import { ARCHETYPES, ArchetypeId, ArchetypeTheme, DEFAULT_ARCHETYPE } from "./archetypes";

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
    // Apply CSS variables to :root with smooth transition
    const root = document.documentElement;
    Object.entries(current.cssVars).forEach(([k, v]) => {
      root.style.setProperty(k, v);
    });
    // Apply font
    root.style.setProperty("--app-font-family", current.fontFamily);

    // Log the theme change
    console.info(`Archetype switched: ${current.displayName}`);

    // play sound stub
    if (current.sound?.enter) {
      // do not load audio in this phase — just log
      console.info(`Playing sound: ${current.sound.enter}`);
    }
  }, [current]);

  const value = useMemo(
    () => ({ current, setArchetype: (id: ArchetypeId) => setCurrentId(id) }),
    [current]
  );

  return <ThemeContext.Provider value={value}>{children}</ThemeContext.Provider>;
};

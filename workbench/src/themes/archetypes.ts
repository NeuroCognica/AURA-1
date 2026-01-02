export type ArchetypeId =
  | "architect"
  | "sentinel"
  | "oracle"
  | "explorer"
  | "mentor"
  | "empath"
  | "jester";

export interface ArchetypeTheme {
  id: ArchetypeId;
  displayName: string;
  cssVars: Record<string, string>;
  fontFamily: string;
  layoutHints?: {
    emphasize?: "left" | "center" | "right";
  };
  sound?: {
    enter?: string;
    ambient?: string;
  };
}

export const ARCHETYPES: Record<ArchetypeId, ArchetypeTheme> = {
  architect: {
    id: "architect",
    displayName: "Architect",
    fontFamily: "Inter, Roboto, system-ui, sans-serif",
    cssVars: {
      "--bg-primary": "#071124",
      "--bg-secondary": "#0b2740",
      "--text-primary": "#e6f0ff",
      "--accent": "#00d1ff",
      "--panel-glass": "rgba(255,255,255,0.04)",
      "--glow-color": "#00d1ff",
    },
    layoutHints: { emphasize: "left" },
    sound: { enter: "architect-chime", ambient: "architect-ambient" },
  },

  sentinel: {
    id: "sentinel",
    displayName: "Sentinel",
    fontFamily: "IBM Plex Sans, system-ui, sans-serif",
    cssVars: {
      "--bg-primary": "#0b0f12",
      "--bg-secondary": "#12161a",
      "--text-primary": "#dbe6e8",
      "--accent": "#ff4d4f",
      "--panel-glass": "rgba(255,255,255,0.03)",
      "--glow-color": "#ff4d4f",
    },
    layoutHints: { emphasize: "center" },
    sound: { enter: "sentinel-beacon", ambient: "sentinel-hum" },
  },

  oracle: {
    id: "oracle",
    displayName: "Oracle",
    fontFamily: "Merriweather, Georgia, serif",
    cssVars: {
      "--bg-primary": "#08132a",
      "--bg-secondary": "#11203a",
      "--text-primary": "#fff7e6",
      "--accent": "#ffd166",
      "--panel-glass": "rgba(255,255,230,0.03)",
      "--glow-color": "#ffd166",
    },
    layoutHints: { emphasize: "right" },
    sound: { enter: "oracle-tone", ambient: "oracle-wind" },
  },

  explorer: {
    id: "explorer",
    displayName: "Explorer",
    fontFamily: "Poppins, system-ui, sans-serif",
    cssVars: {
      "--bg-primary": "#052b14",
      "--bg-secondary": "#0b3f22",
      "--text-primary": "#eafff0",
      "--accent": "#7bff6f",
      "--panel-glass": "rgba(255,255,255,0.03)",
      "--glow-color": "#7bff6f",
    },
    layoutHints: { emphasize: "left" },
    sound: { enter: "explorer-spark", ambient: "explorer-breeze" },
  },

  mentor: {
    id: "mentor",
    displayName: "Mentor",
    fontFamily: "Georgia, 'Times New Roman', serif",
    cssVars: {
      "--bg-primary": "#2b1f12",
      "--bg-secondary": "#3a2a18",
      "--text-primary": "#fff8e6",
      "--accent": "#ffb84d",
      "--panel-glass": "rgba(255,255,240,0.04)",
      "--glow-color": "#ffb84d",
    },
    layoutHints: { emphasize: "center" },
    sound: { enter: "mentor-bell", ambient: "mentor-quiet" },
  },

  empath: {
    id: "empath",
    displayName: "Empath",
    fontFamily: "Comfortaa, system-ui, sans-serif",
    cssVars: {
      "--bg-primary": "#2b0b17",
      "--bg-secondary": "#3a0f24",
      "--text-primary": "#ffeef6",
      "--accent": "#ff63a5",
      "--panel-glass": "rgba(255,240,245,0.04)",
      "--glow-color": "#ff63a5",
    },
    layoutHints: { emphasize: "right" },
    sound: { enter: "empath-sigh", ambient: "empath-pad" },
  },

  jester: {
    id: "jester",
    displayName: "Jester",
    fontFamily: "Comic Neue, 'Segoe UI', sans-serif",
    cssVars: {
      "--bg-primary": "#1b0b2b",
      "--bg-secondary": "#2a123f",
      "--text-primary": "#fff5e6",
      "--accent": "#ffd100",
      "--panel-glass": "rgba(255,255,240,0.05)",
      "--glow-color": "#ffd100",
    },
    layoutHints: { emphasize: "left" },
    sound: { enter: "jester-burst", ambient: "jester-tink" },
  },
};

export const DEFAULT_ARCHETYPE: ArchetypeTheme = ARCHETYPES.architect;

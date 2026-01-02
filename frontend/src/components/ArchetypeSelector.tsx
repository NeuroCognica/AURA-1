import React from "react";
import { ARCHETYPES, ArchetypeId } from "../themes/archetypes";
import { useTheme } from "../themes/ThemeContext";

const ArchetypeSelector: React.FC = () => {
  const { current, setArchetype } = useTheme();

  return (
    <div style={{ display: "flex", gap: 8, alignItems: "center", marginBottom: 12 }}>
      <label style={{ fontSize: 14 }}>Archetype:</label>
      <select
        value={current.id}
        onChange={async (e) => {
          const id = e.target.value as ArchetypeId;
          try {
            const res = await fetch("/api/archetype/activate", {
              method: "POST",
              headers: { "Content-Type": "application/json" },
              body: JSON.stringify({ archetype: id, ritual: true }),
            });
            if (res.ok) {
              // server returns the activation payload; we use it to set local theme
              const j = await res.json();
              // Apply returned theme_vars if present
              if (j?.theme_vars && typeof j.theme_vars === "object") {
                Object.entries(j.theme_vars).forEach(([k, v]) => {
                  document.documentElement.style.setProperty(k, v as string);
                });
              }
              setArchetype(id);
            } else {
              // fallback to local set
              setArchetype(id);
            }
          } catch (err) {
            console.warn("archetype activation failed", err);
            setArchetype(id);
          }
        }}
        style={{ padding: 6, borderRadius: 6 }}
      >
        {Object.values(ARCHETYPES).map((t) => (
          <option key={t.id} value={t.id}>
            {t.displayName}
          </option>
        ))}
      </select>
    </div>
  );
};

export default ArchetypeSelector;

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
        onChange={(e) => setArchetype(e.target.value as ArchetypeId)}
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

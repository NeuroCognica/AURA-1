import React from 'react'
import { useTheme } from '../themes/ThemeContext'
import { ARCHETYPES } from '../themes/archetypes'
import ChatPanel from './ChatPanel'
import ChatHistory from './ChatHistory'

export default function LeftColumn() {
  const { current, setArchetype } = useTheme()

  return (
    <div className="flex flex-col h-full gap-4">
      <div>
        <label className="block text-sm text-neutral-400">Archetype</label>
        <select
          className="w-full mt-2 p-2 bg-neutral-800 border border-neutral-700 rounded"
          value={current.id}
          onChange={(e) => setArchetype(e.target.value as any)}
        >
          {Object.values(ARCHETYPES).map((t) => (
            <option key={t.id} value={t.id}>
              {t.displayName}
            </option>
          ))}
        </select>
      </div>

      <div>
        <label className="block text-sm text-neutral-400">Council Mode</label>
        <input type="checkbox" disabled className="mt-2" />
      </div>

      <div className={`flex-1 flex flex-col p-0 rounded ${current.layoutHints?.emphasize === 'left' ? 'emphasize-left' : current.layoutHints?.emphasize === 'center' ? 'emphasize-center' : current.layoutHints?.emphasize === 'right' ? 'emphasize-right' : ''}`} style={{ background: 'var(--panel-glass)' }}>
        <div className="flex-1 p-3">
          <ChatPanel />
        </div>

        <div className="mt-2">
          <ChatHistory />
        </div>
      </div>
    </div>
  )
}

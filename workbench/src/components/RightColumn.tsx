import React from 'react'
import NotepadPanel from './NotepadPanel'

export default function RightColumn() {
  return (
    <div className="flex flex-col h-full gap-4">
      <div className="grid grid-cols-2 gap-4">
        <div className="bg-neutral-800 p-3 rounded border border-neutral-700">
          <div className="font-semibold">Archetype Profile</div>
          <div className="text-sm text-neutral-400 mt-2">Name: Sentinel</div>
        </div>

        <div className="bg-neutral-800 p-3 rounded border border-neutral-700 flex items-center justify-center">
          <div className="text-neutral-500">Archetype Image / Face (stub)</div>
        </div>
      </div>

      <div className="flex-1">
        <NotepadPanel />
      </div>
    </div>
  )
}

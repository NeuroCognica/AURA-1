import React from 'react'

export default function LeftColumn() {
  return (
    <div className="flex flex-col gap-4">
      <div>
        <label className="block text-sm text-neutral-400">Archetype</label>
        <select className="w-full mt-2 p-2 bg-neutral-800 border border-neutral-700 rounded" disabled>
          <option>Sentinel</option>
          <option>Architect</option>
          <option>Explorer</option>
        </select>
      </div>

      <div>
        <label className="block text-sm text-neutral-400">Council Mode</label>
        <input type="checkbox" disabled className="mt-2" />
      </div>

      <div className="flex-1 bg-neutral-800 p-3 rounded border border-neutral-700">
        <div className="text-sm text-neutral-400">Chat (placeholder)</div>
        <div className="mt-4 text-xs text-neutral-500">No messages — static placeholder.</div>
      </div>

      <div>
        <input className="w-full p-2 bg-neutral-800 border border-neutral-700 rounded" placeholder="Type a message" disabled />
      </div>
    </div>
  )
}

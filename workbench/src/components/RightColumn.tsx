import React from 'react'

export default function RightColumn() {
  return (
    <div className="flex flex-col gap-4">
      <div className="bg-neutral-800 p-3 rounded border border-neutral-700">
        <div className="font-semibold">Archetype Profile</div>
        <div className="text-sm text-neutral-400 mt-2">Name: Sentinel</div>
      </div>

      <div className="flex-1 bg-neutral-800 p-3 rounded border border-neutral-700">
        <div className="font-semibold">Notepad</div>
        <textarea className="w-full h-40 mt-2 p-2 bg-neutral-900 text-neutral-100 rounded" placeholder="Notes..." />
      </div>

      <button className="w-full py-2 bg-neutral-700 rounded disabled:opacity-50" disabled>Submit</button>
    </div>
  )
}

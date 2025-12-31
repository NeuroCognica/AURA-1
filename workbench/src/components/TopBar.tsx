import React from 'react'

export default function TopBar() {
  return (
    <div className="h-14 flex items-center justify-between px-4 bg-neutral-800 border-b border-neutral-700">
      <div className="flex items-center gap-3">
        <div className="text-lg font-semibold">AURA Workbench</div>
        <div className="text-sm text-neutral-400">(Electron Preview)</div>
      </div>
      <div className="flex items-center gap-2">
        <button className="px-3 py-1 bg-neutral-700 rounded disabled:opacity-50" disabled>Nav 1</button>
        <button className="px-3 py-1 bg-neutral-700 rounded disabled:opacity-50" disabled>Nav 2</button>
        <div className="px-3 py-1 bg-neutral-700 rounded">Window</div>
      </div>
    </div>
  )
}

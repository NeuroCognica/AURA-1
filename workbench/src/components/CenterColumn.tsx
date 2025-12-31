import React from 'react'

export default function CenterColumn() {
  return (
    <div className="flex flex-col gap-4">
      <div className="flex-1 bg-neutral-800 rounded border border-neutral-700 flex items-center justify-center">
        <div className="text-neutral-500">Viewport / Three.js canvas (stub)</div>
      </div>

      <div className="grid grid-cols-2 gap-4">
        <div className="bg-neutral-800 p-3 rounded border border-neutral-700">
          <div className="font-semibold">System Monitor</div>
          <div className="text-sm text-neutral-400 mt-2">CPU: 12% | Memory: 1.2 GB</div>
          <div className="text-sm text-neutral-400">WS: connected</div>
        </div>

        <div className="bg-neutral-800 p-3 rounded border border-neutral-700">
          <div className="font-semibold">Calendar</div>
          <div className="text-sm text-neutral-400 mt-2">No events — placeholder</div>
        </div>
      </div>
    </div>
  )
}

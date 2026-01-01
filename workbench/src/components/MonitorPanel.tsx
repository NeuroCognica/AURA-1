import React from 'react'

export default function MonitorPanel() {
  return (
    <div className="bg-neutral-800 p-3 rounded border border-neutral-700 min-h-[120px]">
      <div className="font-semibold">System Monitor</div>
      <div className="text-sm text-neutral-400 mt-2">CPU: 12% | Memory: 1.2 GB</div>
    </div>
  )
}

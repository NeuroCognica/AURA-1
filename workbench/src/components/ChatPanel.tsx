import React from 'react'

export default function ChatPanel() {
  return (
    <div className="flex flex-col h-full">
      <div className="flex-1 overflow-auto bg-neutral-900 p-3 rounded border border-neutral-700">
        <div className="text-neutral-400">Chat messages will appear here. (placeholder)</div>
      </div>

      <div className="mt-2 flex items-center gap-2">
        <input className="flex-1 p-2 bg-neutral-800 border border-neutral-700 rounded" placeholder="Type here" />
        <button className="px-4 py-2 bg-indigo-600 text-white rounded">Send</button>
      </div>
    </div>
  )
}

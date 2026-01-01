import React from 'react'

export default function NotepadPanel() {
  return (
    <div className="flex flex-col h-full">
      <div className="flex-1 bg-neutral-900 p-3 rounded border border-neutral-700 overflow-auto">
        <div className="font-semibold">Notepad</div>
        <textarea className="w-full h-full mt-2 p-2 bg-neutral-900 text-neutral-100 rounded" placeholder="Write notes..." />
      </div>
      <div className="mt-2">
        <button className="w-full py-2 bg-indigo-600 text-white rounded">Submit</button>
      </div>
    </div>
  )
}

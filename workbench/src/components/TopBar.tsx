import React from 'react'

interface TopBarProps {
  username?: string
  onLogout?: () => void
}

export default function TopBar({ username, onLogout }: TopBarProps) {
  return (
    <div className="h-14 flex items-center justify-between px-4 bg-neutral-800 border-b border-neutral-700">
      <div className="flex items-center gap-3">
        <div className="text-lg font-semibold">AURA Workbench</div>
        <div className="text-sm text-neutral-400">(Electron Preview)</div>
      </div>
      <div className="flex items-center gap-3">
        {username && (
          <div className="flex items-center gap-2">
            <div className="text-sm text-neutral-400">
              Logged in as: <span className="text-blue-400 font-medium">{username}</span>
            </div>
            {onLogout && (
              <button
                onClick={onLogout}
                className="px-3 py-1 bg-red-900/30 hover:bg-red-900/50 border border-red-500/50 rounded text-sm text-red-400 transition-colors"
              >
                Logout
              </button>
            )}
          </div>
        )}
        <div className="px-3 py-1 bg-neutral-700 rounded text-sm">Window Controls</div>
      </div>
    </div>
  )
}

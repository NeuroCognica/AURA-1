import React from 'react'
import TopBar from './components/TopBar'
import LeftColumn from './components/LeftColumn'
import CenterColumn from './components/CenterColumn'
import RightColumn from './components/RightColumn'

export default function App() {
  return (
    <div className="h-screen w-screen bg-neutral-900 text-neutral-100">
      <TopBar />
      <div className="flex h-[calc(100vh-56px)]">
        <aside className="w-72 border-r border-neutral-800 p-4">
          <LeftColumn />
        </aside>
        <main className="flex-1 p-4 overflow-auto">
          <CenterColumn />
        </main>
        <aside className="w-80 border-l border-neutral-800 p-4">
          <RightColumn />
        </aside>
      </div>
    </div>
  )
}

import React from 'react'
import TopBar from './components/TopBar'
import LeftColumn from './components/LeftColumn'
import CenterColumn from './components/CenterColumn'
import RightColumn from './components/RightColumn'

export default function App() {
  return (
    <div className="h-screen w-screen bg-neutral-900 text-neutral-100">
      <TopBar />

      <div className="h-[calc(100vh-56px)] p-4">
        <div className="h-full grid gap-4 grid-cols-[minmax(320px,1.2fr)_minmax(260px,0.8fr)_minmax(360px,1fr)]">
          <section className="h-full border border-neutral-800 rounded-lg overflow-hidden">
            <div className="h-full p-4">
              <LeftColumn />
            </div>
          </section>

          <section className="h-full border border-neutral-800 rounded-lg overflow-hidden">
            <div className="h-full p-4">
              <CenterColumn />
            </div>
          </section>

          <section className="h-full border border-neutral-800 rounded-lg overflow-hidden">
            <div className="h-full p-4">
              <RightColumn />
            </div>
          </section>
        </div>
      </div>
    </div>
  )
}

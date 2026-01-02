import React, { useState, useEffect } from 'react'
import TopBar from './components/TopBar'
import LeftColumn from './components/LeftColumn'
import CenterColumn from './components/CenterColumn'
import RightColumn from './components/RightColumn'
import LoginScreen from './components/LoginScreen'

export default function App() {
  const [username, setUsername] = useState<string | null>(null)

  useEffect(() => {
    // Check if user is already logged in
    const storedUsername = localStorage.getItem('aura_username')
    if (storedUsername) {
      setUsername(storedUsername)
    }
  }, [])

  const handleLogin = (username: string) => {
    setUsername(username)
  }

  const handleLogout = () => {
    localStorage.removeItem('aura_username')
    setUsername(null)
  }

  // Show login screen if not logged in
  if (!username) {
    return <LoginScreen onLogin={handleLogin} />
  }

  return (
    <div className="h-screen w-screen bg-neutral-900 text-neutral-100">
      <TopBar username={username} onLogout={handleLogout} />

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

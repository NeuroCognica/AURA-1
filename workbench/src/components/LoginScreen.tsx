import React, { useState } from 'react'

interface LoginScreenProps {
  onLogin: (username: string) => void
}

export default function LoginScreen({ onLogin }: LoginScreenProps) {
  const [username, setUsername] = useState('')
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [mode, setMode] = useState<'login' | 'create'>('login')

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    
    if (!username.trim()) {
      setError('Username is required')
      return
    }

    setIsLoading(true)
    setError(null)

    try {
      const endpoint = mode === 'login' 
        ? 'http://localhost:8080/api/account/login'
        : 'http://localhost:8080/api/account/create'

      const response = await fetch(endpoint, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ username: username.trim() }),
      })

      if (!response.ok) {
        const errorData = await response.json().catch(() => ({ error: 'Unknown error' }))
        throw new Error(errorData.error || `HTTP ${response.status}`)
      }

      const data = await response.json()
      
      // Store username in localStorage
      localStorage.setItem('aura_username', username.trim())
      
      // Call onLogin callback
      onLogin(username.trim())
    } catch (err) {
      console.error('Login error:', err)
      setError(err instanceof Error ? err.message : 'Failed to connect to AURA backend')
    } finally {
      setIsLoading(false)
    }
  }

  return (
    <div className="h-screen w-screen bg-neutral-900 flex items-center justify-center">
      <div className="w-full max-w-md p-8">
        <div className="text-center mb-8">
          <h1 className="text-4xl font-bold text-blue-400 mb-2">AURA-1</h1>
          <p className="text-neutral-400">Constitutional AI System</p>
        </div>

        <div className="bg-neutral-800 rounded-lg p-6 border border-neutral-700">
          <div className="flex gap-2 mb-6">
            <button
              onClick={() => setMode('login')}
              className={`flex-1 py-2 px-4 rounded transition-colors ${
                mode === 'login'
                  ? 'bg-blue-600 text-white'
                  : 'bg-neutral-700 text-neutral-300 hover:bg-neutral-600'
              }`}
            >
              Login
            </button>
            <button
              onClick={() => setMode('create')}
              className={`flex-1 py-2 px-4 rounded transition-colors ${
                mode === 'create'
                  ? 'bg-blue-600 text-white'
                  : 'bg-neutral-700 text-neutral-300 hover:bg-neutral-600'
              }`}
            >
              Create Account
            </button>
          </div>

          <form onSubmit={handleSubmit}>
            <div className="mb-4">
              <label htmlFor="username" className="block text-sm font-medium text-neutral-300 mb-2">
                Username
              </label>
              <input
                id="username"
                type="text"
                value={username}
                onChange={(e) => setUsername(e.target.value)}
                placeholder="Enter your username"
                className="w-full px-4 py-2 bg-neutral-700 border border-neutral-600 rounded text-white placeholder-neutral-500 focus:outline-none focus:border-blue-500"
                disabled={isLoading}
                autoFocus
              />
            </div>

            {error && (
              <div className="mb-4 p-3 bg-red-900/20 border border-red-500/50 rounded text-red-400 text-sm">
                {error}
              </div>
            )}

            <button
              type="submit"
              disabled={isLoading}
              className="w-full py-2 px-4 bg-blue-600 hover:bg-blue-700 disabled:bg-neutral-600 disabled:cursor-not-allowed text-white rounded font-medium transition-colors"
            >
              {isLoading ? 'Connecting...' : mode === 'login' ? 'Login' : 'Create Account'}
            </button>
          </form>

          <div className="mt-6 pt-6 border-t border-neutral-700">
            <p className="text-xs text-neutral-500 text-center">
              {mode === 'login' 
                ? "First time? Click 'Create Account' above."
                : "Accounts are stored locally with Forever Law covenant."}
            </p>
          </div>
        </div>

        <div className="mt-6 text-center">
          <p className="text-sm text-neutral-600">
            Backend: <span className="text-blue-400">localhost:8080</span>
          </p>
          <p className="text-xs text-neutral-700 mt-2">
            Forever Law enforced • Constitutional governance active
          </p>
        </div>
      </div>
    </div>
  )
}

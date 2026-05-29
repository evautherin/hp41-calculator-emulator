import React, { useEffect, useState } from 'react'
import ReactDOM from 'react-dom/client'
import App from './App'
import { computeScale, DESIGN_WIDTH, DESIGN_HEIGHT } from './scale'
import './index.css'
// D-48.11: themes.css must come after index.css so [data-theme] selector specificity wins.
import './themes.css'

/**
 * Wraps <App/> in a fixed-size (DESIGN_WIDTH x DESIGN_HEIGHT) box and applies a
 * uniform CSS transform so the layout fits the current window without scrolling.
 * Used by the macOS menu-bar popover (which may be shorter than 1020px) and any
 * small display. On a full-height window the scale is 1 (no visual change).
 */
function ScaledApp(): React.ReactElement {
  const [scale, setScale] = useState(() =>
    computeScale(window.innerWidth, window.innerHeight),
  )

  useEffect(() => {
    const update = () => setScale(computeScale(window.innerWidth, window.innerHeight))
    update()
    window.addEventListener('resize', update)
    return () => window.removeEventListener('resize', update)
  }, [])

  return (
    <div
      style={{
        width: '100vw',
        height: '100vh',
        overflow: 'hidden',
        display: 'flex',
        justifyContent: 'center',
        alignItems: 'flex-start',
      }}
    >
      <div
        style={{
          width: DESIGN_WIDTH,
          height: DESIGN_HEIGHT,
          transform: `scale(${scale})`,
          transformOrigin: 'top center',
          flex: '0 0 auto',
        }}
      >
        <App />
      </div>
    </div>
  )
}

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <ScaledApp />
  </React.StrictMode>,
)

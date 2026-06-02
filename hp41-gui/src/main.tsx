import React, { useEffect, useState } from 'react'
import ReactDOM from 'react-dom/client'
import App from './App'
import { computeScale, DESIGN_WIDTH, DESIGN_HEIGHT } from './scale'
import './index.css'
// D-48.11: themes.css must come after index.css so [data-theme] selector specificity wins.
import './themes.css'

/**
 * Wraps <App/> in a fixed-size (DESIGN_WIDTH x DESIGN_HEIGHT) box and applies a
 * uniform CSS transform so the layout fills the current window without distortion.
 * Upscales up to MAX_SCALE on large viewports (macOS window mode, 4K displays) and
 * downscales below 1 on small viewports (macOS menu-bar popover, short windows) so
 * nothing is clipped. transformOrigin 'top center' keeps the scaled box anchored at
 * the top; alignItems 'flex-start' on the outer container prevents vertical centering
 * gaps on tall viewports.
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

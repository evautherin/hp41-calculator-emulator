import React, { useEffect, useRef, useState } from 'react'
import ReactDOM from 'react-dom/client'
import App from './App'
import { computeScale, DESIGN_WIDTH, DESIGN_HEIGHT } from './scale'
import './index.css'
// D-48.11: themes.css must come after index.css so [data-theme] selector specificity wins.
import './themes.css'

/**
 * Wraps <App/> in a content-hugging box and applies a uniform CSS transform so
 * the layout fills the current window without distortion.
 *
 * Measure-and-fit: instead of a hardcoded design box, a ResizeObserver watches
 * the INNER content node (the div wrapping <App/>). That node has no explicit
 * width/height so it collapses to the natural size of .calculator (392px wide,
 * height auto). offsetWidth/offsetHeight read the LAYOUT size — they are not
 * affected by the CSS transform applied to the same node, so there is no
 * measurement/feedback loop. The measured size is fed to computeScale on every
 * content resize AND on every window resize.
 *
 * Upscales up to MAX_SCALE on large viewports (macOS window mode, 4K displays)
 * and downscales below 1 on small viewports (macOS menu-bar popover, short
 * windows) so nothing is clipped. transformOrigin 'top center' keeps the scaled
 * box anchored at the top; alignItems 'flex-start' on the outer container
 * prevents vertical centering gaps on tall viewports.
 *
 * DESIGN_WIDTH/DESIGN_HEIGHT are used only as the pre-measurement fallback for
 * the first paint; after the first ResizeObserver callback the real measured
 * dimensions drive scaling.
 */
function ScaledApp(): React.ReactElement {
  const contentRef = useRef<HTMLDivElement>(null)

  // Pre-measurement first paint: fall back to design constants so there is no
  // blank flash before the ResizeObserver callback fires.
  const [scale, setScale] = useState(() =>
    computeScale(window.innerWidth, window.innerHeight, DESIGN_WIDTH, DESIGN_HEIGHT),
  )

  useEffect(() => {
    // Recompute scale from the current window size and the measured content size.
    const recompute = () => {
      const node = contentRef.current
      const measuredW = node ? node.offsetWidth : DESIGN_WIDTH
      const measuredH = node ? node.offsetHeight : DESIGN_HEIGHT
      setScale(computeScale(window.innerWidth, window.innerHeight, measuredW, measuredH))
    }

    // Re-measure across a few frames to outlast async viewport settling — in
    // particular the iOS virtual-keyboard dismiss animation, which restores the
    // viewport AFTER the overlay-close event fires and without a reliable
    // window 'resize' on WKWebView. A single synchronous recompute would read
    // the still-shrunk viewport and leave the keypad clipped.
    const timers: number[] = []
    const recomputeSoon = () => {
      recompute()
      requestAnimationFrame(recompute)
      timers.push(window.setTimeout(recompute, 350))
    }

    // (a) Window resize listener
    window.addEventListener('resize', recompute)

    // (b) Overlay-close re-fit — App dispatches this when the help/settings
    // overlay opens or closes (see App.tsx). The "soon" variant re-measures over
    // ~350ms so the iOS keyboard-dismiss restore is captured.
    window.addEventListener('hp41:recompute-scale', recomputeSoon)

    // (c) visualViewport — the reliable iOS signal for virtual-keyboard show/hide
    // and pinch-zoom; the plain window 'resize' event is flaky on WKWebView for
    // keyboard dismissal. Guarded for environments without the API.
    const vv = window.visualViewport
    if (vv) {
      vv.addEventListener('resize', recompute)
      vv.addEventListener('scroll', recompute)
    }

    // (d) Content resize observer — fires whenever the calculator's layout height
    // changes (different viewport, font-metric variance, etc.).
    // Guard: ResizeObserver is not available in jsdom test environments.
    let ro: ResizeObserver | null = null
    if (typeof ResizeObserver !== 'undefined') {
      ro = new ResizeObserver(recompute)
      if (contentRef.current) {
        ro.observe(contentRef.current)
      }
    }

    // Initial measurement after mount (ResizeObserver fires asynchronously on
    // first observation, so also call synchronously to update before first paint).
    recompute()

    return () => {
      window.removeEventListener('resize', recompute)
      window.removeEventListener('hp41:recompute-scale', recomputeSoon)
      if (vv) {
        vv.removeEventListener('resize', recompute)
        vv.removeEventListener('scroll', recompute)
      }
      if (ro) ro.disconnect()
      timers.forEach(t => window.clearTimeout(t))
    }
  }, [])

  return (
    <div
      style={{
        // Use 100% (not 100vw/100vh) so the outer wrapper tracks the fixed,
        // locked body rather than the iOS toolbar-inclusive viewport unit.
        // This eliminates the few-px scroll that 100vh can introduce on
        // mobile browsers where 100vh > the visible area (#iOS-100vh quirk).
        width: '100%',
        height: '100%',
        overflow: 'hidden',
        display: 'flex',
        justifyContent: 'center',
        alignItems: 'flex-start',
        // Belt-and-suspenders: prevents a white flash on the wrapper itself
        // before the CSS cascade applies the html/body background.
        background: 'var(--calc-bg, #0d0d0d)',
      }}
    >
      <div
        ref={contentRef}
        style={{
          display: 'inline-block',
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

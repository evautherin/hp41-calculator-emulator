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
// Height (px) of the BottomSheet collapsed peek. Must match .bottom-sheet max-height
// in App.css. Used by recompute() to reserve the peek so the keypad scales above the sheet.
const BOTTOM_SHEET_PEEK = 32;

function ScaledApp(): React.ReactElement {
  const contentRef = useRef<HTMLDivElement>(null)
  // outerRef: ref on the outer wrapper div (OUTSIDE the transform) so recompute can
  // measure the content-box area after safe-area padding is applied by the browser.
  const outerRef = useRef<HTMLDivElement>(null)

  // Pre-measurement first paint: fall back to design constants so there is no
  // blank flash before the ResizeObserver callback fires.
  const [scale, setScale] = useState(() =>
    computeScale(window.innerWidth, window.innerHeight, DESIGN_WIDTH, DESIGN_HEIGHT),
  )

  useEffect(() => {
    // Recompute scale from the available content-box area (outer wrapper minus safe-area
    // padding) and the measured content size. Uses the outer wrapper's content box rather
    // than window.innerWidth/innerHeight so that env(safe-area-inset-*) padding applied
    // to the outer wrapper (at device pixels, outside the CSS transform) is already
    // subtracted — no separate :root env() probe element needed.
    const recompute = () => {
      const node = contentRef.current
      const measuredW = node ? node.offsetWidth : DESIGN_WIDTH
      const measuredH = node ? node.offsetHeight : DESIGN_HEIGHT

      // Measure the outer wrapper's content box (area inside safe-area padding).
      // clientWidth/clientHeight exclude scrollbars; subtracting computed padding yields
      // the inset-reduced area available for the scaled calculator.
      // WHY content-box: it reflects the padding actually applied by env() and needs no
      // separate hidden probe element. Falls back to window.inner* when outerRef is null
      // (first paint / jsdom test environment where ref is not attached).
      let availW = window.innerWidth
      let availH = window.innerHeight
      const outerEl = outerRef.current
      if (outerEl) {
        const cs = getComputedStyle(outerEl)
        availW = outerEl.clientWidth - parseFloat(cs.paddingLeft) - parseFloat(cs.paddingRight)
        availH = outerEl.clientHeight - parseFloat(cs.paddingTop) - parseFloat(cs.paddingBottom)
      }

      // Reserve the BottomSheet peek height when any sheet is mounted in the DOM
      // (the sheet mounts/unmounts with visibility; recompute is re-triggered on
      // PRGM toggle by App.tsx, so this DOM query always sees the current state).
      const peek = document.querySelector('.bottom-sheet') ? BOTTOM_SHEET_PEEK : 0;

      setScale(computeScale(availW, availH, measuredW, measuredH, peek))
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

    // (c2) Pinch-zoom lock — WKWebView honours the viewport meta's
    // maximum-scale/user-scalable, but the WebKit gesture* events are the
    // reliable belt-and-suspenders: preventing them blocks two-finger pinch from
    // panning/offsetting the CSS-scaled calculator. Single-finger taps and
    // one-finger scroll (help overlay, stack panel) are unaffected — gesture*
    // only fires for multi-touch, and the touchmove guard checks touches.length.
    const preventGesture = (e: Event) => e.preventDefault()
    const preventMultiTouch = (e: TouchEvent) => {
      if (e.touches.length > 1) e.preventDefault()
    }
    document.addEventListener('gesturestart', preventGesture, { passive: false })
    document.addEventListener('gesturechange', preventGesture, { passive: false })
    document.addEventListener('gestureend', preventGesture, { passive: false })
    document.addEventListener('touchmove', preventMultiTouch, { passive: false })

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
      document.removeEventListener('gesturestart', preventGesture)
      document.removeEventListener('gesturechange', preventGesture)
      document.removeEventListener('gestureend', preventGesture)
      document.removeEventListener('touchmove', preventMultiTouch)
    }
  }, [])

  return (
    <div
      ref={outerRef}
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
        // Safe-area insets applied ONCE at the outer frame (OUTSIDE the CSS transform).
        // Because this node is not scaled, the Dynamic Island / home-indicator gaps render
        // at full device pixels regardless of the scale factor. On desktop env() = 0 →
        // zero padding → unchanged layout. Per-component insets in .calculator-safe-area
        // and .help-overlay-header are removed (they lived INSIDE the transform and
        // rendered as scale × inset, causing a double-sized top gap on iOS). (mxg)
        boxSizing: 'border-box',
        paddingTop: 'env(safe-area-inset-top, 0px)',
        paddingRight: 'env(safe-area-inset-right, 0px)',
        paddingBottom: 'env(safe-area-inset-bottom, 0px)',
        paddingLeft: 'env(safe-area-inset-left, 0px)',
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

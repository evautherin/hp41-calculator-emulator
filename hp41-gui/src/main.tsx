import React from 'react'
import ReactDOM from 'react-dom/client'
import App from './App'
import './index.css'
// D-48.11: themes.css must come after index.css so [data-theme] selector specificity wins.
import './themes.css'

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
)

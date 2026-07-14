// Test setup file
import '@testing-library/jest-dom'
// Import any global mocks or setup here

// Mock resizeObserver if needed
global.ResizeObserver = class ResizeObserver {
  observe() {}
  unobserve() {}
  disconnect() {}
}

// Mock IntersectionObserver if needed
global.IntersectionObserver = class IntersectionObserver {
  observe() {}
  unobserve() {}
  disconnect() {}
}

// MatchMedia matcher
window.matchMedia = window.matchMedia || function() {
  return {
    matches: false,
    media: '',
    onchange: null,
    addListener: () => {}, // deprecated
    removeListener: () => {}, // deprecated
    addEventListener: () => {},
    removeEventListener: () => {},
    dispatchEvent: () => false,
  }
}
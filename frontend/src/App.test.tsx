import { render, screen } from '@testing-library/react'
import { MemoryRouter } from 'react-router-dom'
import App from './App'

describe('App routing', () => {
  it('renders input page on root route', () => {
    render(
      <MemoryRouter initialEntries={['/']}>
        <App />
      </MemoryRouter>,
    )

    expect(
      screen.getByRole('heading', { name: /capture first\. structure later\./i }),
    ).toBeInTheDocument()
    expect(screen.getByRole('heading', { name: /quick capture/i })).toBeInTheDocument()
    expect(screen.getByRole('heading', { name: /import queue/i })).toBeInTheDocument()
  })

  it('redirects unknown routes back to input page', () => {
    render(
      <MemoryRouter initialEntries={['/missing']}>
        <App />
      </MemoryRouter>,
    )

    expect(screen.getByRole('heading', { name: /quick capture/i })).toBeInTheDocument()
  })

  it('renders raw logs page on logs route', () => {
    render(
      <MemoryRouter initialEntries={['/logs']}>
        <App />
      </MemoryRouter>,
    )

    expect(screen.getByRole('heading', { name: /raw logs archive/i })).toBeInTheDocument()
  })

  it('renders ai page on ai route', () => {
    render(
      <MemoryRouter initialEntries={['/ai']}>
        <App />
      </MemoryRouter>,
    )

    expect(screen.getByRole('heading', { name: /ai console/i })).toBeInTheDocument()
    expect(screen.getByRole('button', { name: /run ai/i })).toBeInTheDocument()
  })
})

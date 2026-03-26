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

    expect(screen.getByRole('heading', { name: /quick input/i })).toBeInTheDocument()
    expect(
      screen.getByText(/先保留原始表达，再进入结构化链路/i),
    ).toBeInTheDocument()
    expect(screen.getByRole('heading', { name: /import raw logs/i })).toBeInTheDocument()
  })

  it('redirects unknown routes back to input page', () => {
    render(
      <MemoryRouter initialEntries={['/missing']}>
        <App />
      </MemoryRouter>,
    )

    expect(screen.getByRole('heading', { name: /quick input/i })).toBeInTheDocument()
  })

  it('renders raw logs page on logs route', () => {
    render(
      <MemoryRouter initialEntries={['/logs']}>
        <App />
      </MemoryRouter>,
    )

    expect(screen.getByRole('heading', { name: /raw logs/i })).toBeInTheDocument()
  })
})

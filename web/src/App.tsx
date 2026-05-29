import { NavLink, Outlet } from "react-router-dom";

/** App shell: top navigation + routed page outlet. */
export function App() {
  return (
    <div className="app">
      <header className="topbar">
        <span className="brand">ZBLL Trainer</span>
        <nav>
          <NavLink to="/" end>
            Cases
          </NavLink>
          <NavLink to="/train">Train</NavLink>
          <NavLink to="/algs">Algs</NavLink>
        </nav>
      </header>
      <main className="content">
        <Outlet />
      </main>
    </div>
  );
}

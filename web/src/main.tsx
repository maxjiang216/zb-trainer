import React from "react";
import ReactDOM from "react-dom/client";
import { createBrowserRouter, RouterProvider } from "react-router-dom";

import { App } from "./App";
import { HomePage } from "./pages/HomePage";
import { TrainerPage } from "./pages/TrainerPage";
import { AlgsPage } from "./pages/AlgsPage";
import { CaseAlgsPage } from "./pages/CaseAlgsPage";
import "./styles.css";

const router = createBrowserRouter([
  {
    path: "/",
    element: <App />,
    children: [
      { index: true, element: <HomePage /> },
      { path: "train", element: <TrainerPage /> },
      { path: "algs", element: <AlgsPage /> },
      { path: "algs/:id", element: <CaseAlgsPage /> },
    ],
  },
]);

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <RouterProvider router={router} />
  </React.StrictMode>,
);

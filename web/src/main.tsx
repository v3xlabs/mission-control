import "./index.css";

import { MutationCache, QueryClient, QueryClientProvider } from "@tanstack/react-query";
import React from "react";
import ReactDOM from "react-dom/client";

import { notify } from "./api/notices";
import { App } from "./pages/App";

// Every mutation reports its own failure. Without this a refused request looks to the reader
// exactly like one that worked.
const queryClient = new QueryClient({
  mutationCache: new MutationCache({
    onError: error => notify(error instanceof Error ? error.message : String(error)),
  }),
});

ReactDOM.createRoot(document.querySelector("#root") as HTMLElement).render(
  <React.StrictMode>
    <QueryClientProvider client={queryClient}>
      <App />
    </QueryClientProvider>
  </React.StrictMode>,
);

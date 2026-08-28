import "../index.css";

import ReactDOM from "react-dom/client";

import { Alerts } from "./Alerts";

const query = new URLSearchParams(globalThis.location.search);

// The window the daemon opens says which presentation it is. Nothing else on the page can tell:
// a sidebar and a toast are both small, and a takeover is a tab like any other.
const presentation = query.has("sidebar")
  ? "sidebar"
  : (query.has("toast") ? "toast" : "takeover");

ReactDOM.createRoot(document.querySelector("#root") as HTMLElement)
  .render(<Alerts presentation={presentation} />);

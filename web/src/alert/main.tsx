import "../index.css";

import ReactDOM from "react-dom/client";

import type { Presentation } from "./Alerts";
import { Alerts } from "./Alerts";

const query = new URLSearchParams(globalThis.location.search);

const presentations: Presentation[] = ["sidebar", "toast", "agenda"];
const presentation = presentations.find(name => query.has(name)) ?? "takeover";

ReactDOM.createRoot(document.querySelector("#root") as HTMLElement)
  .render(<Alerts presentation={presentation} />);

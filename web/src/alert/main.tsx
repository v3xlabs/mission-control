import "../index.css";

import ReactDOM from "react-dom/client";

import { Alerts } from "./Alerts";

const isSidebar = new URLSearchParams(globalThis.location.search).has("sidebar");

ReactDOM.createRoot(document.querySelector("#root") as HTMLElement)
  .render(<Alerts isSidebar={isSidebar} />);

import "../index.css";

import ReactDOM from "react-dom/client";

import { Stinger } from "./Stinger";

const name = new URLSearchParams(globalThis.location.search).get("name") ?? "";

ReactDOM.createRoot(document.querySelector("#root") as HTMLElement).render(<Stinger name={name} />);

import { mountGreetingPanel } from "./greeting_panel.js";

const root = document.querySelector("#app");
if (!root) {
  throw new Error("missing #app element");
}

await mountGreetingPanel({ root });

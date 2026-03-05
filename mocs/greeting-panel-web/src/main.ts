import { mountGreetingPanel } from "../preview/greeting_panel.js";

async function main(): Promise<void> {
  const root = document.querySelector<HTMLElement>("#app");
  if (!root) {
    throw new Error("missing #app element");
  }

  await mountGreetingPanel({ root });
}

document.addEventListener("DOMContentLoaded", () => {
  void main();
});

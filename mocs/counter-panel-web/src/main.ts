import { mountCounter } from "../../../blocks/ui.counter.mount/tauri_ts/src/index";

function main(): void {
  const root = document.querySelector<HTMLElement>("#app");
  if (!root) {
    throw new Error("missing #app element");
  }

  mountCounter(root, 3);
}

document.addEventListener("DOMContentLoaded", main);

import { mountCounterView } from "../../../blocks/ui.counter.mount/tauri_ts/src/counter_view.js";

const root = document.querySelector("#app");
if (!root) {
  throw new Error("missing #app element");
}

mountCounterView(root, 3);

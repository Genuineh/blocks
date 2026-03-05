import { mountText } from "../../../blocks/ui.dom.mount_text/tauri_ts/src/index";

function main(): void {
  const root = document.querySelector<HTMLElement>("#app");
  if (!root) {
    throw new Error("missing #app element");
  }

  mountText(root, "hello from frontend moc");
}

document.addEventListener("DOMContentLoaded", main);

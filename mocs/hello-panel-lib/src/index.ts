import { mountText } from "../../../blocks/ui.dom.mount_text/tauri_ts/src/index";

export function mountHelloPanel(target: HTMLElement): { mounted: true } {
  return mountText(target, "hello from frontend lib");
}

import { mountTextView } from "./text_view.js";

export function mountText(target: HTMLElement, text: string): { mounted: true } {
  return mountTextView(target, text);
}

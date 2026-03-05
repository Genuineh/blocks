import { mountCounterView } from "./counter_view.js";

export type CounterMountResult = { mounted: true };

export function mountCounter(
  target: HTMLElement,
  initialCount = 0,
): CounterMountResult {
  return mountCounterView(target, initialCount);
}

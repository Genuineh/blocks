export function mountTextView(target, text) {
  target.textContent = text;
  return { mounted: true };
}

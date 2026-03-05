export function mountCounterView(target, initialCount = 0) {
  let count = initialCount;

  const shell = document.createElement("section");
  shell.style.display = "grid";
  shell.style.gap = "12px";
  shell.style.maxWidth = "320px";
  shell.style.padding = "20px";
  shell.style.border = "1px solid #d4d4d8";
  shell.style.borderRadius = "16px";
  shell.style.background = "#ffffff";
  shell.style.boxShadow = "0 16px 40px rgba(15, 23, 42, 0.08)";

  const title = document.createElement("h1");
  title.textContent = "Counter Panel";
  title.style.margin = "0";
  title.style.fontSize = "20px";

  const value = document.createElement("p");
  value.style.margin = "0";
  value.style.fontSize = "40px";
  value.style.fontWeight = "700";

  const controls = document.createElement("div");
  controls.style.display = "flex";
  controls.style.gap = "8px";

  const decrement = document.createElement("button");
  decrement.textContent = "-1";
  const increment = document.createElement("button");
  increment.textContent = "+1";
  const reset = document.createElement("button");
  reset.textContent = "Reset";

  const render = () => {
    value.textContent = String(count);
  };

  decrement.addEventListener("click", () => {
    count -= 1;
    render();
  });

  increment.addEventListener("click", () => {
    count += 1;
    render();
  });

  reset.addEventListener("click", () => {
    count = initialCount;
    render();
  });

  controls.append(decrement, increment, reset);
  shell.append(title, value, controls);
  target.replaceChildren(shell);
  render();

  return { mounted: true };
}

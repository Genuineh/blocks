export const DEFAULT_GREETING_API_ORIGIN = "http://127.0.0.1:4318";

export function resolveApiOrigin(scope = globalThis) {
  const override =
    scope &&
    typeof scope.__BLOCKS_GREETING_API_ORIGIN__ === "string" &&
    scope.__BLOCKS_GREETING_API_ORIGIN__.trim()
      ? scope.__BLOCKS_GREETING_API_ORIGIN__.trim()
      : "";

  return override || DEFAULT_GREETING_API_ORIGIN;
}

export function buildGreetingUrl(apiOrigin = DEFAULT_GREETING_API_ORIGIN) {
  const normalizedOrigin = apiOrigin.replace(/\/+$/, "");
  return `${normalizedOrigin}/api/v1/greeting`;
}

export function createLoadingState() {
  return { status: "loading" };
}

export function createSuccessState(greeting) {
  const normalizedGreeting = normalizeGreeting(greeting);
  return {
    status: "success",
    title: normalizedGreeting.title,
    message: normalizedGreeting.message,
  };
}

export function createErrorState(error) {
  return {
    status: "error",
    message: toErrorMessage(error),
  };
}

export function normalizeGreeting(payload) {
  if (!payload || typeof payload !== "object") {
    throw new Error("Greeting payload must be an object.");
  }

  if (typeof payload.title !== "string" || payload.title.trim() === "") {
    throw new Error("Greeting payload is missing a valid title.");
  }

  if (typeof payload.message !== "string" || payload.message.trim() === "") {
    throw new Error("Greeting payload is missing a valid message.");
  }

  return {
    title: payload.title,
    message: payload.message,
  };
}

export async function loadGreeting(fetchImpl, apiOrigin = DEFAULT_GREETING_API_ORIGIN) {
  const response = await fetchImpl(buildGreetingUrl(apiOrigin));

  if (!response || response.ok !== true) {
    const status = response && typeof response.status === "number" ? response.status : "unknown";
    throw new Error(`Greeting request failed with status ${status}.`);
  }

  return createSuccessState(await response.json());
}

export function renderGreetingState(state) {
  switch (state.status) {
    case "loading":
      return `
        <section class="panel panel-loading" aria-live="polite">
          <p class="eyebrow">Greeting API</p>
          <h1 class="title">Loading greeting...</h1>
          <p class="message">Waiting for GET /api/v1/greeting.</p>
        </section>
      `;
    case "success":
      return `
        <section class="panel panel-success" aria-live="polite">
          <p class="eyebrow">Greeting API</p>
          <h1 class="title">${escapeHtml(state.title)}</h1>
          <p class="message">${escapeHtml(state.message)}</p>
        </section>
      `;
    case "error":
      return `
        <section class="panel panel-error" aria-live="assertive">
          <p class="eyebrow">Greeting API</p>
          <h1 class="title">Unable to load greeting</h1>
          <p class="message">${escapeHtml(state.message)}</p>
        </section>
      `;
    default:
      throw new Error(`Unknown greeting panel state: ${state.status}`);
  }
}

export async function mountGreetingPanel({
  root,
  fetchImpl = fetch,
  apiOrigin = resolveApiOrigin(),
}) {
  root.innerHTML = renderGreetingState(createLoadingState());

  try {
    const state = await loadGreeting(fetchImpl, apiOrigin);
    root.innerHTML = renderGreetingState(state);
    return state;
  } catch (error) {
    const state = createErrorState(error);
    root.innerHTML = renderGreetingState(state);
    return state;
  }
}

function toErrorMessage(error) {
  if (error instanceof Error && error.message.trim()) {
    return error.message;
  }

  return "Unknown error while contacting the greeting service.";
}

function escapeHtml(value) {
  return String(value)
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#39;");
}

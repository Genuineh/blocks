import test from "node:test";
import assert from "node:assert/strict";

import {
  buildGreetingUrl,
  createErrorState,
  createLoadingState,
  createSuccessState,
  mountGreetingPanel,
  normalizeGreeting,
  renderGreetingState,
} from "../preview/greeting_panel.js";

test("buildGreetingUrl appends the greeting route once", () => {
  assert.equal(
    buildGreetingUrl("http://127.0.0.1:4500/"),
    "http://127.0.0.1:4500/api/v1/greeting",
  );
});

test("normalizeGreeting rejects invalid payloads", () => {
  assert.throws(() => normalizeGreeting({ title: "Hello" }), /valid message/);
});

test("renderGreetingState covers loading and error views", () => {
  const loading = renderGreetingState(createLoadingState());
  const error = renderGreetingState(createErrorState(new Error("backend offline")));

  assert.match(loading, /Loading greeting/);
  assert.match(error, /Unable to load greeting/);
  assert.match(error, /backend offline/);
});

test("mountGreetingPanel renders loading before success", async () => {
  let resolveFetch;
  const fetchPromise = new Promise((resolve) => {
    resolveFetch = resolve;
  });
  const root = { innerHTML: "" };

  const mounted = mountGreetingPanel({
    root,
    apiOrigin: "http://127.0.0.1:4500",
    fetchImpl: () => fetchPromise,
  });

  assert.match(root.innerHTML, /Loading greeting/);

  resolveFetch({
    ok: true,
    async json() {
      return createSuccessState({
        title: "Hello from test",
        message: "The fetch path returned a greeting.",
      });
    },
  });

  const state = await mounted;

  assert.deepEqual(state, {
    status: "success",
    title: "Hello from test",
    message: "The fetch path returned a greeting.",
  });
  assert.match(root.innerHTML, /Hello from test/);
});

test("mountGreetingPanel renders the error state when the request fails", async () => {
  const root = { innerHTML: "" };

  const state = await mountGreetingPanel({
    root,
    apiOrigin: "http://127.0.0.1:4500",
    fetchImpl: async () => ({ ok: false, status: 503 }),
  });

  assert.equal(state.status, "error");
  assert.match(root.innerHTML, /503/);
});

async function fetch_csrf(uri: string, params?: RequestInit) {
  const csrf_token = document.cookie
    .split(";")
    .filter((c) => c.startsWith("session_csrf="))
    .map((c) => c.substring("session_csrf=".length, c.length))
    .pop();
  if (csrf_token === undefined) {
    throw new Error("CSRF token cookie is not set");
  }
  const headers = params ? new Headers(params.headers) : new Headers();
  headers.set("X-CSRF-Token", csrf_token);
  return fetch(uri, {
    ...params,
    headers,
  });
}

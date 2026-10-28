export function endpointRoot(): string {
  return import.meta.env.VITE_API_ENDPOINT || "http://localhost:3030";
}

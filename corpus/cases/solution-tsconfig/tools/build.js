export function build() {
  return step();
}

function step() {
  return JSON.stringify({ ok: true });
}

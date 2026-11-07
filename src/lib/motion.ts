// Transiciones de Svelte con la curva «emphasized decelerate» de M3. Con movimiento reducido
// solo queda un fundido corto, sin desplazamiento.
const decel = (t: number) => 1 - Math.pow(1 - t, 4);
const reduced = () => typeof matchMedia !== "undefined" && matchMedia("(prefers-reduced-motion: reduce)").matches;

/** Aparece subiendo unos píxeles (contenido que cambia en su sitio: detalle, tarjetas). */
export function rise(_node: Element, { delay = 0, duration = 260, y = 10 } = {}) {
  if (reduced()) return { duration: 120, css: (t: number) => `opacity:${t}` };
  return {
    delay,
    duration,
    easing: decel,
    css: (t: number) => `opacity:${t}; transform: translateY(${(1 - t) * y}px)`,
  };
}

/** Entra desde el borde derecho (hojas laterales). */
export function sheet(_node: Element, { duration = 300, x = 32 } = {}) {
  if (reduced()) return { duration: 120, css: (t: number) => `opacity:${t}` };
  return { duration, easing: decel, css: (t: number) => `opacity:${t}; transform: translateX(${(1 - t) * x}px)` };
}

/** Retraso escalonado para listas, con tope para que nunca se sienta lento. */
export const stagger = (i: number, step = 28, max = 8) => Math.min(i, max) * step;

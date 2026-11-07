<script lang="ts">
  import { Tween } from "svelte/motion";
  import { scoreClass } from "$lib/api";

  /**
   * Medidor de compatibilidad: píldora tonal con el puntaje y barra lineal (M3).
   * `chip`: compacto, para listas. `row`: barra ancha con el valor a la derecha (desgloses).
   */
  let {
    value,
    max = 100,
    label,
    layout = "chip",
  }: { value: number; max?: number; label: string; layout?: "chip" | "row" } = $props();

  // Momento propio de la app: la cifra cuenta hasta el puntaje al mismo ritmo que crece la barra,
  // y si el puntaje cambia (CV guardado, pesos nuevos) avanza del valor anterior al nuevo.
  const reduced = typeof matchMedia !== "undefined" && matchMedia("(prefers-reduced-motion: reduce)").matches;
  const shownValue = new Tween(0, { duration: reduced ? 0 : 700, easing: (t) => 1 - Math.pow(1 - t, 4) });
  $effect(() => {
    const id = requestAnimationFrame(() => shownValue.set(value));
    return () => cancelAnimationFrame(id);
  });
  const pct = $derived(max > 0 ? Math.max(0, Math.min(100, (shownValue.current / max) * 100)) : 0);
  const tone = $derived(scoreClass(max > 0 ? (value / max) * 100 : 0));
  // Al terminar de contar un puntaje alto, la píldora late una vez (cempasúchil).
  const landed = $derived(Math.round(shownValue.current) === Math.round(value) && value > 0);
</script>

<span
  class="meter {layout} {tone}" class:landed
  role="meter"
  aria-label={label}
  aria-valuenow={Math.round(value)}
  aria-valuemin={0}
  aria-valuemax={max}
  title={label}
>
  {#if layout === "chip"}
    <span class="num" aria-hidden="true">{Math.round(shownValue.current)}</span>
  {/if}
  <span class="track"><span class="fill" style:width="{pct}%"></span></span>
  {#if layout === "row"}
    <span class="val" aria-hidden="true">{Math.round(shownValue.current)}<span class="of">{` / ${max}`}</span></span>
  {/if}
</span>

<style>
  .meter { display: inline-flex; flex: none; font-variant-numeric: tabular-nums; }
  .chip { flex-direction: column; align-items: stretch; gap: 4px; width: 3.25rem; }
  .row { align-items: center; gap: 0.75rem; width: 100%; }
  .num {
    display: grid; place-items: center; height: 32px; border-radius: var(--radius-full);
    font-weight: 750; font-size: 1rem;
  }
  .track {
    position: relative; display: block; height: 4px; border-radius: var(--radius-full);
    background: var(--md-surface-container-highest); overflow: hidden;
  }
  .row .track { flex: 1; height: 6px; }
  .fill {
    position: absolute; inset: 0 auto 0 0; border-radius: inherit;
  }
  .num { transition: box-shadow 400ms var(--ease-decel); }
  .high.landed .num { animation: land 900ms var(--ease-decel) 1; }
  @keyframes land {
    0% { box-shadow: 0 0 0 0 color-mix(in srgb, var(--md-tertiary) 45%, transparent); }
    100% { box-shadow: 0 0 0 8px color-mix(in srgb, var(--md-tertiary) 0%, transparent); }
  }
  @media (prefers-reduced-motion: reduce) { .high.landed .num { animation: none; } }
  .val { min-width: 4.5rem; text-align: right; font-weight: 700; }
  .of { font-weight: 400; color: var(--md-on-surface-variant); }

  .high .num { background: var(--md-tertiary-container); color: var(--md-on-tertiary-container); }
  .high .fill { background: var(--md-tertiary); }
  .mid .num { background: var(--md-primary-container); color: var(--md-on-primary-container); }
  .mid .fill { background: var(--md-primary); }
  .low .num { background: var(--md-surface-container-highest); color: var(--md-on-surface-variant); }
  .low .fill { background: var(--md-outline); }
</style>

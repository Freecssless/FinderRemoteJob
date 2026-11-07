<script lang="ts">
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

  const pct = $derived(max > 0 ? Math.max(0, Math.min(100, (value / max) * 100)) : 0);
  // La barra crece desde cero al aparecer: es el único momento de movimiento de la app.
  let shown = $state(false);
  $effect(() => {
    const id = requestAnimationFrame(() => (shown = true));
    return () => cancelAnimationFrame(id);
  });
</script>

<span
  class="meter {layout} {scoreClass(pct)}"
  role="meter"
  aria-label={label}
  aria-valuenow={Math.round(value)}
  aria-valuemin={0}
  aria-valuemax={max}
  title={label}
>
  {#if layout === "chip"}
    <span class="num">{Math.round(value)}</span>
  {/if}
  <span class="track"><span class="fill" style:width="{shown ? pct : 0}%"></span></span>
  {#if layout === "row"}
    <span class="val">{Math.round(value)}<span class="of">{` / ${max}`}</span></span>
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
    transition: width 500ms cubic-bezier(0.05, 0.7, 0.1, 1);
  }
  .val { min-width: 4.5rem; text-align: right; font-weight: 700; }
  .of { font-weight: 400; color: var(--md-on-surface-variant); }

  .high .num { background: var(--md-tertiary-container); color: var(--md-on-tertiary-container); }
  .high .fill { background: var(--md-tertiary); }
  .mid .num { background: var(--md-primary-container); color: var(--md-on-primary-container); }
  .mid .fill { background: var(--md-primary); }
  .low .num { background: var(--md-surface-container-highest); color: var(--md-on-surface-variant); }
  .low .fill { background: var(--md-outline); }
</style>

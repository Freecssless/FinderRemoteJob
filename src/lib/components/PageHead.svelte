<script lang="ts">
  import type { Snippet } from "svelte";
  import type { IconName } from "$lib/icons";
  import Icon from "./Icon.svelte";

  /** Cabecera de pantalla: icono con el color de la sección, título, para qué sirve y acciones. */
  let {
    icon,
    title,
    subtitle,
    tone = "primary",
    children,
  }: { icon: IconName; title: string; subtitle?: string; tone?: "primary" | "tertiary" | "secondary"; children?: Snippet } = $props();
</script>

<header class="pagehead">
  <span class="tile {tone}" aria-hidden="true"><Icon name={icon} size={26} /></span>
  <div class="titles">
    <h1>{title}</h1>
    {#if subtitle}<p>{subtitle}</p>{/if}
  </div>
  {#if children}<div class="actions">{@render children()}</div>{/if}
</header>

<style>
  .pagehead { display: flex; align-items: center; gap: 0.9rem 1rem; flex-wrap: wrap; margin-bottom: 0.25rem; }
  .tile {
    display: grid; place-items: center; flex: none; width: 52px; height: 52px; border-radius: var(--radius-l);
    animation: tile-in 260ms var(--ease-decel) both;
  }
  .tile.primary { background: var(--md-primary-container); color: var(--md-on-primary-container); }
  .tile.tertiary { background: var(--md-tertiary-container); color: var(--md-on-tertiary-container); }
  .tile.secondary { background: var(--md-secondary-container); color: var(--md-on-secondary-container); }
  @keyframes tile-in { from { opacity: 0; transform: scale(0.92); } }
  .titles { min-width: 0; margin-right: auto; }
  h1 { margin: 0; }
  p { margin: 0.1rem 0 0; color: var(--md-on-surface-variant); }
  .actions { display: flex; align-items: center; gap: 0.75rem; flex-wrap: wrap; }
  .actions :global(label.inline .select) { width: auto; min-width: 11rem; }
</style>

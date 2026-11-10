<script lang="ts">
  import { ICONS, type IconName } from "#lib/icons.ts";

  // Decorativo por defecto: el texto del control ya lo nombra.
  let { name, size = 20, label }: { name: IconName; size?: number; label?: string } = $props();
</script>

<svg
  class="icon"
  width={size}
  height={size}
  viewBox="0 -960 960 960"
  fill="currentColor"
  role={label ? "img" : undefined}
  aria-label={label}
  aria-hidden={label ? undefined : "true"}
><path d={ICONS[name]} /></svg>

<style>
  .icon { flex: none; display: inline-block; vertical-align: middle; }
</style>

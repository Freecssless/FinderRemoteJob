<script lang="ts" generics="T extends string">
  import Icon from "./Icon.svelte";

  /**
   * Menú desplegable de M3 en lugar de <select>: en WebKitGTK la lista nativa sale con el
   * estilo de GTK. Usa la API `popover` (capa superior: ningún panel la recorta, y se cierra
   * al hacer clic fuera) y el patrón de teclado de un listbox.
   */
  let {
    value = $bindable(),
    options,
    label,
    disabled = false,
    onchange,
  }: {
    value: T | null;
    options: { value: T; label: string }[];
    /** Nombre para lectores de pantalla (el rótulo visible, si lo hay). */
    label: string;
    disabled?: boolean;
    onchange?: (value: T) => void;
  } = $props();

  const id = `sel-${Math.random().toString(36).slice(2, 9)}`;
  let trigger = $state<HTMLButtonElement>();
  let menu = $state<HTMLUListElement>();
  let open = $state(false);
  let active = $state(0);
  let pos = $state({ top: 0, left: 0, width: 0, up: false });

  // Sin valor (null o sin guardar) equivale a la opción vacía «por defecto», si existe.
  const current = $derived(options.find((o) => o.value === (value ?? "")));

  // Abierto, el menú está fijo en pantalla: si algo se desplaza o cambia de tamaño, se cierra.
  $effect(() => {
    if (!open) return;
    const close = (e: Event) => { if (!menu?.contains(e.target as Node)) menu?.hidePopover(); };
    document.addEventListener("scroll", close, true);
    window.addEventListener("resize", close);
    return () => {
      document.removeEventListener("scroll", close, true);
      window.removeEventListener("resize", close);
    };
  });

  function show() {
    if (disabled || !trigger || !menu) return;
    const r = trigger.getBoundingClientRect();
    const below = window.innerHeight - r.bottom;
    const up = below < Math.min(320, options.length * 48 + 16) && r.top > below;
    pos = { top: up ? r.top - 4 : r.bottom + 4, left: r.left, width: r.width, up };
    active = Math.max(0, options.findIndex((o) => o.value === (value ?? "")));
    menu.showPopover();
    requestAnimationFrame(() => menu?.querySelector<HTMLElement>(`[data-i="${active}"]`)?.scrollIntoView({ block: "nearest" }));
  }

  function pick(i: number) {
    const o = options[i];
    if (!o) return;
    menu?.hidePopover();
    trigger?.focus();
    if (o.value !== value) {
      value = o.value;
      onchange?.(o.value);
    }
  }

  function key(e: KeyboardEvent) {
    const last = options.length - 1;
    const move = (i: number) => {
      active = Math.max(0, Math.min(last, i));
      menu?.querySelector<HTMLElement>(`[data-i="${active}"]`)?.scrollIntoView({ block: "nearest" });
    };
    if (!open) {
      if (["ArrowDown", "ArrowUp", "Enter", " "].includes(e.key)) {
        e.preventDefault();
        show();
      }
      return;
    }
    if (e.key === "ArrowDown") move(active + 1);
    else if (e.key === "ArrowUp") move(active - 1);
    else if (e.key === "Home") move(0);
    else if (e.key === "End") move(last);
    else if (e.key === "Enter" || e.key === " ") pick(active);
    else if (e.key === "Escape") menu?.hidePopover();
    else if (e.key === "Tab") menu?.hidePopover();
    else if (e.key.length === 1) {
      // Saltar a la primera opción que empiece con esa letra.
      const k = e.key.toLowerCase();
      const i = options.findIndex((o, j) => j > active && o.label.toLowerCase().startsWith(k));
      const j = i >= 0 ? i : options.findIndex((o) => o.label.toLowerCase().startsWith(k));
      if (j >= 0) move(j);
      return;
    } else return;
    e.preventDefault();
  }
</script>

<button
  bind:this={trigger}
  type="button"
  class="select"
  class:open
  {disabled}
  role="combobox"
  aria-haspopup="listbox"
  aria-expanded={open}
  aria-controls={id}
  aria-label="{label}: {current?.label ?? 'sin elegir'}"
  aria-activedescendant={open ? `${id}-${active}` : undefined}
  onclick={() => (open ? menu?.hidePopover() : show())}
  onkeydown={key}
>
  <span class="text">{current?.label ?? "—"}</span>
  <span class="chev" aria-hidden="true"><Icon name="expand_more" size={22} /></span>
</button>

<ul
  bind:this={menu}
  {id}
  popover="auto"
  role="listbox"
  aria-label={label}
  class="menu"
  class:up={pos.up}
  style:top="{pos.top}px"
  style:left="{pos.left}px"
  style:min-width="{pos.width}px"
  ontoggle={(e) => (open = (e as ToggleEvent).newState === "open")}
>
  {#each options as o, i (o.value)}
    <!-- svelte-ignore a11y_click_events_have_key_events -- el teclado lo maneja el combobox -->
    <li
      id="{id}-{i}"
      data-i={i}
      role="option"
      aria-selected={o.value === (value ?? "")}
      class:active={i === active}
      onclick={() => pick(i)}
      onpointermove={() => (active = i)}
    >
      <span class="check" aria-hidden="true">{#if o.value === (value ?? "")}<Icon name="check" size={20} />{/if}</span>
      {o.label}
    </li>
  {/each}
</ul>

<style>
  .select {
    width: 100%; min-height: 48px; justify-content: space-between; gap: 0.5rem;
    padding: 0 0.6rem 0 0.9rem; border-radius: var(--radius-xs);
    border: 1px solid var(--md-outline); background: transparent;
    color: var(--md-on-surface); font-size: 1rem; font-weight: 400; letter-spacing: 0; text-align: left;
  }
  .select:hover:not(:disabled) { border-color: var(--md-on-surface); background: transparent; }
  .select:active:not(:disabled) { transform: none; background: transparent; }
  .select:focus-visible, .select.open { outline: none; border-color: var(--md-primary); box-shadow: inset 0 0 0 1px var(--md-primary); }
  .select:disabled { opacity: 0.6; }
  .text { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .chev { display: grid; color: var(--md-on-surface-variant); transition: transform 200ms var(--ease); }
  .open .chev { transform: rotate(180deg); color: var(--md-primary); }

  .menu {
    position: fixed; inset: auto; margin: 0; padding: 0.5rem 0;
    max-height: min(320px, 60vh); overflow-y: auto; list-style: none;
    border: 0; border-radius: var(--radius-s);
    background: var(--md-surface-container); color: var(--md-on-surface);
    box-shadow: var(--elev-2);
    transform-origin: top center;
  }
  .menu.up { translate: 0 -100%; transform-origin: bottom center; }
  .menu:popover-open { animation: menu-in 200ms var(--ease-decel) both; }
  @keyframes menu-in { from { opacity: 0; transform: scaleY(0.85); } }
  li {
    display: flex; align-items: center; gap: 0.5rem;
    min-height: 48px; padding: 0 1rem 0 0.6rem; cursor: pointer; white-space: nowrap;
  }
  li.active { background: color-mix(in srgb, var(--md-on-surface) 8%, transparent); }
  li[aria-selected="true"] { background: var(--md-secondary-container); color: var(--md-on-secondary-container); font-weight: 650; }
  .check { width: 20px; display: grid; place-items: center; color: inherit; }
  @media (prefers-reduced-motion: reduce) { .menu:popover-open { animation: none; } }
</style>

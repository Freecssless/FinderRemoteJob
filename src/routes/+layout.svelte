<script lang="ts">
  import { page } from "$app/state";
  import { api, type AppInfo } from "$lib/api";
  import Icon from "$lib/components/Icon.svelte";
  import type { IconName } from "$lib/icons";
  import "../app.css";

  let { children } = $props();
  let info = $state<AppInfo | null>(null);
  // Se vuelve a pedir al navegar: los Ajustes pueden cambiar los modelos.
  $effect(() => {
    void page.url.pathname;
    api.appInfo().then((i) => (info = i)).catch(() => {});
  });

  const nav: { href: string; label: string; icon: IconName }[] = [
    { href: "/perfiles", label: "Perfiles", icon: "person" },
    { href: "/vacantes", label: "Vacantes", icon: "work" },
    { href: "/optimizador", label: "Optimizador", icon: "edit_document" },
    { href: "/cola", label: "Postular", icon: "send" },
    { href: "/tablero", label: "Tablero", icon: "view_kanban" },
    { href: "/ajustes", label: "Ajustes", icon: "settings" },
  ];
  const current = (href: string) => page.url.pathname.startsWith(href);
</script>

<div class="shell">
  <aside>
    <a class="brand" href="/perfiles">
      <span class="mark" aria-hidden="true"><Icon name="travel_explore" size={22} /></span>
      FinderRemoteJob
    </a>
    <nav aria-label="Secciones">
      {#each nav as item}
        <a href={item.href} aria-current={current(item.href) ? "page" : undefined}>
          <Icon name={current(item.href) ? (`${item.icon}-fill` as IconName) : item.icon} size={24} />
          {item.label}
        </a>
      {/each}
    </nav>
    {#if info}
      <footer>
        {#if info.llm_error}
          <a class="ai bad" href="/ajustes">
            <Icon name="error" size={20} />
            <span>La IA no está lista: revisa Ajustes</span>
          </a>
        {:else}
          <div class="ai">
            <Icon name="check_circle" size={20} />
            <span>IA: {info.writer_model ?? "—"}</span>
          </div>
        {/if}
      </footer>
    {/if}
  </aside>
  <main>{@render children()}</main>
</div>

<style>
  .shell { display: grid; grid-template-columns: 248px 1fr; height: 100vh; }
  aside {
    display: flex; flex-direction: column; gap: 1.25rem;
    padding: 1.25rem 0.75rem 1rem;
    background: var(--md-surface-container-low);
  }
  .brand {
    display: flex; align-items: center; gap: 0.6rem;
    padding: 0 0.75rem; min-height: 44px;
    font-weight: 750; font-size: 1.0625rem; letter-spacing: -0.01em;
    color: var(--md-on-surface); text-decoration: none;
  }
  .mark {
    display: grid; place-items: center; width: 36px; height: 36px; border-radius: var(--radius-m);
    background: var(--md-primary); color: var(--md-on-primary);
  }
  nav { display: flex; flex-direction: column; gap: 0.25rem; }
  nav a {
    display: flex; align-items: center; gap: 0.85rem;
    min-height: 52px; padding: 0 1rem;
    border-radius: var(--radius-full);
    color: var(--md-on-surface-variant); text-decoration: none;
    font-weight: 600; font-size: 1rem;
    transition: background-color 150ms var(--ease);
  }
  nav a:hover { background: color-mix(in srgb, var(--md-on-surface) 8%, transparent); color: var(--md-on-surface); }
  nav a[aria-current="page"] {
    background: var(--md-secondary-container); color: var(--md-on-secondary-container); font-weight: 750;
  }
  nav a:focus-visible { outline-offset: -3px; border-radius: var(--radius-full); }
  footer { margin-top: auto; }
  .ai {
    display: flex; align-items: flex-start; gap: 0.5rem;
    padding: 0.75rem 1rem; border-radius: var(--radius-m);
    font-size: 0.875rem; line-height: 1.3rem; color: var(--md-on-surface-variant);
    overflow-wrap: anywhere;
  }
  .ai :global(.icon) { color: var(--md-success); margin-top: 1px; }
  .ai.bad { background: var(--md-error-container); color: var(--md-on-error-container); text-decoration: none; font-weight: 600; }
  .ai.bad :global(.icon) { color: inherit; }
  main { overflow: auto; padding: 2rem 2.5rem 3rem; }
</style>

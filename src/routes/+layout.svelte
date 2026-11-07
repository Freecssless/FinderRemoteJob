<script lang="ts">
  import { onNavigate } from "$app/navigation";
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

  // Transición entre pantallas: el contenido se funde y la píldora de la navegación se desliza.
  // Sin soporte (o con movimiento reducido) se cambia de pantalla al instante.
  onNavigate((nav) => {
    if (!document.startViewTransition || matchMedia("(prefers-reduced-motion: reduce)").matches) return;
    if (nav.from?.url.pathname === nav.to?.url.pathname) return;
    return new Promise((resolve) => {
      document.startViewTransition(async () => {
        resolve();
        await nav.complete;
      });
    });
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
    <a class="brand" href="/perfiles" aria-label="FinderRemoteJob, inicio">
      <span class="mark" aria-hidden="true"><Icon name="travel_explore" size={22} /></span>
      <span class="brandname">FinderRemoteJob</span>
    </a>
    <nav aria-label="Secciones">
      {#each nav as item}
        {@const on = current(item.href)}
        <a href={item.href} aria-current={on ? "page" : undefined}>
          {#if on}<span class="pill" aria-hidden="true"></span>{/if}
          <span class="ico"><Icon name={on ? (`${item.icon}-fill` as IconName) : item.icon} size={24} /></span>
          <span class="label">{item.label}</span>
        </a>
      {/each}
    </nav>
    {#if info}
      <footer>
        {#if info.llm_error}
          <a class="ai bad" href="/ajustes" title="La IA no está lista: revisa Ajustes">
            <Icon name="error" size={20} />
            <span class="ailabel">La IA no está lista: revisa Ajustes</span>
          </a>
        {:else}
          <div class="ai" title="IA: {info.writer_model ?? '—'}">
            <Icon name="check_circle" size={20} />
            <span class="ailabel">IA: {info.writer_model ?? "—"}</span>
          </div>
        {/if}
      </footer>
    {/if}
  </aside>
  <main>
    <div class="content">{@render children()}</div>
  </main>
</div>

<style>
  .shell { display: grid; grid-template-columns: 256px 1fr; height: 100vh; }
  aside {
    display: flex; flex-direction: column; gap: 1.25rem;
    padding: 1.25rem 0.75rem 1rem;
    background: var(--md-surface-container-low);
    overflow-y: auto;
  }
  .brand {
    display: flex; align-items: center; gap: 0.7rem;
    padding: 0 0.75rem; min-height: 44px;
    font-weight: 750; font-size: 1.0625rem; letter-spacing: -0.01em;
    color: var(--md-on-surface); text-decoration: none;
  }
  .mark {
    display: grid; place-items: center; flex: none; width: 38px; height: 38px; border-radius: var(--radius-m);
    background: var(--md-primary); color: var(--md-on-primary);
    box-shadow: var(--elev-1);
  }
  nav { display: flex; flex-direction: column; gap: 0.25rem; }
  nav a {
    position: relative; isolation: isolate;
    display: flex; align-items: center; gap: 0.85rem;
    min-height: 52px; padding: 0 1rem;
    border-radius: var(--radius-full);
    color: var(--md-on-surface-variant); text-decoration: none;
    font-weight: 600; font-size: 1rem;
    transition: background-color 150ms var(--ease), color 150ms var(--ease);
  }
  nav a:hover { background: color-mix(in srgb, var(--md-on-surface) 8%, transparent); color: var(--md-on-surface); }
  nav a[aria-current="page"] { color: var(--md-on-secondary-container); font-weight: 750; }
  nav a[aria-current="page"]:hover { background: transparent; }
  /* La píldora es un elemento propio para que la transición la deslice de un destino a otro. */
  .pill {
    position: absolute; inset: 0; z-index: -1;
    border-radius: var(--radius-full); background: var(--md-secondary-container);
    view-transition-name: nav-pill;
  }
  .ico { display: grid; place-items: center; }
  nav a:focus-visible { outline-offset: -3px; border-radius: var(--radius-full); }
  footer { margin-top: auto; }
  .ai {
    display: flex; align-items: flex-start; gap: 0.5rem;
    padding: 0.75rem 1rem; border-radius: var(--radius-m);
    font-size: 0.9375rem; line-height: 1.35rem; color: var(--md-on-surface-variant);
    overflow-wrap: anywhere;
  }
  .ai :global(.icon) { color: var(--md-success); margin-top: 1px; }
  .ai.bad { background: var(--md-error-container); color: var(--md-on-error-container); text-decoration: none; font-weight: 600; }
  .ai.bad :global(.icon) { color: inherit; }
  main { overflow: auto; container: main / inline-size; scroll-behavior: smooth; }
  .content {
    max-width: 1440px; margin-inline: auto;
    padding: 2rem clamp(1rem, 3vw, 2.5rem) 3rem;
    view-transition-name: page;
  }

  /* Ventanas medianas: riel de navegación (M3), icono con su etiqueta debajo. */
  @media (max-width: 1099px) {
    .shell { grid-template-columns: 96px 1fr; }
    aside { padding: 1rem 0.25rem; align-items: center; gap: 1rem; }
    .brand { padding: 0; justify-content: center; }
    .brandname { position: absolute; width: 1px; height: 1px; overflow: hidden; clip: rect(0 0 0 0); }
    nav { gap: 0.6rem; width: 100%; }
    nav a {
      flex-direction: column; justify-content: center; gap: 0.2rem;
      min-height: 64px; padding: 0.25rem 0;
      border-radius: var(--radius-l); font-size: 0.9375rem; line-height: 1.2;
    }
    nav a:hover { background: transparent; }
    nav a:hover .ico { background: color-mix(in srgb, var(--md-on-surface) 8%, transparent); }
    .ico { width: 56px; height: 32px; border-radius: var(--radius-full); transition: background-color 150ms var(--ease); }
    .pill { inset: 0.25rem auto auto 50%; width: 56px; height: 32px; translate: -50% 0; }
    .label { text-align: center; }
    footer { width: 100%; display: flex; justify-content: center; }
    .ai { padding: 0.6rem; justify-content: center; }
    .ailabel { position: absolute; width: 1px; height: 1px; overflow: hidden; clip: rect(0 0 0 0); }
  }
</style>

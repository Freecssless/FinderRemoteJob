<script lang="ts">
  import { page } from "$app/state";
  import { api, type AppInfo } from "$lib/api";
  import "../app.css";

  let { children } = $props();
  let info = $state<AppInfo | null>(null);
  api.appInfo().then((i) => (info = i)).catch(() => {});

  // Las demás secciones llegan en las partes 10c y 10d.
  const nav = [
    { href: "/perfiles", label: "Perfiles", ready: true },
    { href: "/vacantes", label: "Vacantes", ready: true },
    { href: "/optimizador", label: "Optimizador", ready: false },
    { href: "/tablero", label: "Tablero", ready: false },
  ];
</script>

<div class="shell">
  <aside>
    <div class="brand">FinderRemoteJob</div>
    <nav aria-label="Secciones">
      {#each nav as item}
        {#if item.ready}
          <a href={item.href} aria-current={page.url.pathname.startsWith(item.href) ? "page" : undefined}>
            {item.label}
          </a>
        {:else}
          <span class="soon" title="Próximamente">{item.label}</span>
        {/if}
      {/each}
    </nav>
    {#if info}
      <footer class="muted small">
        <div>IA local: {info.local_model ?? "—"}</div>
        <div>Redactor: {info.writer_model ?? "—"}</div>
        <div title={info.env_file ?? ""}>.env: {info.env_file ? "cargado" : "no encontrado"}</div>
        {#if info.llm_error}<div class="danger">{info.llm_error}</div>{/if}
      </footer>
    {/if}
  </aside>
  <main>{@render children()}</main>
</div>

<style>
  .shell { display: grid; grid-template-columns: 200px 1fr; height: 100vh; }
  aside {
    display: flex; flex-direction: column; gap: 1rem; padding: 1rem;
    background: var(--surface); border-right: 1px solid var(--border);
  }
  .brand { font-weight: 700; }
  nav { display: flex; flex-direction: column; gap: 0.25rem; }
  nav a, nav span { padding: 0.4rem 0.6rem; border-radius: 6px; text-decoration: none; color: var(--text); }
  nav a[aria-current="page"] { background: var(--accent-soft); color: var(--accent); font-weight: 600; }
  nav a:hover { background: var(--hover); }
  .soon { color: var(--muted); cursor: default; }
  footer { margin-top: auto; display: flex; flex-direction: column; gap: 0.2rem; overflow-wrap: anywhere; }
  main { overflow: auto; padding: 1.5rem; }
</style>

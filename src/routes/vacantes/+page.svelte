<script lang="ts">
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { goto } from "$app/navigation";
  import {
    api, boardApi, jobsApi, rememberedProfile, rememberProfile, SOURCE_LABELS,
    type JobDetail, type JobRow, type Profile, type SyncSummary,
  } from "$lib/api";
  import Icon from "$lib/components/Icon.svelte";
  import Meter from "$lib/components/Meter.svelte";
  import { rise, stagger } from "$lib/motion";

  let profiles = $state<Profile[]>([]);
  let profileId = $state<string | null>(null);
  let rows = $state<JobRow[]>([]);
  let query = $state("");
  let source = $state("");
  let hideWeak = $state(true);

  let detail = $state<JobDetail | null>(null);
  let selectedId = $state<string | null>(null);

  let busy = $state<"" | "list" | "sync" | "import" | "analyze" | "embed">("");
  let embedMsg = $state("");
  let error = $state("");
  let syncReport = $state<SyncSummary[]>([]);

  let showImport = $state(false);
  let importMode = $state<"url" | "text">("url");
  let imp = $state({ url: "", title: "", company: "", text: "" });

  let portals = $state<{ name: string; url: string }[]>([]);
  let added = $state<string | null>(null);

  const profile = $derived(profiles.find((p) => p.id === profileId) ?? null);
  const hasScores = $derived(rows.some((r) => r.score !== null));
  const visible = $derived(
    rows.filter(
      (r) =>
        (!source || r.source_id === source) &&
        !(hideWeak && r.score && (r.score.low_relevance || r.score.blockers.length > 0)),
    ),
  );
  const hiddenCount = $derived(rows.length - visible.length);

  async function init() {
    try {
      profiles = await api.listProfiles();
      const saved = rememberedProfile();
      profileId = profiles.find((p) => p.id === saved)?.id ?? profiles[0]?.id ?? null;
      await Promise.all([reload(), loadPortals()]);
    } catch (e) {
      error = String(e);
    }
  }

  async function reload() {
    busy = "list";
    try {
      rows = await jobsApi.list(profileId, query);
      error = "";
    } catch (e) {
      error = String(e);
    } finally {
      busy = "";
    }
  }

  async function loadPortals() {
    portals = await jobsApi.portals(profile?.target_titles[0] ?? "");
  }

  async function changeProfile() {
    if (profileId) rememberProfile(profileId);
    await Promise.all([reload(), loadPortals()]);
    if (selectedId) await select(selectedId);
  }

  async function addToBoard() {
    if (!detail || !profileId) return;
    try {
      await boardApi.add(detail.job.id, profileId);
      added = detail.job.id;
    } catch (e) {
      error = String(e);
    }
  }

  async function select(id: string) {
    selectedId = id;
    try {
      detail = await jobsApi.get(id, profileId);
    } catch (e) {
      error = String(e);
    }
  }

  async function sync() {
    if (!profileId) return;
    busy = "sync";
    syncReport = [];
    try {
      syncReport = await jobsApi.sync(profileId);
      busy = "";
      await reload();
    } catch (e) {
      error = String(e);
    } finally {
      busy = "";
    }
  }

  async function embedAll() {
    if (!profileId) return;
    busy = "embed";
    error = "";
    embedMsg = "";
    try {
      const n = await jobsApi.embed(profileId);
      embedMsg = n ? `Similitud calculada para ${n} textos.` : "Todas las vacantes recientes ya la tenían.";
      busy = "";
      await reload();
    } catch (e) {
      error = String(e);
    } finally {
      busy = "";
    }
  }

  async function importJob() {
    busy = "import";
    try {
      const r =
        importMode === "url"
          ? await jobsApi.importUrl(imp.url)
          : await jobsApi.importText(imp.title, imp.company, imp.url, imp.text);
      imp = { url: "", title: "", company: "", text: "" };
      showImport = false;
      busy = "";
      error = r.created ? "" : "Esa vacante ya estaba guardada; te la muestro.";
      await reload();
      await select(r.id);
    } catch (e) {
      error = String(e);
      // Si el portal bloquea el enlace, lo natural es pegar el texto.
      if (importMode === "url") importMode = "text";
    } finally {
      busy = "";
    }
  }

  async function analyze() {
    if (!detail) return;
    busy = "analyze";
    try {
      detail = await jobsApi.analyze(detail.job.id, profileId);
      error = "";
      const id = detail.job.id;
      busy = "";
      await reload();
      selectedId = id;
    } catch (e) {
      error = String(e);
    } finally {
      busy = "";
    }
  }

  function syncLine(s: SyncSummary): string {
    if (s.error) return s.error;
    if (s.skipped_too_soon) return "consultada hace poco; se omitió para respetar su límite";
    return `${s.new} nuevas (${s.kept} de ${s.fetched} coinciden con tus puestos)`;
  }

  const date = (iso: string | null) => (iso ? iso.slice(0, 10) : "");
  // `found_at` viene en UTC: «nueva» = encontrada en las últimas 24 horas.
  const isNew = (iso: string) => Date.now() - Date.parse(iso) < 86_400_000;

  init();
</script>

<div class="page">
  <header class="row">
    <h1>Vacantes</h1>
    <label class="inline">
      Perfil
      <select bind:value={profileId} onchange={changeProfile} disabled={!profiles.length}>
        {#each profiles as p}<option value={p.id}>{p.name}</option>{/each}
      </select>
    </label>
    <button class="primary" onclick={sync} disabled={!profileId || busy !== ""}>
      {busy === "sync" ? "Buscando en las fuentes…" : "Buscar vacantes nuevas"}
    </button>
    <button onclick={() => (showImport = !showImport)} aria-expanded={showImport}>Importar vacante</button>
  </header>
  {#if embedMsg}<p class="ok small" role="status">{embedMsg}</p>{/if}

  {#if !profiles.length}
    <p class="notice">Primero crea un perfil en <a href="/perfiles">Perfiles</a>: sus puestos buscados filtran las vacantes.</p>
  {:else if profile && !profile.target_titles.length}
    <p class="notice">El perfil «{profile.name}» no tiene puestos buscados: se guardarán todas las vacantes de las fuentes. Agrégalos en <a href="/perfiles">Perfiles</a>.</p>
  {/if}
  {#if rows.length && !hasScores && profileId}
    <p class="notice">Este perfil aún no tiene CV maestro: las vacantes salen por fecha, sin compatibilidad.</p>
  {/if}
  {#if error}<p class="danger" role="alert">{error}</p>{/if}

  {#if syncReport.length}
    <div class="card small" role="status">
      <ul class="plain">
        {#each syncReport as s}
          <li><strong>{SOURCE_LABELS[s.source] ?? s.source}:</strong> <span class:danger={s.error}>{syncLine(s)}</span></li>
        {/each}
      </ul>
    </div>
  {/if}

  {#if showImport}
    <div class="card">
      <div class="segmented" role="tablist">
        <button role="tab" aria-selected={importMode === "url"} onclick={() => (importMode = "url")}>Desde un enlace</button>
        <button role="tab" aria-selected={importMode === "text"} onclick={() => (importMode = "text")}>Pegar el texto</button>
      </div>
      {#if importMode === "url"}
        <p class="muted small">Se hace una sola petición, como tu navegador. LinkedIn y otros portales suelen bloquearla: en ese caso pega el texto.</p>
        <label>Enlace del anuncio<input type="url" bind:value={imp.url} placeholder="https://…" /></label>
      {:else}
        <div class="grid">
          <label>Puesto<input bind:value={imp.title} /></label>
          <label>Empresa (opcional)<input bind:value={imp.company} /></label>
          <label>Enlace (opcional)<input type="url" bind:value={imp.url} /></label>
        </div>
        <label style="margin-top:0.5rem">Texto del anuncio<textarea rows="8" bind:value={imp.text}></textarea></label>
      {/if}
      <div class="row" style="margin-top:0.75rem">
        <button class="primary" onclick={importJob} disabled={busy !== ""}>{busy === "import" ? "Importando…" : "Importar"}</button>
        <button onclick={() => (showImport = false)}>Cancelar</button>
      </div>
    </div>
  {/if}

  {#if portals.length}
    <p class="small muted portals">
      Buscar{profile?.target_titles[0] ? ` «${profile.target_titles[0]}»` : ""} en:
      {#each portals as p, i}
        <button class="link" onclick={() => openUrl(p.url)}>{p.name}</button>{i < portals.length - 1 ? " · " : ""}
      {/each}
      <span>(luego importa aquí las que te interesen)</span>
    </p>
  {/if}

  <div class="filters row">
    <input type="search" placeholder="Buscar en título, empresa o descripción" bind:value={query}
      onkeydown={(e) => e.key === "Enter" && reload()} aria-label="Buscar vacantes" />
    <button onclick={reload} disabled={busy !== ""}>Buscar</button>
    <select bind:value={source} aria-label="Fuente">
      <option value="">Todas las fuentes</option>
      {#each Object.entries(SOURCE_LABELS) as [id, label]}<option value={id}>{label}</option>{/each}
    </select>
    <label class="inline small"><input type="checkbox" bind:checked={hideWeak} /> Ocultar poco relevantes y bloqueadas</label>
  </div>

  <div class="split" class:showing={selectedId}>
    <section class="list" aria-label="Lista de vacantes">
      <div class="listhead">
        <p class="muted small">
          {busy === "list" ? "Cargando…" : `${visible.length} vacantes`}{hiddenCount ? ` · ${hiddenCount} ocultas` : ""}
        </p>
        <button class="link" onclick={embedAll} disabled={!profileId || busy !== ""}
          title="Usa la IA de tu computadora para comparar el sentido de tu CV con cada vacante, no solo las palabras.">
          {busy === "embed" ? "Comparando…" : "Comparar mejor con IA"}
        </button>
      </div>
      {#if busy === "list" && !rows.length}
        <ul class="plain" aria-hidden="true">
          {#each [0, 1, 2, 3, 4] as i}<li class="skeleton skelrow" style:opacity={1 - i * 0.15}></li>{/each}
        </ul>
      {:else if !rows.length && busy === ""}
        <div class="empty">
          <span class="empty-icon"><Icon name="travel_explore" size={32} /></span>
          <h2>Todavía no hay vacantes</h2>
          <p class="muted">Pulsa «Buscar vacantes nuevas» para traer las de varios portales, o importa una que hayas visto.</p>
        </div>
      {/if}
      <ul class="plain">
        {#each visible as r, i (r.id)}
          <li in:rise={{ delay: stagger(i) }}>
            <button class="list-item job" class:active={r.id === selectedId} onclick={() => select(r.id)}>
              {#if r.score}
                <Meter value={r.score.total} label="Compatibilidad" />
              {/if}
              <span class="jobtext">
                <strong>{r.title}</strong>
                {#if isNew(r.found_at)}<span class="new">Nueva</span>{/if}
                <span class="muted small">
                  {[r.company, r.location, SOURCE_LABELS[r.source_id] ?? r.source_id].filter(Boolean).join(" · ")}
                </span>
                {#if r.score?.blockers.length}<span class="danger small">Bloqueada: {r.score.blockers[0]}</span>
                {:else if r.score?.low_relevance}<span class="muted small">Poco relevante</span>{/if}
              </span>
            </button>
          </li>
        {/each}
      </ul>
    </section>

    <section class="detail" aria-label="Detalle de la vacante">
      {#if detail}
        {@const job = detail.job}
        {@const s = detail.score}
        {#key job.id}
        <div class="detailbody" in:rise>
        <button class="link backlist" onclick={() => ((selectedId = null), (detail = null))}>
          <Icon name="arrow_back" size={20} /> Volver a la lista
        </button>
        <div class="jobhead">
          <h2 style="margin-top:0">{job.title}</h2>
          <p class="muted">
            {[job.company, job.location, job.remote_type].filter(Boolean).join(" · ")}<br />
            {SOURCE_LABELS[job.source_id] ?? job.source_id}
            {#if job.posted_at}· publicada {date(job.posted_at)}{/if} · encontrada {date(job.found_at)}
          </p>
          {#if job.salary_min || job.salary_max}
            <p>Salario: {job.salary_min ?? "?"}–{job.salary_max ?? "?"} {job.currency ?? ""}</p>
          {/if}
          <div class="row">
            <button class="primary" onclick={() => goto(`/optimizador?job=${job.id}`)} disabled={!s}
              title={s ? "" : "El perfil necesita CV maestro"}>Adaptar mi CV</button>
            <button onclick={addToBoard} disabled={!s || busy !== ""}>
              {#if added === job.id}<Icon name="check_circle" /> En el tablero{:else}<Icon name="add" /> Agregar al tablero{/if}
            </button>
            {#if job.url}<button onclick={() => openUrl(job.url)}>Abrir anuncio</button>{/if}
            <button onclick={analyze} disabled={busy !== ""}>
              {busy === "analyze" ? "Analizando con IA local…" : job.requirements ? "Volver a analizar con IA" : "Analizar requisitos con IA"}
            </button>
          </div>
          {#if busy === "analyze"}<p class="muted small">Con un modelo local en CPU puede tardar unos minutos.</p>{/if}
          {#if !job.requirements}
            <p class="muted small">Requisitos detectados solo con el diccionario. Analízalos con IA para separar requeridos y deseables, años y bloqueos.</p>
          {/if}
        </div>

        {#if s}
          <div class="card">
            <div class="compat">
              <Meter value={s.total} label="Compatibilidad" />
              <h3>Qué tanto encaja contigo</h3>
            </div>
            <dl class="breakdown">
              <dt>Habilidades</dt><dd><Meter layout="row" value={s.breakdown.skills} max={45} label="Habilidades" /></dd>
              {#if s.breakdown.semantic !== null}
                <dt>Parecido con tu CV</dt><dd><Meter layout="row" value={s.breakdown.semantic} max={15} label="Parecido con tu CV" /></dd>
              {/if}
              <dt>Puesto</dt><dd><Meter layout="row" value={s.breakdown.title} max={10} label="Puesto" /></dd>
              <dt>Nivel de experiencia</dt><dd><Meter layout="row" value={s.breakdown.seniority} max={5} label="Nivel de experiencia" /></dd>
              <dt>Años ({s.years_experience.toFixed(1)})</dt><dd><Meter layout="row" value={s.breakdown.years} max={10} label="Años de experiencia" /></dd>
              <dt>Remoto, región e idiomas</dt><dd><Meter layout="row" value={s.breakdown.logistics} max={15} label="Remoto, región e idiomas" /></dd>
            </dl>
            {#if s.breakdown.semantic === null}
              <p class="muted small">Sin la comparación con IA, sus 15 puntos se reparten entre las demás partes.</p>
            {/if}
            {#if s.blockers.length}
              <p class="danger"><strong>Bloqueos:</strong> {s.blockers.join("; ")}</p>
            {/if}
            {#if s.missing_required.length}
              <p><strong>Requeridos que no están en tu CV:</strong> {s.missing_required.join(", ")}</p>
            {/if}
          </div>
        {/if}

        {#if job.requirements}
          {@const req = job.requirements}
          <div class="card outlined">
            <h3>Requisitos</h3>
            <p class="small">
              <strong>Requeridos:</strong> {req.terms.filter((t) => t.required).map((t) => t.surface).join(", ") || "—"}<br />
              <strong>Deseables:</strong> {req.terms.filter((t) => !t.required).map((t) => t.surface).join(", ") || "—"}
            </p>
            <p class="small muted">
              {#if req.years_min}Años mínimos: {req.years_min} · {/if}
              {#if req.seniority}Nivel: {req.seniority} · {/if}
              {#if req.languages.length}Idiomas: {req.languages.join(", ")}{/if}
            </p>
          </div>
        {/if}

        <div class="card outlined">
          <h3>Anuncio</h3>
          <div class="description">{job.description}</div>
        </div>
      </div>
        {/key}
      {:else}
        <p class="muted">Elige una vacante para ver su detalle.</p>
      {/if}
    </section>
  </div>
</div>

<style>
  .page { display: flex; flex-direction: column; gap: 0.75rem; }
  header h1 { margin: 0 auto 0 0; }
  header select { width: auto; min-width: 12rem; }
  .filters input[type="search"] { flex: 1 1 18rem; width: auto; }
  .filters select { width: auto; }
  .portals { margin: 0; }
  .plain { list-style: none; margin: 0; padding: 0; }
  .split { display: grid; grid-template-columns: minmax(280px, 2fr) 3fr; gap: 1.25rem; align-items: start; }
  .detailbody { display: flex; flex-direction: column; gap: 0.75rem; }
  .skelrow { height: 76px; margin-bottom: 0.4rem; }
  .backlist { display: none; align-self: flex-start; }
  /* Angosto: lista o detalle, uno a la vez (patrón lista-detalle de M3). */
  @container main (max-width: 820px) {
    .split { grid-template-columns: 1fr; }
    .split.showing .list, .split:not(.showing) .detail { display: none; }
    .backlist { display: inline-flex; }
    .detail { position: static; max-height: none; }
  }
  .list { min-width: 0; }
  .list .plain { display: flex; flex-direction: column; gap: 0.25rem; }
  .job { gap: 0.85rem; }
  .jobtext { display: flex; flex-direction: column; gap: 0.1rem; min-width: 0; }
  .detail { display: flex; flex-direction: column; gap: 0.75rem; min-width: 0; position: sticky; top: 0; max-height: calc(100vh - 3rem); overflow: auto; }
  .listhead { display: flex; align-items: center; justify-content: space-between; gap: 0.5rem; }
  .listhead p { margin: 0; }
  .new {
    align-self: flex-start; padding: 0.1rem 0.55rem; border-radius: var(--radius-s);
    background: var(--md-tertiary-container); color: var(--md-on-tertiary-container);
    font-size: 0.875rem; font-weight: 700;
  }
  .jobhead { padding: 0.25rem 0.25rem 0.5rem; }
  .compat { display: flex; align-items: center; gap: 1rem; margin-bottom: 1rem; }
  .compat h3 { margin: 0; }
  .breakdown { display: grid; grid-template-columns: max-content 1fr; align-items: center; gap: 0.6rem 1.25rem; margin: 0 0 0.75rem; }
  .breakdown dt { color: var(--md-on-surface-variant); }
  .breakdown dd { margin: 0; }
  .description { white-space: pre-wrap; overflow-wrap: anywhere; }
</style>

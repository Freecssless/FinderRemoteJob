<script lang="ts">
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { goto } from "$app/navigation";
  import {
    api, boardApi, jobsApi, rememberedProfile, rememberProfile, scoreClass, SOURCE_LABELS,
    type JobDetail, type JobRow, type Profile, type SyncSummary,
  } from "$lib/api";

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
    <button onclick={embedAll} disabled={!profileId || busy !== ""} title="Compara el significado de tu CV con cada vacante (IA local, modelo de embeddings)">
      {busy === "embed" ? "Calculando similitud…" : "Calcular similitud semántica"}
    </button>
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
      <div class="row" role="tablist">
        <button role="tab" aria-selected={importMode === "url"} class:primary={importMode === "url"} onclick={() => (importMode = "url")}>Desde un enlace</button>
        <button role="tab" aria-selected={importMode === "text"} class:primary={importMode === "text"} onclick={() => (importMode = "text")}>Pegar el texto</button>
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

  <div class="split">
    <section class="list" aria-label="Lista de vacantes">
      <p class="muted small">
        {busy === "list" ? "Cargando…" : `${visible.length} vacantes`}{hiddenCount ? ` · ${hiddenCount} ocultas` : ""}
      </p>
      {#if !rows.length && busy === ""}
        <p class="muted">Aún no hay vacantes. Pulsa «Buscar vacantes nuevas» o importa una.</p>
      {/if}
      <ul class="plain">
        {#each visible as r (r.id)}
          <li>
            <button class="job" class:active={r.id === selectedId} onclick={() => select(r.id)}>
              {#if r.score}
                <span class="score {scoreClass(r.score.total)}" title="Compatibilidad">{r.score.total}</span>
              {/if}
              <span class="jobtext">
                <strong>{r.title}</strong>
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
        <div class="card">
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
              {added === job.id ? "En el tablero ✓" : "Agregar al tablero"}
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
            <h3>Compatibilidad: <span class="score {scoreClass(s.total)}">{s.total}</span> / 100</h3>
            <dl class="breakdown small">
              <dt>Habilidades</dt><dd>{s.breakdown.skills.toFixed(0)} / 45</dd>
              {#if s.breakdown.semantic !== null}
                <dt>Similitud con tu CV</dt><dd>{s.breakdown.semantic.toFixed(0)} / 15</dd>
              {/if}
              <dt>Puesto</dt><dd>{s.breakdown.title.toFixed(0)} / 10</dd>
              <dt>Seniority</dt><dd>{s.breakdown.seniority.toFixed(0)} / 5</dd>
              <dt>Años ({s.years_experience.toFixed(1)})</dt><dd>{s.breakdown.years.toFixed(0)} / 10</dd>
              <dt>Remoto, región e idiomas</dt><dd>{s.breakdown.logistics.toFixed(0)} / 15</dd>
            </dl>
            <p class="muted small">Sin similitud semántica, sus 15 puntos se reparten entre los demás componentes.</p>
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
          <div class="card">
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

        <div class="card">
          <h3>Anuncio</h3>
          <div class="description">{job.description}</div>
        </div>
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
  .split { display: grid; grid-template-columns: minmax(280px, 2fr) 3fr; gap: 1rem; align-items: start; }
  .list { min-width: 0; }
  .list .plain { display: flex; flex-direction: column; gap: 0.25rem; }
  .job {
    width: 100%; display: flex; gap: 0.6rem; align-items: flex-start; text-align: left;
    border-color: transparent; background: var(--surface);
  }
  .job.active { border-color: var(--accent); background: var(--accent-soft); }
  .jobtext { display: flex; flex-direction: column; gap: 0.1rem; min-width: 0; }
  .detail { display: flex; flex-direction: column; gap: 0.75rem; min-width: 0; position: sticky; top: 0; max-height: calc(100vh - 3rem); overflow: auto; }
  .breakdown { display: grid; grid-template-columns: auto auto; gap: 0.2rem 1rem; margin: 0 0 0.5rem; justify-content: start; }
  .breakdown dd { margin: 0; font-variant-numeric: tabular-nums; }
  .description { white-space: pre-wrap; overflow-wrap: anywhere; }
  [role="tab"][aria-selected="false"] { background: var(--surface); }
</style>

<script lang="ts">
  import Icon from "$lib/components/Icon.svelte";
  import Meter from "$lib/components/Meter.svelte";
  import { openPath, revealItemInDir } from "@tauri-apps/plugin-opener";
  import { page } from "$app/state";
  import {
    api, jobsApi, optimizerApi, rememberedProfile, TEMPLATES,
    type Choices, type Template, type JobDetail, type OptimizerView, type Profile, type SavedCv,
  } from "$lib/api";

  const jobId = $derived(page.url.searchParams.get("job"));

  let profile = $state<Profile | null>(null);
  let detail = $state<JobDetail | null>(null);
  let view = $state<OptimizerView | null>(null);
  let choices = $state<Choices>({ summary: false, bullets: [] });
  let saved = $state<SavedCv | null>(null);
  let template = $state<Template>("clasica");
  let busy = $state<"" | "load" | "run" | "save">("load");
  let error = $state("");

  const expLabel = $derived(new Map(view?.experiences ?? []));
  const changed = $derived(
    view !== null &&
      (choices.summary !== view.opt.choices.summary ||
        choices.bullets.some((b, i) => b !== view!.opt.choices.bullets[i])),
  );

  async function init() {
    if (!jobId) {
      busy = "";
      return;
    }
    try {
      const profiles = await api.listProfiles();
      const saved = rememberedProfile();
      profile = profiles.find((p) => p.id === saved) ?? profiles[0] ?? null;
      detail = await jobsApi.get(jobId, profile?.id ?? null);
      if (profile) show(await optimizerApi.get(jobId, profile.id));
    } catch (e) {
      error = String(e);
    } finally {
      busy = "";
    }
  }

  function show(v: OptimizerView | null) {
    view = v;
    saved = null;
    if (v) choices = structuredClone(v.opt.choices);
  }

  async function run() {
    if (!jobId || !profile) return;
    busy = "run";
    error = "";
    try {
      show(await optimizerApi.run(jobId, profile.id));
    } catch (e) {
      error = String(e);
    } finally {
      busy = "";
    }
  }

  async function save() {
    if (!jobId) return;
    busy = "save";
    error = "";
    try {
      saved = await optimizerApi.save(jobId, $state.snapshot(choices), template);
    } catch (e) {
      error = String(e);
    } finally {
      busy = "";
    }
  }

  const terms = (ts: { surface: string }[]) => ts.map((t) => t.surface).join(", ");

  init();
</script>

{#if !jobId}
  <h1>Optimizador</h1>
  <div class="empty">
    <span class="empty-icon"><Icon name="edit_document" size={32} /></span>
    <h2>Adapta tu CV a una vacante</h2>
    <p class="muted">Elige una vacante en Vacantes y pulsa «Adaptar mi CV». Aquí verás qué cambia y cuánto mejora.</p>
    <a class="button primary" href="/vacantes"><Icon name="work" size={20} /> Ir a Vacantes</a>
  </div>
{:else}
  <div class="page">
    <header>
      <p class="small"><a class="back" href="/vacantes"><Icon name="arrow_back" size={18} /> Vacantes</a></p>
      <h1>Adaptar CV{detail ? `: ${detail.job.title}` : ""}</h1>
      {#if detail}
        <p class="muted">
          {[detail.job.company, detail.job.location].filter(Boolean).join(" · ")}
          {#if profile}· perfil «{profile.name}»{/if}
        </p>
      {/if}
    </header>

    {#if error}<p class="danger" role="alert">{error}</p>{/if}

    {#if busy === "load"}
      <p class="muted">Cargando…</p>
    {:else if !profile}
      <p class="notice">Primero crea un perfil con su CV maestro en <a href="/perfiles">Perfiles</a>.</p>
    {:else if !view}
      <div class="card">
        <p>
          La IA analiza la vacante, propone cómo redactar tu resumen y tus logros, y el guardián rechaza
          cualquier dato que no esté en tu CV maestro. Tú decides qué se usa. Nada se guarda hasta que pulses «Guardar CV».
        </p>
        <button class="primary" onclick={run} disabled={busy !== ""}>
          {busy === "run" ? "Analizando y redactando…" : "Generar propuesta"}
        </button>
        {#if busy === "run"}
          <p class="muted small">Con modelos locales en CPU puede tardar varios minutos. No cierres la app.</p>
        {/if}
      </div>
    {:else}
      {@const opt = view.opt}
      <div class="card scores">
        <h2>Qué tan bien lo leen los filtros automáticos</h2>
        <p class="muted small">Muchas empresas usan un programa (ATS) que revisa tu CV antes que una persona.</p>
        <dl class="ats">
          <dt>Tu CV actual</dt>
          <dd><Meter layout="row" value={opt.prepared.ats_before.total} label="Tu CV actual" /></dd>
          <dt>CV adaptado{saved ? " (guardado)" : ""}</dt>
          <dd><Meter layout="row" value={saved?.ats_after ?? opt.ats_after.total} label="CV adaptado" /></dd>
        </dl>
        <div class="row">
          <button onclick={run} disabled={busy !== ""}>
            {busy === "run" ? "Generando…" : "Generar otra propuesta"}
          </button>
        </div>
      </div>
      {#if changed && !saved}
        <p class="muted small">Cambiaste decisiones: el puntaje final se recalcula al guardar.</p>
      {/if}

      {#if opt.prepared.notes.length}
        <div class="notice"><ul>{#each opt.prepared.notes as n}<li>{n}</li>{/each}</ul></div>
      {/if}
      {#if !opt.roundtrip.sections_in_order || opt.roundtrip.missing_ratio > 0.02}
        <div class="notice">
          El PDF de prueba no se leyó completo
          {#if opt.roundtrip.missing_words.length}(faltan: {opt.roundtrip.missing_words.slice(0, 10).join(", ")}){/if}.
          El puntaje después ya lo tiene en cuenta.
        </div>
      {/if}

      <div class="card">
        <h3>Lo que pide la vacante y no está en tu CV</h3>
        {#if opt.prepared.gaps.with_evidence.length}
          <p class="small"><strong>Tu CV lo respalda</strong> (el redactor pudo usarlo): {terms(opt.prepared.gaps.with_evidence)}</p>
        {/if}
        {#if opt.prepared.gaps.without_evidence.length}
          <p class="small">
            <strong>Sin respaldo, no se agrega:</strong> {terms(opt.prepared.gaps.without_evidence)}.
            <span class="muted">Si sí lo tienes, agrégalo a tu CV maestro en <a href="/perfiles">Perfiles</a> y genera de nuevo.</span>
          </p>
        {/if}
        {#if !opt.prepared.gaps.with_evidence.length && !opt.prepared.gaps.without_evidence.length}
          <p class="small ok">Tu CV ya menciona todo lo que detectamos.</p>
        {/if}
        {#if opt.ats_after.synonym_only.length}
          <p class="small muted">
            Usa la forma del anuncio:
            {opt.ats_after.synonym_only.map((s) => `«${s.job_surface}» en vez de «${s.cv_surface}»`).join("; ")}
          </p>
        {/if}
      </div>

      <h2>Revisa las propuestas</h2>
      {#if opt.review.summary}
        {@const sm = opt.review.summary}
        <div class="card change" class:rejected={view.summary_issues.length}>
          <label class="inline">
            <input type="checkbox" bind:checked={choices.summary} disabled={view.summary_issues.length > 0} />
            <strong>Resumen profesional</strong>
          </label>
          <p class="proposed">{sm.proposed}</p>
          {#if view.summary_issues.length}
            <p class="danger small">Rechazado por el guardián: {view.summary_issues.join(" ")} El CV saldrá sin resumen.</p>
          {:else if !choices.summary}
            <p class="muted small">El CV saldrá sin resumen.</p>
          {/if}
        </div>
      {/if}

      {#each opt.review.bullets as b, i}
        {@const issues = view.bullet_issues[i]}
        {#if i === 0 || opt.review.bullets[i - 1].exp_id !== b.exp_id}
          <h3 class="exp">{expLabel.get(b.exp_id ?? "") ?? "Experiencia"}</h3>
        {/if}
        <div class="card change" class:rejected={issues.length}>
          <label class="inline">
            <input type="checkbox" bind:checked={choices.bullets[i]} disabled={issues.length > 0} />
            {issues.length ? "No se puede usar" : choices.bullets[i] ? "Usar la propuesta" : "Mantener mi texto"}
          </label>
          <div class="diff">
            <div>
              <span class="muted small">Tu texto</span>
              {#each b.originals as o}<p class="original">{o}</p>{:else}<p class="muted small">—</p>{/each}
            </div>
            <div>
              <span class="muted small">Propuesta</span>
              <p class="proposed">{b.proposed}</p>
            </div>
          </div>
          {#if issues.length}
            <p class="danger small">Rechazado por el guardián: {issues.join(" ")} Se mantiene tu texto.</p>
          {:else if !opt.choices.bullets[i]}
            <p class="muted small">Desmarcada: tu texto tiene términos de la vacante que la propuesta pierde.</p>
          {/if}
        </div>
      {/each}

      <div class="card save">
        <fieldset class="templates">
          <legend>Plantilla</legend>
          {#each TEMPLATES as t}
            <label class="inline" title={t.hint}>
              <input type="radio" name="template" value={t.id} bind:group={template} />
              {t.label} <span class="muted small">{t.hint}</span>
            </label>
          {/each}
        </fieldset>
        <button class="primary" onclick={save} disabled={busy !== ""}>
          {busy === "save" ? "Generando PDF y DOCX…" : saved ? "Guardar otra versión" : "Guardar CV"}
        </button>
        <span class="muted small">Se crean el PDF y el DOCX, y la vacante pasa a «CV adaptado» en el tablero.</span>
      </div>

      {#if saved}
        <div class="card" role="status">
          <p class="ok"><strong>CV guardado.</strong> Filtros automáticos: {saved.ats_before ?? "?"} → {saved.ats_after ?? "?"}</p>
          {#if saved.missing.length}
            <p class="small muted">Siguen faltando: {saved.missing.join(", ")}</p>
          {/if}
          <div class="row">
            {#if saved.pdf_path}
              <button onclick={() => openPath(saved!.pdf_path!)}>Abrir PDF</button>
              <button onclick={() => revealItemInDir(saved!.pdf_path!)}>Mostrar en la carpeta</button>
            {/if}
            {#if saved.docx_path}<button onclick={() => openPath(saved!.docx_path!)}>Abrir DOCX</button>{/if}
          </div>
        </div>
      {/if}
    {/if}

  </div>
{/if}

<style>
  .rejected { background: var(--md-error-container); color: var(--md-on-error-container); }
  .templates { flex-basis: 100%; border: 0; padding: 0; margin: 0 0 0.5rem; display: flex; flex-direction: column; gap: 0.2rem; }
  .templates legend { font-weight: 600; margin-bottom: 0.2rem; }
  .page { display: flex; flex-direction: column; gap: 0.75rem; max-width: 960px; }
  header h1 { margin-bottom: 0.25rem; }
  header p { margin: 0; }
  .scores h2 { margin: 0 0 0.25rem; }
  .ats { display: grid; grid-template-columns: max-content 1fr; align-items: center; gap: 0.75rem 1.25rem; margin: 0.5rem 0 1rem; max-width: 36rem; }
  .ats dt { font-weight: 600; }
  .ats dd { margin: 0; }
  .back { display: inline-flex; align-items: center; gap: 0.3rem; min-height: 40px; font-weight: 600; }
  .exp { margin: 0.75rem 0 0; }
  .change.rejected { border-color: var(--danger); }
  .diff { display: grid; grid-template-columns: 1fr 1fr; gap: 1rem; margin-top: 0.5rem; }
  .diff p, .proposed { margin: 0.2rem 0; }
  .original { color: var(--muted); }
  .save { display: flex; align-items: center; gap: 0.75rem; flex-wrap: wrap; }
</style>

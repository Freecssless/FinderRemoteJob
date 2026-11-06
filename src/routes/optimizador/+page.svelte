<script lang="ts">
  import { openPath, revealItemInDir } from "@tauri-apps/plugin-opener";
  import { page } from "$app/state";
  import {
    api, costLabel, jobsApi, letterApi, optimizerApi, rememberedProfile, settingsApi, TEMPLATES,
    type Choices, type CostView, type LetterView, type Template, type JobDetail, type OptimizerView, type Profile, type SavedCv,
  } from "$lib/api";

  const jobId = $derived(page.url.searchParams.get("job"));

  let profile = $state<Profile | null>(null);
  let detail = $state<JobDetail | null>(null);
  let view = $state<OptimizerView | null>(null);
  let choices = $state<Choices>({ summary: false, bullets: [] });
  let saved = $state<SavedCv | null>(null);
  let template = $state<Template>("clasica");
  let busy = $state<"" | "load" | "run" | "save" | "letter">("load");
  let letter = $state<LetterView | null>(null);
  let letterText = $state("");
  let letterMsg = $state("");
  let error = $state("");
  let cost = $state<CostView | null>(null);

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
      if (profile) {
        show(await optimizerApi.get(jobId, profile.id));
        // El estimado es informativo: si falla, no bloquea la pantalla.
        cost = await settingsApi.estimate(jobId, profile.id).catch(() => null);
      }
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

  async function writeLetter() {
    if (!jobId || !profile) return;
    if (letterText.trim() && letterText !== letter?.text && !confirm("Se reemplazará la carta que editaste. ¿Seguir?")) return;
    busy = "letter";
    error = "";
    letterMsg = "";
    try {
      letter = await letterApi.write(jobId, profile.id);
      letterText = letter.text;
    } catch (e) {
      error = String(e);
    } finally {
      busy = "";
    }
  }

  async function saveLetter() {
    if (!jobId || !profile) return;
    try {
      await letterApi.save(jobId, profile.id, letterText);
      letterMsg = "Carta guardada en la postulación (la ves en el Tablero).";
    } catch (e) {
      letterMsg = String(e);
    }
  }

  async function copyLetter() {
    try {
      await navigator.clipboard.writeText(letterText);
      letterMsg = "Copiada al portapapeles.";
    } catch (e) {
      letterMsg = `No se pudo copiar: ${e}`;
    }
  }

  const terms = (ts: { surface: string }[]) => ts.map((t) => t.surface).join(", ");

  init();
</script>

{#if !jobId}
  <h1>Optimizador</h1>
  <p class="muted">Elige una vacante en <a href="/vacantes">Vacantes</a> y pulsa «Adaptar mi CV».</p>
{:else}
  <div class="page">
    <header>
      <p class="small"><a href="/vacantes">← Vacantes</a></p>
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
        {#if cost}<p class="muted small">Redactor externo: {costLabel(cost.cv)}.</p>{/if}
        {#if busy === "run"}
          <p class="muted small">Con modelos locales en CPU puede tardar varios minutos. No cierres la app.</p>
        {/if}
      </div>
    {:else}
      {@const opt = view.opt}
      <div class="card scores">
        <div>
          <span class="muted small">ATS antes</span>
          <strong class="big">{opt.prepared.ats_before.total}</strong>
        </div>
        <span class="arrow" aria-hidden="true">→</span>
        <div>
          <span class="muted small">ATS después{saved ? " (guardado)" : ", con lo que aprobó el guardián"}</span>
          <strong class="big">{saved?.ats_after ?? opt.ats_after.total}</strong>
        </div>
        <button onclick={run} disabled={busy !== ""} class="regen">
          {busy === "run" ? "Generando…" : "Generar otra propuesta"}
        </button>
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
          <p class="ok"><strong>CV guardado.</strong> ATS {saved.ats_before ?? "?"} → {saved.ats_after ?? "?"}</p>
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

    {#if profile && busy !== "load"}
      <section class="card letter" aria-labelledby="letter-title">
        <h2 id="letter-title">Carta de presentación</h2>
        <p class="muted small">
          Tres párrafos: por qué este puesto, 2 o 3 logros de tu CV y un cierre. El guardián revisa cada párrafo con las
          mismas reglas que el resumen; los rechazados no entran. Puedes editarla antes de guardarla.
        </p>
        <div class="row">
          <button class:primary={!letter} onclick={writeLetter} disabled={busy !== ""}>
            {busy === "letter" ? "Redactando…" : letter ? "Generar otra" : "Generar carta"}
          </button>
          {#if busy === "letter"}<span class="muted small">Con modelos locales puede tardar unos minutos.</span>{/if}
          {#if cost && busy !== "letter"}
            <span class="muted small">{costLabel(cost.letter)}; hasta el doble si el guardián pide repetir.</span>
          {/if}
        </div>
        {#each letter?.paragraphs.filter((p) => p.issues.length) ?? [] as p}
          <div class="rejected small">
            <p class="danger">Párrafo rechazado: {p.issues.join(" ")}</p>
            <p class="muted">{p.text}</p>
          </div>
        {/each}
        {#if letter}
          <label>Texto <textarea rows="14" bind:value={letterText}></textarea></label>
          <div class="row">
            <button class="primary" onclick={saveLetter} disabled={!letterText.trim()}>Guardar carta</button>
            <button onclick={copyLetter} disabled={!letterText.trim()}>Copiar</button>
            {#if letterMsg}<span class="small" role="status">{letterMsg}</span>{/if}
          </div>
        {/if}
      </section>
    {/if}
  </div>
{/if}

<style>
  .letter { display: flex; flex-direction: column; gap: 0.6rem; }
  .letter h2 { margin: 0; }
  .letter p { margin: 0; }
  .rejected { border-left: 3px solid var(--danger); padding-left: 0.6rem; }
  .templates { flex-basis: 100%; border: 0; padding: 0; margin: 0 0 0.5rem; display: flex; flex-direction: column; gap: 0.2rem; }
  .templates legend { font-weight: 600; margin-bottom: 0.2rem; }
  .page { display: flex; flex-direction: column; gap: 0.75rem; max-width: 960px; }
  header h1 { margin-bottom: 0.25rem; }
  header p { margin: 0; }
  .scores { display: flex; align-items: center; gap: 1.25rem; flex-wrap: wrap; }
  .scores > div { display: flex; flex-direction: column; }
  .big { font-size: 2rem; font-variant-numeric: tabular-nums; }
  .arrow { font-size: 1.5rem; color: var(--muted); }
  .regen { margin-left: auto; }
  .exp { margin: 0.75rem 0 0; }
  .change.rejected { border-color: var(--danger); }
  .diff { display: grid; grid-template-columns: 1fr 1fr; gap: 1rem; margin-top: 0.5rem; }
  .diff p, .proposed { margin: 0.2rem 0; }
  .original { color: var(--muted); }
  .save { display: flex; align-items: center; gap: 0.75rem; flex-wrap: wrap; }
</style>

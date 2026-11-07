<script lang="ts">
  import { untrack } from "svelte";
  import PageHead from "$lib/components/PageHead.svelte";
  import Icon from "$lib/components/Icon.svelte";
  import Meter from "$lib/components/Meter.svelte";
  import CvPaper from "$lib/components/CvPaper.svelte";
  import { openPath, revealItemInDir } from "@tauri-apps/plugin-opener";
  import { page } from "$app/state";
  import { profileStore } from "$lib/profile.svelte";
  import {
    jobsApi, optimizerApi, TEMPLATES, TEMPLATE_LABELS,
    type Choices, type CvDocument, type Template, type JobDetail, type OptimizerView, type SavedCv, type SavedEntry,
  } from "$lib/api";

  const jobId = $derived(page.url.searchParams.get("job"));
  const profile = $derived(profileStore.list.find((p) => p.id === profileStore.id) ?? null);

  let detail = $state<JobDetail | null>(null);
  let view = $state<OptimizerView | null>(null);
  let choices = $state<Choices>({ summary: false, bullets: [] });
  let saved = $state<SavedCv | null>(null);
  let template = $state<Template>("clasica");
  let busy = $state<"" | "load" | "run" | "save">("load");
  let error = $state("");
  let preview = $state<CvDocument | null>(null);

  // Biblioteca: CVs ya adaptados del perfil activo.
  let library = $state<SavedEntry[]>([]);
  let selectedId = $state<string | null>(null);
  const selected = $derived(library.find((e) => e.version.id === selectedId) ?? library[0] ?? null);
  const parse = (json: string): CvDocument | null => {
    try { return JSON.parse(json) as CvDocument; } catch { return null; }
  };
  const docs = $derived(new Map(library.map((e) => [e.version.id, parse(e.version.content_json)])));
  const fmtDate = (iso: string) =>
    new Date(iso).toLocaleDateString("es-MX", { day: "numeric", month: "short", year: "numeric" });

  const expLabel = $derived(new Map(view?.experiences ?? []));

  // Corregir un CV ya guardado: se edita una copia y la hoja de abajo la muestra al instante.
  let draft = $state<CvDocument | null>(null);
  let draftBusy = $state(false);
  let draftMsg = $state("");
  const lines = (t: string) => t.split("\n").map((l) => l.trim()).filter(Boolean);
  function startEdit(d: CvDocument) {
    draft = structuredClone($state.snapshot(d)) as CvDocument;
    draftMsg = "";
  }
  async function saveDraft(id: string) {
    if (!draft) return;
    draftBusy = true;
    error = "";
    try {
      const v = await optimizerApi.updateSaved(id, $state.snapshot(draft) as CvDocument);
      library = library.map((e) => (e.version.id === id ? { ...e, version: v } : e));
      draft = null;
      draftMsg = "Cambios guardados: el PDF y el DOCX ya los tienen.";
    } catch (e) {
      error = String(e);
    } finally {
      draftBusy = false;
    }
  }

  // Corrección a mano: una propuesta a la vez ("s" = resumen, número = logro).
  let editing = $state<{ key: "s" | number; text: string } | null>(null);
  let editBusy = $state(false);
  let edited = $state<Set<string>>(new Set());

  $effect(() => {
    const pid = profileStore.id;
    const jid = jobId;
    if (profileStore.ready) untrack(() => load(pid, jid));
  });

  async function load(pid: string | null, jid: string | null) {
    busy = "load";
    error = "";
    view = null;
    detail = null;
    preview = null;
    try {
      if (!jid) {
        library = pid ? await optimizerApi.listSaved(pid) : [];
      } else {
        detail = await jobsApi.get(jid, pid);
        if (pid) show(await optimizerApi.get(jid, pid));
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
    editing = null;
    edited = new Set();
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

  async function saveEdit() {
    if (!jobId || !editing || !view) return;
    const { key, text } = editing;
    editBusy = true;
    error = "";
    try {
      const keep = $state.snapshot(choices);
      const v = await optimizerApi.edit(jobId, key === "s" ? null : key, text);
      view = v;
      saved = null;
      // Se respetan tus otras decisiones; la corregida se usa si el guardián la aprueba.
      if (key === "s") keep.summary = v.summary_issues.length === 0;
      else keep.bullets[key] = v.bullet_issues[key].length === 0;
      choices = keep;
      edited = new Set([...edited, String(key)]);
      editing = null;
    } catch (e) {
      error = String(e);
    } finally {
      editBusy = false;
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

  // Vista previa: se rehace al cambiar decisiones, plantilla o textos.
  $effect(() => {
    if (!jobId || !view) return;
    const c = $state.snapshot(choices);
    const t = template;
    void view.opt;
    const id = setTimeout(() => {
      optimizerApi.preview(jobId, c, t).then((d) => (preview = d)).catch(() => {});
    }, 120);
    return () => clearTimeout(id);
  });

  const terms = (ts: { surface: string }[]) => ts.map((t) => t.surface).join(", ");
</script>

{#snippet editor(key: "s" | number, original: string)}
  {#if editing?.key === key}
    <div class="editbox">
      <label>Tu corrección
        <textarea rows="3" bind:value={editing.text}></textarea>
      </label>
      <div class="row">
        <button class="tonal" onclick={saveEdit} disabled={editBusy || !editing.text.trim() || editing.text.trim() === original.trim()}>
          <Icon name="check" size={20} /> {editBusy ? "Revisando…" : "Guardar corrección"}
        </button>
        <button class="link" onclick={() => (editing = null)} disabled={editBusy}>Cancelar</button>
      </div>
      <p class="muted small">Se revisa de nuevo: no puede agregar cifras ni habilidades que no estén en tu CV.</p>
    </div>
  {:else}
    <p class="proposed">{original}</p>
    <div class="row edrow">
      {#if edited.has(String(key))}<span class="tag">Corregido por ti</span>{/if}
      <button class="link" onclick={() => (editing = { key, text: original })} disabled={editing !== null}>
        <Icon name="edit" size={18} /> Corregir texto
      </button>
    </div>
  {/if}
{/snippet}

{#if !jobId}
  <PageHead icon="edit_document" title="Optimizador" subtitle="Tus CVs adaptados, uno por vacante" tone="tertiary">
    {#if library.length}
      <a class="button primary" href="/vacantes"><Icon name="add" size={20} /> Adaptar para otra vacante</a>
    {/if}
  </PageHead>
  {#if error}<p class="danger" role="alert">{error}</p>{/if}
  {#if busy === "load"}
    <p class="muted">Cargando…</p>
  {:else if !library.length || !selected}
    <div class="empty">
      <span class="empty-icon"><Icon name="edit_document" size={32} /></span>
      <h2>Adapta tu CV a una vacante</h2>
      <p class="muted">Elige una vacante en Vacantes y pulsa «Adaptar mi CV». Aquí verás qué cambia, cuánto mejora y todos los CVs que guardes.</p>
      <a class="button primary" href="/vacantes"><Icon name="work" size={20} /> Ir a Vacantes</a>
    </div>
  {:else}
    {@const v = selected.version}
    {@const sdoc = docs.get(v.id)}
    <div class="library" class:editing={draft !== null}>
      <ul class="cvlist" aria-label="CVs adaptados">
        {#each library as e (e.version.id)}
          {@const d = docs.get(e.version.id)}
          <li>
            <button class="cvitem" aria-current={e.version.id === v.id ? "true" : undefined}
              onclick={() => { selectedId = e.version.id; draft = null; draftMsg = ""; }}>
              <span class="thumb" aria-hidden="true">{#if d}<CvPaper doc={d} />{/if}</span>
              <span class="cvmeta">
                <strong>{e.job_title ?? "Vacante eliminada"}</strong>
                {#if e.company}<span class="muted small">{e.company}</span>{/if}
                <span class="muted small">{fmtDate(e.version.created_at)} · {TEMPLATE_LABELS[e.version.variant] ?? e.version.variant}</span>
                {#if e.version.ats_after !== null}
                  <span class="gain">
                    <Meter value={e.version.ats_after} label="Filtros automáticos" />
                    {#if e.version.ats_before !== null && e.version.ats_after > e.version.ats_before}
                      <span class="small up">+{e.version.ats_after - e.version.ats_before}</span>
                    {/if}
                  </span>
                {/if}
              </span>
            </button>
          </li>
        {/each}
      </ul>

      <section class="viewer" class:editing={draft !== null} aria-label="CV elegido">
        <div class="card viewhead">
          <div>
            <h2>{selected.job_title ?? "Vacante eliminada"}</h2>
            <p class="muted small">{[selected.company, fmtDate(v.created_at), TEMPLATE_LABELS[v.variant]].filter(Boolean).join(" · ")}</p>
          </div>
          <dl class="ats">
            <dt>Antes</dt><dd><Meter layout="row" value={v.ats_before ?? 0} label="Tu CV antes" /></dd>
            <dt>Adaptado</dt><dd><Meter layout="row" value={v.ats_after ?? 0} label="CV adaptado" /></dd>
          </dl>
          {#if v.missing.length}
            <p class="small muted">Siguen faltando: {v.missing.join(", ")}</p>
          {/if}
          <div class="row">
            {#if v.pdf_path}
              <button class="primary" onclick={() => openPath(v.pdf_path!)}><Icon name="open_in_new" size={20} /> Abrir PDF</button>
              <button onclick={() => revealItemInDir(v.pdf_path!)}>Mostrar en la carpeta</button>
            {/if}
            {#if v.docx_path}<button onclick={() => openPath(v.docx_path!)}><Icon name="description" size={20} /> Abrir DOCX</button>{/if}
            {#if sdoc && !draft}
              <button class="tonal" onclick={() => startEdit(sdoc)}><Icon name="edit" size={20} /> Corregir texto</button>
            {/if}
            {#if v.job_id && selected.job_title}
              <a class="button link" href="/optimizador?job={v.job_id}"><Icon name="refresh" size={20} /> Adaptar de nuevo</a>
            {/if}
          </div>
          {#if draftMsg}<p class="ok small" role="status">{draftMsg}</p>{/if}
        </div>
        {#if draft}
          <form class="card docedit" onsubmit={(e) => { e.preventDefault(); saveDraft(v.id); }} aria-label="Corregir el CV">
            <div class="edhead">
              <h2>Corregir el CV</h2>
              <p class="muted small">Cambia lo que quieras: la hoja se actualiza mientras escribes. Al guardar se rehacen el PDF y el DOCX.</p>
            </div>
            <fieldset>
              <legend>Encabezado</legend>
              <div class="grid">
                <label>Nombre<input bind:value={draft.name} /></label>
                <label>Titular<input bind:value={draft.headline} /></label>
              </div>
              <label>Contacto (uno por línea)
                <textarea rows="4" value={draft.contact.join("\n")} oninput={(e) => (draft!.contact = lines(e.currentTarget.value))}></textarea>
              </label>
            </fieldset>
            {#if draft.summary !== null}
              <fieldset>
                <legend>{draft.headings.summary}</legend>
                <textarea rows="3" bind:value={draft.summary} aria-label={draft.headings.summary}></textarea>
              </fieldset>
            {/if}
            {#if draft.experience.length}
              <fieldset>
                <legend>{draft.headings.experience}</legend>
                {#each draft.experience as ex, i}
                  <div class="edblock">
                    <div class="grid three">
                      <label>Puesto<input bind:value={ex.title} /></label>
                      <label>Empresa<input bind:value={ex.company} /></label>
                      <label>Fechas<input bind:value={ex.dates} /></label>
                    </div>
                    {#each ex.bullets as _, j}
                      <textarea rows="2" bind:value={ex.bullets[j]} aria-label="Logro {j + 1} de {ex.title || `experiencia ${i + 1}`}"></textarea>
                    {/each}
                  </div>
                {/each}
              </fieldset>
            {/if}
            {#if draft.education.length}
              <fieldset>
                <legend>{draft.headings.education}</legend>
                {#each draft.education as ed}
                  <div class="grid three">
                    <label>Título<input bind:value={ed.degree} /></label>
                    <label>Institución<input bind:value={ed.institution} /></label>
                    <label>Fechas<input bind:value={ed.dates} /></label>
                  </div>
                {/each}
              </fieldset>
            {/if}
            <fieldset>
              <legend>{draft.headings.skills}, {draft.headings.certifications.toLowerCase()} e {draft.headings.languages.toLowerCase()}</legend>
              <label>{draft.headings.skills} (separadas por comas)
                <textarea rows="2" value={draft.skills.join(", ")}
                  oninput={(e) => (draft!.skills = e.currentTarget.value.split(",").map((x) => x.trim()).filter(Boolean))}></textarea>
              </label>
              <div class="grid">
                <label>{draft.headings.certifications} (una por línea)
                  <textarea rows="2" value={draft.certifications.join("\n")} oninput={(e) => (draft!.certifications = lines(e.currentTarget.value))}></textarea>
                </label>
                <label>{draft.headings.languages} (uno por línea)
                  <textarea rows="2" value={draft.languages.join("\n")} oninput={(e) => (draft!.languages = lines(e.currentTarget.value))}></textarea>
                </label>
              </div>
            </fieldset>
            <div class="row edactions">
              <button type="submit" class="primary" disabled={draftBusy}><Icon name="check" size={20} /> {draftBusy ? "Guardando…" : "Guardar cambios"}</button>
              <button type="button" class="link" onclick={() => (draft = null)} disabled={draftBusy}>Cancelar</button>
            </div>
          </form>
        {/if}
        {#if draft}
          <div class="sheet"><CvPaper doc={draft} label="Vista previa con tus cambios" /></div>
        {:else if sdoc}
          <div class="sheet"><CvPaper doc={sdoc} label="CV para {selected.job_title ?? 'la vacante'}" /></div>
        {/if}
      </section>
    </div>
  {/if}
{:else}
  <header class="crumbs">
    <a class="back" href="/optimizador"><Icon name="arrow_back" size={18} /> Mis CVs</a>
    <span class="muted" aria-hidden="true">·</span>
    <a class="back" href="/vacantes">Vacantes</a>
  </header>
  <PageHead icon="edit_document" tone="tertiary" title={detail ? detail.job.title : "Adaptar CV"}
    subtitle={detail ? [detail.job.company, detail.job.location, profile ? `perfil «${profile.name}»` : ""].filter(Boolean).join(" · ") : undefined} />

  {#if error}<p class="danger" role="alert">{error}</p>{/if}

  {#if busy === "load"}
    <p class="muted">Cargando…</p>
  {:else if !profile}
    <p class="notice">Primero crea un perfil con su CV maestro en <a href="/perfiles">Perfiles</a>.</p>
  {:else if !view}
    <div class="card start">
      <span class="empty-icon"><Icon name="edit_document" size={32} /></span>
      <div>
        <h2>Generar la propuesta</h2>
        <p class="muted">La IA reescribe tu resumen y tus logros para esta vacante, sin inventar nada. Después tú eliges qué usar y puedes corregir cualquier palabra.</p>
        <button class="primary" onclick={run} disabled={busy !== ""}>
          {busy === "run" ? "Analizando y redactando…" : "Generar propuesta"}
        </button>
        {#if busy === "run"}
          <p class="muted small">Con modelos locales en CPU puede tardar varios minutos. No cierres la app.</p>
        {/if}
      </div>
    </div>
  {:else}
    {@const opt = view.opt}
    <div class="work">
      <div class="review">
        <div class="card scores">
          <h2>Qué tan bien lo leen los filtros automáticos</h2>
          <p class="muted small">Muchas empresas usan un programa (ATS) que revisa tu CV antes que una persona.</p>
          <dl class="ats">
            <dt>Tu CV actual</dt>
            <dd><Meter layout="row" value={opt.prepared.ats_before.total} label="Tu CV actual" /></dd>
            <dt>CV adaptado{saved ? " (guardado)" : ""}</dt>
            <dd><Meter layout="row" value={saved?.ats_after ?? opt.ats_after.total} label="CV adaptado" /></dd>
          </dl>
          <button onclick={run} disabled={busy !== ""}>
            <Icon name="refresh" size={20} /> {busy === "run" ? "Generando…" : "Generar otra propuesta"}
          </button>
        </div>

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
          <h3>Lo que pide la vacante</h3>
          {#if opt.prepared.gaps.with_evidence.length}
            <p class="small"><strong>Ya lo tienes:</strong> {terms(opt.prepared.gaps.with_evidence)}</p>
          {/if}
          {#if opt.prepared.gaps.without_evidence.length}
            <p class="small">
              <strong>No está en tu CV:</strong> {terms(opt.prepared.gaps.without_evidence)}.
              <span class="muted">Si lo tienes, agrégalo en <a href="/perfiles">Perfiles</a>.</span>
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

        <h2 class="section">Revisa las propuestas</h2>
        <p class="muted small lead">Marca las que quieres usar. Si la IA se equivocó en una palabra, pulsa «Corregir texto».</p>
        {#if opt.review.summary}
          {@const sm = opt.review.summary}
          <div class="card change" class:rejected={view.summary_issues.length}>
            <label class="inline">
              <input type="checkbox" bind:checked={choices.summary} disabled={view.summary_issues.length > 0} />
              <strong>Resumen profesional</strong>
            </label>
            {@render editor("s", sm.proposed)}
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
                {@render editor(i, b.proposed)}
              </div>
            </div>
            {#if issues.length}
              <p class="danger small">Rechazado por el guardián: {issues.join(" ")} Se mantiene tu texto.</p>
            {:else if !opt.choices.bullets[i] && !edited.has(String(i))}
              <p class="muted small">Desmarcada: tu versión ya usa palabras del anuncio.</p>
            {/if}
          </div>
        {/each}
      </div>

      <aside class="side" aria-label="Vista previa y guardado">
        <div class="card sidecard">
          <div class="sidehead">
            <h2>Así quedará tu CV</h2>
            <div class="segmented" role="group" aria-label="Plantilla">
              {#each TEMPLATES as t}
                <button aria-pressed={template === t.id} title={t.hint} onclick={() => (template = t.id)}>{t.label}</button>
              {/each}
            </div>
          </div>
          {#if preview}
            <div class="sheet"><CvPaper doc={preview} /></div>
          {:else}
            <p class="muted small">Preparando la vista previa…</p>
          {/if}
          <button class="primary wide" onclick={save} disabled={busy !== ""}>
            <Icon name="download" size={20} />
            {busy === "save" ? "Generando PDF y DOCX…" : saved ? "Guardar otra versión" : "Guardar CV"}
          </button>
          <p class="muted small">Se crean el PDF y el DOCX, y la vacante pasa a «CV adaptado» en el tablero.</p>
          {#if saved}
            <div class="savedbox" role="status">
              <p><strong>CV guardado.</strong> Filtros automáticos: {saved.ats_before ?? "?"} → {saved.ats_after ?? "?"}</p>
              {#if saved.missing.length}<p class="small">Siguen faltando: {saved.missing.join(", ")}</p>{/if}
              <div class="row">
                {#if saved.pdf_path}
                  <button onclick={() => openPath(saved!.pdf_path!)}>Abrir PDF</button>
                  <button onclick={() => revealItemInDir(saved!.pdf_path!)}>Mostrar en la carpeta</button>
                {/if}
                {#if saved.docx_path}<button onclick={() => openPath(saved!.docx_path!)}>Abrir DOCX</button>{/if}
              </div>
            </div>
          {/if}
        </div>
      </aside>
    </div>
  {/if}
{/if}

<style>
  /* Biblioteca: lista de CVs a la izquierda, el elegido en grande a la derecha. */
  .library { display: grid; grid-template-columns: minmax(300px, 2fr) 3fr; gap: 1.25rem; align-items: start; margin-top: 1rem; }
  .cvlist { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 0.5rem; }
  .cvitem {
    display: flex; align-items: flex-start; justify-content: flex-start; gap: 1rem; width: 100%; height: auto; min-height: 0;
    padding: 0.75rem; border: 0; border-radius: var(--radius-l); text-align: left;
    background: var(--md-surface-container-low); color: var(--md-on-surface); font-weight: 400;
  }
  .cvitem:hover { background: color-mix(in srgb, var(--md-on-surface) 6%, var(--md-surface-container-low)); }
  .cvitem[aria-current="true"] { background: var(--md-secondary-container); color: var(--md-on-secondary-container); }
  .cvitem[aria-current="true"] .muted { color: inherit; }
  .thumb { flex: none; width: 84px; pointer-events: none; }
  .cvmeta { display: flex; flex-direction: column; gap: 0.15rem; min-width: 0; }
  .cvmeta strong { font-size: 1.0625rem; line-height: 1.4rem; overflow-wrap: anywhere; }
  .gain { display: inline-flex; align-items: center; gap: 0.5rem; margin-top: 0.35rem; }
  .up { color: var(--md-success); font-weight: 750; font-variant-numeric: tabular-nums; }
  .cvitem[aria-current="true"] .up { color: inherit; }
  .viewer { position: sticky; top: 0; display: flex; flex-direction: column; gap: 1rem; min-width: 0; }
  .viewhead { display: flex; flex-direction: column; gap: 0.75rem; }
  .viewhead h2 { margin: 0; }
  .viewhead p { margin: 0.15rem 0 0; }
  .sheet { max-width: 640px; width: 100%; margin-inline: auto; }
  /* Al corregir: sin la lista, el formulario a la izquierda y la hoja fija a la derecha. */
  .library.editing { grid-template-columns: minmax(0, 1fr); }
  .library.editing .cvlist { display: none; }
  .viewer.editing { position: static; display: grid; grid-template-columns: minmax(0, 1fr) minmax(0, 1fr); align-items: start; }
  .viewer.editing > .viewhead, .viewer.editing > .docedit { grid-column: 1; }
  .viewer.editing > .sheet { grid-column: 2; grid-row: 1 / span 2; position: sticky; top: 0; }
  @container main (max-width: 1000px) {
    .viewer.editing { grid-template-columns: minmax(0, 1fr); }
    .viewer.editing > .sheet { grid-column: 1; grid-row: auto; position: static; }
  }
  .docedit { display: flex; flex-direction: column; gap: 1.25rem; animation: rise-in 220ms var(--ease-decel) both; }
  .edhead h2 { margin: 0; }
  .edhead p { margin: 0.2rem 0 0; }
  .docedit fieldset { border: 0; padding: 0; margin: 0; display: flex; flex-direction: column; gap: 0.6rem; min-width: 0; }
  .docedit legend { padding: 0; margin-bottom: 0.6rem; font-size: 1.0625rem; font-weight: 700; color: var(--md-on-surface); }
  .edblock { display: flex; flex-direction: column; gap: 0.5rem; padding: 0.9rem; border-radius: var(--radius-m); background: var(--md-surface-container); }
  .edblock + .edblock { margin-top: 0.25rem; }
  .grid.three { grid-template-columns: repeat(auto-fit, minmax(160px, 1fr)); gap: 0.75rem; }
  .edactions { position: sticky; bottom: 0; padding: 0.75rem 0 0.25rem; background: var(--md-surface-container-low); }

  /* Adaptar: revisión a la izquierda, hoja y guardado fijos a la derecha. */
  .crumbs { display: flex; align-items: center; gap: 0.5rem; margin-bottom: 0.25rem; }
  .back { display: inline-flex; align-items: center; gap: 0.3rem; min-height: 40px; font-weight: 600; }
  .start { display: flex; gap: 1.25rem; align-items: flex-start; max-width: 46rem; margin-top: 1rem; }
  .start .empty-icon { flex: none; }
  .start h2 { margin: 0.25rem 0 0.35rem; }
  .start p { margin: 0 0 1rem; }
  .work { display: grid; grid-template-columns: minmax(0, 1fr) minmax(320px, 440px); gap: 1.25rem; align-items: start; margin-top: 1rem; }
  .review { display: flex; flex-direction: column; gap: 0.75rem; min-width: 0; }
  .side { position: sticky; top: 0; min-width: 0; }
  .sidecard { display: flex; flex-direction: column; gap: 0.9rem; }
  .sidehead { display: flex; flex-direction: column; gap: 0.6rem; }
  .sidehead h2 { margin: 0; }
  .sidehead .segmented button { padding: 0 0.85rem; }
  .sidecard p { margin: 0; }
  .wide { width: 100%; justify-content: center; }
  .savedbox {
    display: flex; flex-direction: column; gap: 0.5rem; padding: 0.85rem 1rem; border-radius: var(--radius-m);
    background: var(--md-success-container); color: var(--md-on-success-container);
  }

  .scores h2 { margin: 0 0 0.25rem; }
  .ats { display: grid; grid-template-columns: max-content 1fr; align-items: center; gap: 0.75rem 1.25rem; margin: 0.5rem 0 1rem; max-width: 36rem; }
  .viewhead .ats { margin: 0; }
  .ats dt { font-weight: 600; }
  .ats dd { margin: 0; }
  .section { margin: 1.25rem 0 0; }
  .lead { margin: 0; }
  .exp { margin: 0.75rem 0 0; }
  .rejected { background: var(--md-error-container); color: var(--md-on-error-container); }
  .diff { display: grid; grid-template-columns: 1fr 1fr; gap: 1rem; margin-top: 0.5rem; }
  .diff p, .proposed { margin: 0.2rem 0; }
  .original { color: var(--md-on-surface-variant); }
  .edrow { gap: 0.5rem; margin-top: 0.15rem; }
  .edrow button { margin-left: -0.75rem; }
  .tag {
    padding: 0.1rem 0.55rem; border-radius: var(--radius-s); font-size: 0.875rem; font-weight: 650;
    background: var(--md-secondary-container); color: var(--md-on-secondary-container);
  }
  .tag + button { margin-left: 0; }
  .editbox {
    display: flex; flex-direction: column; gap: 0.6rem; margin-top: 0.35rem; padding: 0.85rem;
    border-radius: var(--radius-m); background: var(--md-surface-container-lowest);
    animation: rise-in 220ms var(--ease-decel) both;
  }
  .editbox p { margin: 0; }

  @container main (max-width: 1100px) {
    .work { grid-template-columns: 1fr; }
    .side { position: static; }
    .sidecard .sheet { max-width: 520px; }
  }
  @container main (max-width: 820px) {
    .library { grid-template-columns: 1fr; }
    .viewer { position: static; }
  }
  @container main (max-width: 700px) {
    .diff { grid-template-columns: 1fr; }
    .ats { grid-template-columns: 1fr; gap: 0.25rem; }
  }
</style>

<script lang="ts">
  import { untrack } from "svelte";
  import { profileStore } from "#lib/profile.svelte.ts";
  import PageHead from "#lib/components/PageHead.svelte";
  import Select from "#lib/components/Select.svelte";
  import Icon from "#lib/components/Icon.svelte";
  import Meter from "#lib/components/Meter.svelte";
  import { receive, rise, send, sheet } from "#lib/motion.ts";
  import { flip } from "svelte/animate";
  import { openPath, openUrl } from "@tauri-apps/plugin-opener";
  import { goto } from "$app/navigation";
  import {
    api, boardApi, rememberedProfile, statsApi, rememberProfile, REMINDER_LABELS, SOURCE_LABELS, STATUSES, TEMPLATE_LABELS,
    STATUS_LABELS, type ApplicationDetail, type Stats, type BoardRow, type Profile, type Reminder, type ReminderDays,
    type Status,
  } from "#lib/api.ts";

  let profiles = $state<Profile[]>([]);
  let profileId = $state<string | null>(null);
  let rows = $state<BoardRow[]>([]);
  let query = $state("");
  let view = $state<"kanban" | "lista">("kanban");
  const NO_FILTERS = { status: "", source: "", from: "", to: "", minScore: "" };
  let filters = $state({ ...NO_FILTERS });
  let error = $state("");
  let loading = $state(true);

  let selectedId = $state<string | null>(null);
  let detail = $state<ApplicationDetail | null>(null);
  let form = $state({ notes: "", next_step: "", next_step_at: "" });
  let saving = $state(false);
  let savedMsg = $state("");

  let due = $state<Reminder[]>([]);
  let stats = $state<Stats | null>(null);
  let days = $state<ReminderDays>({ follow_up: 7, no_response: 21 });
  let daysMsg = $state("");

  let dragging = $state<string | null>(null);
  let dropTarget = $state<Status | null>(null);
  // Etapa de cada estado, para el punto de color de su columna: preparar, enviada, buena noticia o cerrada.
  const stage = (s: Status) =>
    ["Encontrada", "Evaluada", "CvAdaptado"].includes(s) ? "prep"
    : ["Postulada", "EnProceso"].includes(s) ? "sent"
    : ["Entrevista", "Oferta"].includes(s) ? "good"
    : "closed";

  const today = new Date().toISOString().slice(0, 10);
  const day = (iso: string | null) => iso?.slice(0, 10) ?? "";

  // Fecha de la fila: cuándo te postulaste o, si aún no, cuándo se encontró (igual que el filtro de la base).
  const rowDate = (r: BoardRow) => day(r.application.applied_at ?? r.found_at);
  const sources = $derived([...new Set(rows.map((r) => r.source_id))].sort());
  const filtered = $derived(Object.values(filters).some(Boolean));

  const visible = $derived.by(() => {
    const q = query.trim().toLowerCase();
    const { status, source, from, to, minScore } = filters;
    const min = Number(minScore) || 0;
    return rows.filter(
      (r) =>
        (!q || `${r.job_title} ${r.company ?? ""}`.toLowerCase().includes(q)) &&
        (view === "kanban" || !status || r.application.status === status) &&
        (!source || r.source_id === source) &&
        (!from || rowDate(r) >= from) &&
        (!to || rowDate(r) <= to) &&
        (!min || (r.application.match_score ?? 0) >= min),
    );
  });
  const columns = $derived(
    STATUSES.map((s) => ({ ...s, items: visible.filter((r) => r.application.status === s.id) })),
  );
  const selected = $derived(rows.find((r) => r.application.id === selectedId) ?? null);
  const dirty = $derived(
    selected !== null &&
      (form.notes !== (selected.application.notes ?? "") ||
        form.next_step !== (selected.application.next_step ?? "") ||
        form.next_step_at !== (selected.application.next_step_at ?? "")),
  );

  async function init() {
    try {
      profiles = await api.listProfiles();
      const saved = rememberedProfile();
      profileId = profiles.find((p) => p.id === saved)?.id ?? profiles[0]?.id ?? null;
      days = await boardApi.reminderDays();
      await reload();
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  async function reload() {
    if (!profileId) return;
    try {
      [rows, due, stats] = await Promise.all([
        boardApi.list(profileId),
        boardApi.dueReminders(profileId),
        statsApi.get(profileId),
      ]);
      error = "";
    } catch (e) {
      error = String(e);
    }
  }

  async function changeProfile() {
    if (profileId) rememberProfile(profileId);
    close();
    await reload();
  }

  async function select(r: BoardRow) {
    if (dirty && !confirm("Tienes notas sin guardar. ¿Descartarlas?")) return;
    selectedId = r.application.id;
    savedMsg = "";
    form = {
      notes: r.application.notes ?? "",
      next_step: r.application.next_step ?? "",
      next_step_at: r.application.next_step_at ?? "",
    };
    detail = null;
    try {
      detail = await boardApi.detail(r.application.id);
    } catch (e) {
      error = String(e);
    }
  }

  function close() {
    selectedId = null;
    detail = null;
  }

  async function move(id: string, status: Status) {
    const row = rows.find((r) => r.application.id === id);
    if (!row || row.application.status === status) return;
    try {
      row.application = await boardApi.setStatus(id, status);
      due = await boardApi.dueReminders(row.application.profile_id);
      if (id === selectedId) detail = await boardApi.detail(id);
    } catch (e) {
      error = String(e);
      await reload();
    }
  }

  async function saveNotes() {
    if (!selected) return;
    saving = true;
    try {
      await boardApi.saveNotes(selected.application.id, form.notes, form.next_step, form.next_step_at);
      const empty = (s: string) => (s.trim() ? s.trim() : null);
      selected.application.notes = empty(form.notes);
      selected.application.next_step = empty(form.next_step);
      selected.application.next_step_at = empty(form.next_step_at);
      form = {
        notes: selected.application.notes ?? "",
        next_step: selected.application.next_step ?? "",
        next_step_at: selected.application.next_step_at ?? "",
      };
      savedMsg = "Guardado.";
      [due, detail] = await Promise.all([
        boardApi.dueReminders(selected.application.profile_id),
        boardApi.detail(selected.application.id),
      ]);
    } catch (e) {
      error = String(e);
    } finally {
      saving = false;
    }
  }

  async function done(r: Reminder) {
    try {
      await boardApi.completeReminder(r.id);
      due = due.filter((d) => d.id !== r.id);
      if (detail) detail.reminders = detail.reminders.filter((d) => d.id !== r.id);
    } catch (e) {
      error = String(e);
    }
  }

  function openFor(r: Reminder) {
    const row = rows.find((x) => x.application.id === r.application_id);
    if (row) select(row);
  }

  async function saveDays() {
    daysMsg = "";
    try {
      await boardApi.setReminderDays($state.snapshot(days));
      daysMsg = "Guardado. Aplica a las que pases a «Postulada» desde ahora.";
    } catch (e) {
      daysMsg = String(e);
    }
  }

  function onDrop(e: DragEvent, status: Status) {
    e.preventDefault();
    const id = e.dataTransfer?.getData("text/plain") || dragging;
    dropTarget = null;
    dragging = null;
    if (id) move(id, status);
  }

  // El perfil se elige en el menú lateral: al cambiarlo, esta pantalla se recarga.
  $effect(() => {
    const id = profileStore.id;
    if (id && id !== profileId && profiles.some((p) => p.id === id)) {
      profileId = id;
      untrack(() => changeProfile());
    }
  });

  init();
</script>

<svelte:window onkeydown={(e) => e.key === "Escape" && selected && close()} />

<div class="page" class:with-detail={selected}>
  <div class="main">
    <PageHead icon="view_kanban" title="Tablero" subtitle="Sigue cada postulación hasta la respuesta" tone="secondary">
      <div class="segmented" role="group" aria-label="Vista">
        <button aria-pressed={view === "kanban"} onclick={() => (view = "kanban")}>Kanban</button>
        <button aria-pressed={view === "lista"} onclick={() => (view = "lista")}>Lista</button>
      </div>
      <input class="search" type="search" placeholder="Buscar puesto o empresa" bind:value={query} aria-label="Buscar" />
    </PageHead>

    {#if error}<p class="danger" role="alert">{error}</p>{/if}

    {#if stats && stats.funnel.applied}
      <ul class="summary" aria-label="Resumen">
        <li><strong>{stats.funnel.applied}</strong> {stats.funnel.applied === 1 ? "postulada" : "postuladas"}</li>
        <li><strong>{stats.funnel.interview}</strong> {stats.funnel.interview === 1 ? "entrevista" : "entrevistas"}</li>
        <li><strong>{stats.funnel.offer}</strong> {stats.funnel.offer === 1 ? "oferta" : "ofertas"}</li>
        {#if stats.response_rate !== null}
          <li title="De las postuladas hace 14 días o más">Te respondieron el <strong>{Math.round(stats.response_rate)} %</strong></li>
        {/if}
      </ul>
    {/if}

    {#if loading}
      <p class="muted">Cargando…</p>
    {:else if !profiles.length}
      <p class="notice">Primero crea un perfil en <a href="/perfiles">Perfiles</a>.</p>
    {:else if !rows.length}
      <p class="notice">
        Aún no hay postulaciones. En <a href="/vacantes">Vacantes</a> abre una vacante y pulsa «Agregar al tablero»
        o «Adaptar mi CV» (al guardar el CV entra aquí como «CV adaptado»).
      </p>
    {:else}
      {#if due.length}
        <section class="due notice" aria-label="Recordatorios vencidos">
          <h2>Recordatorios ({due.length})</h2>
          <ul>
            {#each due as r (r.id)}
              <li class="row">
                <span class="grow">
                  <strong>{REMINDER_LABELS[r.kind]}</strong> · {r.job_title}{r.company ? ` — ${r.company}` : ""}
                  <span class="muted small">({day(r.due_at)})</span>
                </span>
                <button class="link" onclick={() => openFor(r)}>Abrir</button>
                {#if r.kind === "SinRespuesta"}
                  <button onclick={() => move(r.application_id, "SinRespuesta")}>Marcar sin respuesta</button>
                {/if}
                <button onclick={() => done(r)}>Hecho</button>
              </li>
            {/each}
          </ul>
        </section>
      {/if}
      <details class="filterbox" open={filtered}>
        <summary>Filtros{filtered ? " (activos)" : ""}</summary>
      <div class="filters row">
        {#if view === "lista"}
          <label>Estado
            <Select bind:value={filters.status} label="Estado"
              options={[{ value: "", label: "Todos" }, ...STATUSES.map((s) => ({ value: s.id, label: s.label }))]} />
          </label>
        {/if}
        <label>Portal
          <Select bind:value={filters.source} label="Portal"
            options={[{ value: "", label: "Todos" }, ...sources.map((s) => ({ value: s, label: SOURCE_LABELS[s] ?? s }))]} />
        </label>
        <label>Desde <input type="date" bind:value={filters.from} /></label>
        <label>Hasta <input type="date" bind:value={filters.to} /></label>
        <label>Compatibilidad mínima
          <input type="number" min="0" max="100" step="5" bind:value={filters.minScore} placeholder="0" />
        </label>
        {#if filtered}<button class="link" onclick={() => (filters = { ...NO_FILTERS })}>Quitar filtros</button>{/if}
      </div>
      </details>
      {#if view === "lista"}<p class="muted small">{visible.length} de {rows.length} postulaciones</p>{/if}
      {#if view === "lista"}
        <div class="list">
          {#if visible.length}
            <table>
              <thead>
                <tr>
                  <th scope="col">Puesto</th>
                  <th scope="col">Empresa</th>
                  <th scope="col">Portal</th>
                  <th scope="col">Estado</th>
                  <th scope="col">Fecha</th>
                  <th scope="col" class="num">Compat.</th>
                </tr>
              </thead>
              <tbody>
                {#each visible as r (r.application.id)}
                  {@const a = r.application}
                  <tr class:active={a.id === selectedId}>
                    <th scope="row"><button class="link" onclick={() => select(r)}>{r.job_title}</button></th>
                    <td>{r.company ?? "—"}</td>
                    <td>{SOURCE_LABELS[r.source_id] ?? r.source_id}</td>
                    <td>{STATUS_LABELS[a.status]}</td>
                    <td>{rowDate(r)}</td>
                    <td class="num">
                      {#if a.match_score !== null}<Meter value={a.match_score} label="Compatibilidad" />{:else}—{/if}
                    </td>
                  </tr>
                {/each}
              </tbody>
            </table>
          {:else}
            <p class="muted">Ninguna postulación coincide con los filtros.</p>
          {/if}
        </div>
      {:else}
      <div class="boardnav">
        <span class="muted small">Arrastra las tarjetas para cambiar su estado</span>
      </div>
      <div class="board">
        {#each columns as col (col.id)}
          <section
            class="col"
            class:target={dropTarget === col.id}
            aria-label={col.label}
            ondragover={(e) => { e.preventDefault(); dropTarget = col.id; }}
            ondragleave={() => { if (dropTarget === col.id) dropTarget = null; }}
            ondrop={(e) => onDrop(e, col.id)}
          >
            <h2><span class="dot {stage(col.id)}" aria-hidden="true"></span>{col.label} <span class="count">{col.items.length}</span></h2>
            {#if !col.items.length}<div class="drop" aria-hidden="true">Suelta aquí</div>{/if}
            <div class="colcards">
            {#each col.items as r (r.application.id)}
              {@const a = r.application}
              <button
                in:receive={{ key: a.id }}
                out:send={{ key: a.id }}
                animate:flip={{ duration: 260 }}
                class="cardbtn"
                class:active={a.id === selectedId}
                class:dragging={dragging === a.id}
                draggable="true"
                ondragstart={(e) => { dragging = a.id; e.dataTransfer?.setData("text/plain", a.id); }}
                ondragend={() => { dragging = null; dropTarget = null; }}
                onclick={() => select(r)}
              >
                <span class="top">
                  <strong>{r.job_title}</strong>
                  {#if a.match_score !== null}
                    <Meter value={a.match_score} label="Compatibilidad" />
                  {/if}
                </span>
                <span class="muted small">{[r.company, SOURCE_LABELS[r.source_id] ?? r.source_id].filter(Boolean).join(" · ")}</span>
                {#if a.next_step || a.next_step_at}
                  <span class="small" class:danger={a.next_step_at && a.next_step_at < today}>
                    {a.next_step ?? "Próximo paso"}{a.next_step_at ? ` · ${a.next_step_at}` : ""}
                  </span>
                {/if}
              </button>
            {/each}
            </div>
          </section>
        {/each}
      </div>
      {/if}
      <details class="days small">
        <summary>Plazos de los recordatorios</summary>
        <div class="row">
          <label>Seguimiento (días después de postularte)
            <input type="number" min="1" max="365" bind:value={days.follow_up} />
          </label>
          <label>«Sin respuesta» (días)
            <input type="number" min="1" max="365" bind:value={days.no_response} />
          </label>
          <button onclick={saveDays}>Guardar plazos</button>
        </div>
        {#if daysMsg}<p class="muted" role="status">{daysMsg}</p>{/if}
      </details>
    {/if}
  </div>

  {#if selected}
    {@const a = selected.application}
    <button class="scrim" aria-label="Cerrar ficha" tabindex="-1" onclick={close}></button>
    <aside class="detail card" aria-label="Ficha de la postulación" in:sheet>
      <div class="row head">
        <h2>{selected.job_title}</h2>
        <button class="link" onclick={close} aria-label="Cerrar ficha"><Icon name="close" size={24} /></button>
      </div>
      <p class="muted small">
        {[selected.company, SOURCE_LABELS[selected.source_id] ?? selected.source_id].filter(Boolean).join(" · ")}
        {#if a.match_score !== null}<span class="detailscore">Compatibilidad <Meter value={a.match_score} label="Compatibilidad" /></span>{/if}
      </p>
      <div class="row">
        {#if selected.url}<button onclick={() => openUrl(selected.url)}>Abrir anuncio</button>{/if}
        <button onclick={() => goto(`/optimizador?job=${a.job_id}`)}>Adaptar CV</button>
      </div>

      <label>
        Estado
        <Select value={a.status} label="Estado" onchange={(v) => move(a.id, v as Status)}
          options={STATUSES.map((s) => ({ value: s.id, label: s.label }))} />
      </label>

      <dl class="dates small">
        <dt>Encontrada</dt><dd>{day(selected.found_at)}</dd>
        <dt>Postulada</dt><dd>{day(a.applied_at) || "—"}</dd>
        <dt>Primera respuesta</dt><dd>{day(a.first_response_at) || "—"}</dd>
      </dl>

      <h3>CV enviado</h3>
      {#if !detail}
        <p class="muted small">Cargando…</p>
      {:else if detail.cv}
        {@const cv = detail.cv}
        <p class="small">
          {TEMPLATE_LABELS[cv.variant] ?? cv.variant} · {day(cv.created_at)} · filtros automáticos {cv.ats_before ?? "?"} → <strong>{cv.ats_after ?? "?"}</strong>
        </p>
        {#if cv.missing.length}<p class="muted small">Le falta: {cv.missing.join(", ")}</p>{/if}
        <div class="row">
          {#if cv.pdf_path}<button onclick={() => openPath(cv.pdf_path!)}>Abrir PDF</button>{/if}
          {#if cv.docx_path}<button onclick={() => openPath(cv.docx_path!)}>Abrir DOCX</button>{/if}
        </div>
      {:else}
        <p class="muted small">Todavía no adaptaste tu CV para esta vacante.</p>
      {/if}

      <h3>Seguimiento</h3>
      <label>Notas <textarea rows="4" bind:value={form.notes}></textarea></label>
      <div class="next">
        <label>Próximo paso <input bind:value={form.next_step} placeholder="Ej.: entrevista técnica" /></label>
        <label>Fecha <input type="date" bind:value={form.next_step_at} /></label>
      </div>
      <div class="row">
        <button class="primary" onclick={saveNotes} disabled={!dirty || saving}>
          {saving ? "Guardando…" : "Guardar"}
        </button>
        {#if savedMsg && !dirty}<span class="ok small" role="status">{savedMsg}</span>{/if}
      </div>

      {#if detail?.reminders.length}
        <h3>Recordatorios</h3>
        <ul class="history small">
          {#each detail.reminders as r (r.id)}
            <li>
              {day(r.due_at)}: {REMINDER_LABELS[r.kind]}
              {#if day(r.due_at) <= today}<button class="link" onclick={() => done(r)}>Hecho</button>{/if}
            </li>
          {/each}
        </ul>
      {/if}

      {#if detail?.history.length}
        <h3>Historial</h3>
        <ol class="history small">
          {#each detail.history as h}
            <li>{day(h.at)}: {h.from ? `${STATUS_LABELS[h.from]} → ` : ""}{STATUS_LABELS[h.to]}</li>
          {/each}
        </ol>
      {/if}
    </aside>
  {/if}
</div>

<style>
  .summary { display: flex; flex-wrap: wrap; gap: 0.5rem; list-style: none; margin: 0 0 1rem; padding: 0; }
  .summary li {
    padding: 0.35rem 0.9rem; border-radius: var(--radius-full);
    background: var(--md-surface-container); color: var(--md-on-surface-variant);
  }
  .summary strong { color: var(--md-on-surface); font-size: 1.0625rem; }
  .filterbox { margin-bottom: 0.5rem; }
  .filterbox .filters { margin-top: 0.5rem; }
  .detailscore { display: flex; align-items: center; gap: 0.6rem; margin-top: 0.5rem; }
  .page { display: grid; grid-template-columns: minmax(0, 1fr); gap: 1rem; min-height: calc(100vh - 5rem); }
  .page.with-detail { grid-template-columns: minmax(0, 1fr) 340px; }
  .main { display: flex; flex-direction: column; min-width: 0; min-height: 0; }
  .search { width: 15rem; }
  .filters { align-items: end; margin-bottom: 0.25rem; }
  .filters label { width: auto; }
  .filters :global(.select) { min-width: 10rem; }
  .filters input[type="number"] { width: auto; max-width: 11rem; }
  .list { flex: 1; min-height: 0; overflow: auto; }
  .list table { width: 100%; border-collapse: collapse; }
  .list th, .list td { padding: 0.35rem 0.5rem; border-bottom: 1px solid var(--border); text-align: left; }
  .list thead th { font-size: 0.875rem; color: var(--muted); font-weight: 650; position: sticky; top: 0; background: var(--bg); }
  .list tbody th { font-weight: 500; }
  .list tbody th button { text-align: left; }
  .list tr.active { background: var(--accent-soft); }
  .list .num { text-align: right; }
  /* Estados en cuadrícula: caben todos sin desplazarse de lado y llenan el espacio de abajo. */
  .board {
    display: grid; grid-template-columns: repeat(auto-fill, minmax(232px, 1fr)); gap: 0.75rem;
    align-items: stretch; padding: 0.25rem 0 0.75rem;
  }
  .boardnav { display: flex; align-items: center; gap: 0.25rem; margin: -0.25rem 0 0.25rem; }
  .boardnav span { margin-right: auto; }
  .cardbtn { cursor: grab; }
  .cardbtn:active { cursor: grabbing; }
  .col {
    min-width: 0; display: flex; flex-direction: column; gap: 0.5rem;
    background: var(--md-surface-container); border: 2px dashed transparent; border-radius: var(--radius-l); padding: 0.5rem;
    transition: background-color 150ms var(--ease), border-color 150ms var(--ease);
  }
  .col.target { border-color: var(--md-primary); background: var(--md-primary-container); }
  /* Una columna con muchas tarjetas se desplaza dentro y no estira su fila. */
  .colcards { display: flex; flex-direction: column; gap: 0.5rem; max-height: 26rem; overflow-y: auto; padding: 2px; margin: -2px; scrollbar-width: thin; }
  .colcards:empty { display: none; }
  .col h2 { font-size: 0.9375rem; margin: 0.25rem 0.35rem 0.1rem; display: flex; align-items: center; justify-content: flex-start; color: var(--md-on-surface); }
  .col h2 .count { margin-left: auto; }
  .dot { flex: none; width: 10px; height: 10px; margin-right: 0.5rem; border-radius: 50%; }
  .dot.prep { background: var(--md-outline); }
  .dot.sent { background: var(--md-primary); }
  .dot.good { background: var(--md-success); box-shadow: 0 0 0 3px color-mix(in srgb, var(--md-success) 22%, transparent); }
  .dot.closed { background: transparent; border: 2px solid var(--md-outline); }
  .count {
    min-width: 1.75rem; padding: 0 0.45rem; border-radius: var(--radius-full); text-align: center;
    background: var(--md-surface-container-highest); color: var(--md-on-surface-variant); font-size: 0.875rem; font-weight: 700;
  }
  .drop {
    display: grid; place-items: center; min-height: 56px; border-radius: var(--radius-m);
    border: 1.5px dashed var(--md-outline-variant); color: var(--md-on-surface-variant); font-size: 0.875rem;
  }
  .cardbtn {
    display: flex; flex-direction: column; align-items: stretch; justify-content: flex-start; gap: 0.25rem;
    text-align: left; width: 100%; min-height: 0; padding: 0.7rem 0.8rem;
    background: var(--md-surface-container-lowest); color: var(--md-on-surface);
    border: 0; border-radius: var(--radius-m); box-shadow: var(--elev-1);
    font-size: 1rem; font-weight: 400; letter-spacing: 0;
  }
  .cardbtn:hover:not(:disabled) { background: var(--md-surface-container-lowest); box-shadow: var(--elev-2); }
  .cardbtn.active { outline: 2px solid var(--md-primary); outline-offset: 0; }
  .cardbtn.dragging { opacity: 0.5; }
  .top { display: flex; justify-content: space-between; gap: 0.4rem; align-items: start; }
  .detail { display: flex; flex-direction: column; gap: 0.6rem; overflow-y: auto; min-height: 0; max-height: calc(100vh - 5rem); position: sticky; top: 0; }
  .scrim { display: none; }
  /* Angosto: la ficha pasa a hoja lateral modal (M3) sobre el tablero. */
  @container main (max-width: 900px) {
    .page.with-detail { grid-template-columns: minmax(0, 1fr); }
    .detail {
      position: fixed; inset: 0.75rem 0.75rem 0.75rem auto; z-index: 20;
      width: min(400px, calc(100vw - 1.5rem)); max-height: none;
      background: var(--md-surface-container-low); box-shadow: var(--elev-2);
    }
    .scrim {
      display: block; position: fixed; inset: 0; z-index: 19; min-height: 0; padding: 0; border: 0; border-radius: 0;
      background: color-mix(in srgb, var(--md-scrim) 32%, transparent); animation: fade-in 200ms var(--ease) both;
    }
    .scrim:hover:not(:disabled) { background: color-mix(in srgb, var(--md-scrim) 32%, transparent); }
  }
  @keyframes fade-in { from { opacity: 0; } }
  .detail .head { justify-content: space-between; flex-wrap: nowrap; align-items: start; }
  .detail h2 { margin: 0; }
  .detail h3 { margin: 0.5rem 0 0; }
  .detail p { margin: 0; }
  .dates { display: grid; grid-template-columns: auto 1fr; gap: 0.15rem 0.75rem; margin: 0; }
  .dates dt { color: var(--muted); }
  .dates dd { margin: 0; }
  .next { display: grid; grid-template-columns: 1fr; gap: 0.5rem; }
  .history { margin: 0; padding-left: 1.2rem; }
  .due { margin-bottom: 0.75rem; }
  .due h2 { font-size: 1rem; margin: 0 0 0.4rem; }
  .due ul { list-style: none; padding: 0; margin: 0; display: flex; flex-direction: column; gap: 0.3rem; }
  .due .grow { flex: 1; min-width: 12rem; }
  .days { margin-top: 0.5rem; }
  .days .row { align-items: end; }
  .days label { max-width: 16rem; }
</style>

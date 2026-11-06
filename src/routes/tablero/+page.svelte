<script lang="ts">
  import { openPath, openUrl } from "@tauri-apps/plugin-opener";
  import { goto } from "$app/navigation";
  import {
    api, boardApi, rememberedProfile, rememberProfile, scoreClass, SOURCE_LABELS, STATUSES, STATUS_LABELS,
    type ApplicationDetail, type BoardRow, type Profile, type Status,
  } from "$lib/api";

  let profiles = $state<Profile[]>([]);
  let profileId = $state<string | null>(null);
  let rows = $state<BoardRow[]>([]);
  let query = $state("");
  let error = $state("");
  let loading = $state(true);

  let selectedId = $state<string | null>(null);
  let detail = $state<ApplicationDetail | null>(null);
  let form = $state({ notes: "", next_step: "", next_step_at: "" });
  let saving = $state(false);
  let savedMsg = $state("");

  let dragging = $state<string | null>(null);
  let dropTarget = $state<Status | null>(null);

  const today = new Date().toISOString().slice(0, 10);
  const day = (iso: string | null) => iso?.slice(0, 10) ?? "";

  const visible = $derived.by(() => {
    const q = query.trim().toLowerCase();
    return q
      ? rows.filter((r) => `${r.job_title} ${r.company ?? ""}`.toLowerCase().includes(q))
      : rows;
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
      rows = await boardApi.list(profileId);
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
    } catch (e) {
      error = String(e);
    } finally {
      saving = false;
    }
  }

  function onDrop(e: DragEvent, status: Status) {
    e.preventDefault();
    const id = e.dataTransfer?.getData("text/plain") || dragging;
    dropTarget = null;
    dragging = null;
    if (id) move(id, status);
  }

  init();
</script>

<div class="page" class:with-detail={selected}>
  <div class="main">
    <header class="row">
      <h1>Tablero</h1>
      <label class="inline">
        Perfil
        <select bind:value={profileId} onchange={changeProfile} disabled={!profiles.length}>
          {#each profiles as p}<option value={p.id}>{p.name}</option>{/each}
        </select>
      </label>
      <input class="search" type="search" placeholder="Buscar puesto o empresa" bind:value={query} aria-label="Buscar" />
    </header>

    {#if error}<p class="danger" role="alert">{error}</p>{/if}

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
      <p class="muted small">Arrastra una tarjeta a otra columna para cambiar su estado, o cámbialo en su ficha.</p>
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
            <h2>{col.label} <span class="muted">{col.items.length}</span></h2>
            {#each col.items as r (r.application.id)}
              {@const a = r.application}
              <button
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
                    <span class="score {scoreClass(a.match_score)}" title="Compatibilidad">{a.match_score}</span>
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
          </section>
        {/each}
      </div>
    {/if}
  </div>

  {#if selected}
    {@const a = selected.application}
    <aside class="detail card" aria-label="Ficha de la postulación">
      <div class="row head">
        <h2>{selected.job_title}</h2>
        <button class="link" onclick={close} aria-label="Cerrar ficha">✕</button>
      </div>
      <p class="muted small">
        {[selected.company, SOURCE_LABELS[selected.source_id] ?? selected.source_id].filter(Boolean).join(" · ")}
        {#if a.match_score !== null}· compatibilidad <span class="score {scoreClass(a.match_score)}">{a.match_score}</span>{/if}
      </p>
      <div class="row">
        {#if selected.url}<button onclick={() => openUrl(selected.url)}>Abrir anuncio</button>{/if}
        <button onclick={() => goto(`/optimizador?job=${a.job_id}`)}>Adaptar CV</button>
      </div>

      <label>
        Estado
        <select value={a.status} onchange={(e) => move(a.id, e.currentTarget.value as Status)}>
          {#each STATUSES as s}<option value={s.id}>{s.label}</option>{/each}
        </select>
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
          Versión del {day(cv.created_at)} · ATS {cv.ats_before ?? "?"} → <strong>{cv.ats_after ?? "?"}</strong>
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
  .page { display: grid; grid-template-columns: minmax(0, 1fr); gap: 1rem; height: calc(100vh - 3rem); }
  .page.with-detail { grid-template-columns: minmax(0, 1fr) 340px; }
  .main { display: flex; flex-direction: column; min-width: 0; min-height: 0; }
  header { margin-bottom: 0.5rem; }
  header h1 { margin: 0 1rem 0 0; }
  header label.inline select { width: auto; }
  .search { max-width: 260px; margin-left: auto; }
  .board { display: flex; gap: 0.75rem; overflow-x: auto; flex: 1; min-height: 0; padding-bottom: 0.5rem; }
  .col {
    flex: 0 0 220px; display: flex; flex-direction: column; gap: 0.5rem;
    background: var(--hover); border: 1px dashed transparent; border-radius: 8px; padding: 0.5rem; overflow-y: auto;
  }
  .col.target { border-color: var(--accent); background: var(--accent-soft); }
  .col h2 { font-size: 0.85rem; margin: 0.1rem 0.2rem 0.2rem; display: flex; justify-content: space-between; }
  .cardbtn {
    display: flex; flex-direction: column; gap: 0.2rem; text-align: left; width: 100%;
    background: var(--surface); border: 1px solid var(--border); border-radius: 8px; padding: 0.5rem 0.6rem;
  }
  .cardbtn.active { border-color: var(--accent); }
  .cardbtn.dragging { opacity: 0.5; }
  .top { display: flex; justify-content: space-between; gap: 0.4rem; align-items: start; }
  .detail { display: flex; flex-direction: column; gap: 0.6rem; overflow-y: auto; min-height: 0; }
  .detail .head { justify-content: space-between; flex-wrap: nowrap; align-items: start; }
  .detail h2 { margin: 0; }
  .detail h3 { margin: 0.5rem 0 0; }
  .detail p { margin: 0; }
  .dates { display: grid; grid-template-columns: auto 1fr; gap: 0.15rem 0.75rem; margin: 0; }
  .dates dt { color: var(--muted); }
  .dates dd { margin: 0; }
  .next { display: grid; grid-template-columns: 1fr 9.5rem; gap: 0.5rem; }
  .history { margin: 0; padding-left: 1.2rem; }
</style>

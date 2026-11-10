<script lang="ts">
  import { untrack } from "svelte";
  import { profileStore } from "#lib/profile.svelte.ts";
  import PageHead from "#lib/components/PageHead.svelte";
  import Icon from "#lib/components/Icon.svelte";
  import Meter from "#lib/components/Meter.svelte";
  import { rise, stagger } from "#lib/motion.ts";
  import { openUrl, revealItemInDir } from "@tauri-apps/plugin-opener";
  import { goto } from "$app/navigation";
  import {
    api, boardApi, queueApi, rememberedProfile, rememberProfile, STATUS_LABELS,
    type Faq, type FaqView, type Profile, type QueueItem,
  } from "#lib/api.ts";

  let profiles = $state<Profile[]>([]);
  let profileId = $state<string | null>(null);
  let items = $state<QueueItem[]>([]);
  let faq = $state<FaqView | null>(null);
  let custom = $state<Faq[]>([]);
  let loading = $state(true);
  let error = $state("");
  let msg = $state("");
  let faqMsg = $state("");
  let editingFaq = $state<number | null>(null);
  // Pestañas para que el bloque no crezca más que la cola: «De tu CV» o «Tuyas».
  let faqTab = $state<"cv" | "own">("cv");

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
    error = "";
    msg = "";
    faqMsg = "";
    [items, faq] = await Promise.all([queueApi.list(profileId), queueApi.faq(profileId)]);
    custom = $state.snapshot(faq.custom);
  }

  async function changeProfile() {
    if (profileId) rememberProfile(profileId);
    try {
      await reload();
    } catch (e) {
      error = String(e);
    }
  }

  // El botón que copió muestra una palomita un momento: confirma sin mover nada de sitio.
  let copied = $state<string | null>(null);
  let copiedTimer: ReturnType<typeof setTimeout> | undefined;
  async function copy(text: string, key = text) {
    try {
      await navigator.clipboard.writeText(text);
      faqMsg = "";
      copied = key;
      clearTimeout(copiedTimer);
      copiedTimer = setTimeout(() => (copied = null), 1400);
    } catch (e) {
      faqMsg = `No se pudo copiar: ${e}`;
    }
  }

  /** Abre el anuncio y muestra el PDF en su carpeta para arrastrarlo al formulario. */
  async function openAndApply(it: QueueItem) {
    error = "";
    msg = "";
    try {
      if (it.pdf_path) await revealItemInDir(it.pdf_path);
      await openUrl(it.url);
      msg = it.pdf_path ? "Anuncio abierto; tu CV adaptado está en la ventana de archivos." : "Anuncio abierto.";
    } catch (e) {
      error = String(e);
    }
  }

  async function markApplied(it: QueueItem) {
    if (!profileId) return;
    error = "";
    try {
      const id = it.application_id ?? (await boardApi.add(it.job_id, profileId));
      await boardApi.setStatus(id, "Postulada");
      items = items.filter((i) => i.job_id !== it.job_id);
      msg = `«${it.title}» quedó como Postulada; el recordatorio de seguimiento está programado.`;
    } catch (e) {
      error = String(e);
    }
  }

  async function saveFaq() {
    if (!profileId) return;
    try {
      await queueApi.saveFaq(profileId, $state.snapshot(custom));
      faq = await queueApi.faq(profileId);
      custom = $state.snapshot(faq.custom);
      faqMsg = "Respuestas guardadas.";
    } catch (e) {
      faqMsg = String(e);
    }
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

<div class="page">
  <PageHead icon="send" title="Postular" subtitle="Envía tus solicitudes en minutos" tone="tertiary" />

  {#if error}<p class="danger" role="alert">{error}</p>{/if}

  {#if loading}
    <p class="muted">Cargando…</p>
  {:else if !profiles.length}
    <p class="notice">Primero crea un perfil en <a href="/perfiles">Perfiles</a>.</p>
  {:else}
    <div class="cols">
      <section class="queuecol" aria-labelledby="queue-title">
        <h2 id="queue-title">Cola de hoy</h2>
        <p class="muted small">
          Las que más encajan contigo (70 o más) y aún no envías.
        </p>
        {#if msg}<p class="ok small" role="status">{msg}</p>{/if}
        {#if !items.length}
          <p class="notice">
            No hay vacantes pendientes que encajen. Busca más en <a href="/vacantes">Vacantes</a>.
          </p>
        {/if}
        <ul class="queue">
          {#each items as it, i (it.job_id)}
            <li class="card" in:rise={{ delay: stagger(i) }}>
              <div class="row head">
                <Meter value={it.score} label="Compatibilidad" />
                <strong>{it.title}</strong>
                {#if it.company}<span class="muted">— {it.company}</span>{/if}
              </div>
              <p class="chips">
                {#if it.pdf_path}<span class="chip ok"><Icon name="check" size={16} /> CV adaptado</span>{/if}
                {#if it.status && !(it.pdf_path && it.status === "CvAdaptado")}<span class="chip">{STATUS_LABELS[it.status]}</span>{/if}
              </p>
              <div class="row">
                {#if !it.pdf_path}
                  <button class="primary" onclick={() => goto(`/optimizador?job=${encodeURIComponent(it.job_id)}`)}>Adaptar CV</button>
                {/if}
                <button class:primary={!!it.pdf_path} onclick={() => openAndApply(it)}>Abrir y postular</button>
                <button onclick={() => markApplied(it)}>Ya me postulé</button>
              </div>
            </li>
          {/each}
        </ul>
      </section>

      {#if faq}
        <section class="card faq" aria-labelledby="faq-title">
          <h2 id="faq-title">Respuestas frecuentes</h2>
          <p class="muted small">Para pegar en los formularios de los portales.</p>
          {#if faqMsg}<p class="small" role="status">{faqMsg}</p>{/if}
          <span class="sr-only" aria-live="polite">{copied ? "Copiado" : ""}</span>

          {#if faq.from_cv.length}
            <div class="segmented tabs" role="tablist" aria-label="Respuestas">
              <button role="tab" aria-selected={faqTab === "cv"} onclick={() => (faqTab = "cv")}>De tu CV</button>
              <button role="tab" aria-selected={faqTab === "own"} onclick={() => (faqTab = "own")}>Tuyas <span class="n">{custom.length}</span></button>
            </div>
          {/if}

          <div class="qascroll" role="tabpanel">
          {#if faqTab === "cv" && faq.from_cv.length}
            <ul class="qalist">
              {#each faq.from_cv as f}
                <li class="qa">
                  <span class="qtext"><span class="q">{f.question}</span><span class="a" title={f.answer}>{f.answer}</span></span>
                  <button class="icon-btn copybtn" class:done={copied === "cv:"+f.question} onclick={() => copy(f.answer, "cv:"+f.question)} aria-label="Copiar respuesta: {f.question}" title={copied === "cv:"+f.question ? "Copiado" : "Copiar"}><Icon name={copied === "cv:"+f.question ? "check" : "content_copy"} size={20} /></button>
                </li>
              {/each}
            </ul>
          {:else}
          <ul class="qalist">
            {#each custom as f, i}
              {#if editingFaq === i}
                <li class="qaedit">
                  <label>Pregunta<input bind:value={f.question} placeholder="Salario esperado" /></label>
                  <label>Respuesta<textarea bind:value={f.answer} rows="2"></textarea></label>
                  <div class="row">
                    <button class="tonal" onclick={() => (editingFaq = null)}><Icon name="check" size={20} /> Listo</button>
                    <button class="link danger" onclick={() => { custom.splice(i, 1); editingFaq = null; }}>Quitar</button>
                  </div>
                </li>
              {:else}
                <li class="qa">
                  <span class="qtext"><span class="q">{f.question || "Sin pregunta"}</span><span class="a" title={f.answer}>{f.answer || "—"}</span></span>
                  <button class="icon-btn copybtn" class:done={copied === "own:"+i} onclick={() => copy(f.answer, "own:"+i)} disabled={!f.answer.trim()} aria-label="Copiar respuesta: {f.question}" title={copied === "own:"+i ? "Copiado" : "Copiar"}><Icon name={copied === "own:"+i ? "check" : "content_copy"} size={20} /></button>
                  <button class="icon-btn" onclick={() => (editingFaq = i)} aria-label="Editar: {f.question}" title="Editar"><Icon name="edit" size={20} /></button>
                </li>
              {/if}
            {:else}
              <li class="muted small">Agrega las preguntas que te repiten los formularios.</li>
            {/each}
          </ul>
          {/if}
          </div>
          <div class="row faqactions">
            <button onclick={() => { custom.push({ question: "", answer: "" }); editingFaq = custom.length - 1; faqTab = "own"; }}><Icon name="add" /> Pregunta</button>
            <button class="primary" onclick={saveFaq}>Guardar</button>
          </div>
        </section>
      {/if}
    </div>
  {/if}
</div>

<style>
  .page { display: flex; flex-direction: column; gap: 1rem; }
  .cols { display: grid; grid-template-columns: minmax(0, 1fr) minmax(260px, 360px); gap: 1rem; align-items: start; }
  @container main (max-width: 820px) { .cols { grid-template-columns: 1fr; } }
  section h2, section p { margin: 0 0 0.5rem; }
  .queue { list-style: none; padding: 0; margin: 0; display: flex; flex-direction: column; gap: 0.6rem; }
  .queue p { margin: 0.3rem 0 0.5rem; }
  .faq { display: flex; flex-direction: column; gap: 0.6rem; padding: 1.1rem 1.25rem; }
  /* Junto a la cola, el bloque toma su alto (sin bajar de 24rem) y la lista se desplaza dentro. */
  @container main (min-width: 821px) {
    .cols { align-items: stretch; }
    .queuecol { align-self: start; }
    .faq { contain: size; min-height: 24rem; }
  }
  .tabs { display: flex; }
  .tabs button { flex: 1; padding: 0 0.6rem; justify-content: center; }
  .n { font-variant-numeric: tabular-nums; opacity: 0.8; }
  .qascroll { flex: 1 1 auto; min-height: 0; overflow-y: auto; margin: 0 -0.5rem; padding: 0 0.5rem; scrollbar-width: thin; }
  .copybtn { transition: background-color 150ms var(--ease), color 150ms var(--ease); }
  .copybtn.done { background: var(--md-success-container); color: var(--md-on-success-container); }
  .copybtn.done :global(.icon) { animation: pop-in 260ms var(--ease-decel); }
  @keyframes pop-in { from { transform: scale(0.4) rotate(-20deg); opacity: 0; } }
  .faqactions { margin-top: auto; padding-top: 0.25rem; }
  .qalist { list-style: none; padding: 0; margin: 0; display: flex; flex-direction: column; }
  .qa { display: flex; align-items: center; gap: 0.25rem; padding: 0.5rem 0; border-bottom: 1px solid var(--md-outline-variant); }
  .qa:last-child { border-bottom: 0; }
  .qtext { flex: 1; min-width: 0; display: flex; flex-direction: column; }
  .q { font-size: 0.9375rem; color: var(--md-on-surface-variant); }
  .a { font-weight: 650; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .qaedit { display: flex; flex-direction: column; gap: 0.6rem; padding: 0.9rem; margin: 0.25rem 0; border-radius: var(--radius-m); background: var(--md-surface-container); animation: rise-in 220ms var(--ease-decel) both; }
  .chips { display: flex; gap: 0.4rem; flex-wrap: wrap; margin: 0.4rem 0 0.6rem; }
  .chips:empty { display: none; }
  .chip {
    display: inline-flex; align-items: center; gap: 0.25rem; padding: 0.15rem 0.6rem; border-radius: var(--radius-s);
    background: var(--md-surface-container-highest); color: var(--md-on-surface-variant); font-size: 0.875rem; font-weight: 650;
  }
  .chip.ok { background: var(--md-success-container); color: var(--md-on-success-container); }
  /* Muy ancho: la cola en dos columnas de tarjetas y las respuestas con más espacio. */
  @container main (min-width: 1600px) {
    .cols { grid-template-columns: minmax(0, 1fr) minmax(360px, 440px); }
    .faq { min-height: 30rem; }
    .queue { align-items: stretch; gap: 0.75rem; display: grid; grid-template-columns: repeat(auto-fill, minmax(480px, 1fr)); }
  }
</style>

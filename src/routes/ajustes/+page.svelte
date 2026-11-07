<script lang="ts">
  import Icon from "$lib/components/Icon.svelte";
  import PageHead from "$lib/components/PageHead.svelte";
  import Select from "$lib/components/Select.svelte";
  import { revealItemInDir } from "@tauri-apps/plugin-opener";
  import { backupApi, jobsApi, settingsApi, weightsApi, type LlmSettings, type Weights, type WeightsStatus } from "$lib/api";
  import Meter from "$lib/components/Meter.svelte";

  const NO_KEYS = { ANTHROPIC_API_KEY: "", FRJ_OPENAI_API_KEY: "" };
  /** Proveedores con la dirección y el modelo ya puestos (compatibles con la API de OpenAI). */
  const PRESETS = {
    gemini: {
      label: "Gemini (Google)",
      url: "https://generativelanguage.googleapis.com/v1beta/openai",
      model: "gemini-3.8-flash",
      keyUrl: "aistudio.google.com/apikey",
      hint: "Tiene plan gratuito.",
    },
    openai: {
      label: "ChatGPT (OpenAI)",
      url: "https://api.openai.com/v1",
      model: "gpt-6.1-terra",
      keyUrl: "platform.openai.com/api-keys",
      hint: "De pago por uso.",
    },
  } as const;
  type Preset = keyof typeof PRESETS;
  const FIELDS = [
    "FRJ_WRITER", "FRJ_LOCAL_URL", "FRJ_LOCAL_MODEL", "FRJ_EMBED_MODEL",
    "FRJ_ANTHROPIC_MODEL", "FRJ_ANTHROPIC_EFFORT", "FRJ_OPENAI_URL", "FRJ_OPENAI_MODEL",
  ];
  const EFFORTS = ["low", "medium", "high", "xhigh", "max"];

  let s = $state<LlmSettings | null>(null);
  let values = $state<Record<string, string>>({});
  let keys = $state<Record<string, string>>({ ...NO_KEYS });
  let error = $state("");
  let msg = $state("");
  let saving = $state(false);

  let boards = $state("");
  let boardsMsg = $state("");
  let boardsError = $state("");

  const writer = $derived(values.FRJ_WRITER || "local");
  /** Qué proveedor de la tercera opción está elegido, según su dirección. */
  const preset = $derived<Preset | null>(
    (Object.keys(PRESETS) as Preset[]).find((p) => (values.FRJ_OPENAI_URL ?? "").startsWith(PRESETS[p].url)) ?? null,
  );
  const locked = (k: string) => s?.overridden.includes(k) ?? false;
  const hasKey = (k: string) => !!keys[k]?.trim() || (s?.saved_keys.includes(k) ?? false) || locked(k);

  function show(v: LlmSettings) {
    s = v;
    // Sin elección guardada, se marca la que está en uso.
    values = { FRJ_WRITER: v.writer_kind ?? "local", ...v.values };
    keys = { ...NO_KEYS };
  }

  function choosePreset(p: Preset) {
    values.FRJ_OPENAI_URL = PRESETS[p].url;
    values.FRJ_OPENAI_MODEL = PRESETS[p].model;
  }

  function chooseWriter(w: string) {
    values.FRJ_WRITER = w;
    if (w === "openai" && !values.FRJ_OPENAI_URL) choosePreset("gemini");
  }

  /** Lo que falta para que la opción elegida funcione, en palabras simples. */
  function missing(): string {
    if (writer === "anthropic" && !hasKey("ANTHROPIC_API_KEY")) return "Pega tu clave de Claude.";
    if (writer === "openai") {
      if (!values.FRJ_OPENAI_URL?.trim() || !values.FRJ_OPENAI_MODEL?.trim()) return "Elige Gemini o ChatGPT.";
      if (preset && !hasKey("FRJ_OPENAI_API_KEY")) return `Pega tu clave de ${PRESETS[preset].label}.`;
    }
    return "";
  }

  async function init() {
    try {
      boards = await jobsApi.boards();
      show(await settingsApi.get());
    } catch (e) {
      error = String(e);
    }
  }

  async function save() {
    error = "";
    msg = "";
    const why = missing();
    if (why) {
      error = why;
      return;
    }
    saving = true;
    try {
      // Se envían todos los campos conocidos: uno vacío vuelve al valor por defecto.
      const all = Object.fromEntries(FIELDS.map((f) => [f, values[f] ?? ""]));
      show(await settingsApi.save(all, $state.snapshot(keys)));
      msg = "Guardado. Ya se usa, sin reiniciar.";
    } catch (e) {
      error = String(e);
    } finally {
      saving = false;
    }
  }

  async function removeKey(k: string) {
    if (!confirm("¿Borrar la clave guardada?")) return;
    try {
      show(await settingsApi.deleteKey(k));
      msg = "Clave borrada.";
    } catch (e) {
      error = String(e);
    }
  }

  async function saveBoards() {
    boardsMsg = "";
    boardsError = "";
    try {
      boards = await jobsApi.saveBoards(boards);
      boardsMsg = "Guardado. Se consultan al buscar vacantes nuevas.";
    } catch (e) {
      boardsError = String(e);
    }
  }

  // Copia de seguridad
  let backupPath = $state("");
  let backupBusy = $state(false);
  let backupError = $state("");
  let restoreFile = $state<File | null>(null);

  async function makeBackup() {
    backupBusy = true;
    backupError = "";
    try {
      backupPath = await backupApi.create();
    } catch (e) {
      backupError = String(e);
    } finally {
      backupBusy = false;
    }
  }

  async function restore() {
    if (!restoreFile) return;
    backupBusy = true;
    backupError = "";
    try {
      await backupApi.restore(restoreFile); // si sale bien, la app se reinicia
    } catch (e) {
      backupError = String(e);
      backupBusy = false;
    }
  }

  // Compatibilidad a tu medida: pesos sugeridos por tus resultados (nunca se aplican solos).
  let ws = $state<WeightsStatus | null>(null);
  let wsMsg = $state("");
  const PARTS: [keyof Weights, string][] = [
    ["skills", "Habilidades"], ["semantic", "Parecido con tu CV"], ["title", "Puesto"],
    ["seniority", "Nivel de experiencia"], ["years", "Años"], ["logistics", "Remoto, región e idiomas"],
  ];
  const custom = $derived(ws ? PARTS.some(([k]) => Math.abs(ws!.current[k] - ws!.default[k]) > 0.01) : false);
  async function loadWeights() {
    try { ws = await weightsApi.status(); } catch (e) { wsMsg = String(e); }
  }
  async function applyWeights() {
    if (!ws?.suggested) return;
    await weightsApi.apply(ws.suggested);
    wsMsg = "Listo: la compatibilidad ya usa tus pesos.";
    await loadWeights();
  }
  async function resetWeights() {
    await weightsApi.reset();
    wsMsg = "Volviste a los pesos normales.";
    await loadWeights();
  }
  loadWeights();

  init();
</script>

{#snippet field(key: string, label: string, placeholder: string)}
  <label>
    {label}
    <input bind:value={values[key]} {placeholder} disabled={locked(key)} />
    {#if locked(key)}<span class="muted small">Lo fija el archivo .env.</span>{/if}
  </label>
{/snippet}

{#snippet apiKey(key: string, label: string)}
  <label>
    {label}
    <input
      type="password"
      autocomplete="off"
      bind:value={keys[key]}
      placeholder={s?.saved_keys.includes(key) ? "Ya está guardada (pega otra para cambiarla)" : "Pega aquí tu clave"}
      disabled={locked(key) || !!s?.keyring_error}
    />
  </label>
  {#if s?.saved_keys.includes(key)}
    <button class="link small" onclick={() => removeKey(key)}>Borrar clave guardada</button>
  {/if}
{/snippet}

<div class="page">
  <PageHead icon="settings" title="Ajustes" subtitle="Qué IA usar y copias de seguridad" tone="secondary" />
  {#if error}<p class="danger" role="alert">{error}</p>{/if}

  <div class="cols">
    <div class="col">
  {#if s}
    <section class="card">
      <h2>¿Qué IA adapta tus CV?</h2>
      <p class="muted small">
        Tu nombre, correo y teléfono nunca se envían a una IA en internet.
      </p>

      <fieldset class="choice" disabled={locked("FRJ_WRITER")}>
        <legend class="sr-only">IA</legend>
        <label class="option" class:on={writer === "local"}>
          <input type="radio" name="writer" checked={writer === "local"} onchange={() => chooseWriter("local")} />
          <span><strong>En tu computadora (gratis)</strong><br />
            <span class="muted small">Privado y sin costo. Necesita Ollama instalado y puede tardar varios minutos por CV.</span></span>
        </label>
        <label class="option" class:on={writer === "anthropic"}>
          <input type="radio" name="writer" checked={writer === "anthropic"} onchange={() => chooseWriter("anthropic")} />
          <span><strong>Claude</strong><br />
            <span class="muted small">La mejor calidad y rápido. De pago por uso (centavos por CV).</span></span>
        </label>
        <label class="option" class:on={writer === "openai"}>
          <input type="radio" name="writer" checked={writer === "openai"} onchange={() => chooseWriter("openai")} />
          <span><strong>Gemini o ChatGPT</strong><br />
            <span class="muted small">Rápido. Gemini tiene plan gratuito.</span></span>
        </label>
        {#if locked("FRJ_WRITER")}<span class="muted small">La elección la fija el archivo .env.</span>{/if}
      </fieldset>

      {#if writer === "anthropic"}
        <p class="small">Crea tu clave en <strong>console.anthropic.com</strong> → «API Keys» y pégala aquí.</p>
        {@render apiKey("ANTHROPIC_API_KEY", "Clave de Claude")}
      {:else if writer === "openai"}
        <div class="segmented" role="group" aria-label="Proveedor">
          {#each Object.entries(PRESETS) as [id, p]}
            <button aria-pressed={preset === id} onclick={() => choosePreset(id as Preset)}>{p.label}</button>
          {/each}
        </div>
        {#if preset}
          <p class="small">{PRESETS[preset].hint} Crea tu clave en <strong>{PRESETS[preset].keyUrl}</strong> y pégala aquí.</p>
        {:else}
          <p class="muted small">Usas un proveedor propio (lo configuraste en Opciones avanzadas).</p>
        {/if}
        {@render apiKey("FRJ_OPENAI_API_KEY", "Clave")}
      {/if}

      {#if s.keyring_error}
        <p class="danger small">No se pueden guardar claves: el llavero del sistema no responde ({s.keyring_error}).</p>
      {:else if writer !== "local"}
        <p class="muted small">Se guarda cifrada en tu computadora.</p>
      {/if}

      <div class="row">
        <button class="primary" onclick={save} disabled={saving}>{saving ? "Guardando…" : "Guardar"}</button>
        {#if msg}<span class="ok small" role="status">{msg}</span>{/if}
      </div>
      {#if s.error}
        <p class="danger small">La IA elegida no está lista: revisa que la clave esté guardada. <span class="muted">({s.error})</span></p>
      {:else}
        <p class="muted small">En uso: {s.writer}</p>
      {/if}
    </section>

  {:else if !error}
    <p class="muted">Cargando…</p>
  {/if}
    </div>
    <div class="col">
  <section class="card">
    <h2>Copia de seguridad</h2>
    <p class="muted small">
      Guarda en un solo archivo tus perfiles, CV, vacantes, postulaciones y los CV generados.
      Las claves de IA no se incluyen.
    </p>
    {#if backupError}<p class="danger small" role="alert">{backupError}</p>{/if}
    <div class="row">
      <button class="primary" onclick={makeBackup} disabled={backupBusy}>Hacer copia ahora</button>
      {#if backupPath}
        <span class="ok small" role="status">Guardada en {backupPath}</span>
        <button class="link small" onclick={() => revealItemInDir(backupPath)}>Mostrar en la carpeta</button>
      {/if}
    </div>
    <label class="filebtn">
      <input type="file" accept=".zip" onchange={(e) => { restoreFile = e.currentTarget.files?.[0] ?? null; e.currentTarget.value = ""; }} disabled={backupBusy} />
      <span><Icon name="history" size={20} /> Restaurar una copia…</span>
    </label>
    {#if restoreFile}
      <p class="notice small">
        Tus datos actuales se reemplazarán por los de «{restoreFile.name}» (los actuales se guardan aparte
        por si acaso) y la app se reiniciará.
      </p>
      <div class="row">
        <button class="danger" onclick={restore} disabled={backupBusy}>Sí, restaurar</button>
        <button onclick={() => (restoreFile = null)} disabled={backupBusy}>Cancelar</button>
      </div>
    {/if}
  </section>
  {#if ws}
    <section class="card tune">
      <h2>Compatibilidad a tu medida</h2>
      {#if ws.suggested}
        <p class="muted small">Según {ws.samples} postulaciones con resultado, esto pesaría más o menos:</p>
        <dl class="wtable">
          {#each PARTS as [k, label]}
            {@const d = ws.default[k]}
            {@const n = ws.suggested[k]}
            <dt>{label}</dt>
            <dd><span class="muted">{Math.round(d)}</span> → <strong class:up={n > d + 0.5} class:down={n < d - 0.5}>{Math.round(n)}</strong></dd>
          {/each}
        </dl>
        <div class="row">
          <button class="primary" onclick={applyWeights}>Aplicar</button>
          {#if custom}<button onclick={resetWeights}>Volver a lo normal</button>{/if}
        </div>
      {:else}
        <p class="muted small">
          La app aprende qué te funciona cuando tengas {ws.needed} postulaciones con resultado
          (al menos {ws.min_each} que avanzaron a entrevista y {ws.min_each} sin respuesta o rechazadas).
        </p>
        <Meter layout="row" value={ws.samples} max={ws.needed} label="Postulaciones con resultado" />
        <p class="small">{ws.advanced} avanzaron · {ws.not_advanced} no avanzaron</p>
        {#if custom}<div class="row"><button onclick={resetWeights}>Volver a lo normal</button></div>{/if}
      {/if}
      {#if custom}<p class="small ok">Usando pesos ajustados a tus resultados.</p>{/if}
      {#if wsMsg}<p class="small" role="status">{wsMsg}</p>{/if}
    </section>
  {/if}
  {#if s}
    <details class="card advanced">
      <summary>Opciones avanzadas</summary>

      <h3>IA en tu computadora (Ollama)</h3>
      <div class="grid">
        {@render field("FRJ_LOCAL_URL", "Dirección de Ollama", "http://localhost:11434/v1")}
        {@render field("FRJ_LOCAL_MODEL", "Modelo", "qwen3:8b")}
        {@render field("FRJ_EMBED_MODEL", "Modelo de similitud", "bge-m3")}
      </div>
      <p class="muted small">En uso: {s.local ?? "—"}</p>

      <h3>Claude</h3>
      <div class="grid">
        {@render field("FRJ_ANTHROPIC_MODEL", "Modelo", "claude-opus-5-5")}
        <label>
          Esfuerzo (más alto = mejor y más caro)
          <Select bind:value={values.FRJ_ANTHROPIC_EFFORT} label="Esfuerzo" disabled={locked("FRJ_ANTHROPIC_EFFORT")}
            options={[{ value: "", label: "Por defecto (medium)" }, ...EFFORTS.map((e) => ({ value: e, label: e }))]} />
        </label>
      </div>

      <h3>Otro proveedor compatible con OpenAI</h3>
      <div class="grid">
        {@render field("FRJ_OPENAI_URL", "Dirección", "https://api.openai.com/v1")}
        {@render field("FRJ_OPENAI_MODEL", "Modelo", "")}
      </div>

      <div class="row">
        <button onclick={save} disabled={saving}>{saving ? "Guardando…" : "Guardar"}</button>
      </div>

      <h3>Empresas que sigues</h3>
      <p class="muted small">
        Vacantes de empresas concretas que publican en Greenhouse, Lever o Ashby. Pega la dirección de su página de
        empleos, una por línea.
      </p>
      <label>
        Páginas de empleo
        <textarea rows="4" bind:value={boards} placeholder={"https://boards.greenhouse.io/empresa\nhttps://jobs.lever.co/empresa"}></textarea>
      </label>
      {#if boardsError}<p class="danger small" role="alert">{boardsError}</p>{/if}
      <div class="row">
        <button onclick={saveBoards}>Guardar empresas</button>
        {#if boardsMsg}<span class="ok small" role="status">{boardsMsg}</span>{/if}
      </div>

      {#if s.env_file}<p class="muted small">Archivo .env: {s.env_file} (lo que fija manda sobre esta pantalla).</p>{/if}
    </details>
  {/if}
    </div>
  </div>

</div>

<style>
  .page { display: flex; flex-direction: column; gap: 1rem; }
  .tune h2 { margin: 0; }
  .wtable { display: grid; grid-template-columns: 1fr auto; gap: 0.35rem 1rem; margin: 0; font-variant-numeric: tabular-nums; }
  .wtable dt { color: var(--md-on-surface-variant); }
  .wtable dd { margin: 0; text-align: right; }
  .wtable .up { color: var(--md-success); }
  .wtable .down { color: var(--md-tertiary); }
  /* Dos columnas en ventanas anchas: la IA a la izquierda; respaldo y avanzadas a la derecha. */
  .cols { display: grid; grid-template-columns: 1fr; gap: 1rem; align-items: start; }
  .col { display: flex; flex-direction: column; gap: 1rem; min-width: 0; }
  @container main (min-width: 880px) { .cols { grid-template-columns: minmax(0, 1.15fr) minmax(0, 1fr); } }
  section, .advanced { display: flex; flex-direction: column; gap: 0.6rem; }
  section h2, section p, .advanced p { margin: 0; }
  .advanced h3 { margin: 1.25rem 0 0.1rem; }
  .advanced > .row { margin-top: 0.5rem; }
  .advanced > summary { font-size: 1.375rem; }
  .choice { border: 0; padding: 0; margin: 0; display: flex; flex-direction: column; gap: 0.4rem; }
  .option { display: flex; flex-direction: row; gap: 0.85rem; align-items: flex-start; padding: 0.9rem 1rem; border: 1px solid var(--md-outline-variant); border-radius: var(--radius-m); cursor: pointer; color: var(--md-on-surface); transition: background-color 150ms var(--ease); }
  .option:hover { background: color-mix(in srgb, var(--md-on-surface) 6%, transparent); }
  .option.on { border-color: var(--md-primary); background: var(--md-secondary-container); color: var(--md-on-secondary-container); }
  .option:has(input:focus-visible) { outline: 3px solid var(--md-primary); outline-offset: 2px; }
  .option input { width: auto; margin-top: 0.2rem; }
  .sr-only { position: absolute; width: 1px; height: 1px; overflow: hidden; clip: rect(0 0 0 0); }
  .link.small { align-self: flex-start; }
</style>

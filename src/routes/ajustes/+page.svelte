<script lang="ts">
  import { jobsApi, settingsApi, type LlmSettings } from "$lib/api";

  let s = $state<LlmSettings | null>(null);
  let values = $state<Record<string, string>>({});
  let keys = $state<Record<string, string>>({ ANTHROPIC_API_KEY: "", FRJ_OPENAI_API_KEY: "", FRJ_ADZUNA_APP_KEY: "", FRJ_JOOBLE_KEY: "" });
  let error = $state("");
  let msg = $state("");
  let saving = $state(false);

  const EFFORTS = ["low", "medium", "high", "xhigh", "max"];
  const writer = $derived(values.FRJ_WRITER || "local");

  function show(v: LlmSettings) {
    s = v;
    // Sin elección guardada, se marca la que está en uso (p. ej. Claude si hay clave en .env).
    const inUse = v.writer?.startsWith("Claude") ? "anthropic" : v.writer?.startsWith("Compatible") ? "openai" : "local";
    values = { FRJ_WRITER: inUse, ...v.values };
    keys = { ANTHROPIC_API_KEY: "", FRJ_OPENAI_API_KEY: "", FRJ_ADZUNA_APP_KEY: "", FRJ_JOOBLE_KEY: "" };
  }

  let boards = $state("");
  let boardsMsg = $state("");
  let boardsError = $state("");

  async function saveBoards() {
    boardsMsg = "";
    boardsError = "";
    try {
      boards = await jobsApi.saveBoards(boards);
      boardsMsg = "Guardado. Se consultan al sincronizar en Vacantes.";
    } catch (e) {
      boardsError = String(e);
    }
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
    saving = true;
    error = "";
    msg = "";
    try {
      // Se envían todos los campos conocidos: uno vacío vuelve al valor por defecto.
      const all = Object.fromEntries(FIELDS.map((f) => [f, values[f] ?? ""]));
      show(await settingsApi.save(all, $state.snapshot(keys)));
      msg = "Guardado. Se usa desde ahora, sin reiniciar.";
    } catch (e) {
      error = String(e);
    } finally {
      saving = false;
    }
  }

  async function removeKey(k: string) {
    if (!confirm("¿Borrar la clave del llavero del sistema?")) return;
    try {
      show(await settingsApi.deleteKey(k));
      msg = "Clave borrada.";
    } catch (e) {
      error = String(e);
    }
  }

  const FIELDS = [
    "FRJ_WRITER", "FRJ_LOCAL_URL", "FRJ_LOCAL_MODEL", "FRJ_EMBED_MODEL",
    "FRJ_ANTHROPIC_MODEL", "FRJ_ANTHROPIC_EFFORT", "FRJ_OPENAI_URL", "FRJ_OPENAI_MODEL",
    "FRJ_ANTHROPIC_PRICE_IN", "FRJ_ANTHROPIC_PRICE_OUT", "FRJ_OPENAI_PRICE_IN", "FRJ_OPENAI_PRICE_OUT",
    "FRJ_ADZUNA_APP_ID", "FRJ_ADZUNA_COUNTRY", "FRJ_JOOBLE_DOMAIN",
  ];
  const ADZUNA_COUNTRIES = ["mx", "us", "ca", "br", "es", "gb", "de", "fr", "it", "nl", "be", "at", "ch", "pl", "au", "nz", "sg", "in", "za"];
  const locked = (k: string) => s?.overridden.includes(k) ?? false;

  init();
</script>

{#snippet field(key: string, label: string, placeholder: string)}
  <label>
    {label}
    <input bind:value={values[key]} {placeholder} disabled={locked(key)} />
    {#if locked(key)}<span class="muted small">Definido en el entorno o en .env: ese valor manda.</span>{/if}
  </label>
{/snippet}

{#snippet prices(p: string)}
  <div class="prices">
    {@render field(`${p}_PRICE_IN`, "Precio de entrada (USD por millón de tokens)", "")}
    {@render field(`${p}_PRICE_OUT`, "Precio de salida (USD por millón de tokens)", "")}
  </div>
  <p class="muted small">Cópialos de la página de precios de tu proveedor. Solo sirven para mostrarte el costo estimado antes de generar.</p>
{/snippet}

{#snippet apiKey(key: string, label: string)}
  <label>
    {label}
    <input
      type="password"
      autocomplete="off"
      bind:value={keys[key]}
      placeholder={s?.saved_keys.includes(key) ? "Guardada en el llavero (escribe otra para cambiarla)" : "Pega tu clave"}
      disabled={locked(key) || !!s?.keyring_error}
    />
    {#if locked(key)}<span class="muted small">Definida en el entorno o en .env: esa manda.</span>{/if}
  </label>
  {#if s?.saved_keys.includes(key)}
    <button class="link small" onclick={() => removeKey(key)}>Borrar clave guardada</button>
  {/if}
{/snippet}

<div class="page">
  <h1>Ajustes</h1>
  {#if error}<p class="danger" role="alert">{error}</p>{/if}

  {#if s}
    <section class="card status">
      <h2>En uso</h2>
      {#if s.error}
        <p class="danger">{s.error}</p>
      {:else}
        <p>IA local: <strong>{s.local}</strong></p>
        <p>Redactor de CV y cartas: <strong>{s.writer}</strong></p>
      {/if}
      {#if s.env_file}<p class="muted small">Archivo .env: {s.env_file}</p>{/if}
    </section>

    <section class="card">
      <h2>IA local</h2>
      <p class="muted small">Analiza vacantes e importa tu CV. Nada sale de tu equipo.</p>
      {@render field("FRJ_LOCAL_URL", "URL de Ollama", "http://localhost:11434/v1")}
      {@render field("FRJ_LOCAL_MODEL", "Modelo", "qwen3:8b")}
      {@render field("FRJ_EMBED_MODEL", "Modelo de embeddings", "bge-m3")}
    </section>

    <section class="card">
      <h2>Redactor de CV y cartas</h2>
      <p class="muted small">
        Con una API externa la redacción suele ser mejor y más rápida. Se envían la vacante y tu CV maestro
        <strong>sin</strong> nombre, correo, teléfono, ubicación ni enlaces.
      </p>
      <fieldset class="choice" disabled={locked("FRJ_WRITER")}>
        <legend class="sr-only">Proveedor</legend>
        {#each [["local", "Local (el mismo modelo de Ollama)"], ["anthropic", "Claude (Anthropic)"], ["openai", "Compatible con OpenAI (OpenAI, Gemini, Groq, OpenRouter…)"]] as [id, label]}
          <label class="inline"><input type="radio" value={id} bind:group={values.FRJ_WRITER} /> {label}</label>
        {/each}
        {#if locked("FRJ_WRITER")}<span class="muted small">Definido en el entorno o en .env: ese valor manda.</span>{/if}
      </fieldset>

      {#if writer === "anthropic"}
        {@render apiKey("ANTHROPIC_API_KEY", "Clave de API de Anthropic")}
        {@render field("FRJ_ANTHROPIC_MODEL", "Modelo", "claude-opus-5-5")}
        <label>
          Esfuerzo (más alto = mejor y más caro)
          <select bind:value={values.FRJ_ANTHROPIC_EFFORT} disabled={locked("FRJ_ANTHROPIC_EFFORT")}>
            <option value="">Por defecto (medium)</option>
            {#each EFFORTS as e}<option value={e}>{e}</option>{/each}
          </select>
        </label>
        {@render prices("FRJ_ANTHROPIC")}
      {:else if writer === "openai"}
        {@render field("FRJ_OPENAI_URL", "URL base", "https://api.openai.com/v1")}
        {@render apiKey("FRJ_OPENAI_API_KEY", "Clave de API")}
        {@render field("FRJ_OPENAI_MODEL", "Modelo", "")}
        {@render prices("FRJ_OPENAI")}
      {/if}

      {#if s.keyring_error}
        <p class="danger small">El llavero del sistema no está disponible ({s.keyring_error}); no se pueden guardar claves.</p>
      {:else}
        <p class="muted small">Las claves se guardan en el llavero del sistema (KWallet, GNOME Keyring o el de Windows), no en la base de datos.</p>
      {/if}
    </section>

    <section class="card">
      <h2>Fuentes de vacantes con clave</h2>
      <p class="muted small">
        Opcionales y gratuitas. Sin clave, estas fuentes no se consultan. Las claves van al llavero del sistema.
      </p>
      <h3 class="small">Adzuna</h3>
      <p class="muted small">Crea tu clave en developer.adzuna.com (uso personal permitido).</p>
      {@render field("FRJ_ADZUNA_APP_ID", "App ID", "")}
      {@render apiKey("FRJ_ADZUNA_APP_KEY", "App Key")}
      <label>
        País
        <select bind:value={values.FRJ_ADZUNA_COUNTRY} disabled={locked("FRJ_ADZUNA_COUNTRY")}>
          <option value="">Por defecto (mx)</option>
          {#each ADZUNA_COUNTRIES as c}<option value={c}>{c}</option>{/each}
        </select>
      </label>
      <h3 class="small">Jooble</h3>
      <p class="muted small">
        Pide la clave en el sitio de tu país (por ejemplo mx.jooble.org/api/about): cada clave solo da vacantes de su
        país. La gratuita tiene 500 consultas en total, así que se consulta una vez al día.
      </p>
      {@render field("FRJ_JOOBLE_DOMAIN", "Dominio", "mx.jooble.org")}
      {@render apiKey("FRJ_JOOBLE_KEY", "Clave")}
    </section>

    <div class="row">
      <button class="primary" onclick={save} disabled={saving}>{saving ? "Guardando…" : "Guardar"}</button>
      {#if msg}<span class="ok small" role="status">{msg}</span>{/if}
    </div>

    <section class="card">
      <h2>Empresas que sigues</h2>
      <p class="muted small">
        Vacantes de empresas concretas que publican en Greenhouse, Lever o Ashby. Pega la dirección de su página de
        empleos, una por línea (por ejemplo <code>https://jobs.lever.co/empresa</code>). Al sincronizar se traen las que
        coinciden con los puestos de tu perfil.
      </p>
      <label>
        Páginas de empleo
        <textarea rows="5" bind:value={boards} placeholder={"https://boards.greenhouse.io/empresa\nhttps://jobs.ashbyhq.com/empresa"}></textarea>
      </label>
      {#if boardsError}<p class="danger small" role="alert">{boardsError}</p>{/if}
      <div class="row">
        <button onclick={saveBoards}>Guardar empresas</button>
        {#if boardsMsg}<span class="ok small" role="status">{boardsMsg}</span>{/if}
      </div>
    </section>
  {:else if !error}
    <p class="muted">Cargando…</p>
  {/if}
</div>

<style>
  .page { display: flex; flex-direction: column; gap: 0.75rem; max-width: 720px; }
  section { display: flex; flex-direction: column; gap: 0.6rem; }
  section h2, section p { margin: 0; }
  section h3 { margin: 0.4rem 0 0; }
  .choice { border: 0; padding: 0; margin: 0; display: flex; flex-direction: column; gap: 0.3rem; }
  .sr-only { position: absolute; width: 1px; height: 1px; overflow: hidden; clip: rect(0 0 0 0); }
  .link.small { align-self: flex-start; }
  .prices { display: grid; grid-template-columns: repeat(auto-fit, minmax(220px, 1fr)); gap: 0.6rem; }
</style>

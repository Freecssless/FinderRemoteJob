<script lang="ts">
  import { settingsApi, type LlmSettings } from "$lib/api";

  let s = $state<LlmSettings | null>(null);
  let values = $state<Record<string, string>>({});
  let keys = $state<Record<string, string>>({ ANTHROPIC_API_KEY: "", FRJ_OPENAI_API_KEY: "" });
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
    keys = { ANTHROPIC_API_KEY: "", FRJ_OPENAI_API_KEY: "" };
  }

  async function init() {
    try {
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
  ];
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
      {:else if writer === "openai"}
        {@render field("FRJ_OPENAI_URL", "URL base", "https://api.openai.com/v1")}
        {@render apiKey("FRJ_OPENAI_API_KEY", "Clave de API")}
        {@render field("FRJ_OPENAI_MODEL", "Modelo", "")}
      {/if}

      {#if s.keyring_error}
        <p class="danger small">El llavero del sistema no está disponible ({s.keyring_error}); no se pueden guardar claves.</p>
      {:else}
        <p class="muted small">Las claves se guardan en el llavero del sistema (KWallet, GNOME Keyring o el de Windows), no en la base de datos.</p>
      {/if}
    </section>

    <div class="row">
      <button class="primary" onclick={save} disabled={saving}>{saving ? "Guardando…" : "Guardar"}</button>
      {#if msg}<span class="ok small" role="status">{msg}</span>{/if}
    </div>
  {:else if !error}
    <p class="muted">Cargando…</p>
  {/if}
</div>

<style>
  .page { display: flex; flex-direction: column; gap: 0.75rem; max-width: 720px; }
  section { display: flex; flex-direction: column; gap: 0.6rem; }
  section h2, section p { margin: 0; }
  .choice { border: 0; padding: 0; margin: 0; display: flex; flex-direction: column; gap: 0.3rem; }
  .sr-only { position: absolute; width: 1px; height: 1px; overflow: hidden; clip: rect(0 0 0 0); }
  .link.small { align-self: flex-start; }
</style>

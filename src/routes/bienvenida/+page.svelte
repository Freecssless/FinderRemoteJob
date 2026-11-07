<script lang="ts">
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { goto } from "$app/navigation";
  import { api, jobsApi, rememberProfile, REMOTE_LABELS, type CvMaster, type Profile, type RemotePref } from "$lib/api";

  const STEPS = ["Tu perfil", "Tu CV", "Qué puestos buscas", "Buscar vacantes"];
  let step = $state(0);
  let busy = $state(false);
  let error = $state("");

  // Paso 1
  let name = $state("");
  let remote = $state<RemotePref>("SoloRemoto");
  let profile = $state<Profile | null>(null);

  // Paso 2
  let ollama = $state<{ running: boolean; model: string; installed: boolean } | null>(null);
  let cv = $state<CvMaster | null>(null);
  let sourceText = $state<string | null>(null);
  let warnings = $state<string[]>([]);

  // Paso 3
  let titles = $state("");

  // Paso 4
  let found = $state<number | null>(null);

  async function run(f: () => Promise<void>) {
    busy = true;
    error = "";
    try {
      await f();
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }

  const createProfile = () =>
    run(async () => {
      profile = await api.createProfile({ name: name.trim(), target_titles: [], remote_pref: remote, locations: [] });
      rememberProfile(profile.id);
      step = 1;
      await checkOllama();
    });

  async function checkOllama() {
    ollama = null;
    ollama = await api.ollamaStatus().catch(() => ({ running: false, model: "qwen3:8b", installed: false }));
  }

  async function importFile(e: Event & { currentTarget: HTMLInputElement }) {
    const file = e.currentTarget.files?.[0];
    e.currentTarget.value = "";
    if (!file) return;
    await run(async () => {
      const preview = await api.importCv(file);
      cv = preview.cv;
      warnings = preview.warnings;
      sourceText = preview.source_text;
    });
  }

  const saveCv = () =>
    run(async () => {
      if (!profile || !cv) return;
      await api.saveCv(profile.id, $state.snapshot(cv), sourceText);
      titles ||= cv.basics.headline;
      step = 2;
    });

  const saveTitles = () =>
    run(async () => {
      if (!profile) return;
      const target_titles = titles.split(",").map((t) => t.trim()).filter(Boolean);
      await api.updateProfile({ ...profile, target_titles });
      profile = { ...profile, target_titles };
      step = 3;
    });

  const search = () =>
    run(async () => {
      if (!profile) return;
      const report = await jobsApi.sync(profile.id);
      found = report.reduce((n, r) => n + r.kept, 0);
    });
</script>

<div class="wizard">
  <h1>Bienvenido a FinderRemoteJob</h1>
  <p class="muted">
    Te ayudamos a encontrar vacantes, adaptar tu CV a cada una y llevar el seguimiento.
    Empecemos en cuatro pasos.
  </p>

  <ol class="steps">
    {#each STEPS as s, i}
      <li class:current={i === step} class:done={i < step}>{s}</li>
    {/each}
  </ol>

  {#if error}<p class="danger" role="alert">{error}</p>{/if}

  <div class="card">
    {#if step === 0}
      <h2>Tu perfil</h2>
      <p class="muted small">
        Un perfil guarda tu CV y las vacantes que te interesan. Puedes crear más después,
        por ejemplo uno por tipo de trabajo.
      </p>
      <label>¿Cómo quieres llamarlo?<input bind:value={name} placeholder="Mi búsqueda" /></label>
      <label>¿Qué tipo de trabajo buscas?
        <select bind:value={remote}>
          {#each Object.entries(REMOTE_LABELS) as [value, label]}<option {value}>{label}</option>{/each}
        </select>
      </label>
      <div class="row">
        <button class="primary" onclick={createProfile} disabled={busy || !name.trim()}>Siguiente</button>
      </div>
    {:else if step === 1}
      <h2>Tu CV</h2>
      {#if !ollama}
        <p class="muted">Comprobando la IA de tu computadora…</p>
      {:else if !ollama.running || !ollama.installed}
        <p>
          Para leer tu CV usamos una IA que corre <strong>en tu computadora</strong>: tus datos no
          salen de ella. Es gratis, pero hay que instalarla una vez:
        </p>
        <ol>
          {#if !ollama.running}
            <li>
              Descarga e instala Ollama desde
              <button class="link" onclick={() => openUrl("https://ollama.com/download")}>ollama.com/download</button>
              y ábrelo.
            </li>
          {/if}
          <li>
            Abre una terminal (en Windows: «Símbolo del sistema») y escribe:
            <code>ollama pull {ollama.model}</code>
            <span class="muted small">Descarga unos 5 GB; puede tardar un rato.</span>
          </li>
          <li>Vuelve aquí y pulsa «Comprobar de nuevo».</li>
        </ol>
        <div class="row">
          <button class="primary" onclick={checkOllama}>Comprobar de nuevo</button>
          <button onclick={() => (step = 2)}>Saltar: escribiré mi CV a mano en Perfiles</button>
        </div>
      {:else if !cv}
        <p>Elige tu CV en PDF o Word. La IA lo leerá y lo pasará a un formato que la app entiende.</p>
        <label class="filebtn">
          <input type="file" accept=".pdf,.docx" onchange={importFile} disabled={busy} />
          <span>{busy ? "Leyendo tu CV…" : "Elegir archivo"}</span>
        </label>
        {#if busy}<p class="muted small">Puede tardar varios minutos. No cierres la app.</p>{/if}
        <div class="row">
          <button onclick={() => (step = 2)} disabled={busy}>Saltar: lo haré después en Perfiles</button>
        </div>
      {:else}
        <p class="ok">
          Leímos tu CV: {cv.experience.length} experiencias y {cv.skills.length} habilidades.
        </p>
        {#if warnings.length}
          <div class="notice" role="status">
            <strong>Revisa estos datos</strong> (no aparecen tal cual en tu archivo). Podrás corregirlos en Perfiles:
            <ul>{#each warnings as w}<li>{w}</li>{/each}</ul>
          </div>
        {/if}
        <div class="row">
          <button class="primary" onclick={saveCv} disabled={busy}>Guardar y seguir</button>
          <button onclick={() => (cv = null)} disabled={busy}>Elegir otro archivo</button>
        </div>
      {/if}
    {:else if step === 2}
      <h2>¿Qué puestos buscas?</h2>
      <p class="muted small">
        Escribe los nombres de los puestos como aparecen en los anuncios, separados por comas.
        Con ellos buscamos vacantes.
      </p>
      <label>Puestos<input bind:value={titles} placeholder="Editor de video, Motion designer" /></label>
      <div class="row">
        <button class="primary" onclick={saveTitles} disabled={busy || !titles.trim()}>Siguiente</button>
      </div>
    {:else}
      <h2>Buscar vacantes</h2>
      {#if found === null}
        <p>Buscaremos en varios portales de empleo remoto las vacantes que encajan con tus puestos.</p>
        <div class="row">
          <button class="primary" onclick={search} disabled={busy}>{busy ? "Buscando…" : "Buscar vacantes"}</button>
        </div>
      {:else}
        {#if found}
          <p class="ok">¡Listo! Encontramos {found} vacantes para ti.</p>
        {:else}
          <p>No encontramos vacantes con esos puestos. En Perfiles puedes probar con otros nombres más comunes (también en inglés).</p>
        {/if}
        <p class="muted small">
          En Vacantes verás cuánto encaja cada una con tu CV. Desde ahí puedes adaptar tu CV a la que
          te guste y seguirla en el Tablero.
        </p>
        <div class="row">
          <button class="primary" onclick={() => goto("/vacantes")}>Ver vacantes</button>
        </div>
      {/if}
    {/if}
  </div>
</div>

<style>
  .wizard { max-width: 640px; margin: 0 auto; }
  .steps { display: flex; gap: 0.5rem; padding: 0; list-style: none; counter-reset: s; margin: 1rem 0; }
  .steps li { flex: 1; counter-increment: s; font-size: 0.9375rem; color: var(--muted); border-top: 3px solid var(--border); padding-top: 0.4rem; }
  .steps li::before { content: counter(s) ". "; }
  .steps li.done { border-color: var(--accent); }
  .steps li.current { border-color: var(--accent); color: var(--text); font-weight: 600; }
  .card { display: flex; flex-direction: column; gap: 0.75rem; }
  .card h2 { margin: 0; }
  code { display: block; margin: 0.3rem 0; padding: 0.3rem 0.5rem; background: var(--surface); border: 1px solid var(--border); border-radius: 4px; user-select: all; }
  .link { border: none; background: none; padding: 0; color: var(--accent); text-decoration: underline; cursor: pointer; }
</style>

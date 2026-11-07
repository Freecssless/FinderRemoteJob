<script lang="ts">
  import { chooseProfile, refreshProfiles } from "$lib/profile.svelte";
  import PageHead from "$lib/components/PageHead.svelte";
  import Select from "$lib/components/Select.svelte";
  import { api, emptyCv, rememberedProfile, REMOTE_LABELS, type CvMaster, type Profile, type RemotePref } from "$lib/api";
  import CvEditor from "$lib/components/CvEditor.svelte";
  import Icon from "$lib/components/Icon.svelte";

  let profiles = $state<Profile[]>([]);
  let selectedId = $state<string | null>(null);
  let creating = $state(false);

  // Formulario del perfil (comas para listas).
  let form = $state({ name: "", titles: "", remote: "SoloRemoto" as RemotePref, locations: "" });

  // CV en edición y lo último guardado, para detectar cambios sin guardar.
  let cv = $state<CvMaster | null>(null);
  let savedSnapshot = $state("");
  let sourceText = $state<string | null>(null);
  let warnings = $state<string[]>([]);

  let busy = $state<"" | "import" | "save" | "profile">("");
  let error = $state("");
  let message = $state("");
  let confirmDelete = $state(false);
  let pendingSwitch = $state<string | "new" | null>(null);

  const selected = $derived(profiles.find((p) => p.id === selectedId) ?? null);
  const profileDirty = $derived(
    selected !== null &&
      (form.name !== selected.name ||
        form.titles !== selected.target_titles.join(", ") ||
        form.remote !== selected.remote_pref ||
        form.locations !== selected.locations.join(", ")),
  );
  const cvDirty = $derived(cv !== null && JSON.stringify(cv) !== savedSnapshot);
  const dirty = $derived(profileDirty || cvDirty);

  const splitList = (s: string) => s.split(",").map((x) => x.trim()).filter(Boolean);

  function fail(e: unknown) {
    error = String(e);
    message = "";
  }
  function done(msg: string) {
    message = msg;
    error = "";
  }

  async function load() {
    try {
      profiles = await api.listProfiles();
      if (!selectedId && profiles.length) {
        const saved = rememberedProfile();
        await open(profiles.some((p) => p.id === saved) ? saved! : profiles[0].id);
      }
      if (!profiles.length) startNew();
    } catch (e) {
      fail(e);
    }
  }

  function requestSwitch(target: string | "new") {
    if (dirty) {
      pendingSwitch = target;
      return;
    }
    target === "new" ? startNew() : open(target);
  }

  function discardAndSwitch() {
    const t = pendingSwitch;
    pendingSwitch = null;
    if (t) t === "new" ? startNew() : open(t);
  }

  function startNew() {
    creating = true;
    selectedId = null;
    cv = null;
    warnings = [];
    sourceText = null;
    confirmDelete = false;
    form = { name: "", titles: "", remote: "SoloRemoto", locations: "" };
  }

  async function open(id: string) {
    const p = profiles.find((x) => x.id === id);
    if (!p) return;
    creating = false;
    selectedId = id;
    chooseProfile(id);
    confirmDelete = false;
    warnings = [];
    sourceText = null;
    error = "";
    message = "";
    form = {
      name: p.name,
      titles: p.target_titles.join(", "),
      remote: p.remote_pref,
      locations: p.locations.join(", "),
    };
    try {
      const stored = (await api.getCv(id)) ?? emptyCv();
      cv = stored;
      savedSnapshot = JSON.stringify(stored);
    } catch (e) {
      fail(e);
    }
  }

  async function saveProfile() {
    busy = "profile";
    try {
      const fields = {
        name: form.name.trim(),
        target_titles: splitList(form.titles),
        remote_pref: form.remote,
        locations: splitList(form.locations),
      };
      if (creating) {
        const p = await api.createProfile(fields);
        profiles = [...profiles, p];
        await open(p.id);
        done("Perfil creado. Ahora importa o escribe tu CV maestro.");
        void refreshProfiles();
      } else if (selected) {
        const p = { ...selected, ...fields };
        await api.updateProfile(p);
        profiles = profiles.map((x) => (x.id === p.id ? p : x));
        form.titles = p.target_titles.join(", ");
        form.locations = p.locations.join(", ");
        done("Perfil guardado.");
        void refreshProfiles();
      }
    } catch (e) {
      fail(e);
    } finally {
      busy = "";
    }
  }

  async function deleteProfile() {
    if (!selected) return;
    busy = "profile";
    try {
      await api.deleteProfile(selected.id);
      profiles = profiles.filter((p) => p.id !== selected.id);
      selectedId = null;
      cv = null;
      profiles.length ? await open(profiles[0].id) : startNew();
      done("Perfil borrado.");
      void refreshProfiles();
    } catch (e) {
      fail(e);
    } finally {
      busy = "";
      confirmDelete = false;
    }
  }

  async function importFile(e: Event & { currentTarget: HTMLInputElement }) {
    const file = e.currentTarget.files?.[0];
    e.currentTarget.value = "";
    if (!file || !cv) return;
    busy = "import";
    error = "";
    message = "";
    try {
      const preview = await api.importCv(file);
      cv = preview.cv;
      warnings = preview.warnings;
      sourceText = preview.source_text;
      done("CV leído. Revisa cada campo y pulsa «Guardar CV»: todavía no se ha guardado nada.");
    } catch (e) {
      fail(e);
    } finally {
      busy = "";
    }
  }

  async function saveCv() {
    if (!selectedId || !cv) return;
    busy = "save";
    try {
      await api.saveCv(selectedId, $state.snapshot(cv), sourceText);
      savedSnapshot = JSON.stringify(cv);
      sourceText = null;
      warnings = [];
      done("CV maestro guardado.");
    } catch (e) {
      fail(e);
    } finally {
      busy = "";
    }
  }

  function discardCv() {
    cv = JSON.parse(savedSnapshot);
    warnings = [];
    sourceText = null;
  }

  load();
</script>

<div class="page">
  <PageHead icon="person" title="Perfiles" subtitle="Tu CV y lo que buscas">
    <a class="button tonal" href="/bienvenida"><Icon name="person_add" size={20} /> Crear paso a paso</a>
    <button onclick={() => requestSwitch("new")} disabled={creating}><Icon name="add" /> Perfil vacío</button>
  </PageHead>

  {#if profiles.length}
    <div class="tabs" role="tablist" aria-label="Tus perfiles">
      {#each profiles as p (p.id)}
        <button role="tab" class="tab" aria-selected={p.id === selectedId} onclick={() => requestSwitch(p.id)}>
          <span class="avatar" aria-hidden="true">{p.name.trim().charAt(0).toUpperCase() || "?"}</span>
          <span class="tabtext"><strong>{p.name}</strong><span>{REMOTE_LABELS[p.remote_pref]}</span></span>
        </button>
      {/each}
      {#if creating}
        <span class="tab" aria-selected="true"><span class="avatar" aria-hidden="true">+</span><span class="tabtext"><strong>Nuevo perfil</strong></span></span>
      {/if}
    </div>
  {/if}

  <section class="detail">
    {#if pendingSwitch}
      <div class="notice row" role="alert">
        <span>Tienes cambios sin guardar.</span>
        <button onclick={discardAndSwitch}>Descartar cambios</button>
        <button onclick={() => (pendingSwitch = null)}>Seguir editando</button>
      </div>
    {/if}
    {#if error}<p class="danger" role="alert">{error}</p>{/if}
    {#if message}<p class="ok" role="status">{message}</p>{/if}

    {#if creating || selected}
      <div class="card">
        <h2 style="margin-top:0">{creating ? "Nuevo perfil" : "Perfil"}</h2>
        <div class="grid">
          <label>Nombre del perfil<input bind:value={form.name} placeholder="Editor de video" /></label>
          <label>Puestos buscados (separados por comas)
            <input bind:value={form.titles} placeholder="Editor de video, Motion designer" />
          </label>
          <label>Modalidad
            <Select bind:value={form.remote} label="Modalidad"
              options={Object.entries(REMOTE_LABELS).map(([value, label]) => ({ value: value as RemotePref, label }))} />
          </label>
          <label>Ubicaciones aceptadas (separadas por comas)
            <input bind:value={form.locations} placeholder="México, LATAM" />
          </label>
        </div>
        <div class="row" style="margin-top:0.75rem">
          <button class="primary" onclick={saveProfile} disabled={busy !== "" || !form.name.trim() || (!creating && !profileDirty)}>
            {creating ? "Crear perfil" : "Guardar perfil"}
          </button>
          {#if creating && profiles.length}
            <button onclick={() => open(profiles[0].id)}>Cancelar</button>
          {/if}
          {#if selected}
            {#if confirmDelete}
              <span class="danger small">Se borrarán su CV maestro, versiones y postulaciones.</span>
              <button class="danger" onclick={deleteProfile} disabled={busy !== ""}>Sí, borrar todo</button>
              <button onclick={() => (confirmDelete = false)}>Cancelar</button>
            {:else}
              <button class="danger" onclick={() => (confirmDelete = true)}>Borrar perfil</button>
            {/if}
          {/if}
        </div>
      </div>
    {/if}

    {#if selected && cv}
      <div class="row cvbar">
        <h2>CV maestro</h2>
        <label class="filebtn">
          <input type="file" accept=".pdf,.docx" onchange={importFile} disabled={busy !== ""} />
          <span>{busy === "import" ? "Leyendo con IA local…" : "Importar PDF o DOCX"}</span>
        </label>
        <button class="primary" onclick={saveCv} disabled={busy !== "" || !cvDirty}>
          {busy === "save" ? "Guardando…" : "Guardar CV"}
        </button>
        {#if cvDirty}<button onclick={discardCv} disabled={busy !== ""}>Descartar cambios</button>{/if}
      </div>
      {#if busy === "import"}
        <p class="muted small">Con un modelo local en CPU esto puede tardar varios minutos.</p>
      {/if}
      {#if warnings.length}
        <div class="notice" role="status">
          <strong>Revisa estos datos antes de guardar</strong>
          <span class="muted small">(no aparecen tal cual en tu archivo original):</span>
          <ul>{#each warnings as w}<li>{w}</li>{/each}</ul>
        </div>
      {/if}
      <CvEditor bind:cv />
      <div class="row" style="margin-top:1.5rem">
        <button class="primary" onclick={saveCv} disabled={busy !== "" || !cvDirty}>Guardar CV</button>
      </div>
    {/if}
  </section>
</div>

<style>
  .page { display: flex; flex-direction: column; gap: 1rem; }
  /* Pestañas de perfil: una ficha por perfil, con su inicial. */
  .tabs { display: flex; gap: 0.5rem; flex-wrap: wrap; }
  .tab {
    justify-content: flex-start; gap: 0.6rem; min-height: 56px; padding: 0.4rem 1rem 0.4rem 0.45rem;
    border-radius: var(--radius-full); border: 1px solid var(--md-outline-variant);
    background: var(--md-surface-container-low); color: var(--md-on-surface); font-weight: 400; text-align: left;
  }
  .tab[aria-selected="true"] { background: var(--md-secondary-container); border-color: transparent; color: var(--md-on-secondary-container); }
  .avatar {
    display: grid; place-items: center; flex: none; width: 40px; height: 40px; border-radius: 50%;
    background: var(--md-primary); color: var(--md-on-primary); font-weight: 750; font-size: 1.0625rem;
  }
  .tabtext { display: flex; flex-direction: column; line-height: 1.2; }
  .tabtext span { font-size: 0.875rem; color: var(--md-on-surface-variant); }
  .tab[aria-selected="true"] .tabtext span { color: inherit; }
  .detail { min-width: 0; display: flex; flex-direction: column; gap: 1rem; }
  .detail > .card .grid { grid-template-columns: repeat(2, minmax(0, 1fr)); }
  @container main (max-width: 640px) { .detail > .card .grid { grid-template-columns: 1fr; } }
  /* Barra del CV fija arriba al desplazarse: «Guardar CV» siempre a mano. */
  /* Misma barra flotante que «Corregir el CV» del Optimizador: tonal, redondeada, sobre el contenido. */
  .cvbar {
    position: sticky; top: 0.5rem; z-index: 5; margin-top: 0.75rem;
    padding: 0.7rem 0.8rem 0.7rem 1.5rem; border-radius: var(--radius-l);
    background: color-mix(in srgb, var(--md-surface-container-high) 90%, transparent);
    backdrop-filter: blur(10px);
    box-shadow: 0 6px 12px -10px rgb(var(--md-shadow) / 0.5);
  }
  .cvbar h2 { margin: 0 auto 0 0; }
</style>

<script lang="ts">
  import Icon from "$lib/components/Icon.svelte";
  import type { CvMaster } from "$lib/api";

  let { cv = $bindable() }: { cv: CvMaster } = $props();

  // Ids estables y únicos ("exp_03", "b_12"): el guardián del optimizador cita estos ids.
  function nextId(prefix: string, ids: string[]): string {
    const n = ids.reduce((max, id) => {
      const m = id.match(new RegExp(`^${prefix}_(\\d+)$`));
      return m ? Math.max(max, Number(m[1])) : max;
    }, 0);
    return `${prefix}_${String(n + 1).padStart(2, "0")}`;
  }
  const allBulletIds = () => cv.experience.flatMap((e) => e.bullets.map((b) => b.id));

  const addExperience = () =>
    cv.experience.push({
      id: nextId("exp", cv.experience.map((e) => e.id)),
      company: "", title: "", start: "", end: null, bullets: [],
    });
  const addBullet = (i: number) =>
    cv.experience[i].bullets.push({ id: nextId("b", allBulletIds()), text: "", metrics: [] });
  // Habilidades como fichas: se agregan escribiendo y se editan de una en una.
  let newSkill = $state("");
  let editingSkill = $state<string | null>(null);
  function addSkill(e: SubmitEvent) {
    e.preventDefault();
    for (const name of newSkill.split(",").map((s) => s.trim()).filter(Boolean)) {
      if (cv.skills.some((s) => s.name.toLowerCase() === name.toLowerCase())) continue;
      cv.skills.push({ id: nextId("sk", cv.skills.map((s) => s.id)), name, level: null, years: null });
    }
    newSkill = "";
  }
  function removeSkill(i: number) {
    if (cv.skills[i].id === editingSkill) editingSkill = null;
    cv.skills.splice(i, 1);
  }
  const addEducation = () =>
    cv.education.push({
      id: nextId("edu", cv.education.map((e) => e.id)),
      institution: "", degree: "", start: null, end: null,
    });
  const addCert = () =>
    cv.certifications.push({
      id: nextId("cert", cv.certifications.map((c) => c.id)),
      name: "", issuer: null, date: null,
    });
  const addLanguage = () => cv.languages.push({ name: "", level: "" });

  // Campos opcionales: texto vacío se guarda como null.
  const opt = (v: string) => (v.trim() === "" ? null : v);
</script>

<section class="card">
  <h3>Datos básicos</h3>
  <p class="muted small">No se envían a la IA.</p>
  <div class="grid">
    <label>Nombre completo<input bind:value={cv.basics.name} /></label>
    <label>Titular<input bind:value={cv.basics.headline} placeholder="Editor de video" /></label>
    <label>Correo<input type="email" bind:value={cv.basics.email} /></label>
    <label>Teléfono<input type="tel" bind:value={cv.basics.phone} /></label>
    <label>Ubicación<input bind:value={cv.basics.location} /></label>
  </div>
  <label style="margin-top:0.75rem">Enlaces (uno por línea)
    <textarea
      rows="2"
      value={cv.basics.links.join("\n")}
      oninput={(e) => (cv.basics.links = e.currentTarget.value.split("\n").map((l) => l.trim()).filter(Boolean))}
    ></textarea>
  </label>
</section>

<h2>Experiencia</h2>
{#each cv.experience as exp, i (exp.id)}
  <section class="card item">
    <div class="grid">
      <label>Puesto<input bind:value={exp.title} /></label>
      <label>Empresa<input bind:value={exp.company} /></label>
      <label>Inicio (AAAA-MM)<input bind:value={exp.start} placeholder="2022-01" /></label>
      <label>Fin (AAAA-MM)
        <input
          value={exp.end ?? ""}
          disabled={exp.end === null}
          placeholder={exp.end === null ? "Actual" : "2024-06"}
          oninput={(e) => (exp.end = e.currentTarget.value)}
        />
      </label>
    </div>
    <label class="inline small">
      <input type="checkbox" checked={exp.end === null} onchange={(e) => (exp.end = e.currentTarget.checked ? null : "")} />
      Trabajo actual
    </label>
    <h3 style="margin-top:0.75rem">Logros</h3>
    {#each exp.bullets as b, j (b.id)}
      <div class="row nowrap">
        <textarea rows="2" bind:value={b.text} aria-label="Logro {j + 1}"></textarea>
        <button class="link danger" onclick={() => exp.bullets.splice(j, 1)} aria-label="Quitar logro">Quitar</button>
      </div>
    {/each}
    <div class="row">
      <button onclick={() => addBullet(i)}><Icon name="add" /> Logro</button>
      <button class="danger" onclick={() => cv.experience.splice(i, 1)}>Quitar experiencia</button>
    </div>
  </section>
{/each}
<button onclick={addExperience}><Icon name="add" /> Experiencia</button>

<h2>Habilidades <span class="count">{cv.skills.length}</span></h2>
<section class="card">
  {#if cv.skills.length}
    <ul class="skills" aria-label="Habilidades">
      {#each cv.skills as s, i (s.id)}
        <li class="skill" class:sel={editingSkill === s.id}>
          <button class="skillname" aria-expanded={editingSkill === s.id}
            onclick={() => (editingSkill = editingSkill === s.id ? null : s.id)}
            title="Editar nivel y años">
            {s.name || "Sin nombre"}{#if s.years}<span class="yrs">{s.years} {s.years === 1 ? "año" : "años"}</span>{/if}
          </button>
          <button class="skillx" aria-label="Quitar {s.name}" onclick={() => removeSkill(i)}><Icon name="close" size={18} /></button>
        </li>
      {/each}
    </ul>
  {/if}
  {#each cv.skills as s (s.id)}
    {#if editingSkill === s.id}
      <div class="skilledit">
        <label>Habilidad<input bind:value={s.name} /></label>
        <label>Nivel<input value={s.level ?? ""} oninput={(e) => (s.level = opt(e.currentTarget.value))} placeholder="Opcional" /></label>
        <label>Años
          <input type="number" min="0" step="0.5" value={s.years ?? ""} placeholder="Opcional"
            oninput={(e) => (s.years = e.currentTarget.value === "" ? null : Number(e.currentTarget.value))} />
        </label>
        <button class="tonal" onclick={() => (editingSkill = null)}><Icon name="check" size={20} /> Listo</button>
      </div>
    {/if}
  {/each}
  <form class="addskill" onsubmit={addSkill}>
    <input bind:value={newSkill} placeholder="Escribe una habilidad y pulsa Enter (o varias separadas por comas)" aria-label="Nueva habilidad" />
    <button type="submit" disabled={!newSkill.trim()}><Icon name="add" /> Agregar</button>
  </form>
</section>

<h2>Educación</h2>
{#each cv.education as ed, i (ed.id)}
  <section class="card item">
    <div class="grid">
      <label>Título<input bind:value={ed.degree} /></label>
      <label>Institución<input bind:value={ed.institution} /></label>
      <label>Inicio<input value={ed.start ?? ""} oninput={(e) => (ed.start = opt(e.currentTarget.value))} /></label>
      <label>Fin<input value={ed.end ?? ""} oninput={(e) => (ed.end = opt(e.currentTarget.value))} /></label>
    </div>
    <button class="link danger" onclick={() => cv.education.splice(i, 1)}>Quitar</button>
  </section>
{/each}
<button onclick={addEducation}><Icon name="add" /> Educación</button>

<h2>Certificaciones</h2>
<section class="card">
  {#each cv.certifications as c, i (c.id)}
    <div class="row nowrap">
      <input bind:value={c.name} placeholder="Certificación" aria-label="Certificación" />
      <input value={c.issuer ?? ""} oninput={(e) => (c.issuer = opt(e.currentTarget.value))} placeholder="Emisor" aria-label="Emisor" />
      <input value={c.date ?? ""} oninput={(e) => (c.date = opt(e.currentTarget.value))} placeholder="Fecha" aria-label="Fecha" class="narrow" />
      <button class="link danger" onclick={() => cv.certifications.splice(i, 1)}>Quitar</button>
    </div>
  {/each}
  <button onclick={addCert}><Icon name="add" /> Certificación</button>
</section>

<h2>Idiomas</h2>
<section class="card">
  {#each cv.languages as l, i}
    <div class="row nowrap">
      <input bind:value={l.name} placeholder="Idioma" aria-label="Idioma" />
      <input bind:value={l.level} placeholder="Nivel (B2, nativo…)" aria-label="Nivel del idioma" />
      <button class="link danger" onclick={() => cv.languages.splice(i, 1)}>Quitar</button>
    </div>
  {/each}
  <button onclick={addLanguage}><Icon name="add" /> Idioma</button>
</section>

<style>
  .count {
    display: inline-grid; place-items: center; min-width: 1.75rem; padding: 0 0.45rem; margin-left: 0.4rem;
    border-radius: var(--radius-full); background: var(--md-surface-container-highest);
    color: var(--md-on-surface-variant); font-size: 0.875rem; font-weight: 700; vertical-align: middle;
  }
  .skills { display: flex; flex-wrap: wrap; gap: 0.5rem; list-style: none; padding: 0; margin: 0 0 1rem; }
  /* Ficha de entrada (M3): nombre que abre el editor y una X para quitar. */
  .skill {
    display: inline-flex; align-items: center; height: 36px; border-radius: var(--radius-s);
    border: 1px solid var(--md-outline-variant); background: var(--md-surface-container-lowest);
    transition: background-color 150ms var(--ease), border-color 150ms var(--ease);
  }
  .skill.sel { background: var(--md-secondary-container); border-color: transparent; }
  .skill button { min-height: 0; height: 100%; border: 0; border-radius: inherit; background: transparent; font-weight: 600; font-size: 0.9375rem; color: var(--md-on-surface); }
  .skill.sel button { color: var(--md-on-secondary-container); }
  .skillname { padding: 0 0.4rem 0 0.75rem; gap: 0.4rem; }
  .yrs { font-weight: 400; color: var(--md-on-surface-variant); }
  .skillx { width: 32px; padding: 0; color: var(--md-on-surface-variant) !important; }
  .skilledit {
    display: grid; grid-template-columns: 2fr 1.5fr 1fr auto; align-items: end; gap: 0.75rem;
    padding: 1rem; margin-bottom: 1rem; border-radius: var(--radius-m); background: var(--md-surface-container);
    animation: rise-in 220ms var(--ease-decel) both;
  }
  @container main (max-width: 700px) { .skilledit { grid-template-columns: 1fr 1fr; } }
  .addskill { display: flex; gap: 0.75rem; }
  .item { margin-bottom: 0.75rem; }
  .nowrap { flex-wrap: nowrap; margin-bottom: 0.5rem; }
  .narrow { max-width: 7rem; }
</style>

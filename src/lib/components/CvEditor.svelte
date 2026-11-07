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
  // Educación: fichas con el título; al agregar se abre su editor para institución y fechas.
  let newEdu = $state("");
  let editingEdu = $state<string | null>(null);
  function addEducation(e: SubmitEvent) {
    e.preventDefault();
    const degree = newEdu.trim();
    if (!degree) return;
    const id = nextId("edu", cv.education.map((x) => x.id));
    cv.education.push({ id, institution: "", degree, start: null, end: null });
    newEdu = "";
    editingEdu = id;
  }
  // Certificaciones e idiomas: mismas fichas que las habilidades.
  let newCert = $state("");
  let editingCert = $state<string | null>(null);
  function addCert(e: SubmitEvent) {
    e.preventDefault();
    const name = newCert.trim();
    if (!name) return;
    const id = nextId("cert", cv.certifications.map((c) => c.id));
    cv.certifications.push({ id, name, issuer: null, date: null });
    newCert = "";
    editingCert = id; // para completar emisor y fecha
  }
  let newLang = $state({ name: "", level: "" });
  let editingLang = $state<number | null>(null);
  function addLanguage(e: SubmitEvent) {
    e.preventDefault();
    const name = newLang.name.trim();
    if (!name || cv.languages.some((l) => l.name.toLowerCase() === name.toLowerCase())) return;
    cv.languages.push({ name, level: newLang.level.trim() });
    newLang = { name: "", level: "" };
  }

  // Campos opcionales: texto vacío se guarda como null.
  const opt = (v: string) => (v.trim() === "" ? null : v);
</script>

<div class="cvgrid">
<section class="blk card basics">
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

<section class="blk wide">
<h2>Experiencia <span class="count">{cv.experience.length}</span></h2>
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
</section>

<section class="blk skillsblk">
<h2>Habilidades <span class="count">{cv.skills.length}</span></h2>
<section class="card">
  {#if cv.skills.length}
    <ul class="chips" aria-label="Habilidades">
      {#each cv.skills as s, i (s.id)}
        <li class="chip" class:sel={editingSkill === s.id}>
          <button class="chipname" aria-expanded={editingSkill === s.id}
            onclick={() => (editingSkill = editingSkill === s.id ? null : s.id)}
            title="Editar nivel y años">
            {s.name || "Sin nombre"}{#if s.years}<span class="yrs">{s.years} {s.years === 1 ? "año" : "años"}</span>{/if}
          </button>
          <button class="chipx" aria-label="Quitar {s.name}" onclick={() => removeSkill(i)}><Icon name="close" size={18} /></button>
        </li>
      {/each}
    </ul>
  {/if}
  {#each cv.skills as s (s.id)}
    {#if editingSkill === s.id}
      <div class="chipedit">
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
  <form class="addrow" onsubmit={addSkill}>
    <input bind:value={newSkill} placeholder="Escribe una habilidad y pulsa Enter (o varias separadas por comas)" aria-label="Nueva habilidad" />
    <button type="submit" disabled={!newSkill.trim()}><Icon name="add" /> Agregar</button>
  </form>
</section>
</section>

<section class="blk">
<h2>Educación <span class="count">{cv.education.length}</span></h2>
<section class="card">
  {#if cv.education.length}
    <ul class="chips tall" aria-label="Educación">
      {#each cv.education as ed, i (ed.id)}
        {@const years = [ed.start, ed.end].filter(Boolean).join(" – ")}
        <li class="chip" class:sel={editingEdu === ed.id}>
          <button class="chipname" aria-expanded={editingEdu === ed.id}
            onclick={() => (editingEdu = editingEdu === ed.id ? null : ed.id)} title="Editar institución y fechas">
            <span class="eduname">{ed.degree || "Sin título"}</span>
            {#if ed.institution || years}
              <span class="edumeta">
                {#if ed.institution}<span>{ed.institution}</span>{/if}
                {#if years}<span class="eduyears">{years}</span>{/if}
              </span>
            {/if}
          </button>
          <button class="chipx" aria-label="Quitar {ed.degree}"
            onclick={() => { if (editingEdu === ed.id) editingEdu = null; cv.education.splice(i, 1); }}><Icon name="close" size={18} /></button>
        </li>
      {/each}
    </ul>
  {/if}
  {#each cv.education as ed (ed.id)}
    {#if editingEdu === ed.id}
      <div class="chipedit">
        <label>Título<input bind:value={ed.degree} /></label>
        <label>Institución<input bind:value={ed.institution} /></label>
        <label>Inicio<input value={ed.start ?? ""} oninput={(e) => (ed.start = opt(e.currentTarget.value))} placeholder="2016" /></label>
        <label>Fin<input value={ed.end ?? ""} oninput={(e) => (ed.end = opt(e.currentTarget.value))} placeholder="2020" /></label>
        <button class="tonal" onclick={() => (editingEdu = null)}><Icon name="check" size={20} /> Listo</button>
      </div>
    {/if}
  {/each}
  <form class="addrow" onsubmit={addEducation}>
    <input bind:value={newEdu} placeholder="Escribe tu título o carrera y pulsa Enter" aria-label="Nueva educación" />
    <button type="submit" disabled={!newEdu.trim()}><Icon name="add" /> Agregar</button>
  </form>
</section>
</section>

<section class="blk">
<h2>Certificaciones <span class="count">{cv.certifications.length}</span></h2>
<section class="card">
  {#if cv.certifications.length}
    <ul class="chips" aria-label="Certificaciones">
      {#each cv.certifications as c, i (c.id)}
        <li class="chip" class:sel={editingCert === c.id}>
          <button class="chipname" aria-expanded={editingCert === c.id}
            onclick={() => (editingCert = editingCert === c.id ? null : c.id)} title="Editar emisor y fecha">
            {c.name || "Sin nombre"}{#if c.issuer || c.date}<span class="yrs">{[c.issuer, c.date].filter(Boolean).join(" · ")}</span>{/if}
          </button>
          <button class="chipx" aria-label="Quitar {c.name}"
            onclick={() => { if (editingCert === c.id) editingCert = null; cv.certifications.splice(i, 1); }}><Icon name="close" size={18} /></button>
        </li>
      {/each}
    </ul>
  {/if}
  {#each cv.certifications as c (c.id)}
    {#if editingCert === c.id}
      <div class="chipedit">
        <label>Certificación<input bind:value={c.name} /></label>
        <label>Emisor<input value={c.issuer ?? ""} oninput={(e) => (c.issuer = opt(e.currentTarget.value))} placeholder="Opcional" /></label>
        <label>Fecha<input value={c.date ?? ""} oninput={(e) => (c.date = opt(e.currentTarget.value))} placeholder="2024-05" /></label>
        <button class="tonal" onclick={() => (editingCert = null)}><Icon name="check" size={20} /> Listo</button>
      </div>
    {/if}
  {/each}
  <form class="addrow" onsubmit={addCert}>
    <input bind:value={newCert} placeholder="Escribe una certificación y pulsa Enter" aria-label="Nueva certificación" />
    <button type="submit" disabled={!newCert.trim()}><Icon name="add" /> Agregar</button>
  </form>
</section>
</section>

<section class="blk">
<h2>Idiomas <span class="count">{cv.languages.length}</span></h2>
<section class="card">
  {#if cv.languages.length}
    <ul class="chips" aria-label="Idiomas">
      {#each cv.languages as l, i}
        <li class="chip" class:sel={editingLang === i}>
          <button class="chipname" aria-expanded={editingLang === i}
            onclick={() => (editingLang = editingLang === i ? null : i)} title="Editar nivel">
            {l.name || "Sin nombre"}{#if l.level}<span class="yrs">{l.level}</span>{/if}
          </button>
          <button class="chipx" aria-label="Quitar {l.name}"
            onclick={() => { editingLang = null; cv.languages.splice(i, 1); }}><Icon name="close" size={18} /></button>
        </li>
      {/each}
    </ul>
  {/if}
  {#if editingLang !== null && cv.languages[editingLang]}
    {@const l = cv.languages[editingLang]}
    <div class="chipedit">
      <label>Idioma<input bind:value={l.name} /></label>
      <label>Nivel<input bind:value={l.level} placeholder="B2, nativo…" /></label>
      <button class="tonal" onclick={() => (editingLang = null)}><Icon name="check" size={20} /> Listo</button>
    </div>
  {/if}
  <form class="addrow" onsubmit={addLanguage}>
    <input bind:value={newLang.name} placeholder="Idioma" aria-label="Nuevo idioma" />
    <input class="narrow" bind:value={newLang.level} placeholder="Nivel (B2…)" aria-label="Nivel del nuevo idioma" />
    <button type="submit" disabled={!newLang.name.trim()}><Icon name="add" /> Agregar</button>
  </form>
</section>
</section>
</div>

<style>
  /* Dos columnas en ventanas anchas: los bloques cortos se acomodan juntos, Experiencia ocupa todo el ancho. */
  .cvgrid { display: grid; grid-template-columns: 1fr; gap: 1.5rem 1.25rem; align-items: start; }
  @container main (min-width: 1000px) {
    .cvgrid { grid-template-columns: 1fr 1fr; }
    .wide { grid-column: 1 / -1; }
  }
  /* Datos básicos y Habilidades lado a lado; Experiencia debajo, a todo el ancho. */
  .blk { order: 4; }
  .blk.basics { order: 1; }
  .blk.skillsblk { order: 2; }
  .blk.wide { order: 3; }
  .basics .grid { grid-template-columns: repeat(auto-fit, minmax(180px, 1fr)); }
  .blk { min-width: 0; display: flex; flex-direction: column; gap: 0.6rem; }
  .blk > h2 { margin: 0; }
  .blk > button { align-self: flex-start; }
  .count {
    display: inline-grid; place-items: center; min-width: 1.75rem; padding: 0 0.45rem; margin-left: 0.4rem;
    border-radius: var(--radius-full); background: var(--md-surface-container-highest);
    color: var(--md-on-surface-variant); font-size: 0.875rem; font-weight: 700; vertical-align: middle;
  }
  .chips { display: flex; flex-wrap: wrap; gap: 0.5rem; list-style: none; padding: 0; margin: 0 0 1rem; }
  /* Ficha de entrada (M3): nombre que abre el editor y una X para quitar. */
  .chip {
    display: inline-flex; align-items: center; height: 36px; border-radius: var(--radius-s);
    border: 1px solid var(--md-outline-variant); background: var(--md-surface-container-lowest);
    transition: background-color 150ms var(--ease), border-color 150ms var(--ease);
  }
  .chip.sel { background: var(--md-secondary-container); border-color: transparent; }
  .chip button { min-height: 0; height: 100%; border: 0; border-radius: inherit; background: transparent; font-weight: 600; font-size: 0.9375rem; color: var(--md-on-surface); }
  .chip.sel button { color: var(--md-on-secondary-container); }
  .chipname { padding: 0 0.4rem 0 0.75rem; gap: 0.4rem; }
  .yrs { font-weight: 400; color: var(--md-on-surface-variant); }
  /* Educación: fichas de dos líneas (título; institución y años) con aire alrededor. */
  .chips.tall { gap: 0.75rem; }
  .tall .chip { height: auto; min-height: 64px; border-radius: var(--radius-m); align-items: stretch; }
  .tall .chipname { flex-direction: column; align-items: flex-start; justify-content: center; gap: 0.3rem; padding: 0.7rem 0.75rem 0.7rem 1rem; text-align: left; line-height: 1.3; }
  .tall .chipx { width: 44px; align-self: center; }
  .eduname { font-weight: 700; font-size: 1rem; }
  .edumeta { display: flex; flex-wrap: wrap; align-items: center; gap: 0.25rem 0.6rem; font-weight: 400; font-size: 0.9375rem; color: var(--md-on-surface-variant); }
  .chip.sel .edumeta { color: inherit; }
  .eduyears { padding: 0.05rem 0.5rem; border-radius: var(--radius-s); background: var(--md-surface-container-high); font-variant-numeric: tabular-nums; }
  .chip.sel .eduyears { background: color-mix(in srgb, var(--md-on-secondary-container) 10%, transparent); }
  .chipx { width: 32px; padding: 0; color: var(--md-on-surface-variant) !important; }
  .chipedit {
    display: grid; grid-template-columns: repeat(auto-fit, minmax(150px, 1fr)); align-items: end; gap: 0.75rem;
    padding: 1rem; margin-bottom: 1rem; border-radius: var(--radius-m); background: var(--md-surface-container);
    animation: rise-in 220ms var(--ease-decel) both;
  }
  .addrow { display: flex; gap: 0.75rem; }
  .addrow input { flex: 1 1 8rem; min-width: 0; }
  .addrow input.narrow { flex: 0 1 8rem; }
  .addrow button { flex: none; }
  .item { margin-bottom: 0.75rem; }
  .nowrap { flex-wrap: nowrap; margin-bottom: 0.5rem; }
  .narrow { max-width: 7rem; }
</style>

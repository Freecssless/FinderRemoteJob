<script lang="ts">
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
  const addSkill = () =>
    cv.skills.push({ id: nextId("sk", cv.skills.map((s) => s.id)), name: "", level: null, years: null });
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
  <p class="muted small">Estos datos solo los usa la plantilla; nunca se envían al redactor de IA.</p>
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
      <button onclick={() => addBullet(i)}>+ Logro</button>
      <button class="danger" onclick={() => cv.experience.splice(i, 1)}>Quitar experiencia</button>
    </div>
  </section>
{/each}
<button onclick={addExperience}>+ Experiencia</button>

<h2>Habilidades</h2>
<section class="card">
  {#each cv.skills as s, i (s.id)}
    <div class="row nowrap">
      <input bind:value={s.name} placeholder="Habilidad" aria-label="Habilidad" />
      <input value={s.level ?? ""} oninput={(e) => (s.level = opt(e.currentTarget.value))} placeholder="Nivel" aria-label="Nivel" class="narrow" />
      <input
        type="number" min="0" step="0.5" class="narrow"
        value={s.years ?? ""}
        oninput={(e) => (s.years = e.currentTarget.value === "" ? null : Number(e.currentTarget.value))}
        placeholder="Años" aria-label="Años"
      />
      <button class="link danger" onclick={() => cv.skills.splice(i, 1)}>Quitar</button>
    </div>
  {/each}
  <button onclick={addSkill}>+ Habilidad</button>
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
<button onclick={addEducation}>+ Educación</button>

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
  <button onclick={addCert}>+ Certificación</button>
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
  <button onclick={addLanguage}>+ Idioma</button>
</section>

<style>
  .item { margin-bottom: 0.75rem; }
  .nowrap { flex-wrap: nowrap; margin-bottom: 0.5rem; }
  .narrow { max-width: 7rem; }
</style>

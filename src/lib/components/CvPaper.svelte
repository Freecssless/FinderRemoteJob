<script lang="ts">
  import type { CvDocument } from "#lib/api.ts";

  /** Hoja del CV como se imprime. Las medidas van en `cqi`: la misma hoja sirve de miniatura y de vista grande. */
  let { doc, label = "Vista previa del CV" }: { doc: CvDocument; label?: string } = $props();

  type Key = keyof CvDocument["headings"];
  // Mismo orden que `Template::order` en crates/core/src/document.rs.
  const order = $derived<Key[]>(
    doc.template === "tecnica"
      ? ["summary", "skills", "experience", "education", "certifications", "languages"]
      : ["summary", "experience", "education", "skills", "certifications", "languages"],
  );
  const has = (k: Key) =>
    k === "summary" ? !!doc.summary : (doc[k] as unknown[]).length > 0;
</script>

<div class="paperbox">
  <article class="paper {doc.template}" aria-label={label}>
    <header>
      <h1>{doc.name || "Tu nombre"}</h1>
      {#if doc.headline}<p class="headline">{doc.headline}</p>{/if}
      {#if doc.contact.length}<p class="contact">{doc.contact.join("  ·  ")}</p>{/if}
    </header>
    {#each order.filter(has) as k}
      <section>
        <h2>{doc.headings[k]}</h2>
        {#if k === "summary"}
          <p>{doc.summary}</p>
        {:else if k === "experience"}
          {#each doc.experience as e}
            <div class="entry">
              <div class="line"><strong>{e.title}</strong><span>{e.dates}</span></div>
              <div class="sub">{e.company}</div>
              <ul>{#each e.bullets as b}<li>{b}</li>{/each}</ul>
            </div>
          {/each}
        {:else if k === "education"}
          {#each doc.education as e}
            <div class="entry">
              <div class="line"><strong>{e.degree}</strong><span>{e.dates}</span></div>
              <div class="sub">{e.institution}</div>
            </div>
          {/each}
        {:else if k === "skills"}
          <p>{doc.skills.join(" · ")}</p>
        {:else}
          <ul>{#each doc[k] as item}<li>{item}</li>{/each}</ul>
        {/if}
      </section>
    {/each}
  </article>
</div>

<style>
  .paperbox { container-type: inline-size; }
  /* Una hoja es papel en los dos temas: sus colores no cambian con el sistema. */
  .paper {
    --u: 1cqi;
    aspect-ratio: 216 / 279; /* carta, como el PDF */ overflow: hidden;
    padding: calc(var(--u) * 7) calc(var(--u) * 7.5);
    background: var(--paper); color: var(--on-paper);
    border-radius: calc(var(--u) * 0.8);
    box-shadow: var(--elev-1);
    font-family: "Liberation Sans", Arial, Helvetica, sans-serif;
    font-size: calc(var(--u) * 1.75); line-height: 1.38;
    text-align: left; user-select: text;
  }
  .paper.compacta { font-size: calc(var(--u) * 1.55); padding: calc(var(--u) * 5.5) calc(var(--u) * 6); }
  header { margin-bottom: 1.1em; }
  h1 { margin: 0; font-size: 2em; line-height: 1.1; font-weight: 700; color: var(--on-paper); }
  .headline { margin: 0.25em 0 0; font-size: 1.15em; font-weight: 700; color: var(--paper-accent); }
  .contact { margin: 0.35em 0 0; color: var(--on-paper-variant); }
  section { margin-top: 0.9em; }
  h2 {
    margin: 0 0 0.4em; padding-bottom: 0.2em; border-bottom: max(1px, calc(var(--u) * 0.15)) solid var(--paper-rule);
    font-size: 1.05em; font-weight: 700; letter-spacing: 0; color: var(--paper-accent);
  }
  p { margin: 0; }
  .entry + .entry { margin-top: 0.6em; }
  .line { display: flex; justify-content: space-between; gap: 1em; }
  .line span, .sub { color: var(--on-paper-variant); }
  .line span { flex: none; font-variant-numeric: tabular-nums; }
  ul { margin: 0.2em 0 0; padding-left: 1.1em; }
  li + li { margin-top: 0.15em; }
</style>

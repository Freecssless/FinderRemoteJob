<script lang="ts">
  import { api, statsApi, SOURCE_LABELS, TEMPLATE_LABELS, type GroupStat, type Profile, type Stats } from "$lib/api";

  let profiles = $state<Profile[]>([]);
  let profileId = $state<string>("");
  let stats = $state<Stats | null>(null);
  let error = $state("");

  const profileName = (id: string) => profiles.find((p) => p.id === id)?.name ?? id;
  const fmt = (n: number | null, suffix = "") => (n === null ? "—" : `${Math.round(n * 10) / 10}${suffix}`);
  const share = (part: number, total: number) => (total ? Math.round((100 * part) / total) : 0);

  const sections = $derived(
    stats
      ? [
          { title: "Por portal", rows: stats.by_source, label: (g: string) => SOURCE_LABELS[g] ?? g },
          ...(profileId ? [] : [{ title: "Por perfil", rows: stats.by_profile, label: profileName }]),
          { title: "Por plantilla de CV", rows: stats.by_variant, label: (g: string) => TEMPLATE_LABELS[g] ?? g },
          { title: "Por ATS del CV enviado", rows: stats.by_ats, label: (g: string) => g },
        ]
      : [],
  );

  async function init() {
    try {
      profiles = await api.listProfiles();
      await load();
    } catch (e) {
      error = String(e);
    }
  }

  async function load() {
    try {
      stats = await statsApi.get(profileId || null);
      error = "";
    } catch (e) {
      error = String(e);
    }
  }

  init();
</script>

{#snippet table(title: string, rows: GroupStat[], label: (g: string) => string)}
  <section class="card">
    <h2>{title}</h2>
    <table>
      <thead>
        <tr>
          <th scope="col">Grupo</th>
          <th scope="col" class="num">n</th>
          <th scope="col" class="num">Respondieron</th>
          <th scope="col" class="num">Entrevistas</th>
          <th scope="col" class="rate">Tasa de respuesta</th>
        </tr>
      </thead>
      <tbody>
        {#each rows as g (g.group)}
          <tr class:few={g.few_data}>
            <th scope="row">{label(g.group)}</th>
            <td class="num">{g.applied}</td>
            <td class="num">{g.responded}</td>
            <td class="num">{g.interviews}</td>
            <td class="rate">
              <span class="bar" title="{g.responded} de {g.applied} respondieron">
                <span class="fill" style="width: {g.response_rate}%"></span>
              </span>
              <span class="pct">{fmt(g.response_rate, " %")}</span>
              {#if g.few_data}<span class="tag">pocos datos</span>{/if}
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  </section>
{/snippet}

<div class="page">
  <header class="row">
    <h1>Estadísticas</h1>
    <label class="inline">
      Perfil
      <select bind:value={profileId} onchange={load}>
        <option value="">Todos los perfiles</option>
        {#each profiles as p}<option value={p.id}>{p.name}</option>{/each}
      </select>
    </label>
  </header>

  {#if error}<p class="danger" role="alert">{error}</p>{/if}

  {#if stats}
    {#if stats.eligible === 0}
      <p class="notice">
        Todavía no hay datos: solo cuentan las postulaciones con al menos 14 días desde que te postulaste
        (para no castigar las recientes).
        {#if stats.too_recent}Tienes {stats.too_recent} postulada{stats.too_recent === 1 ? "" : "s"} que contará{stats.too_recent === 1 ? "" : "n"} pronto.{:else}Marca tus vacantes como «Postulada» en el <a href="/tablero">Tablero</a>.{/if}
      </p>
    {:else}
      <p class="muted small">
        Cuentan las postulaciones con al menos 14 días ({stats.eligible}).
        {#if stats.too_recent}{stats.too_recent} más recientes todavía no cuentan.{/if}
        Los grupos con menos de 10 se marcan «pocos datos» y van al final.
      </p>

      <div class="tiles">
        <div class="card tile">
          <span class="muted small">Tasa de respuesta</span>
          <strong class="big">{fmt(stats.response_rate, " %")}</strong>
          <span class="muted small">n = {stats.eligible}</span>
        </div>
        <div class="card tile">
          <span class="muted small">Días hasta la primera respuesta (mediana)</span>
          <strong class="big">{fmt(stats.median_days_to_response)}</strong>
        </div>
        <div class="card tile">
          <span class="muted small">Compatibilidad promedio</span>
          <strong class="big">{fmt(stats.avg_match_responded)}</strong>
          <span class="muted small">de las que respondieron · {fmt(stats.avg_match_not_responded)} de las que no</span>
        </div>
      </div>

      <section class="card">
        <h2>Embudo</h2>
        {#each [
          { label: "Postuladas", n: stats.funnel.applied },
          { label: "Llegaron a entrevista", n: stats.funnel.interview },
          { label: "Llegaron a oferta", n: stats.funnel.offer },
        ] as step}
          <div class="funnel">
            <span>{step.label}</span>
            <span class="bar" title="{step.n} de {stats.funnel.applied}">
              <span class="fill" style="width: {share(step.n, stats.funnel.applied)}%"></span>
            </span>
            <span class="pct">{step.n} ({share(step.n, stats.funnel.applied)} %)</span>
          </div>
        {/each}
      </section>

      {#each sections as s (s.title)}
        {@render table(s.title, s.rows, s.label)}
      {/each}
    {/if}
  {:else if !error}
    <p class="muted">Cargando…</p>
  {/if}
</div>

<style>
  .page { display: flex; flex-direction: column; gap: 0.75rem; max-width: 960px; }
  header h1 { margin: 0 1rem 0 0; }
  header select { width: auto; }
  .tiles { display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 0.75rem; }
  .tile { display: flex; flex-direction: column; gap: 0.2rem; }
  .big { font-size: 1.8rem; font-variant-numeric: tabular-nums; }
  section h2 { margin-top: 0; }
  table { width: 100%; border-collapse: collapse; font-variant-numeric: tabular-nums; }
  th, td { padding: 0.35rem 0.5rem; border-bottom: 1px solid var(--border); text-align: left; }
  thead th { font-size: 0.8rem; color: var(--muted); font-weight: 600; }
  tbody th { font-weight: 500; }
  .num { text-align: right; width: 6rem; }
  .rate { width: 40%; }
  td.rate { display: flex; align-items: center; gap: 0.5rem; }
  tr.few th, tr.few td { color: var(--muted); }
  .bar { flex: 1; height: 8px; background: var(--hover); border-radius: 4px; overflow: hidden; min-width: 4rem; }
  .fill { display: block; height: 100%; background: var(--chart-bar); border-radius: 0 4px 4px 0; }
  tr.few .fill { opacity: 0.45; }
  .pct { min-width: 4.5rem; text-align: right; color: var(--text); }
  .tag { font-size: 0.72rem; border: 1px solid var(--border); border-radius: 4px; padding: 0 0.3rem; color: var(--muted); }
  .funnel { display: grid; grid-template-columns: 11rem 1fr 6.5rem; align-items: center; gap: 0.75rem; padding: 0.25rem 0; }
</style>

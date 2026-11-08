<script lang="ts">
  import { onMount } from "svelte";
  import { openUrl, revealItemInDir } from "@tauri-apps/plugin-opener";
  import PageHead from "$lib/components/PageHead.svelte";
  import { LICENSES_URL, PRIVACY_URL, TERMS_URL, issueUrl, legalApi, type ErrorReport } from "$lib/legal";

  let report = $state<ErrorReport | null>(null);
  let what = $state("");
  let error = $state("");
  // Borrar es irreversible: se pide confirmar escribiendo una palabra.
  let confirmText = $state("");
  let deleting = $state(false);

  onMount(() => void legalApi.report().then((r) => (report = r)).catch((e) => (error = String(e))));

  async function reportIssue(kind: "fallo" | "contenido") {
    error = "";
    try {
      report = await legalApi.report();
      await openUrl(issueUrl(kind, report, what.trim()));
    } catch (e) {
      error = String(e);
    }
  }

  async function deleteAll() {
    deleting = true;
    error = "";
    try {
      await legalApi.deleteAll(); // reinicia la app
    } catch (e) {
      error = String(e);
      deleting = false;
    }
  }

  /** Qué guarda la app, para qué y a dónde va (inventario completo; ver PRIVACIDAD.md). */
  const DATA = [
    ["Tu CV (nombre, contacto, experiencia, estudios, habilidades)", "Leerlo, adaptarlo a vacantes y generar el PDF/DOCX", "Solo en tu computadora", "Titular, experiencia y habilidades (sin nombre, contacto ni estudios), solo a la IA externa que elijas"],
    ["Perfiles de búsqueda (puestos, preferencia de remoto)", "Buscar y ordenar vacantes", "Solo en tu computadora", "Los puestos, a los portales de empleo al buscar"],
    ["Vacantes, postulaciones, notas y recordatorios", "Tu tablero de seguimiento", "Solo en tu computadora", "A nadie"],
    ["CVs generados (PDF y DOCX)", "Que los envíes tú", "Solo en tu computadora", "A nadie (los envías tú)"],
    ["Claves de API de Claude, Gemini o ChatGPT", "Usar esa IA si la eliges", "Llavero cifrado del sistema", "Solo a su proveedor, por HTTPS"],
    ["Registro de errores (versión, sistema, mensajes de error)", "Reportar fallos", "Solo en tu computadora", "A nadie, salvo que tú lo pegues en un reporte"],
  ];

  /** Servicios externos con los que habla la app. No hay analítica, publicidad ni rastreo. */
  const SERVICES = [
    ["Portales de empleo (Remote OK, Remotive, We Work Remotely, Get on Board, Jobicy, Himalayas, Greenhouse, Lever, Ashby)", "Al buscar vacantes", "Los puestos que buscas y tu dirección IP"],
    ["Página de una vacante que pegues", "Al importar un enlace", "Tu dirección IP"],
    ["Ollama (en tu computadora)", "Al leer tu CV y analizar vacantes", "Nada sale del equipo"],
    ["Anthropic (Claude), Google (Gemini) u OpenAI (ChatGPT)", "Solo si los eliges en Ajustes y aceptas", "Texto de la vacante, tu titular, experiencia y habilidades (sin nombre, contacto ni estudios)"],
    ["ollama.com y GitHub", "Solo al descargar Ollama o sus modelos", "Tu dirección IP"],
    ["GitHub", "Solo si decides reportar algo", "Lo que tú escribas en el reporte"],
  ];
</script>

<div class="page">
  <PageHead icon="info" title="Privacidad y ayuda" subtitle="Qué datos usa la app, tus derechos y cómo reportar un problema" />
  {#if error}<p class="danger" role="alert">{error}</p>{/if}

  <section class="card">
    <h2>Tus datos se quedan en tu computadora</h2>
    <p>
      FinderRemoteJob no tiene cuentas, servidores ni publicidad. No usa cookies, analítica ni
      rastreo. Todo lo que escribes o importas se guarda en tu equipo, y solo sale de él en los
      casos de esta tabla.
    </p>
    <div class="scroll">
      <table>
        <thead><tr><th>Dato</th><th>Para qué</th><th>Dónde se guarda</th><th>Se envía a</th></tr></thead>
        <tbody>{#each DATA as row}<tr>{#each row as cell}<td>{cell}</td>{/each}</tr>{/each}</tbody>
      </table>
    </div>
  </section>

  <section class="card">
    <h2>Servicios externos</h2>
    <p class="muted small">La app no incluye kits de terceros (SDK) de analítica, anuncios ni reporte automático de fallos.</p>
    <div class="scroll">
      <table>
        <thead><tr><th>Servicio</th><th>Cuándo</th><th>Qué recibe</th></tr></thead>
        <tbody>{#each SERVICES as row}<tr>{#each row as cell}<td>{cell}</td>{/each}</tr>{/each}</tbody>
      </table>
    </div>
  </section>

  <div class="cols">
    <section class="card">
      <h2>Permisos</h2>
      <ul>
        <li><strong>Notificaciones:</strong> solo para los recordatorios que tú creas en el Tablero. Puedes desactivarlas en la configuración de tu sistema.</li>
        <li><strong>Archivos:</strong> la app solo lee el archivo que eliges (tu CV o un respaldo) y solo abre los CVs que generó.</li>
        <li><strong>Cámara, micrófono, ubicación, contactos y cookies:</strong> no se usan.</li>
      </ul>
    </section>

    <section class="card">
      <h2>Documentos</h2>
      <div class="row">
        <button class="tonal" onclick={() => openUrl(PRIVACY_URL)}>Política de privacidad</button>
        <button class="tonal" onclick={() => openUrl(TERMS_URL)}>Términos de uso</button>
        <button class="tonal" onclick={() => openUrl(LICENSES_URL)}>Licencias de terceros</button>
      </div>
      <p class="muted small">
        FinderRemoteJob {report?.version ?? ""} es software libre (MIT o Apache-2.0) © 2026 freecss.
        Se ofrece tal cual, sin garantías: revisa siempre tu CV antes de enviarlo.
      </p>
    </section>
  </div>

  <section class="card">
    <h2>Reportar un problema</h2>
    <p class="small">
      Se abre un reporte en GitHub ya rellenado. Lo ves y lo editas antes de enviarlo: nada se
      manda solo. Para reportar necesitas una cuenta de GitHub, que es gratis.
    </p>
    <label>¿Qué pasó? (opcional)
      <textarea bind:value={what} maxlength="2000" placeholder="Por ejemplo: al importar mi CV en PDF salió un error"></textarea>
    </label>
    <div class="row">
      <button class="primary" onclick={() => reportIssue("fallo")}>Reportar un fallo</button>
      <button onclick={() => reportIssue("contenido")}>Reportar contenido o privacidad</button>
      {#if report?.log_path}
        <button class="link" onclick={() => revealItemInDir(report!.log_path).catch((e) => (error = String(e)))}>Ver registro de errores</button>
      {/if}
    </div>
    {#if report}
      <p class="muted small">
        {report.recent.length === 1
          ? `El reporte incluirá el último error registrado, la versión y tu sistema (${report.os}).`
          : report.recent.length
            ? `El reporte incluirá los últimos ${report.recent.length} errores registrados, la versión y tu sistema (${report.os}).`
            : "No hay errores registrados."}
        Nunca incluye tu CV.
      </p>
    {/if}
  </section>

  <section class="card danger-zone">
    <h2>Borrar todos mis datos</h2>
    <p class="small">
      Borra tus perfiles, tu CV, vacantes, postulaciones, CVs generados, el registro de errores y
      tus claves de API del llavero. La app se reinicia vacía. No se puede deshacer: si quieres
      conservar algo, crea antes una copia de seguridad en Ajustes.
    </p>
    <p class="muted small">
      No toca las copias de seguridad que guardaste en Documentos ni los modelos de Ollama: bórralos
      tú si ya no los quieres. Para dejar de usar la app, desinstálala después.
    </p>
    <label><span>Escribe <strong>BORRAR</strong> para confirmar</span>
      <input bind:value={confirmText} autocomplete="off" />
    </label>
    <div class="row">
      <button class="danger-btn" onclick={deleteAll} disabled={confirmText.trim() !== "BORRAR" || deleting}>
        {deleting ? "Borrando…" : "Borrar todo y reiniciar"}
      </button>
    </div>
  </section>
</div>

<style>
  .page { display: flex; flex-direction: column; gap: 1rem; }
  section { display: flex; flex-direction: column; gap: 0.6rem; }
  section h2, section p { margin: 0; }
  section ul { margin: 0; padding-left: 1.2rem; display: flex; flex-direction: column; gap: 0.4rem; }
  .scroll { overflow-x: auto; }
  td { vertical-align: top; }
  .cols { display: grid; grid-template-columns: 1fr; gap: 1rem; }
  @container main (min-width: 880px) { .cols { grid-template-columns: 1fr 1fr; } }
  .danger-zone { border: 1px solid var(--md-error); }
  .danger-btn { background: var(--md-error); color: var(--md-on-error); border-color: var(--md-error); }
  .danger-btn:disabled { background: transparent; color: var(--md-on-surface-variant); border-color: var(--md-outline-variant); }
</style>

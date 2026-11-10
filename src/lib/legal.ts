import { invoke } from "@tauri-apps/api/core";

/** Repositorio público: reportes, política completa y términos. Cambiar aquí si se mueve. */
export const REPO_URL = "https://github.com/Freecssless/FinderRemoteJob";
/** Sube cuando cambien la política de privacidad o los términos (se vuelve a pedir aceptarlos). */
export const LEGAL_VERSION = "2026-11-08";
export const PRIVACY_URL = `${REPO_URL}/blob/main/PRIVACIDAD.md`;
export const TERMS_URL = `${REPO_URL}/blob/main/TERMINOS.md`;
export const LICENSES_URL = `${REPO_URL}/blob/main/LICENCIAS-TERCEROS.md`;

export interface ErrorReport { version: string; os: string; arch: string; log_path: string; recent: string[] }

export const legalApi = {
  report: () => invoke<ErrorReport>("error_report"),
  /** Borra las claves del llavero y reinicia la app, que vacía su carpeta de datos al arrancar. */
  deleteAll: () => invoke<void>("delete_all_data"),
  accepted: () => invoke<string | null>("legal_accepted"),
  accept: () => invoke<void>("accept_legal", { version: LEGAL_VERSION }),
  logError: (message: string) => invoke<void>("log_frontend_error", { message }).catch(() => {}),
};

/** Issue de GitHub ya rellenado. El usuario lo ve y lo edita antes de enviarlo: nada se manda solo. */
export function issueUrl(kind: "fallo" | "contenido", r: ErrorReport | null, what = ""): string {
  const title = kind === "fallo" ? "Fallo: " : "Reporte de contenido o privacidad: ";
  const body = kind === "fallo"
    ? [
        "**¿Qué estabas haciendo y qué pasó?**", what || "(descríbelo aquí)", "",
        "**Equipo**", `FinderRemoteJob ${r?.version ?? "?"} · ${r?.os ?? "?"} ${r?.arch ?? ""}`, "",
        "**Últimos errores registrados** (revisa que no haya datos personales antes de enviar)",
        "```", ...(r?.recent.length ? r.recent : ["(sin errores registrados)"]), "```",
      ].join("\n")
    : [
        "**¿Qué quieres reportar?** (contenido indebido de un anuncio, una fuente de vacantes, un tema de privacidad o de derechos de autor)",
        what || "(descríbelo aquí; no pegues tu CV ni datos personales)",
      ].join("\n");
  // Las URL muy largas fallan: se recorta el cuerpo.
  return `${REPO_URL}/issues/new?labels=${kind === "fallo" ? "bug" : "reporte"}&title=${encodeURIComponent(title)}&body=${encodeURIComponent(body.slice(0, 6000))}`;
}

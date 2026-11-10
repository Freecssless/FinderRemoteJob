import { redirect } from "@sveltejs/kit";
import { api } from "#lib/api.ts";

// Sin perfiles es el primer uso: va al asistente de bienvenida.
export async function load() {
  const hasProfiles = await api.listProfiles().then((p) => p.length > 0).catch(() => true);
  redirect(307, hasProfiles ? "/perfiles" : "/bienvenida");
}

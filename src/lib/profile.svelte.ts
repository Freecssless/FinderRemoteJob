import { api, rememberedProfile, rememberProfile, type Profile } from "$lib/api";

/** Perfil activo de toda la app: se elige en el menú lateral (o en las pestañas de Perfiles). */
export const profileStore = $state({ list: [] as Profile[], id: null as string | null, ready: false });

/** Recarga la lista (tras crear, renombrar o borrar) y conserva el perfil elegido si sigue existiendo. */
export async function refreshProfiles() {
  const list = await api.listProfiles();
  profileStore.list = list;
  const saved = profileStore.id ?? rememberedProfile();
  profileStore.id = list.find((p) => p.id === saved)?.id ?? list[0]?.id ?? null;
  profileStore.ready = true;
}

export function chooseProfile(id: string) {
  profileStore.id = id;
  rememberProfile(id);
}

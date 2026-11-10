<script lang="ts">
  import { api, settingsApi } from "#lib/api.ts";

  /**
   * Mientras se lee un CV: segundos transcurridos y «Cancelar». Si la IA local pasa de un minuto
   * (equipo lento), ofrece leerlo con la IA en la nube o explica cómo configurarla.
   */
  let { cloud, onusecloud }: { cloud: boolean; onusecloud: () => void } = $props();

  const SLOW_SECS = 60;
  let secs = $state(0);
  let cloudReady = $state(false);

  $effect(() => {
    const t = setInterval(() => secs++, 1000);
    return () => clearInterval(t);
  });
  $effect(() => {
    settingsApi.get().then((v) => (cloudReady = !!v.writer_kind && v.writer_kind !== "local")).catch(() => {});
  });
</script>

<div class="progress" role="status">
  <p class="muted small">
    Leyendo tu CV con la IA {cloud ? "en la nube" : "de tu computadora"}… {secs} s
  </p>
  {#if !cloud && secs >= SLOW_SECS}
    <p class="small">
      Está tardando: la IA local va lenta en este equipo.
      {#if cloudReady}
        Puedes leerlo con la IA en la nube (tarda unos segundos; tu CV se envía a ese servicio).
      {:else}
        Si te pasa seguido, configura una IA en la nube en <a href="/ajustes">Ajustes</a>: tarda unos
        segundos en cualquier equipo.
      {/if}
    </p>
  {/if}
  <div class="row">
    {#if !cloud && secs >= SLOW_SECS && cloudReady}
      <button class="primary" onclick={onusecloud}>Leer con la IA en la nube</button>
    {/if}
    <button onclick={() => api.cancelImport()}>Cancelar</button>
  </div>
</div>

<style>
  .progress p {
    margin: 0.5rem 0;
  }
</style>

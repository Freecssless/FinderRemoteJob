---
name: FinderRemoteJob
description: Material Design 3 con identidad propia, azul talavera y cempasúchil, para buscar trabajo remoto sin estrés.
colors:
  scrim: "#000000"
  talavera: "#1f50a0"
  on-talavera: "#ffffff"
  talavera-container: "#d8e2ff"
  on-talavera-container: "#002e6b"
  talavera-dark: "#aec6ff"
  on-talavera-dark: "#002e6b"
  talavera-container-dark: "#0a4393"
  secondary-container: "#dbe2f9"
  on-secondary-container: "#3f4759"
  secondary-container-dark: "#3f4759"
  on-secondary-container-dark: "#dbe2f9"
  cempasuchil: "#8c5000"
  cempasuchil-container: "#ffdcbf"
  on-cempasuchil-container: "#2d1600"
  cempasuchil-dark: "#ffb873"
  cempasuchil-container-dark: "#6a3b00"
  on-cempasuchil-container-dark: "#ffdcbf"
  error: "#ba1a1a"
  on-error: "#ffffff"
  error-container: "#ffdad6"
  on-error-container: "#93000a"
  error-dark: "#ffb4ab"
  success: "#196c40"
  success-container: "#a4f4bc"
  on-success-container: "#00210f"
  success-dark: "#88d7a1"
  surface: "#faf9ff"
  surface-container-lowest: "#ffffff"
  surface-container-low: "#f3f3fa"
  surface-container: "#ededf5"
  surface-container-high: "#e8e7ef"
  surface-container-highest: "#e2e2e9"
  on-surface: "#1a1b21"
  on-surface-variant: "#434751"
  outline: "#737783"
  outline-variant: "#c3c6d3"
  surface-dark: "#111318"
  surface-container-low-dark: "#1a1b21"
  surface-container-dark: "#1e1f25"
  surface-container-highest-dark: "#33353a"
  on-surface-dark: "#e2e2e9"
  on-surface-variant-dark: "#c3c6d3"
  outline-dark: "#8d909d"
  outline-variant-dark: "#434751"
typography:
  headline:
    fontFamily: "Atkinson Hyperlegible Next Variable, system-ui, sans-serif"
    fontSize: "1.75rem"
    fontWeight: 650
    lineHeight: "2.25rem"
    letterSpacing: "-0.01em"
  title-large:
    fontFamily: "Atkinson Hyperlegible Next Variable, system-ui, sans-serif"
    fontSize: "1.375rem"
    fontWeight: 650
    lineHeight: "1.75rem"
  title-medium:
    fontFamily: "Atkinson Hyperlegible Next Variable, system-ui, sans-serif"
    fontSize: "1.0625rem"
    fontWeight: 700
    lineHeight: "1.5rem"
  body:
    fontFamily: "Atkinson Hyperlegible Next Variable, system-ui, sans-serif"
    fontSize: "1rem"
    fontWeight: 400
    lineHeight: 1.5
  body-small:
    fontFamily: "Atkinson Hyperlegible Next Variable, system-ui, sans-serif"
    fontSize: "0.9375rem"
    fontWeight: 400
    lineHeight: "1.4rem"
  label:
    fontFamily: "Atkinson Hyperlegible Next Variable, system-ui, sans-serif"
    fontSize: "0.9375rem"
    fontWeight: 650
    letterSpacing: "0.01em"
  label-small:
    fontFamily: "Atkinson Hyperlegible Next Variable, system-ui, sans-serif"
    fontSize: "0.875rem"
    fontWeight: 650
  nav:
    fontFamily: "Atkinson Hyperlegible Next Variable, system-ui, sans-serif"
    fontSize: "1rem"
    fontWeight: 600
  score:
    fontFamily: "Atkinson Hyperlegible Next Variable, system-ui, sans-serif"
    fontSize: "1rem"
    fontWeight: 750
    fontFeature: "tnum"
  paper:
    fontFamily: "Liberation Sans, Arial, Helvetica, sans-serif"
    fontSize: "10.5pt"
    fontWeight: 400
  code:
    fontFamily: "ui-monospace, Cascadia Mono, DejaVu Sans Mono, monospace"
    fontSize: "0.92em"
    fontWeight: 400
rounded:
  xs: "4px"
  s: "8px"
  m: "12px"
  l: "16px"
  full: "999px"
spacing:
  "1": "4px"
  "2": "8px"
  "3": "12px"
  "4": "16px"
  "5": "20px"
  "6": "24px"
  "8": "32px"
  "10": "40px"
components:
  button-filled:
    backgroundColor: "{colors.talavera}"
    textColor: "{colors.on-talavera}"
    typography: "{typography.label}"
    rounded: "{rounded.full}"
    padding: "0 1.4rem"
    height: "44px"
  button-outlined:
    backgroundColor: "transparent"
    textColor: "{colors.talavera}"
    typography: "{typography.label}"
    rounded: "{rounded.full}"
    padding: "0 1.4rem"
    height: "44px"
  button-tonal:
    backgroundColor: "{colors.secondary-container}"
    textColor: "{colors.on-secondary-container}"
    typography: "{typography.label}"
    rounded: "{rounded.full}"
    padding: "0 1.4rem"
    height: "44px"
  button-text:
    backgroundColor: "transparent"
    textColor: "{colors.talavera}"
    typography: "{typography.label}"
    rounded: "{rounded.full}"
    padding: "0 0.75rem"
    height: "40px"
  button-danger:
    backgroundColor: "transparent"
    textColor: "{colors.error}"
    rounded: "{rounded.full}"
    padding: "0 1.4rem"
    height: "44px"
  button-danger-filled:
    backgroundColor: "{colors.error}"
    textColor: "{colors.on-error}"
    rounded: "{rounded.full}"
    padding: "0 1.4rem"
    height: "44px"
  segmented-selected:
    backgroundColor: "{colors.secondary-container}"
    textColor: "{colors.on-secondary-container}"
    padding: "0 1.1rem"
    height: "44px"
  field:
    backgroundColor: "transparent"
    textColor: "{colors.on-surface}"
    typography: "{typography.body}"
    rounded: "{rounded.xs}"
    padding: "0.65rem 0.9rem"
    height: "48px"
  card:
    backgroundColor: "{colors.surface-container-low}"
    rounded: "{rounded.l}"
    padding: "1.25rem 1.5rem"
  card-outlined:
    backgroundColor: "transparent"
    rounded: "{rounded.l}"
    padding: "1.25rem 1.5rem"
  notice:
    backgroundColor: "{colors.secondary-container}"
    textColor: "{colors.on-secondary-container}"
    rounded: "{rounded.m}"
    padding: "0.85rem 1.1rem"
  alert-error:
    backgroundColor: "{colors.error-container}"
    textColor: "{colors.on-error-container}"
    rounded: "{rounded.m}"
    padding: "0.75rem 1rem"
  status-ok:
    backgroundColor: "{colors.success-container}"
    textColor: "{colors.on-success-container}"
    rounded: "{rounded.m}"
    padding: "0.75rem 1rem"
  nav-drawer:
    backgroundColor: "{colors.surface-container-low}"
    width: "248px"
    padding: "1.25rem 0.75rem 1rem"
  nav-item:
    backgroundColor: "transparent"
    textColor: "{colors.on-surface-variant}"
    typography: "{typography.nav}"
    rounded: "{rounded.full}"
    padding: "0 1rem"
    height: "52px"
  nav-item-active:
    backgroundColor: "{colors.secondary-container}"
    textColor: "{colors.on-secondary-container}"
    rounded: "{rounded.full}"
    height: "52px"
  list-item:
    backgroundColor: "transparent"
    textColor: "{colors.on-surface}"
    rounded: "{rounded.m}"
    padding: "0.75rem 1rem"
  list-item-active:
    backgroundColor: "{colors.secondary-container}"
    textColor: "{colors.on-secondary-container}"
    rounded: "{rounded.m}"
  meter-high:
    backgroundColor: "{colors.cempasuchil-container}"
    textColor: "{colors.on-cempasuchil-container}"
    typography: "{typography.score}"
    rounded: "{rounded.full}"
    width: "3.25rem"
    height: "32px"
  meter-mid:
    backgroundColor: "{colors.talavera-container}"
    textColor: "{colors.on-talavera-container}"
    typography: "{typography.score}"
    rounded: "{rounded.full}"
    width: "3.25rem"
    height: "32px"
  meter-low:
    backgroundColor: "{colors.surface-container-highest}"
    textColor: "{colors.on-surface-variant}"
    typography: "{typography.score}"
    rounded: "{rounded.full}"
    width: "3.25rem"
    height: "32px"
  badge-new:
    backgroundColor: "{colors.cempasuchil-container}"
    textColor: "{colors.on-cempasuchil-container}"
    typography: "{typography.label-small}"
    rounded: "{rounded.s}"
    padding: "0.1rem 0.55rem"
  empty-state:
    backgroundColor: "{colors.surface-container-low}"
    rounded: "{rounded.l}"
    padding: "2rem"
    width: "34rem"
  kanban-card:
    backgroundColor: "{colors.surface-container-lowest}"
    textColor: "{colors.on-surface}"
    rounded: "{rounded.m}"
    padding: "0.7rem 0.8rem"
---

# Design System: FinderRemoteJob

## Overview

**Creative North Star: "La loza poblana en la mesa de trabajo"**

Material Design 3, fijado por el usuario, llevado a una identidad propia. El azul talavera de la loza poblana sustituye al morado de referencia de M3; el cempasúchil aparece solo cuando algo encaja mucho o es nuevo. Las superficies son los neutros de M3 con un ligero tinte azul, apiladas por tono, no por sombra. Es una app de escritorio tranquila para sesiones cortas: el usuario debe saber de un vistazo en qué pantalla está y cuál es el siguiente paso.

La densidad es media-baja a propósito: texto base de 16px, controles de 40 a 52px de alto, tipografía Atkinson Hyperlegible Next, pensada para la legibilidad. El tema claro u oscuro lo decide el sistema (`prefers-color-scheme`); los dos temas comparten roles y cambian solo los tonos. Todo recurso (fuente, iconos) va incluido en la app, porque la CSP solo permite recursos locales.

Lo que se rechaza, confirmado en la dirección: el M3 de fábrica (morado de referencia, Roboto, tarjetas grises idénticas).

**Key Characteristics:**
- Principal azul talavera de la paleta tonal de su propia semilla (#1D4E9E): T35 en claro, T80 en oscuro.
- Cempasúchil reservado para la compatibilidad alta y la etiqueta «Nueva».
- Formas de píldora en botones, navegación y medidor; contorno de 4px en campos.
- Profundidad por tono de superficie; dos sombras suaves solo para estado y para tarjetas que se arrastran.
- Movimiento con significado: al cambiar de pantalla el contenido se funde (fade through de M3) y la píldora de la navegación se desliza al nuevo destino. La barra del medidor crece desde cero. Los paneles y avisos entran subiendo unos píxeles.
- Iconos Material Symbols Rounded 400 en SVG en línea, con variante rellena para el destino activo.

## Colors

Una paleta M3 generada con `@material/material-color-utilities`: un azul con oficio, un naranja de fiesta usado con cuentagotas y neutros fríos.

### Primary
- **Azul talavera** (`talavera` / `talavera-dark`): botones rellenos, enlaces, texto de botones delineados, anillo de foco, barra del medidor en compatibilidad media, la marca del cajón. Se toma de la paleta tonal de la semilla (T35 en claro, T80 en oscuro), no del tono 40 genérico.
- **Talavera suave** (`talavera-container`): píldora del medidor en compatibilidad media, icono del estado vacío, selección de texto, fila activa de la vista Lista y columna destino al arrastrar en el Tablero.

### Secondary
- **Azul pizarra tonal** (`secondary-container` / `on-secondary-container`): todo lo «seleccionado» y lo tonal: píldora del destino activo, elemento de lista activo, segmento elegido, botón tonal, botón de elegir archivo y avisos (`notice`).

### Tertiary
- **Cempasúchil** (`cempasuchil` / `cempasuchil-dark`): solo la barra del medidor cuando el puntaje es alto (≥ 70).
- **Cempasúchil suave** (`cempasuchil-container`): la píldora del puntaje alto y la etiqueta «Nueva». En ningún otro lugar.

### Estado
- **Error** (`error`, `error-container`): botones destructivos, alertas (`role="alert"`) y el aviso de IA no lista en el pie del cajón.
- **Éxito** (`success`, `success-container`, derivados de #2E7D4F): mensajes de estado (`role="status"`) y el icono de IA lista.

### Neutral
- **Papel azulado** (`surface`): fondo del contenido.
- **Escalera de contenedores** (`surface-container-lowest` → `-highest`): cajón y tarjetas en `-low`; columnas del Kanban y fichas de resumen en `surface-container`; tarjetas del Kanban en `-lowest`; pista del medidor y medidor bajo en `-highest`.
- **Tinta** (`on-surface`) y **tinta secundaria** (`on-surface-variant`): texto principal y secundario (etiquetas, metadatos, encabezados de tabla).
- **Contorno** (`outline`) para campos y botones delineados; **contorno suave** (`outline-variant`) para divisores de tabla y tarjetas delineadas.
- **Velo** (`scrim`, negro al 32 %): detrás de la hoja lateral modal del Tablero en ventanas angostas.

### Named Rules
**The Cempasúchil Rule.** El terciario existe para decir «esto encaja mucho» o «esto es nuevo». Nunca se usa en botones, navegación, encabezados ni decoración.

**The Tonal Selection Rule.** Lo seleccionado se marca con `secondary-container`, no con el principal. El talavera relleno es para la acción, no para el estado.

**The Paired Theme Rule.** Todo color se escribe con su variable `--md-*`, que tiene valor claro y oscuro. Ningún hex suelto en las pantallas.

**The Paper Rule.** La hoja del CV (`CvPaper`) representa el papel impreso: usa los tokens `--paper`, `--on-paper`, `--on-paper-variant`, `--paper-accent` y `--paper-rule`, que no cambian en tema oscuro. Sus medidas van en `cqi`, así la misma hoja es miniatura o vista grande. Usa Liberation Sans y tamaño carta, igual que la plantilla Typst (`crates/infra/templates/cv.typ`) y el DOCX: lo que se ve es lo que se descarga.

## Typography

**Display Font:** no hay; el título más grande es headline.
**Body Font:** Atkinson Hyperlegible Next Variable (con `system-ui, sans-serif`), autoalojada vía `@fontsource-variable`.
**Hoja del CV:** Liberation Sans, solo dentro de `CvPaper`, porque es la fuente incrustada en el PDF y el DOCX.
**Mono:** `ui-monospace, "Cascadia Mono", "DejaVu Sans Mono", monospace`, solo para `code`.

**Character:** una sola familia variable, hecha para lectores con baja visión; la jerarquía sale del peso (400 a 750) y del tamaño, no de una segunda familia.

### Hierarchy
- **Headline** (650, 28px / 36px, -0.01em): el título de cada pantalla, uno por pantalla, con `text-wrap: balance`.
- **Title large** (650, 22px / 28px): secciones dentro de una pantalla y título del detalle.
- **Title medium** (700, 17px / 24px): subsecciones y títulos de tarjeta.
- **Body** (400, 16px / 1.5): todo el texto corrido; la descripción de vacantes conserva saltos de línea.
- **Body small** (400, 15px / 22.4px): texto secundario, siempre en `on-surface-variant`. Es el tamaño mínimo para texto de lectura.
- **Label** (650, 15px, 0.01em): botones y segmentos. Las etiquetas de campo usan 15px a 500.
- **Label small** (650, 14px): encabezados de tabla, etiqueta «Nueva», pie del cajón.
- **Score** (750, 16px, cifras tabulares): el número del medidor; tablas y medidores usan `font-variant-numeric: tabular-nums`.

### Named Rules
**The 15px Floor Rule.** El texto que se lee no baja de 15px. Los 14px quedan para etiquetas cortas (encabezados de tabla, insignias, estado de la IA).

**The No Uppercase Rule.** Sin mayúsculas sostenidas ni tracking amplio: las etiquetas se escriben en tipo oración, como el resto de la interfaz.

## Layout

Un shell de dos columnas a pantalla completa: navegación a la izquierda y contenido con desplazamiento propio. El contenido tiene relleno de 32px arriba, `clamp(16px, 3vw, 40px)` a los lados y 48px abajo, y un ancho máximo de 1440px centrado. El orden de destinos es fijo: Perfiles, Vacantes, Optimizador, Postular, Tablero, Ajustes.

**Tamaños de ventana (clases de M3).** La ventana mínima es de 720px. En monitores grandes el ancho máximo del contenido sube a 1760px (≥ 1920px de ventana) y 2320px (≥ 2560px, ultrawide); cada pantalla usa el espacio extra con `@container main (min-width: 1600px)`.
- **≥ 1100px:** cajón de 256px.
- **720–1099px:** riel de 96px, con el icono en una píldora de 56×32 y la etiqueta debajo, a 15px.
- **Adaptación de pantallas:** `<main>` es un contenedor (`container: main / inline-size`) y cada pantalla se adapta con `@container main`, no con el ancho de la ventana.
  - Vacantes pasa a lista **o** detalle (con «Volver a la lista») por debajo de 820px.
  - Perfiles apila la lista arriba, en fila, por debajo de 760px.
  - Postular apila sus dos columnas por debajo de 820px.
  - La ficha del Tablero se vuelve hoja lateral modal con fondo atenuado (se cierra con Esc o tocando fuera) por debajo de 900px.
  - El Optimizador apila la hoja de vista previa bajo las propuestas por debajo de 1100px, la lista de «Mis CVs» sobre el CV elegido por debajo de 820px, y el antes/después de cada propuesta por debajo de 700px.

Cada pantalla abre con una cabecera en fila: título headline a la izquierda, selector de perfil a su lado y la acción principal (botón relleno) a la derecha, con `flex-wrap` para ventanas angostas. Las pantallas de lista y detalle (Vacantes) usan una rejilla `minmax(280px, 2fr) 3fr` con el detalle fijo (`sticky`). El Tablero muestra sus estados en una cuadrícula (`auto-fill, minmax(232px, 1fr)`) que llena el espacio sin desplazamiento lateral; una columna con muchas tarjetas se desplaza dentro. Abre una ficha lateral de 340px. El Optimizador usa dos rejillas: «Mis CVs» (lista con miniaturas `minmax(300px, 2fr)` y el CV elegido en grande `3fr`, fijo) y la revisión (propuestas a la izquierda y la hoja «Así quedará tu CV» fija a la derecha, de 320 a 440px). Ajustes tiene dos columnas que terminan a la misma altura y «Opciones avanzadas» a todo el ancho debajo.

El ritmo es de 4px: los huecos habituales son 4, 8, 12, 16, 20 y 24px; las separaciones entre bloques, 12 a 16px; entre secciones (h2), 32px.

## Elevation & Depth

Plano por defecto. La profundidad se expresa con la escalera de contenedores de M3 (de `surface` a `surface-container-highest`). Hay dos sombras, ambas teñidas con `--md-shadow` (negro azulado en claro, negro en oscuro), y solo aparecen en respuesta a un estado o en piezas que se pueden arrastrar.

### Shadow Vocabulary
- **Nivel 1** (`box-shadow: 0 1px 2px rgb(var(--md-shadow) / 0.3), 0 1px 3px 1px rgb(var(--md-shadow) / 0.15)`): hover de botones rellenos y tonales; estado de reposo de las tarjetas del Kanban.
- **Nivel 2** (`box-shadow: 0 1px 2px rgb(var(--md-shadow) / 0.3), 0 2px 6px 2px rgb(var(--md-shadow) / 0.15)`): hover de las tarjetas del Kanban.

### Named Rules
**The Tone-Not-Shadow Rule.** Un contenedor se separa del fondo cambiando de tono de superficie. Las sombras están reservadas para el hover y para lo que se arrastra.

## Shapes

Cinco radios, cada uno con su papel: 4px para campos y el anillo de foco; 8px para la etiqueta «Nueva»; 12px para avisos, alertas, elementos de lista, tarjetas del Kanban y la marca; 16px para tarjetas, estados vacíos y columnas del Kanban; píldora completa para botones, destinos de navegación, segmentos, fichas de resumen y el medidor. Los bordes son de 1px (`outline` en controles, `outline-variant` en divisores). La columna destino al arrastrar usa un borde discontinuo de 2px en talavera.

## Components

### Buttons
Píldoras firmes y amables, de 44px de alto.
- **Shape:** píldora completa (999px), relleno lateral 1.4rem.
- **Delineado (por defecto):** borde `outline` de 1px, texto talavera. Es el botón de cualquier acción secundaria.
- **Relleno (`.primary`):** fondo talavera, texto `on-talavera`. Una acción principal por bloque.
- **Tonal (`.tonal`):** `secondary-container`; también el aspecto del botón de elegir archivo.
- **Texto (`.link`):** sin borde, 40px de alto, relleno 0.75rem.
- **Peligro (`.danger`):** delineado con texto y borde en error; relleno en error para la confirmación final.
- **Hover / Active:** capa de estado M3 por `color-mix` (8% en hover, 12% al presionar) con transición de 150ms en `cubic-bezier(0.2, 0, 0, 1)`; rellenos y tonales suben a Nivel 1 en hover.
- **Disabled:** más legible que el 38 % de M3. El texto es `on-surface` mezclado al 58 % (62 % en rellenos) con la superficie, y el fondo al 10 %. El contraste queda en ≥ 3:1, porque el usuario debe poder leer qué acción falta.
- **Pressed:** escala 0.97 (0.99 en elementos de lista) durante 120ms.
- **Enlace con aspecto de botón:** los mismos estilos sobre `<a>` cuando la acción navega a otra pantalla.

### Segmented Button
Grupo de botones delineados unidos, con extremos en píldora. El segmento elegido (`aria-pressed` o `aria-selected`) toma `secondary-container` y una marca de verificación de 18px antepuesta. Se usa para Kanban / Lista.

### Chips
- **Fichas de resumen del Tablero:** píldoras en `surface-container` con texto `on-surface-variant` y la cifra en negrita `on-surface`. Solo informativas.
- **Etiqueta «Nueva»:** cempasúchil suave, 8px de radio, 14px a 700.

### Cards / Containers
- **Corner Style:** 16px.
- **Background:** `surface-container-low` (tarjeta rellena) o transparente con borde `outline-variant` (tarjeta delineada).
- **Shadow Strategy:** ninguna (ver Elevation & Depth).
- **Internal Padding:** 20px vertical, 24px horizontal.
- **Avisos:** `secondary-container`, 12px de radio. Alertas en `error-container` y estados en `success-container`, ambos con 12px de radio y texto a 600.
- **Estado vacío:** contenedor de 34rem como máximo en `surface-container-low`, con icono de 56px sobre talavera suave, un título, una frase y el botón del siguiente paso.

### Inputs / Fields
- **Style:** contorno de 1px en `outline`, fondo transparente, 4px de radio, 48px de alto mínimo. Los `select` llevan un chevron SVG propio.
- **Hover:** el borde pasa a `on-surface`.
- **Focus:** borde talavera más un trazo interior de 1px (borde visual de 2px), sin anillo exterior.
- **Disabled:** opacidad 0.6. Casillas de 18px con `accent-color` talavera.
- **Etiqueta:** encima del campo, 15px a 500 en `on-surface-variant`. Las etiquetas en línea (casillas) miden 40px de alto.

### Menú desplegable (`Select.svelte`)
Reemplaza a `<select>`, que en WebKitGTK abre una lista con el estilo de GTK.
- **Campo:** contorno de 4px de radio y 48px de alto, como los demás campos. Una flecha `expand_more` gira 180° al abrir. El borde se vuelve talavera al abrirse o con el foco.
- **Menú:** usa la API `popover`. Está en la capa superior, así que ningún panel lo recorta, y se cierra al hacer clic fuera, al desplazar la página o al cambiar el tamaño de la ventana.
  - Fondo `surface-container`, Nivel 2 y radio de 8px.
  - Opciones de 48px; la elegida en `secondary-container` con una marca `check`.
  - Entra en 200ms con `--ease-decel`, escalando desde 85 % de alto.
  - Se abre hacia arriba si no cabe abajo.
- **Teclado:** patrón de combobox de solo selección. Flechas, Inicio y Fin recorren; Enter o Espacio eligen; Esc cierra; una letra salta a la opción que empieza con ella.
- **Sin valor:** equivale a la opción vacía («Todos», «Por defecto»).

### Panel expandible (`<details>`)
Fila de 48px con radio de 12px y título a 650. La flecha `expand_more` va a la derecha y gira al abrir, pintada en talavera. Al pasar el ratón, capa de estado del 6 %; el foco va por dentro. El contenido entra con `rise-in`. Se usa para «Filtros», «Plazos de los recordatorios», el informe de la búsqueda y «Opciones avanzadas».

### Botón de icono
`.icon-btn`: círculo de 40px sin borde, con el icono en `on-surface-variant`. Deshabilitado al 38 %. Lo usan las flechas del Kanban.

### Kanban
- **Columnas:** 248px de ancho y alto según su contenido, con el título y un contador en píldora. Las vacías muestran un hueco punteado «Suelta aquí».
- **Navegación:** la rueda del ratón desplaza el tablero de lado, y al llegar al borde la deja pasar a la página. Las flechas ‹ › avanzan 260px.
- **Bordes:** el lado con más columnas se desvanece con `mask-image`.
- **Tarjetas:** cursor de agarrar.

### Fichas de entrada (habilidades)
- **Ficha:** 36px de alto, radio de 8px y borde `outline-variant` sobre `surface-container-lowest`. Contiene el nombre a 600, los años en tinta secundaria y una X de 32px para quitarla.
- **Edición:** al tocar el nombre, la ficha pasa a `secondary-container` y abre debajo un editor con nombre, nivel y años, sobre `surface-container`. Solo hay uno abierto a la vez.
- **Agregar:** un campo al pie; Enter agrega, admite varias separadas por comas e ignora las repetidas.

### Lista de respuestas
Filas separadas por divisores `outline-variant`: la pregunta en tinta secundaria y la respuesta a 650 en una línea, con botones de icono para copiar y editar. Solo se abre a edición la fila elegida.

### Paneles de pantalla
Cada bloque de una pantalla va en su propio panel `surface-container-low` con radio de 16px y 16px de separación entre bloques.
- **Vacantes:** barra de búsqueda y filtros, lista, y detalle. La búsqueda es un campo en píldora con lupa.
- **Dentro de un panel:** las secciones se separan con un divisor (`.block`), nunca con tarjetas anidadas.

### Cabecera de pantalla (`PageHead.svelte`)
Cuadro de 52px con radio de 16px y el icono de la sección en un contenedor tonal:
- `primary` en Perfiles y Vacantes;
- `tertiary` en Optimizador y Postular;
- `secondary` en Tablero y Ajustes.

Al aparecer, el cuadro escala y gira levemente (420ms). Al lado van el título (headline) y una línea que dice para qué sirve la pantalla, en tinta secundaria. Las acciones van a la derecha y bajan a una segunda fila si no caben.

### Perfil activo
En el cajón, bajo la marca, aparece el rótulo «Perfil activo» y un menú desplegable con radio de 12px sobre `surface-container-lowest`. En el riel se reduce a un círculo de 40px con la inicial, en `secondary-container`, que lleva a Perfiles. Es la única forma de elegir perfil: ninguna pantalla repite el selector.

### Avisos de navegación
Píldora `tertiary-container` (cempasúchil, porque es «lo nuevo») a la derecha del destino en el cajón, o sobre el icono en el riel. Entra con un pequeño rebote. Muestra:
- **Vacantes:** vacantes encontradas en las últimas 24 horas.
- **Postular:** vacantes por enviar.
- **Tablero:** recordatorios vencidos.

El número lleva su texto para lectores de pantalla.

### Onda (ripple)
Al pulsar un botón, un enlace-botón, un destino o una pestaña, nace un círculo de `currentColor` al 16 % donde tocas, que crece y se desvanece en 520ms. No aparece en menús ni con movimiento reducido.

### Pestañas de perfil
Fichas en píldora de 56px con una inicial en un círculo talavera de 40px, el nombre y la modalidad. La pestaña activa usa `secondary-container`.

### Densidad
- **Vacantes:** en ventanas anchas, la lista y el detalle llenan la altura de la ventana y se desplazan cada uno por su cuenta.
- **Perfiles:**
  - El CV se reparte en dos columnas: Datos básicos y Habilidades lado a lado, Experiencia a todo el ancho, y Educación, Certificaciones e Idiomas debajo.
  - La barra «CV maestro» se queda fija arriba al desplazar, con «Guardar CV» siempre a mano.
- **Ajustes:** la IA a la izquierda; la copia de seguridad y las opciones avanzadas a la derecha.

### Lists and Tables
- **Elemento de lista seleccionable:** fila a lo ancho, 12px de radio, sin borde; hover con capa del 8%; activo en `secondary-container`.
- **Tabla:** cifras tabulares, encabezados en label small `on-surface-variant`, divisores de 1px en `outline-variant`.

### Navigation
- **Cajón:** 256px sobre `surface-container-low`; marca arriba (cuadro talavera de 36px con 12px de radio y el icono `travel_explore`, nombre a 750); seis destinos; estado de la IA al pie.
- **Destino:** 52px de alto, píldora, icono de 24px y etiqueta a 16px / 600 en `on-surface-variant`.
- **Hover:** capa del 8% de `on-surface`, texto `on-surface`.
- **Activo:** píldora `secondary-container`, texto a 750 e icono en su variante rellena. La píldora es un elemento propio con `view-transition-name: nav-pill`, así que se desliza al nuevo destino en 340ms.
- **Riel (720–1099px):** el destino es una columna de 64px, con la píldora de 56×32 alrededor del icono y la etiqueta a 15px debajo. La marca y el estado de la IA quedan solo como icono, con su texto en `title` y para lectores de pantalla.
- **Foco:** anillo talavera de 3px dentro de la píldora (offset -3px).

### Icons
Material Symbols Rounded, peso 400, en SVG en línea (`viewBox 0 -960 960 960`, `fill: currentColor`). Decorativos por defecto (`aria-hidden`), porque el texto del control ya los nombra. 20px dentro de controles, 24px en la navegación. Las variantes `-fill` se reservan para el destino activo.

### Medidor de compatibilidad (firma)
El mismo componente en Vacantes, Optimizador (antes → después), Postular y Tablero.
- **Chip (listas):** columna de 3.25rem: píldora de 32px con el puntaje (16px / 750, cifras tabulares) y, debajo, una pista de 4px.
- **Fila (desgloses):** pista de 6px a todo lo ancho con el valor a la derecha («39 / 45», el máximo en peso 400 y tinta secundaria).
- **Tramos:** alto (≥ 70 %) en cempasúchil; medio (≥ 45 %) en talavera; bajo en `surface-container-highest` con barra `outline`.
- **Movimiento:** la barra crece desde 0 al montarse (500ms, `cubic-bezier(0.05, 0.7, 0.1, 1)`).

### Movimiento
- **Curvas:** `--ease` (estándar, `0.2, 0, 0, 1`), `--ease-decel` (enfatizada al llegar, `0.05, 0.7, 0.1, 1`) y `--ease-accel` (al salir, `0.3, 0, 0.8, 0.15`).
- **Cambio de pantalla:** View Transitions. El contenido sale en 100ms y entra en 260ms subiendo 12px. La barra lateral queda quieta y solo su píldora viaja. Sin soporte del motor, el cambio es instantáneo.
- **Entradas (`$lib/motion.ts`):**
  - `rise`: sube 10px y se funde en 260ms. Lo usan el detalle de la vacante (al cambiar de vacante), las tarjetas del kanban y la cola.
  - `stagger`: escalona las listas 28ms por elemento, con tope en el 8.º.
  - `sheet`: la ficha lateral entra desde la derecha.
  - Los avisos con `role=status/alert` entran con `rise-in` (240ms).
- **Carga:** siluetas `.skeleton` con un brillo lineal de 1,4s en lugar de «Cargando…».
- **Desplegables:** `<details>` se abre animando la altura donde el motor soporta `::details-content` e `interpolate-size`.
- **Movimiento reducido:** sin desplazamientos, escalas ni View Transitions. Quedan solo fundidos cortos (≤ 120ms) y cambios de color.
- **Accesibilidad:** `role="meter"` con `aria-valuenow`, `aria-valuemin`, `aria-valuemax` y una etiqueta.

## Do's and Don'ts

### Do:
- **Do** usar las variables `--md-*` para todo color, para que el tema oscuro funcione solo.
- **Do** mostrar cualquier puntaje de compatibilidad con el medidor (chip en listas, fila en desgloses), nunca como número suelto.
- **Do** marcar lo seleccionado con `secondary-container` y lo accionable principal con el botón relleno talavera, uno por bloque.
- **Do** mantener los controles en 40px de alto o más (botones 44px, campos 48px, destinos 52px) y el foco visible (3px talavera, offset 2px).
- **Do** usar 150ms con `--ease` para cambios de estado (color, borde, sombra), 240–340ms con `--ease-decel` para entradas y desplazamientos, y respetar `prefers-reduced-motion`.
- **Do** adaptar cada pantalla con `@container main`, nunca con el ancho de la ventana, para que el riel y las hojas laterales no rompan la cuenta.
- **Do** incluir fuentes e iconos dentro de la app: los iconos nuevos se añaden a `icons.ts` desde `@material-symbols/svg-400`.
- **Do** acompañar cada estado vacío del botón del siguiente paso.

### Don't:
- **Don't** usar cempasúchil fuera del medidor alto y de la etiqueta «Nueva».
- **Don't** volver al M3 de fábrica: nada de morado de referencia, Roboto ni tarjetas grises idénticas.
- **Don't** usar sombras para separar contenedores en reposo; solo las tarjetas que se arrastran las llevan.
- **Don't** añadir animaciones de entrada además de la barra del medidor.
- **Don't** cargar recursos externos (CDN de fuentes o iconos): la CSP los bloquea.
- **Don't** bajar el texto de lectura de 15px ni usar mayúsculas sostenidas.

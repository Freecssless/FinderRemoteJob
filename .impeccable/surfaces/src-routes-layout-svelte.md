---
version: 1
slug: "src-routes-layout-svelte"
primary_target: "src/routes/+layout.svelte"
related_targets: ["src/app.css"]
---

# Superficie: la app completa (shell + todas las pantallas)

Modo: Operate. Usuario no técnico de LATAM, sesiones cortas diarias: revisar vacantes nuevas, adaptar el CV y postularse, y saber en qué va cada postulación. Restricciones: no mover pantallas ni su orden (barra lateral izquierda: Perfiles, Vacantes, Optimizador, Postular, Tablero, Ajustes); texto y controles grandes; teclado completo; tema claro u oscuro según el sistema; recursos solo locales (CSP).

## Direction contract

THESIS: Material Design 3 fijado por el usuario, con identidad propia. Rechaza el M3 de fábrica: morado de referencia, Roboto y tarjetas grises idénticas.

OWN-WORLD: el principal es azul talavera (#1F50A0, el de la loza poblana) y el acento es cempasúchil (#8C5000 / #FFB873), reservado para la compatibilidad y lo nuevo. Superficies neutras de M3 con un ligero tinte azul. Tipografía Atkinson Hyperlegible Next, a 16px de base. Botones en píldora, campos con contorno, el cajón de navegación con la píldora de estado activo e iconos Material Symbols Rounded en SVG.

STORY: el usuario entiende de un vistazo en qué pantalla está y cuál es el siguiente paso. Encuentra vacantes, ve cuánto encajan, adapta su CV y se postula sin dudar.

FIRST VIEWPORT: un cajón de navegación de 248px a la izquierda, sobre surface-container-low, con la marca arriba, seis destinos con icono y etiqueta, y el estado de la IA al pie. El contenido va sobre surface. El título de la pantalla está en headline-small, y la acción principal es un botón relleno arriba a la derecha o al final del bloque.

FORM: Material Design 3 fijado por el usuario; no se lanzó ninguna ronda de conceptos porque la dirección vino fijada. Signature move: el medidor de compatibilidad, una píldora tonal con el puntaje y una barra lineal M3 (cempasúchil cuando encaja mucho), el mismo en Vacantes, Optimizador (antes → después) y Postular. seed: none (dirección fijada por el usuario, sin ronda)

FINISH: unreviewed and undocumented is unfinished; this build ends with the finish review, the verdict, DESIGN.md, and every shipping raster carrying its provenance

# Product

<!-- impeccable:product-schema 1 -->

## Platform

web

App de escritorio (Tauri 2 + Svelte 5) que se ve en el webview del sistema: Windows y Linux. Un futuro Android está planeado (Fase 3), sin fecha.

## Users

Personas hispanohablantes de México y Latinoamérica que buscan trabajo remoto. No tienen conocimientos técnicos y usan la app en su computadora de casa.

## Product Purpose

Ayudar a encontrar la mayor cantidad posible de vacantes remotas, adaptar el CV a cada una sin inventar datos y llevar el seguimiento de las postulaciones hasta la respuesta. El éxito es que el usuario se postule a más vacantes que encajan, con menos esfuerzo, y sepa siempre en qué va cada una.

## Positioning

Todo pasa en la computadora del usuario. Puede usar una IA local y gratis (Ollama), y sus datos personales nunca se envían a una IA externa. El CV adaptado solo reordena y redacta lo que el usuario ya tiene: un guardián bloquea cifras o habilidades inventadas.

## Operating Context

- **Uso:** sesiones cortas, unos minutos al día, para revisar vacantes nuevas, adaptar el CV a las que gustan y postularse. Al terminar, el usuario debe sentir que avanza, sin estrés.
- **Flujo principal:** Bienvenida (primer uso) → Perfiles (CV maestro) → Vacantes (búsqueda y compatibilidad) → Optimizador (CV adaptado, PDF/DOCX) → Postular (cola con «Abrir y postular») → Tablero (seguimiento por estado, con recordatorios).
- **Ajustes:** qué IA adapta los CV (en tu computadora, Claude, o Gemini/ChatGPT), copia de seguridad y opciones avanzadas cerradas.

## Capabilities and Constraints

- La interfaz está solo en español.
- La CSP solo permite recursos locales (`default-src 'self'`): las fuentes e iconos deben ir incluidos en la app, sin CDN.
- La IA local puede tardar varios minutos: las esperas largas deben explicarse.
- Las claves de API viven en el llavero del sistema, nunca en la base de datos.

## Brand Commitments

- Nombre: FinderRemoteJob.
- Voz: tú, cercana, frases cortas y sin jerga técnica. Si un término técnico es inevitable, se explica.
- Por petición del usuario, la interfaz se basa en Material Design 3 de Google.

## Evidence on Hand

No hay testimonios, métricas de uso ni logotipo propio, aparte del favicon por defecto en `static/favicon.png`. No se deben inventar.

## Product Principles

1. **Sencillo antes que completo:** lo técnico va escondido en «Opciones avanzadas». Si una función no ayuda a encontrar vacantes, adaptar el CV o seguir postulaciones, se quita.
2. **Siempre un siguiente paso claro:** cada pantalla dice qué hacer después.
3. **Privacidad por defecto:** los datos personales se quedan en el equipo.
4. **Nunca inventar:** ni en el CV ni en la interfaz.

## Accessibility & Inclusion

- Texto grande y legible, con alto contraste: muchos usuarios no son jóvenes ni técnicos. El mínimo es WCAG AA.
- Todo debe poder hacerse con teclado, con el foco siempre visible.

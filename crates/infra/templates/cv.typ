// Plantillas del CV (plan §4.6): Clásica, Compacta y Técnica. Una sola columna, sin tablas,
// íconos ni gráficos, nada en encabezado/pie de página, encabezados estándar y fuente incrustada.
// Los datos llegan como JSON en sys.inputs.data: los textos del CV son valores, nunca marcado.
// El orden de las secciones lo decide Rust (`cv.sections`); aquí solo se dibuja.
// El aspecto es el de la vista previa de la app (src/lib/components/CvPaper.svelte): mismos
// tamaños relativos, el azul talavera en titular y encabezados, y fechas a la derecha.
#let cv = json(bytes(sys.inputs.data))
#let compact = cv.template == "compacta"
#let ink = rgb("#1a1b21")
#let soft = rgb("#4a4e58")
#let accent = rgb("#1f50a0")
#let rule = rgb("#c3c6d3")

#set document(title: cv.name, author: cv.name)
#set page(paper: "us-letter", margin: if compact { (x: 1.5cm, y: 1.3cm) } else { (x: 1.9cm, y: 1.7cm) })
#set text(font: "Liberation Sans", size: if compact { 9.5pt } else { 10.5pt }, lang: cv.lang, fill: ink)
#set par(justify: false, leading: if compact { 0.45em } else { 0.55em }, spacing: if compact { 0.55em } else { 0.75em })
#set list(indent: 0.2em, body-indent: 0.5em, spacing: if compact { 0.3em } else { 0.4em })

#show heading.where(level: 1): it => block(
  above: if compact { 1em } else { 1.3em },
  below: if compact { 0.5em } else { 0.65em },
  width: 100%,
  stroke: (bottom: 0.6pt + rule),
  inset: (bottom: 0.3em),
  text(size: if compact { 10pt } else { 11pt }, weight: "bold", fill: accent, it.body),
)

// Línea con el título a la izquierda y las fechas a la derecha; debajo, el lugar en gris.
#let entry(title, dates, place) = block(above: if compact { 0.7em } else { 0.95em }, below: if compact { 0.3em } else { 0.4em })[
  #grid(columns: (1fr, auto), column-gutter: 1em, text(weight: "bold", title), text(fill: soft, dates))
  #if place != "" { v(-0.35em); text(fill: soft, place) }
]

// Nombre, titular y contacto en el cuerpo del documento.
#text(size: 2em, weight: "bold", cv.name)
#if cv.headline != "" { v(-0.3em); text(size: 1.15em, weight: "bold", fill: accent, cv.headline) }
#if cv.contact.len() > 0 { v(-0.2em); text(fill: soft, cv.contact.join("  ·  ")) }

#for s in cv.sections {
  heading(level: 1, cv.headings.at(s))
  if s == "summary" {
    cv.summary
  } else if s == "experience" {
    for e in cv.experience {
      entry(e.title, e.dates, e.company)
      list(..e.bullets)
    }
  } else if s == "education" {
    for e in cv.education { entry(e.degree, e.dates, e.institution) }
  } else if s == "skills" {
    cv.skills.join("  ·  ")
  } else {
    list(..cv.at(s))
  }
}

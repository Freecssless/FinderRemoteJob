// Plantillas del CV (plan §4.6): Clásica, Compacta y Técnica. Una sola columna, sin tablas,
// íconos ni gráficos, nada en encabezado/pie de página, encabezados estándar y fuente incrustada.
// Los datos llegan como JSON en sys.inputs.data: los textos del CV son valores, nunca marcado.
// El orden de las secciones lo decide Rust (`cv.sections`); aquí solo se dibuja.
#let cv = json(bytes(sys.inputs.data))
#let compact = cv.template == "compacta"

#set document(title: cv.name, author: cv.name)
#set page(paper: "us-letter", margin: if compact { (x: 1.5cm, y: 1.3cm) } else { (x: 1.9cm, y: 1.7cm) })
#set text(font: "Liberation Sans", size: if compact { 9.5pt } else { 10.5pt }, lang: cv.lang)
#set par(justify: false, leading: if compact { 0.45em } else { 0.55em }, spacing: if compact { 0.55em } else { 0.75em })
#set list(indent: 0.4em, body-indent: 0.5em, spacing: if compact { 0.3em } else { 0.45em })

#show heading.where(level: 1): it => block(
  above: if compact { 0.85em } else { 1.15em },
  below: if compact { 0.4em } else { 0.55em },
  width: 100%,
)[
  #text(size: if compact { 10.5pt } else { 11.5pt }, weight: "bold", upper(it.body))
  #v(-0.45em)
  #line(length: 100%, stroke: 0.5pt)
]

// Nombre, titular y contacto en el cuerpo del documento.
#text(size: if compact { 15pt } else { 18pt }, weight: "bold", cv.name) \
#if cv.headline != "" [#text(size: if compact { 10pt } else { 11pt }, cv.headline) \ ]
#cv.contact.join(" | ")

#for s in cv.sections {
  heading(level: 1, cv.headings.at(s))
  if s == "summary" {
    cv.summary
  } else if s == "experience" {
    for e in cv.experience [
      #block(above: if compact { 0.65em } else { 0.9em }, below: if compact { 0.3em } else { 0.4em })[
        #text(weight: "bold", e.title) — #e.company \
        #text(style: "italic", e.dates)
      ]
      #list(..e.bullets)
    ]
  } else if s == "education" {
    for e in cv.education [
      #text(weight: "bold", e.degree) — #e.institution#if e.dates != "" [, #e.dates] \
    ]
  } else {
    cv.at(s).join(", ")
  }
}

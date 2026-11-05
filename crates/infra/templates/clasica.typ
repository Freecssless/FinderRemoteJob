// Plantilla "Clásica" (plan §4.6): una sola columna, sin tablas, íconos ni gráficos,
// nada en encabezado/pie de página, encabezados estándar y fuente incrustada.
// Los datos llegan como JSON en sys.inputs.data: los textos del CV son valores, nunca marcado.
#let cv = json(bytes(sys.inputs.data))

#set document(title: cv.name, author: cv.name)
#set page(paper: "us-letter", margin: (x: 1.9cm, y: 1.7cm))
#set text(font: "Liberation Sans", size: 10.5pt, lang: cv.lang)
#set par(justify: false, leading: 0.55em, spacing: 0.75em)
#set list(indent: 0.4em, body-indent: 0.5em, spacing: 0.45em)

#show heading.where(level: 1): it => block(above: 1.15em, below: 0.55em, width: 100%)[
  #text(size: 11.5pt, weight: "bold", upper(it.body))
  #v(-0.45em)
  #line(length: 100%, stroke: 0.5pt)
]

#let section(title, body) = [
  #heading(level: 1, title)
  #body
]

// Nombre, titular y contacto en el cuerpo del documento.
#text(size: 18pt, weight: "bold", cv.name) \
#if cv.headline != "" [#text(size: 11pt, cv.headline) \ ]
#cv.contact.join(" | ")

#if cv.summary != none {
  section(cv.headings.summary, cv.summary)
}

#if cv.experience.len() > 0 {
  section(cv.headings.experience, for e in cv.experience [
    #block(above: 0.9em, below: 0.4em)[
      #text(weight: "bold", e.title) — #e.company \
      #text(style: "italic", e.dates)
    ]
    #list(..e.bullets)
  ])
}

#if cv.education.len() > 0 {
  section(cv.headings.education, for e in cv.education [
    #text(weight: "bold", e.degree) — #e.institution#if e.dates != "" [, #e.dates] \
  ])
}

#for (title, items) in (
  (cv.headings.skills, cv.skills),
  (cv.headings.certifications, cv.certifications),
  (cv.headings.languages, cv.languages),
) {
  if items.len() > 0 {
    section(title, items.join(", "))
  }
}

# Política de privacidad de FinderRemoteJob

Versión 2026-11-08 · Última actualización: 8 de noviembre de 2026.

FinderRemoteJob es una aplicación de escritorio gratuita y de código abierto que te ayuda a buscar
empleo remoto, adaptar tu CV y dar seguimiento a tus postulaciones. La desarrolla **freecss**
(«nosotros»). Esta política explica qué datos usa la app, dónde se guardan, con quién se comparten
y cómo puedes borrarlos.

**En resumen:** la app no tiene cuentas ni servidores propios. Tus datos se guardan en tu
computadora y nosotros no los recibimos nunca. Solo salen de tu equipo hacia los servicios que tú
eliges usar, en los casos que se explican abajo.

## 1. Qué datos usa la app

| Dato | Para qué | Dónde se guarda |
|---|---|---|
| Tu CV: nombre, titular, contacto (correo, teléfono, ubicación, enlaces), experiencia, estudios, habilidades, certificaciones e idiomas | Leerlo, compararlo con vacantes, adaptarlo y generar el PDF/DOCX | Base de datos local de la app |
| Perfiles de búsqueda: nombre del perfil, puestos buscados, preferencia de trabajo remoto | Buscar y ordenar vacantes | Base de datos local |
| Vacantes, postulaciones, notas, recordatorios y respuestas frecuentes | Tu tablero de seguimiento | Base de datos local |
| CVs generados (PDF y DOCX) | Que los envíes a las empresas | Carpeta `cvs/` de la app |
| Claves de API de Claude, Gemini o ChatGPT (si las pegas) | Usar esa IA | Llavero cifrado del sistema operativo (Windows Credential Manager, GNOME Keyring/KWallet, Keychain), nunca en la base de datos |
| Ajustes (qué IA usar, modelos, pesos de compatibilidad) | Recordar tus preferencias | Base de datos local |
| Registro de errores: fecha, versión, sistema y mensajes técnicos de error | Que puedas reportar un fallo | Archivo `registro.log` de la app |

**No recopilamos:** analítica de uso, identificadores de dispositivo, ubicación, contactos, fotos,
cámara, micrófono, historial de navegación ni cookies. La app no muestra publicidad.

La carpeta de datos de la app está en:

- Windows: `%APPDATA%\com.finderremotejob.app`
- Linux: `~/.local/share/com.finderremotejob.app` (solo tu usuario puede leerla)
- macOS: `~/Library/Application Support/com.finderremotejob.app`

## 2. Con quién se comparten

Nosotros no recibimos ninguno de tus datos. La app se conecta a internet solo en estos casos:

| Destino | Cuándo | Qué recibe |
|---|---|---|
| Portales de empleo: Remote OK, Remotive, We Work Remotely, Get on Board, Jobicy, Himalayas, Greenhouse, Lever y Ashby | Al buscar vacantes | Los puestos que buscas y tu dirección IP. **Nunca tu CV.** |
| La página de una vacante cuyo enlace pegas | Al importar ese enlace | Tu dirección IP |
| Ollama, en tu propia computadora | Al leer tu CV, analizar vacantes y (por defecto) adaptar tu CV | Se procesa en tu equipo: no sale a internet |
| Anthropic (Claude), Google (Gemini) u OpenAI (ChatGPT) | Solo si eliges una de ellas en Ajustes **y aceptas** el aviso de envío | El texto de la vacante, tu titular, tu experiencia (puestos, empresas, fechas y logros) y tus habilidades. **Nunca** tu nombre, correo, teléfono, ubicación, enlaces ni estudios. Viaja cifrado (HTTPS) y lo tratan según sus propias políticas de privacidad. |
| ollama.com y GitHub | Solo al pulsar «Instalar Ollama» o «Descargar los modelos» | Tu dirección IP |
| GitHub | Solo si pulsas «Reportar un problema» y envías el reporte | Lo que tú escribas y decidas enviar. El reporte se abre rellenado en tu navegador para que lo revises antes. |

No vendemos ni cedemos datos a nadie, y no hay kits de terceros (SDK) de analítica, anuncios ni
reporte automático de fallos dentro de la app.

## 3. Consentimiento y permisos

- **Al empezar**, la app te pide aceptar esta política y los términos de uso.
- **Antes de enviar textos a una IA en internet**, Ajustes te dice exactamente qué se envía y a
  qué empresa, y no se guarda la elección hasta que marques la casilla. Si cambias de proveedor, se
  te vuelve a preguntar. Puedes volver a la IA local cuando quieras.
- **Archivos:** la app solo lee el archivo que tú eliges (tu CV o un respaldo).
- **Notificaciones:** solo se usan para los recordatorios que tú creas en el Tablero. Puedes
  desactivarlas en la configuración de tu sistema.
- **Cámara, micrófono, ubicación, contactos y cookies:** la app no los usa ni los pide.

## 4. Cuánto tiempo se guardan

Mientras tú quieras: no se borran solos. El registro de errores se recorta automáticamente cuando
pasa de 512 KB.

## 5. Cómo ver, corregir o borrar tus datos

- **Ver y corregir:** todo está en la app (Perfiles, Vacantes, Optimizador, Tablero, Ajustes).
- **Llevártelos:** Ajustes → «Crear copia de seguridad» genera un `.zip` con tu base de datos y tus
  CVs.
- **Borrar una parte:** puedes borrar perfiles (con su CV) en Perfiles y tus claves de API en
  Ajustes.
- **Borrar todo:** «Privacidad y ayuda» → «Borrar todos mis datos». Borra la base de datos, los CVs
  generados, el registro de errores y tus claves del llavero, y reinicia la app vacía. No toca las
  copias de seguridad que guardaste en Documentos ni los modelos de Ollama; esos bórralos tú.
- **Datos en servicios externos:** si usaste Claude, Gemini o ChatGPT, lo que se les envió está
  sujeto a sus políticas. Pídeles a ellos el borrado.

Como nosotros no tenemos tus datos, no podemos verlos, entregarlos ni borrarlos por ti: los
controlas tú desde la app. Esto cubre tus derechos de acceso, rectificación, cancelación y
oposición (derechos ARCO, en México) y equivalentes en otros países.

## 6. Seguridad

- Las claves de API van cifradas en el llavero del sistema.
- Todo envío a internet usa HTTPS (salvo a un servidor en tu propia computadora).
- En Linux y macOS la carpeta de datos y los respaldos solo los puede leer tu usuario.
- El código es público y se revisa con análisis de seguridad antes de cada versión.

Aun así, la seguridad de tus datos depende también de tu equipo: protégelo con contraseña y, si
puedes, activa el cifrado de disco del sistema.

## 7. Menores de edad

La app está pensada para personas mayores de 18 años que buscan empleo.

## 8. Cambios a esta política

Si la cambiamos, actualizaremos la versión y la fecha de arriba, y la app te mostrará un aviso para
que la revises y la aceptes de nuevo.

## 9. Contacto

Para dudas, reportes o solicitudes sobre privacidad, abre un reporte en
<https://github.com/Freecssless/FinderRemoteJob/issues> (o desde la app: «Privacidad y ayuda» →
«Reportar contenido o privacidad»). No incluyas tu CV ni datos personales en el reporte: es público.

# Propiedad intelectual y licencias de terceros

Última revisión: 8 de noviembre de 2026.

## FinderRemoteJob

© 2026 freecss. El código de FinderRemoteJob es software libre bajo **MIT o Apache-2.0**, a tu
elección (ver [LICENSE-MIT](LICENSE-MIT) y [LICENSE-APACHE](LICENSE-APACHE)). Puedes usarlo,
copiarlo, modificarlo y distribuirlo, también con fines comerciales, siempre que conserves el aviso
de copyright y la licencia. Se entrega **sin garantías** de ningún tipo.

El nombre «FinderRemoteJob» y su icono identifican a este proyecto: si publicas una versión
modificada, usa otro nombre para que nadie la confunda con la original.

## Resumen de licencias

| Tipo | Qué permite | ¿Está en la app? |
|---|---|---|
| MIT, Apache-2.0, BSD, ISC, Zlib, Unlicense, 0BSD, CC0, BSL-1.0, Unicode-3.0 | Uso libre, también comercial; hay que conservar el aviso de copyright. Apache-2.0 añade una licencia de patentes. | Sí: casi todas las librerías. |
| MPL-2.0 | Como las anteriores, pero si **modificas** un archivo con esta licencia debes publicar ese archivo. | Sí: 7 librerías para leer HTML (`cssparser`, `selectors`, `dtoa-short`, `option-ext`). No se modifican. |
| SIL Open Font License 1.1 (OFL) | Usar, incrustar y redistribuir la fuente; no venderla sola ni usar su nombre reservado en una versión modificada. | Sí: Liberation Sans y Atkinson Hyperlegible Next, sin modificar. |
| GPL, LGPL, AGPL (copyleft) | Obligarían a publicar el código de toda la app bajo la misma licencia. | **No.** Revisado: ninguna librería de la app usa estas licencias. |
| Uso personal / no comercial / propietaria | Prohíben o limitan la distribución. | **No.** Ningún recurso de la app tiene estas licencias. |

## Recursos visuales

| Recurso | Autor | Licencia | Uso |
|---|---|---|---|
| Iconos de la interfaz: Material Symbols Rounded | Google | Apache-2.0 | Trazados SVG en `src/lib/icons.ts`. |
| Icono de la app (`src-tauri/icons/icon.svg`) | freecss, a partir de Material Symbols «travel_explore» | Apache-2.0 (icono base) | Icono de ventana, instalador y favicon. |
| Fuente de la interfaz: Atkinson Hyperlegible Next | Braille Institute of America | OFL-1.1 | Paquete `@fontsource-variable/atkinson-hyperlegible-next`. |
| Fuente de los CV: Liberation Sans | Red Hat (datos de Google: Arimo) | OFL-1.1 ([texto](crates/infra/assets/fonts/LICENSE-LiberationSans.txt)) | Incrustada en los PDF; vista previa del CV. |

La app no incluye fotografías, ilustraciones, sonidos ni otras imágenes de terceros.

## Contenido que no es de este proyecto

- **Vacantes:** los anuncios pertenecen a las empresas que los publican y a los portales de donde
  vienen (Remote OK, Remotive, We Work Remotely, Get on Board, Jobicy, Himalayas, Greenhouse, Lever y
  Ashby). La app los muestra para tu uso personal, siempre con el nombre del portal y un enlace al
  anuncio original. No los republiques como propios.
- **Modelos de IA:** la app no incluye ningún modelo. Si los descargas con Ollama, cada uno tiene su
  licencia (por defecto `qwen3:8b`, Apache-2.0, y `bge-m3`, MIT). Claude, Gemini y ChatGPT se rigen
  por los términos de Anthropic, Google y OpenAI.
- **Tu CV y tus textos** son tuyos. La app no reclama ningún derecho sobre ellos.

## Librerías de JavaScript dentro de la app

| Librería | Versión | Licencia |
|---|---|---|
| [svelte](https://github.com/sveltejs/svelte) | 5.57.3 | MIT |
| [@sveltejs/kit](https://github.com/sveltejs/kit) | 2.70.4 | MIT |
| [@tauri-apps/api](https://github.com/tauri-apps/tauri) | 2.12.4 | Apache-2.0 OR MIT |
| [@tauri-apps/plugin-opener](https://github.com/tauri-apps/plugins-workspace) | 2.7.2 | MIT OR Apache-2.0 |
| [@fontsource-variable/atkinson-hyperlegible-next](https://github.com/fontsource/font-files) | 5.3.0 | OFL-1.1 |

Las herramientas de compilación (Vite, TypeScript, svelte-check, la CLI de Tauri) no se incluyen en
la app instalada.

## Lista completa de librerías Rust (696)

Generada de `cargo metadata` (lo que se compila dentro de la app en Windows y Linux).

| Librería | Versión | Licencia |
|---|---|---|
| [adler2](https://github.com/oyvindln/adler2) | 2.0.1 | 0BSD OR MIT OR Apache-2.0 |
| [adobe-cmap-parser](https://github.com/jrmuizel/adobe-cmap-parser) | 0.4.1 | MIT |
| [aes](https://github.com/RustCrypto/block-ciphers) | 0.8.4 | MIT OR Apache-2.0 |
| [aho-corasick](https://github.com/BurntSushi/aho-corasick) | 1.1.5 | Unlicense OR MIT |
| [alloc-no-stdlib](https://github.com/dropbox/rust-alloc-no-stdlib) | 3.0.0 | BSD-3-Clause |
| [alloc-stdlib](https://github.com/dropbox/rust-alloc-no-stdlib) | 0.3.0 | BSD-3-Clause |
| [allocator-api2](https://github.com/zakarumych/allocator-api2) | 0.2.21 | MIT OR Apache-2.0 |
| [anyhow](https://github.com/dtolnay/anyhow) | 1.0.104 | MIT OR Apache-2.0 |
| [approx](https://github.com/brendanzab/approx) | 0.5.1 | Apache-2.0 |
| [ar_archive_writer](https://github.com/rust-lang/ar_archive_writer) | 0.5.3 | Apache-2.0 WITH LLVM-exception |
| [arrayref](https://github.com/droundy/arrayref) | 0.3.9 | BSD-2-Clause |
| [arrayvec](https://github.com/bluss/arrayvec) | 0.7.8 | MIT OR Apache-2.0 |
| [async-broadcast](https://github.com/smol-rs/async-broadcast) | 0.7.2 | MIT OR Apache-2.0 |
| [async-channel](https://github.com/smol-rs/async-channel) | 2.5.0 | Apache-2.0 OR MIT |
| [async-executor](https://github.com/smol-rs/async-executor) | 1.14.1 | Apache-2.0 OR MIT |
| [async-io](https://github.com/smol-rs/async-io) | 2.6.0 | Apache-2.0 OR MIT |
| [async-lock](https://github.com/smol-rs/async-lock) | 3.5.0 | Apache-2.0 OR MIT |
| [async-process](https://github.com/smol-rs/async-process) | 2.5.0 | Apache-2.0 OR MIT |
| [async-recursion](https://github.com/dcchut/async-recursion) | 1.2.0 | MIT OR Apache-2.0 |
| [async-signal](https://github.com/smol-rs/async-signal) | 0.2.14 | Apache-2.0 OR MIT |
| [async-task](https://github.com/smol-rs/async-task) | 4.7.1 | Apache-2.0 OR MIT |
| [async-trait](https://github.com/dtolnay/async-trait) | 0.1.92 | MIT OR Apache-2.0 |
| [atk](https://github.com/gtk-rs/gtk3-rs) | 0.18.2 | MIT |
| [atk-sys](https://github.com/gtk-rs/gtk3-rs) | 0.18.2 | MIT |
| [atoi](https://github.com/pacman82/atoi-rs) | 2.0.0 | MIT |
| [atomic-waker](https://github.com/smol-rs/atomic-waker) | 1.1.2 | Apache-2.0 OR MIT |
| [autocfg](https://github.com/cuviper/autocfg) | 1.5.1 | Apache-2.0 OR MIT |
| [aws-lc-rs](https://github.com/aws/aws-lc-rs) | 1.19.0 | ISC AND (Apache-2.0 OR ISC) |
| [aws-lc-sys](https://github.com/aws/aws-lc-rs) | 0.47.0 | ISC AND (Apache-2.0 OR ISC) AND Apache-2.0 AND MIT AND BSD-3-Clause AND (Apache-2.0 OR ISC OR MIT) AND (Apache-2.0 OR ISC OR MIT-0) |
| [az](https://gitlab.com/tspiteri/az) | 1.3.0 | MIT/Apache-2.0 |
| [base64](https://github.com/marshallpierce/rust-base64) | 0.22.1 | MIT OR Apache-2.0 |
| [base64](https://github.com/marshallpierce/rust-base64) | 0.23.1 | MIT OR Apache-2.0 |
| [biblatex](https://github.com/typst/biblatex) | 0.12.0 | MIT OR Apache-2.0 |
| [bincode](https://github.com/servo/bincode) | 1.3.3 | MIT |
| [bit-set](https://github.com/contain-rs/bit-set) | 0.8.0 | Apache-2.0 OR MIT |
| [bit-vec](https://github.com/contain-rs/bit-vec) | 0.8.0 | Apache-2.0 OR MIT |
| [bitflags](https://github.com/bitflags/bitflags) | 1.3.2 | MIT/Apache-2.0 |
| [bitflags](https://github.com/bitflags/bitflags) | 2.13.2 | MIT OR Apache-2.0 |
| [block-buffer](https://github.com/RustCrypto/utils) | 0.10.4 | MIT OR Apache-2.0 |
| [block-buffer](https://github.com/RustCrypto/utils) | 0.12.1 | MIT OR Apache-2.0 |
| [block-padding](https://github.com/RustCrypto/utils) | 0.3.3 | MIT OR Apache-2.0 |
| [blocking](https://github.com/smol-rs/blocking) | 1.7.0 | Apache-2.0 OR MIT |
| [brotli](https://github.com/dropbox/rust-brotli) | 9.1.1 | BSD-3-Clause AND MIT |
| [brotli-decompressor](https://github.com/dropbox/rust-brotli-decompressor) | 6.2.0 | BSD-3-Clause/MIT |
| [bs58](https://github.com/Nullus157/bs58-rs) | 0.5.1 | MIT/Apache-2.0 |
| [bumpalo](https://github.com/fitzgen/bumpalo) | 3.20.3 | MIT OR Apache-2.0 |
| [by_address](https://github.com/mbrubeck/by_address) | 1.2.1 | MIT OR Apache-2.0 |
| [bytemuck](https://github.com/Lokathor/bytemuck) | 1.25.2 | Zlib OR Apache-2.0 OR MIT |
| [bytemuck_derive](https://github.com/Lokathor/bytemuck) | 1.12.1 | Zlib OR Apache-2.0 OR MIT |
| [byteorder](https://github.com/BurntSushi/byteorder) | 1.5.0 | Unlicense OR MIT |
| [byteorder-lite](https://github.com/image-rs/byteorder-lite) | 0.1.0 | Unlicense OR MIT |
| [bytes](https://github.com/tokio-rs/bytes) | 1.12.1 | MIT |
| [cairo-rs](https://github.com/gtk-rs/gtk-rs-core) | 0.18.5 | MIT |
| [cairo-sys-rs](https://github.com/gtk-rs/gtk-rs-core) | 0.18.2 | MIT |
| [camino](https://github.com/camino-rs/camino) | 1.2.6 | MIT OR Apache-2.0 |
| [cargo-platform](https://github.com/rust-lang/cargo) | 0.1.9 | MIT OR Apache-2.0 |
| [cargo_metadata](https://github.com/oli-obk/cargo_metadata) | 0.19.2 | MIT |
| [cargo_toml](https://gitlab.com/lib.rs/cargo_toml) | 1.0.1 | Apache-2.0 OR MIT |
| [cbc](https://github.com/RustCrypto/block-modes) | 0.1.2 | MIT OR Apache-2.0 |
| [cc](https://github.com/rust-lang/cc-rs) | 1.11.0 | MIT OR Apache-2.0 |
| [cfb](https://github.com/mdsteele/rust-cfb) | 0.14.0 | MIT |
| [cff-parser](https://github.com/jrmuizel/cff-parser) | 0.2.0 | MIT OR Apache-2.0 |
| [cfg-expr](https://github.com/EmbarkStudios/cfg-expr) | 0.15.8 | MIT OR Apache-2.0 |
| [cfg-if](https://github.com/rust-lang/cfg-if) | 1.0.5 | MIT OR Apache-2.0 |
| [cfg_aliases](https://github.com/katharostech/cfg_aliases) | 0.2.2 | MIT |
| [chacha20](https://github.com/RustCrypto/stream-ciphers) | 0.10.2 | MIT OR Apache-2.0 |
| [chinese-number](https://github.com/magiclen/chinese-number) | 0.7.8 | MIT |
| [chinese-variant](https://github.com/magiclen/chinese-variant) | 1.1.6 | MIT |
| [chrono](https://github.com/chronotope/chrono) | 0.4.45 | MIT OR Apache-2.0 |
| [ciborium](https://github.com/enarx/ciborium) | 0.2.2 | Apache-2.0 |
| [ciborium-io](https://github.com/enarx/ciborium) | 0.2.2 | Apache-2.0 |
| [ciborium-ll](https://github.com/enarx/ciborium) | 0.2.2 | Apache-2.0 |
| [cipher](https://github.com/RustCrypto/traits) | 0.4.4 | MIT OR Apache-2.0 |
| [citationberg](https://github.com/typst/citationberg) | 0.7.0 | MIT OR Apache-2.0 |
| [cmake](https://github.com/rust-lang/cmake-rs) | 0.1.58 | MIT OR Apache-2.0 |
| [cmov](https://github.com/RustCrypto/utils) | 0.5.4 | Apache-2.0 OR MIT |
| [cobs](https://github.com/jamesmunns/cobs.rs) | 0.3.0 | MIT OR Apache-2.0 |
| [codex](https://github.com/typst/codex) | 0.3.0 | Apache-2.0 |
| [color](https://github.com/linebender/color) | 0.3.3 | Apache-2.0 OR MIT |
| [color_quant](https://github.com/image-rs/color_quant.git) | 1.1.0 | MIT |
| [comemo](https://github.com/typst/comemo) | 0.5.1 | MIT OR Apache-2.0 |
| [comemo-macros](https://github.com/typst/comemo) | 0.5.1 | MIT OR Apache-2.0 |
| [concurrent-queue](https://github.com/smol-rs/concurrent-queue) | 2.5.0 | Apache-2.0 OR MIT |
| [constant_time_eq](https://codeberg.org/cesarb/constant_time_eq) | 0.6.1 | CC0-1.0 OR MIT-0 OR Apache-2.0 |
| [cookie](https://github.com/SergioBenitez/cookie-rs) | 0.18.2 | MIT OR Apache-2.0 |
| [core_detect](https://github.com/thomcc/core_detect) | 1.0.0 | MIT/Apache-2.0 |
| [core_maths](https://github.com/robertbastian/core_maths) | 0.1.1 | MIT |
| [cpufeatures](https://github.com/RustCrypto/utils) | 0.2.17 | MIT OR Apache-2.0 |
| [cpufeatures](https://github.com/RustCrypto/utils) | 0.3.1 | MIT OR Apache-2.0 |
| [crc](https://github.com/mrhooray/crc-rs.git) | 3.4.0 | MIT OR Apache-2.0 |
| [crc-catalog](https://github.com/akhilles/crc-catalog.git) | 2.5.0 | MIT OR Apache-2.0 |
| [crc32fast](https://github.com/srijs/rust-crc32fast) | 1.5.4 | MIT OR Apache-2.0 |
| [crossbeam-channel](https://github.com/crossbeam-rs/crossbeam) | 0.5.17 | MIT OR Apache-2.0 |
| [crossbeam-deque](https://github.com/crossbeam-rs/crossbeam) | 0.8.8 | MIT OR Apache-2.0 |
| [crossbeam-epoch](https://github.com/crossbeam-rs/crossbeam) | 0.9.21 | MIT OR Apache-2.0 |
| [crossbeam-queue](https://github.com/crossbeam-rs/crossbeam) | 0.3.14 | MIT OR Apache-2.0 |
| [crossbeam-utils](https://github.com/crossbeam-rs/crossbeam) | 0.8.23 | MIT OR Apache-2.0 |
| [crypto-common](https://github.com/RustCrypto/traits) | 0.1.6 | MIT OR Apache-2.0 |
| [crypto-common](https://github.com/RustCrypto/traits) | 0.2.2 | MIT OR Apache-2.0 |
| [cssparser](https://github.com/servo/rust-cssparser) | 0.37.0 | MPL-2.0 |
| [cssparser](https://github.com/servo/rust-cssparser) | 0.38.0 | MPL-2.0 |
| [cssparser-macros](https://github.com/servo/rust-cssparser) | 0.7.1 | MPL-2.0 |
| [csv](https://github.com/BurntSushi/rust-csv) | 1.4.0 | Unlicense/MIT |
| [csv-core](https://github.com/BurntSushi/rust-csv) | 0.1.13 | Unlicense/MIT |
| [ctor](https://github.com/mmastrac/linktime) | 1.0.13 | Apache-2.0 OR MIT |
| [ctutils](https://github.com/RustCrypto/utils) | 0.4.8 | Apache-2.0 OR MIT |
| [darling](https://github.com/TedDriggs/darling) | 0.24.1 | MIT |
| [darling_core](https://github.com/TedDriggs/darling) | 0.24.1 | MIT |
| [darling_macro](https://github.com/TedDriggs/darling) | 0.24.1 | MIT |
| [data-url](https://github.com/servo/rust-url) | 0.3.2 | MIT OR Apache-2.0 |
| [dbus](https://github.com/diwic/dbus-rs) | 0.9.12 | Apache-2.0/MIT |
| [dbus-secret-service](https://github.com/brotskydotcom/dbus-secret-service.git) | 4.1.0 | MIT OR Apache-2.0 |
| [defmt](https://github.com/knurling-rs/defmt) | 1.2.0 | MIT OR Apache-2.0 |
| [defmt-macros](https://github.com/knurling-rs/defmt) | 1.2.0 | MIT OR Apache-2.0 |
| [defmt-parser](https://github.com/knurling-rs/defmt) | 1.1.0 | MIT OR Apache-2.0 |
| [deranged](https://github.com/jhpratt/deranged) | 0.5.9 | MIT OR Apache-2.0 |
| [derive_more](https://github.com/JelteF/derive_more) | 2.1.1 | MIT |
| [derive_more-impl](https://github.com/JelteF/derive_more) | 2.1.1 | MIT |
| [deunicode](https://github.com/kornelski/deunicode/) | 1.6.2 | BSD-3-Clause |
| [digest](https://github.com/RustCrypto/traits) | 0.10.7 | MIT OR Apache-2.0 |
| [digest](https://github.com/RustCrypto/traits) | 0.11.3 | MIT OR Apache-2.0 |
| [dirs](https://github.com/soc/dirs-rs) | 6.0.0 | MIT OR Apache-2.0 |
| [dirs](https://codeberg.org/dirs/dirs-rs) | 7.0.0 | MIT OR Apache-2.0 |
| [dirs-sys](https://github.com/dirs-dev/dirs-sys-rs) | 0.5.0 | MIT OR Apache-2.0 |
| [displaydoc](https://github.com/yaahc/displaydoc) | 0.2.7 | MIT OR Apache-2.0 |
| [dlopen2](https://github.com/OpenByteDev/dlopen2) | 0.8.2 | MIT |
| [dlopen2_derive](https://github.com/OpenByteDev/dlopen2) | 0.4.3 | MIT |
| [docx-rs](https://github.com/bokuweb/docx-rs) | 0.4.22 | MIT |
| [dom_query](https://github.com/niklak/dom_query) | 0.28.0 | MIT |
| [dotenvy](https://github.com/allan2/dotenvy) | 0.15.7 | MIT |
| [dpi](https://github.com/rust-windowing/winit) | 0.1.2 | Apache-2.0 AND MIT |
| [dtoa](https://github.com/dtolnay/dtoa) | 1.0.11 | MIT OR Apache-2.0 |
| [dtoa-short](https://github.com/upsuper/dtoa-short) | 0.3.5 | MPL-2.0 |
| [dunce](https://gitlab.com/kornelski/dunce) | 1.0.5 | CC0-1.0 OR MIT-0 OR Apache-2.0 |
| [dyn-clone](https://github.com/dtolnay/dyn-clone) | 1.0.20 | MIT OR Apache-2.0 |
| [ecb](https://github.com/magic-akari/ecb) | 0.1.2 | MIT |
| [ecow](https://github.com/typst/ecow) | 0.2.6 | MIT OR Apache-2.0 |
| [ego-tree](https://github.com/rust-scraper/ego-tree) | 0.11.1 | ISC |
| [either](https://github.com/rayon-rs/either) | 1.19.0 | MIT OR Apache-2.0 |
| [embed-resource](https://github.com/nabijaczleweli/rust-embed-resource) | 3.0.12 | MIT |
| [encoding_rs](https://github.com/hsivonen/encoding_rs) | 0.8.44 | (Apache-2.0 OR MIT) AND BSD-3-Clause |
| [endi](https://github.com/zeenix/endi) | 1.1.1 | MIT |
| [enum-ordinalize](https://github.com/magiclen/enum-ordinalize) | 4.4.2 | MIT |
| [enum-ordinalize-derive](https://github.com/magiclen/enum-ordinalize) | 4.4.2 | MIT |
| [enumflags2](https://github.com/meithecatte/enumflags2) | 0.7.12 | MIT OR Apache-2.0 |
| [enumflags2_derive](https://github.com/meithecatte/enumflags2) | 0.7.12 | MIT OR Apache-2.0 |
| [equivalent](https://github.com/indexmap-rs/equivalent) | 1.0.3 | Apache-2.0 OR MIT |
| [erased-serde](https://github.com/dtolnay/erased-serde) | 0.4.11 | MIT OR Apache-2.0 |
| [errno](https://github.com/lambda-fairy/rust-errno) | 0.3.14 | MIT OR Apache-2.0 |
| [etcetera](https://github.com/lunacookies/etcetera) | 0.11.0 | MIT OR Apache-2.0 |
| [euclid](https://github.com/servo/euclid) | 0.20.14 | MIT / Apache-2.0 |
| [euclid](https://github.com/servo/euclid) | 0.22.14 | MIT OR Apache-2.0 |
| [event-listener](https://github.com/smol-rs/event-listener) | 5.4.2 | Apache-2.0 OR MIT |
| [event-listener-strategy](https://github.com/smol-rs/event-listener-strategy) | 0.5.4 | Apache-2.0 OR MIT |
| [fancy-regex](https://github.com/fancy-regex/fancy-regex) | 0.16.2 | MIT |
| [fastrand](https://github.com/smol-rs/fastrand) | 2.5.0 | Apache-2.0 OR MIT |
| [fax](https://github.com/pdf-rs/fax) | 0.2.7 | MIT |
| [fdeflate](https://github.com/image-rs/fdeflate) | 0.3.7 | MIT OR Apache-2.0 |
| [fearless_simd](https://github.com/linebender/fearless_simd) | 0.4.1 | Apache-2.0 OR MIT |
| [fearless_simd](https://github.com/linebender/fearless_simd) | 1.2.0 | Apache-2.0 OR MIT |
| [field-offset](https://github.com/Diggsey/rust-field-offset) | 0.3.6 | MIT OR Apache-2.0 |
| [find-msvc-tools](https://github.com/rust-lang/cc-rs) | 0.1.15 | MIT OR Apache-2.0 |
| [flate2](https://github.com/rust-lang/flate2-rs) | 1.1.10 | MIT OR Apache-2.0 |
| [float-cmp](https://github.com/mikedilger/float-cmp) | 0.9.0 | MIT |
| [flume](https://github.com/zesterer/flume) | 0.12.0 | Apache-2.0/MIT |
| [fnv](https://github.com/servo/rust-fnv) | 1.0.7 | Apache-2.0 / MIT |
| [foldhash](https://github.com/orlp/foldhash) | 0.2.0 | Zlib |
| [font-types](https://github.com/googlefonts/fontations) | 0.11.3 | MIT OR Apache-2.0 |
| [fontconfig-parser](https://github.com/Riey/fontconfig-parser) | 0.5.8 | MIT |
| [fontdb](https://github.com/RazrFalcon/fontdb) | 0.23.0 | MIT |
| [form_urlencoded](https://github.com/servo/rust-url) | 1.2.2 | MIT OR Apache-2.0 |
| [fs_extra](https://github.com/webdesus/fs_extra) | 1.3.0 | MIT |
| [futures-channel](https://github.com/rust-lang/futures-rs) | 0.3.34 | MIT OR Apache-2.0 |
| [futures-core](https://github.com/rust-lang/futures-rs) | 0.3.34 | MIT OR Apache-2.0 |
| [futures-executor](https://github.com/rust-lang/futures-rs) | 0.3.34 | MIT OR Apache-2.0 |
| [futures-intrusive](https://github.com/Matthias247/futures-intrusive) | 0.5.0 | MIT OR Apache-2.0 |
| [futures-io](https://github.com/rust-lang/futures-rs) | 0.3.34 | MIT OR Apache-2.0 |
| [futures-lite](https://github.com/smol-rs/futures-lite) | 2.6.1 | Apache-2.0 OR MIT |
| [futures-macro](https://github.com/rust-lang/futures-rs) | 0.3.34 | MIT OR Apache-2.0 |
| [futures-sink](https://github.com/rust-lang/futures-rs) | 0.3.34 | MIT OR Apache-2.0 |
| [futures-task](https://github.com/rust-lang/futures-rs) | 0.3.34 | MIT OR Apache-2.0 |
| [futures-util](https://github.com/rust-lang/futures-rs) | 0.3.34 | MIT OR Apache-2.0 |
| [gdk](https://github.com/gtk-rs/gtk3-rs) | 0.18.2 | MIT |
| [gdk-pixbuf](https://github.com/gtk-rs/gtk-rs-core) | 0.18.5 | MIT |
| [gdk-pixbuf-sys](https://github.com/gtk-rs/gtk-rs-core) | 0.18.0 | MIT |
| [gdk-sys](https://github.com/gtk-rs/gtk3-rs) | 0.18.2 | MIT |
| [gdkwayland-sys](https://github.com/gtk-rs/gtk3-rs) | 0.18.2 | MIT |
| [gdkx11](https://github.com/gtk-rs/gtk3-rs) | 0.18.2 | MIT |
| [gdkx11-sys](https://github.com/gtk-rs/gtk3-rs) | 0.18.2 | MIT |
| [generic-array](https://github.com/fizyk20/generic-array.git) | 0.14.9 | MIT |
| [getopts](https://github.com/rust-lang/getopts) | 0.2.24 | MIT OR Apache-2.0 |
| [getrandom](https://github.com/rust-random/getrandom) | 0.2.17 | MIT OR Apache-2.0 |
| [getrandom](https://github.com/rust-random/getrandom) | 0.3.4 | MIT OR Apache-2.0 |
| [getrandom](https://github.com/rust-random/getrandom) | 0.4.3 | MIT OR Apache-2.0 |
| [gif](https://github.com/image-rs/image-gif) | 0.14.2 | MIT OR Apache-2.0 |
| [gio](https://github.com/gtk-rs/gtk-rs-core) | 0.18.4 | MIT |
| [gio-sys](https://github.com/gtk-rs/gtk-rs-core) | 0.18.1 | MIT |
| [glib](https://github.com/gtk-rs/gtk-rs-core) | 0.18.5 | MIT |
| [glib-macros](https://github.com/gtk-rs/gtk-rs-core) | 0.18.5 | MIT |
| [glib-sys](https://github.com/gtk-rs/gtk-rs-core) | 0.18.1 | MIT |
| [glidesort](https://github.com/orlp/glidesort) | 0.1.2 | MIT OR Apache-2.0 |
| [glifo](https://github.com/linebender/vello) | 0.1.1 | Apache-2.0 OR MIT |
| [glob](https://github.com/rust-lang/glob) | 0.3.4 | MIT OR Apache-2.0 |
| [gobject-sys](https://github.com/gtk-rs/gtk-rs-core) | 0.18.0 | MIT |
| [gtk](https://github.com/gtk-rs/gtk3-rs) | 0.18.2 | MIT |
| [gtk-sys](https://github.com/gtk-rs/gtk3-rs) | 0.18.2 | MIT |
| [gtk3-macros](https://github.com/gtk-rs/gtk3-rs) | 0.18.2 | MIT |
| [guillotiere](https://github.com/nical/guillotiere) | 0.7.0 | MIT/Apache-2.0 |
| [h2](https://github.com/hyperium/h2) | 0.4.20 | MIT |
| [half](https://github.com/VoidStarKat/half-rs) | 2.7.1 | MIT OR Apache-2.0 |
| [hashbrown](https://github.com/rust-lang/hashbrown) | 0.12.3 | MIT OR Apache-2.0 |
| [hashbrown](https://github.com/rust-lang/hashbrown) | 0.16.1 | MIT OR Apache-2.0 |
| [hashbrown](https://github.com/rust-lang/hashbrown) | 0.17.1 | MIT OR Apache-2.0 |
| [hashlink](https://github.com/djc/hashlink) | 0.11.1 | MIT OR Apache-2.0 |
| [hayagriva](https://github.com/typst/hayagriva) | 0.10.1 | MIT OR Apache-2.0 |
| [hayro](https://github.com/LaurenzV/hayro) | 0.7.1 | Apache-2.0 OR MIT |
| [hayro-ccitt](https://github.com/LaurenzV/hayro) | 0.3.0 | Apache-2.0 OR MIT |
| [hayro-ccitt](https://github.com/LaurenzV/hayro) | 0.4.0 | Apache-2.0 OR MIT |
| [hayro-cmap](https://github.com/LaurenzV/hayro) | 0.1.1 | Apache-2.0 OR MIT |
| [hayro-interpret](https://github.com/LaurenzV/hayro) | 0.7.0 | Apache-2.0 OR MIT |
| [hayro-jbig2](https://github.com/LaurenzV/hayro) | 0.3.1 | Apache-2.0 OR MIT |
| [hayro-jpeg2000](https://github.com/LaurenzV/hayro) | 0.3.5 | Apache-2.0 OR MIT |
| [hayro-postscript](https://github.com/LaurenzV/hayro) | 0.2.0 | Apache-2.0 OR MIT |
| [hayro-svg](https://github.com/LaurenzV/hayro) | 0.7.0 | Apache-2.0 OR MIT |
| [hayro-syntax](https://github.com/LaurenzV/hayro) | 0.7.2 | Apache-2.0 OR MIT |
| [hayro-write](https://github.com/LaurenzV/hayro) | 0.7.0 | Apache-2.0 OR MIT |
| [heck](https://github.com/withoutboats/heck) | 0.4.1 | MIT OR Apache-2.0 |
| [heck](https://github.com/withoutboats/heck) | 0.5.0 | MIT OR Apache-2.0 |
| [hex](https://github.com/KokaKiwi/rust-hex) | 0.4.3 | MIT OR Apache-2.0 |
| [hkdf](https://github.com/RustCrypto/KDFs/) | 0.12.4 | MIT OR Apache-2.0 |
| [hkdf](https://github.com/RustCrypto/KDFs/) | 0.13.0 | MIT OR Apache-2.0 |
| [hmac](https://github.com/RustCrypto/MACs) | 0.12.1 | MIT OR Apache-2.0 |
| [hmac](https://github.com/RustCrypto/MACs) | 0.13.0 | MIT OR Apache-2.0 |
| [html5ever](https://github.com/servo/html5ever) | 0.39.0 | MIT OR Apache-2.0 |
| [html5ever](https://github.com/servo/html5ever) | 0.40.1 | MIT OR Apache-2.0 |
| [http](https://github.com/hyperium/http) | 1.5.0 | MIT OR Apache-2.0 |
| [http-body](https://github.com/hyperium/http-body) | 1.1.0 | MIT |
| [http-body-util](https://github.com/hyperium/http-body) | 0.1.5 | MIT |
| [httparse](https://github.com/seanmonstar/httparse) | 1.10.1 | MIT OR Apache-2.0 |
| [hybrid-array](https://github.com/RustCrypto/hybrid-array) | 0.4.15 | MIT OR Apache-2.0 |
| [hyper](https://github.com/hyperium/hyper) | 1.12.0 | MIT |
| [hyper-rustls](https://github.com/rustls/hyper-rustls) | 0.27.10 | Apache-2.0 OR ISC OR MIT |
| [hyper-util](https://github.com/hyperium/hyper-util) | 0.1.21 | MIT |
| [hypher](https://github.com/typst/hypher) | 0.1.8 | MIT OR Apache-2.0 |
| [iana-time-zone](https://github.com/strawlab/iana-time-zone) | 0.1.65 | MIT OR Apache-2.0 |
| [ico](https://github.com/mdsteele/rust-ico) | 0.6.0 | MIT |
| [icu_collator](https://github.com/unicode-org/icu4x) | 2.4.0 | Unicode-3.0 |
| [icu_collator_data](https://github.com/unicode-org/icu4x) | 2.4.0 | Unicode-3.0 |
| [icu_collections](https://github.com/unicode-org/icu4x) | 2.4.0 | Unicode-3.0 |
| [icu_locale](https://github.com/unicode-org/icu4x) | 2.4.0 | Unicode-3.0 |
| [icu_locale_core](https://github.com/unicode-org/icu4x) | 2.4.0 | Unicode-3.0 |
| [icu_locale_data](https://github.com/unicode-org/icu4x) | 2.4.0 | Unicode-3.0 |
| [icu_locale_fallback](https://github.com/unicode-org/icu4x) | 2.4.0 | Unicode-3.0 |
| [icu_locale_fallback_data](https://github.com/unicode-org/icu4x) | 2.4.0 | Unicode-3.0 |
| [icu_normalizer](https://github.com/unicode-org/icu4x) | 2.4.0 | Unicode-3.0 |
| [icu_normalizer_data](https://github.com/unicode-org/icu4x) | 2.4.0 | Unicode-3.0 |
| [icu_properties](https://github.com/unicode-org/icu4x) | 2.4.0 | Unicode-3.0 |
| [icu_properties_data](https://github.com/unicode-org/icu4x) | 2.4.0 | Unicode-3.0 |
| [icu_provider](https://github.com/unicode-org/icu4x) | 2.4.0 | Unicode-3.0 |
| [icu_provider_blob](https://github.com/unicode-org/icu4x) | 2.4.0 | Unicode-3.0 |
| [icu_segmenter](https://github.com/unicode-org/icu4x) | 2.4.0 | Unicode-3.0 |
| [icu_segmenter_data](https://github.com/unicode-org/icu4x) | 2.4.0 | Unicode-3.0 |
| [ident_case](https://github.com/TedDriggs/ident_case) | 1.0.1 | MIT/Apache-2.0 |
| [idna](https://github.com/servo/rust-url/) | 1.1.0 | MIT OR Apache-2.0 |
| [idna_adapter](https://github.com/hsivonen/idna_adapter) | 1.2.2 | Apache-2.0 OR MIT |
| [image](https://github.com/image-rs/image) | 0.25.10 | MIT OR Apache-2.0 |
| [image-webp](https://github.com/image-rs/image-webp) | 0.2.4 | MIT OR Apache-2.0 |
| [imagesize](https://github.com/Roughsketch/imagesize) | 0.14.0 | MIT |
| [indexmap](https://github.com/bluss/indexmap) | 1.9.3 | Apache-2.0 OR MIT |
| [indexmap](https://github.com/indexmap-rs/indexmap) | 2.14.2 | Apache-2.0 OR MIT |
| [infer](https://github.com/bojand/infer) | 0.19.0 | MIT |
| [infer](https://github.com/bojand/infer) | 0.22.0 | MIT |
| [inout](https://github.com/RustCrypto/utils) | 0.1.4 | MIT OR Apache-2.0 |
| [ipnet](https://github.com/krisprice/ipnet) | 2.12.2 | MIT OR Apache-2.0 |
| [is-docker](https://github.com/TheLarkInn/is-docker) | 0.2.0 | MIT |
| [is-wsl](https://github.com/TheLarkInn/is-wsl) | 0.4.0 | MIT |
| [itoa](https://github.com/dtolnay/itoa) | 1.0.18 | MIT OR Apache-2.0 |
| [javascriptcore-rs](https://github.com/tauri-apps/javascriptcore-rs) | 1.1.2 | MIT |
| [javascriptcore-rs-sys](https://github.com/tauri-apps/javascriptcore-rs) | 1.1.1 | MIT |
| [jiff](https://github.com/BurntSushi/jiff) | 0.2.39 | Unlicense OR MIT |
| [jiff-core](https://github.com/BurntSushi/jiff) | 0.1.2 | Unlicense OR MIT |
| [jiff-tzdb](https://github.com/BurntSushi/jiff) | 0.1.9 | Unlicense OR MIT |
| [jiff-tzdb-platform](https://github.com/BurntSushi/jiff) | 0.1.3 | Unlicense OR MIT |
| [jobserver](https://github.com/rust-lang/jobserver-rs) | 0.1.35 | MIT OR Apache-2.0 |
| [json-patch](https://github.com/idubrov/json-patch) | 4.2.0 | MIT/Apache-2.0 |
| [jsonptr](https://github.com/chanced/jsonptr) | 0.7.1 | MIT OR Apache-2.0 |
| [kamadak-exif](https://github.com/kamadak/exif-rs) | 0.6.2 | BSD-2-Clause |
| [keyboard-types](https://github.com/rust-windowing/keyboard-types) | 0.8.3 | MIT OR Apache-2.0 |
| [keyring](https://github.com/hwchen/keyring-rs.git) | 3.6.3 | MIT OR Apache-2.0 |
| [krilla](https://github.com/LaurenzV/krilla) | 0.8.2 | MIT OR Apache-2.0 |
| [krilla-svg](https://github.com/LaurenzV/krilla) | 0.8.1 | MIT OR Apache-2.0 |
| [kurbo](https://github.com/linebender/kurbo) | 0.13.1 | Apache-2.0 OR MIT |
| libappindicator | 0.9.0 | Apache-2.0 OR MIT |
| libappindicator-sys | 0.9.0 | Apache-2.0 OR MIT |
| [libc](https://github.com/rust-lang/libc) | 0.2.191 | MIT OR Apache-2.0 |
| [libdbus-sys](https://github.com/diwic/dbus-rs) | 0.2.7 | Apache-2.0/MIT |
| [libloading](https://github.com/nagisa/rust_libloading/) | 0.7.4 | ISC |
| [libm](https://github.com/rust-lang/compiler-builtins) | 0.2.16 | MIT |
| [libsqlite3-sys](https://github.com/rusqlite/rusqlite) | 0.37.0 | MIT |
| [linebender_resource_handle](https://github.com/linebender/raw_resource_handle) | 0.1.1 | Apache-2.0 OR MIT |
| [linked-hash-map](https://github.com/contain-rs/linked-hash-map) | 0.5.6 | MIT/Apache-2.0 |
| [linux-raw-sys](https://github.com/sunfishcode/linux-raw-sys) | 0.12.1 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT |
| [lipsum](https://github.com/mgeisler/lipsum/) | 0.9.1 | MIT |
| [litemap](https://github.com/unicode-org/icu4x) | 0.8.3 | Unicode-3.0 |
| [lock_api](https://github.com/Amanieu/parking_lot) | 0.4.14 | MIT OR Apache-2.0 |
| [log](https://github.com/rust-lang/log) | 0.4.35 | MIT OR Apache-2.0 |
| [lopdf](https://github.com/J-F-Liu/lopdf.git) | 0.42.0 | MIT |
| [lru-slab](https://github.com/Ralith/lru-slab) | 0.1.3 | MIT OR Apache-2.0 OR Zlib |
| [markup5ever](https://github.com/servo/html5ever) | 0.39.0 | MIT OR Apache-2.0 |
| [markup5ever](https://github.com/servo/html5ever) | 0.40.0 | MIT OR Apache-2.0 |
| [md-5](https://github.com/RustCrypto/hashes) | 0.10.6 | MIT OR Apache-2.0 |
| [md-5](https://github.com/RustCrypto/hashes) | 0.11.0 | MIT OR Apache-2.0 |
| [memchr](https://github.com/BurntSushi/memchr) | 2.8.3 | Unlicense OR MIT |
| [memmap2](https://github.com/RazrFalcon/memmap2-rs) | 0.9.12 | MIT OR Apache-2.0 |
| [memoffset](https://github.com/Gilnaa/memoffset) | 0.9.1 | MIT |
| [mime](https://github.com/hyperium/mime) | 0.3.18 | MIT OR Apache-2.0 |
| [miniz_oxide](https://github.com/Frommi/miniz_oxide/tree/master/miniz_oxide) | 0.8.9 | MIT OR Zlib OR Apache-2.0 |
| [miniz_oxide](https://github.com/Frommi/miniz_oxide/tree/master/miniz_oxide) | 0.9.1 | MIT OR Zlib OR Apache-2.0 |
| [mio](https://github.com/tokio-rs/mio) | 1.2.4 | MIT |
| [moxcms](https://github.com/awxkee/moxcms.git) | 0.8.1 | BSD-3-Clause OR Apache-2.0 |
| [muda](https://github.com/tauri-apps/muda) | 0.20.0 | Apache-2.0 OR MIT |
| [multiversion_no_op](https://github.com/hsivonen/multiversion_no_op) | 1.0.0 | Apache-2.0 OR MIT |
| [mutate_once](https://github.com/kamadak/mutate_once-rs) | 0.1.2 | BSD-2-Clause |
| [new_debug_unreachable](https://github.com/mbrubeck/rust-debug-unreachable) | 1.0.6 | MIT |
| [nix](https://github.com/nix-rust/nix) | 0.29.0 | MIT |
| [nom](https://github.com/rust-bakery/nom) | 8.0.0 | MIT |
| [notify-rust](https://github.com/hoodie/notify-rust) | 4.18.2 | MIT OR Apache-2.0 |
| [num](https://github.com/rust-num/num) | 0.4.3 | MIT OR Apache-2.0 |
| [num-bigint](https://github.com/rust-num/num-bigint) | 0.4.8 | MIT OR Apache-2.0 |
| [num-complex](https://github.com/rust-num/num-complex) | 0.4.6 | MIT OR Apache-2.0 |
| [num-conv](https://github.com/jhpratt/num-conv) | 0.2.2 | MIT OR Apache-2.0 |
| [num-integer](https://github.com/rust-num/num-integer) | 0.1.47 | MIT OR Apache-2.0 |
| [num-iter](https://github.com/rust-num/num-iter) | 0.1.46 | MIT OR Apache-2.0 |
| [num-rational](https://github.com/rust-num/num-rational) | 0.4.2 | MIT OR Apache-2.0 |
| [num-traits](https://github.com/rust-num/num-traits) | 0.2.19 | MIT OR Apache-2.0 |
| [object](https://github.com/gimli-rs/object) | 0.39.1 | Apache-2.0 OR MIT |
| [once_cell](https://github.com/matklad/once_cell) | 1.21.4 | MIT OR Apache-2.0 |
| [open](https://github.com/Byron/open-rs) | 5.4.5 | MIT |
| [openssl-probe](https://github.com/rustls/openssl-probe) | 0.2.2 | MIT OR Apache-2.0 |
| [option-ext](https://github.com/soc/option-ext.git) | 0.2.0 | MPL-2.0 |
| [ordered-stream](https://github.com/danieldg/ordered-stream) | 0.2.0 | MIT OR Apache-2.0 |
| [palette](https://github.com/Ogeon/palette) | 0.7.7 | MIT OR Apache-2.0 |
| [palette_derive](https://github.com/Ogeon/palette) | 0.7.7 | MIT OR Apache-2.0 |
| [palette_math](https://github.com/Ogeon/palette) | 0.7.7 | MIT OR Apache-2.0 |
| [pango](https://github.com/gtk-rs/gtk-rs-core) | 0.18.3 | MIT |
| [pango-sys](https://github.com/gtk-rs/gtk-rs-core) | 0.18.0 | MIT |
| [parking](https://github.com/smol-rs/parking) | 2.2.1 | Apache-2.0 OR MIT |
| [parking_lot](https://github.com/Amanieu/parking_lot) | 0.12.5 | MIT OR Apache-2.0 |
| [parking_lot_core](https://github.com/Amanieu/parking_lot) | 0.9.12 | MIT OR Apache-2.0 |
| [paste](https://github.com/dtolnay/paste) | 1.0.15 | MIT OR Apache-2.0 |
| [pdf-extract](https://github.com/jrmuizel/pdf-extract) | 0.12.1 | MIT |
| [pdf-writer](https://github.com/typst/pdf-writer) | 0.15.0 | MIT OR Apache-2.0 |
| [peniko](https://github.com/linebender/peniko) | 0.6.1 | Apache-2.0 OR MIT |
| [percent-encoding](https://github.com/servo/rust-url/) | 2.3.2 | MIT OR Apache-2.0 |
| [phf](https://github.com/rust-phf/rust-phf) | 0.13.1 | MIT |
| [phf](https://github.com/rust-phf/rust-phf) | 0.14.0 | MIT |
| [phf_codegen](https://github.com/rust-phf/rust-phf) | 0.13.1 | MIT |
| [phf_codegen](https://github.com/rust-phf/rust-phf) | 0.14.0 | MIT |
| [phf_generator](https://github.com/rust-phf/rust-phf) | 0.13.1 | MIT |
| [phf_generator](https://github.com/rust-phf/rust-phf) | 0.14.0 | MIT |
| [phf_macros](https://github.com/rust-phf/rust-phf) | 0.13.1 | MIT |
| [phf_macros](https://github.com/rust-phf/rust-phf) | 0.14.0 | MIT |
| [phf_shared](https://github.com/rust-phf/rust-phf) | 0.13.1 | MIT |
| [phf_shared](https://github.com/rust-phf/rust-phf) | 0.14.0 | MIT |
| [pic-scale](https://github.com/awxkee/pic-scale) | 0.7.12 | BSD-3-Clause OR Apache-2.0 |
| [pico-args](https://github.com/RazrFalcon/pico-args) | 0.5.0 | MIT |
| [pin-project-lite](https://github.com/taiki-e/pin-project-lite) | 0.2.17 | Apache-2.0 OR MIT |
| [piper](https://github.com/smol-rs/piper) | 0.2.5 | MIT OR Apache-2.0 |
| [pkg-config](https://github.com/rust-lang/pkg-config-rs) | 0.3.34 | MIT OR Apache-2.0 |
| [plist](https://github.com/ebarnard/rust-plist/) | 1.10.3 | MIT |
| [png](https://github.com/image-rs/image-png) | 0.18.1 | MIT OR Apache-2.0 |
| [polling](https://github.com/smol-rs/polling) | 3.11.0 | Apache-2.0 OR MIT |
| [polycool](https://github.com/linebender/kurbo) | 0.4.0 | MIT OR Apache-2.0 |
| [pom](https://github.com/J-F-Liu/pom.git) | 1.1.0 | MIT |
| [portable-atomic](https://github.com/taiki-e/portable-atomic) | 1.15.0 | Apache-2.0 OR MIT |
| [postcard](https://github.com/jamesmunns/postcard) | 1.1.3 | MIT OR Apache-2.0 |
| [postscript](https://github.com/bodoni/postscript) | 0.14.1 | Apache-2.0/MIT |
| [potential_utf](https://github.com/unicode-org/icu4x) | 0.1.6 | Unicode-3.0 |
| [powerfmt](https://github.com/jhpratt/powerfmt) | 0.2.1 | MIT OR Apache-2.0 |
| [ppv-lite86](https://github.com/cryptocorrosion/cryptocorrosion) | 0.2.21 | MIT OR Apache-2.0 |
| [precomputed-hash](https://github.com/emilio/precomputed-hash) | 0.1.1 | MIT |
| [proc-macro-crate](https://github.com/bkchr/proc-macro-crate) | 1.3.1 | MIT OR Apache-2.0 |
| [proc-macro-crate](https://github.com/bkchr/proc-macro-crate) | 2.0.2 | MIT OR Apache-2.0 |
| [proc-macro-crate](https://github.com/bkchr/proc-macro-crate) | 3.5.0 | MIT OR Apache-2.0 |
| [proc-macro-error](https://gitlab.com/CreepySkeleton/proc-macro-error) | 1.0.4 | MIT OR Apache-2.0 |
| [proc-macro-error-attr](https://gitlab.com/CreepySkeleton/proc-macro-error) | 1.0.4 | MIT OR Apache-2.0 |
| [proc-macro-hack](https://github.com/dtolnay/proc-macro-hack) | 0.5.20+deprecated | MIT OR Apache-2.0 |
| [proc-macro2](https://github.com/dtolnay/proc-macro2) | 1.0.107 | MIT OR Apache-2.0 |
| [psm](https://github.com/rust-lang/stacker/) | 0.1.32 | MIT OR Apache-2.0 |
| [pxfm](https://github.com/awxkee/pxfm) | 0.1.30 | BSD-3-Clause OR Apache-2.0 |
| [quick-error](http://github.com/tailhook/quick-error) | 2.0.1 | MIT/Apache-2.0 |
| [quick-xml](https://github.com/tafia/quick-xml) | 0.38.4 | MIT |
| [quick-xml](https://github.com/tafia/quick-xml) | 0.41.0 | MIT |
| [quick-xml](https://github.com/tafia/quick-xml) | 0.42.0 | MIT |
| [quinn](https://github.com/quinn-rs/quinn) | 0.11.12 | MIT OR Apache-2.0 |
| [quinn-proto](https://github.com/quinn-rs/quinn) | 0.11.19 | MIT OR Apache-2.0 |
| [quinn-udp](https://github.com/quinn-rs/quinn) | 0.5.16 | MIT OR Apache-2.0 |
| [quote](https://github.com/dtolnay/quote) | 1.0.47 | MIT OR Apache-2.0 |
| [rand](https://github.com/rust-random/rand) | 0.10.3 | MIT OR Apache-2.0 |
| [rand](https://github.com/rust-random/rand) | 0.8.8 | MIT OR Apache-2.0 |
| [rand_chacha](https://github.com/rust-random/rand) | 0.3.1 | MIT OR Apache-2.0 |
| [rand_core](https://github.com/rust-random/rand_core) | 0.10.1 | MIT OR Apache-2.0 |
| [rand_core](https://github.com/rust-random/rand) | 0.6.4 | MIT OR Apache-2.0 |
| [rand_pcg](https://github.com/rust-random/rngs) | 0.10.2 | MIT OR Apache-2.0 |
| [rangemap](https://github.com/jeffparsons/rangemap) | 1.8.0 | MIT/Apache-2.0 |
| [raw-window-handle](https://github.com/rust-windowing/raw-window-handle) | 0.6.2 | MIT OR Apache-2.0 OR Zlib |
| [rayon](https://github.com/rayon-rs/rayon) | 1.12.0 | MIT OR Apache-2.0 |
| [rayon-core](https://github.com/rayon-rs/rayon) | 1.13.0 | MIT OR Apache-2.0 |
| [read-fonts](https://github.com/googlefonts/fontations) | 0.39.2 | MIT OR Apache-2.0 |
| [ref-cast](https://github.com/dtolnay/ref-cast) | 1.0.27 | MIT OR Apache-2.0 |
| [ref-cast-impl](https://github.com/dtolnay/ref-cast) | 1.0.27 | MIT OR Apache-2.0 |
| [regex](https://github.com/rust-lang/regex) | 1.13.1 | MIT OR Apache-2.0 |
| [regex-automata](https://github.com/rust-lang/regex) | 0.4.18 | MIT OR Apache-2.0 |
| [regex-syntax](https://github.com/rust-lang/regex) | 0.8.11 | MIT OR Apache-2.0 |
| [reqwest](https://github.com/seanmonstar/reqwest) | 0.13.5 | MIT OR Apache-2.0 |
| [resvg](https://github.com/linebender/resvg) | 0.47.0 | Apache-2.0 OR MIT |
| [rgb](https://github.com/kornelski/rust-rgb) | 0.8.53 | MIT |
| [ring](https://github.com/briansmith/ring) | 0.17.14 | Apache-2.0 AND ISC |
| [roman-numerals-rs](https://github.com/AA-Turner/roman-numerals/) | 3.1.0 | 0BSD OR CC0-1.0 |
| [roxmltree](https://github.com/RazrFalcon/roxmltree) | 0.20.0 | MIT OR Apache-2.0 |
| [roxmltree](https://github.com/RazrFalcon/roxmltree) | 0.21.2 | MIT OR Apache-2.0 |
| [rust_decimal](https://github.com/paupino/rust-decimal) | 1.43.0 | MIT |
| [rustc-hash](https://github.com/rust-lang/rustc-hash) | 2.1.3 | Apache-2.0 OR MIT |
| [rustc_version](https://github.com/djc/rustc-version-rs) | 0.4.1 | MIT OR Apache-2.0 |
| [rustix](https://github.com/bytecodealliance/rustix) | 1.1.6 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT |
| [rustls](https://github.com/rustls/rustls) | 0.23.46 | Apache-2.0 OR ISC OR MIT |
| [rustls-native-certs](https://github.com/rustls/rustls-native-certs) | 0.8.5 | Apache-2.0 OR ISC OR MIT |
| [rustls-pki-types](https://github.com/rustls/pki-types) | 1.15.1 | MIT OR Apache-2.0 |
| [rustls-platform-verifier](https://github.com/rustls/rustls-platform-verifier) | 0.7.1 | MIT OR Apache-2.0 |
| [rustls-webpki](https://github.com/rustls/webpki) | 0.103.16 | ISC |
| [rustversion](https://github.com/dtolnay/rustversion) | 1.0.23 | MIT OR Apache-2.0 |
| [rustybuzz](https://github.com/harfbuzz/rustybuzz) | 0.20.1 | MIT |
| [ryu](https://github.com/dtolnay/ryu) | 1.0.23 | Apache-2.0 OR BSL-1.0 |
| [same-file](https://github.com/BurntSushi/same-file) | 1.0.6 | Unlicense/MIT |
| [schemars](https://github.com/GREsau/schemars) | 0.8.22 | MIT |
| [schemars](https://github.com/GREsau/schemars) | 0.9.0 | MIT |
| [schemars](https://github.com/GREsau/schemars) | 1.2.3 | MIT |
| [schemars_derive](https://github.com/GREsau/schemars) | 0.8.22 | MIT |
| [scopeguard](https://github.com/bluss/scopeguard) | 1.2.0 | MIT OR Apache-2.0 |
| [scraper](https://github.com/rust-scraper/scraper) | 0.29.0 | ISC |
| [secret-service](https://github.com/hwchen/secret-service-rs.git) | 4.0.0 | MIT OR Apache-2.0 |
| [selectors](https://github.com/servo/stylo) | 0.38.0 | MPL-2.0 |
| [selectors](https://github.com/servo/stylo) | 0.41.0 | MPL-2.0 |
| [semver](https://github.com/dtolnay/semver) | 1.0.28 | MIT OR Apache-2.0 |
| [serde](https://github.com/serde-rs/serde) | 1.0.229 | MIT OR Apache-2.0 |
| [serde-untagged](https://github.com/dtolnay/serde-untagged) | 0.1.9 | MIT OR Apache-2.0 |
| [serde_core](https://github.com/serde-rs/serde) | 1.0.229 | MIT OR Apache-2.0 |
| [serde_derive](https://github.com/serde-rs/serde) | 1.0.229 | MIT OR Apache-2.0 |
| [serde_derive_internals](https://github.com/serde-rs/serde) | 0.29.1 | MIT OR Apache-2.0 |
| [serde_json](https://github.com/serde-rs/json) | 1.0.154 | MIT OR Apache-2.0 |
| [serde_path_to_error](https://github.com/dtolnay/path-to-error) | 0.1.20 | MIT OR Apache-2.0 |
| [serde_repr](https://github.com/dtolnay/serde-repr) | 0.1.21 | MIT OR Apache-2.0 |
| [serde_spanned](https://github.com/toml-rs/toml) | 0.6.9 | MIT OR Apache-2.0 |
| [serde_spanned](https://github.com/toml-rs/toml) | 1.1.2 | MIT OR Apache-2.0 |
| [serde_with](https://github.com/jonasbb/serde_with/) | 3.24.0 | MIT OR Apache-2.0 |
| [serde_with_macros](https://github.com/jonasbb/serde_with/) | 3.24.0 | MIT OR Apache-2.0 |
| [serde_yaml](https://github.com/dtolnay/serde-yaml) | 0.9.34+deprecated | MIT OR Apache-2.0 |
| [serialize-to-javascript](https://github.com/chippers/serialize-to-javascript) | 0.1.2 | MIT OR Apache-2.0 |
| [serialize-to-javascript-impl](https://github.com/chippers/serialize-to-javascript) | 0.1.2 | MIT OR Apache-2.0 |
| [servo_arc](https://github.com/servo/stylo) | 0.4.3 | MIT OR Apache-2.0 |
| [servo_arc](https://github.com/servo/stylo) | 0.5.0 | MIT OR Apache-2.0 |
| [sha1](https://github.com/RustCrypto/hashes) | 0.10.7 | MIT OR Apache-2.0 |
| [sha1](https://github.com/RustCrypto/hashes) | 0.11.0 | MIT OR Apache-2.0 |
| [sha2](https://github.com/RustCrypto/hashes) | 0.10.9 | MIT OR Apache-2.0 |
| [sha2](https://github.com/RustCrypto/hashes) | 0.11.1 | MIT OR Apache-2.0 |
| [shlex](https://github.com/comex/rust-shlex) | 2.0.1 | MIT OR Apache-2.0 |
| [signal-hook-registry](https://github.com/vorner/signal-hook) | 1.4.8 | MIT OR Apache-2.0 |
| [simd-adler32](https://github.com/mcountryman/simd-adler32) | 0.3.10 | MIT |
| [simdutf8](https://github.com/rusticstuff/simdutf8) | 0.1.5 | MIT OR Apache-2.0 |
| [simplecss](https://github.com/linebender/simplecss) | 0.2.2 | Apache-2.0 OR MIT |
| [siphasher](https://github.com/jedisct1/rust-siphash) | 1.0.4 | MIT OR Apache-2.0 |
| [skrifa](https://github.com/googlefonts/fontations) | 0.42.1 | MIT OR Apache-2.0 |
| [slab](https://github.com/tokio-rs/slab) | 0.4.12 | MIT |
| [slotmap](https://github.com/orlp/slotmap) | 1.1.1 | Zlib |
| [smallvec](https://github.com/servo/rust-smallvec) | 1.16.4 | MIT OR Apache-2.0 |
| [socket2](https://github.com/rust-lang/socket2) | 0.6.5 | MIT OR Apache-2.0 |
| [softbuffer](https://github.com/rust-windowing/softbuffer) | 0.4.8 | MIT OR Apache-2.0 |
| [soup3](https://gitlab.gnome.org/World/Rust/soup3-rs) | 0.5.0 | MIT |
| [soup3-sys](https://gitlab.gnome.org/World/Rust/soup3-rs) | 0.5.0 | MIT |
| [spin](https://github.com/mvdnes/spin-rs.git) | 0.9.9 | MIT |
| [sqlx](https://github.com/launchbadge/sqlx) | 0.9.0 | MIT OR Apache-2.0 |
| [sqlx-core](https://github.com/launchbadge/sqlx) | 0.9.0 | MIT OR Apache-2.0 |
| [sqlx-macros](https://github.com/launchbadge/sqlx) | 0.9.0 | MIT OR Apache-2.0 |
| [sqlx-macros-core](https://github.com/launchbadge/sqlx) | 0.9.0 | MIT OR Apache-2.0 |
| [sqlx-mysql](https://github.com/launchbadge/sqlx) | 0.9.0 | MIT OR Apache-2.0 |
| [sqlx-postgres](https://github.com/launchbadge/sqlx) | 0.9.0 | MIT OR Apache-2.0 |
| [sqlx-sqlite](https://github.com/launchbadge/sqlx) | 0.9.0 | MIT OR Apache-2.0 |
| [stable_deref_trait](https://github.com/storyyeller/stable_deref_trait) | 1.2.1 | MIT OR Apache-2.0 |
| [stacker](https://github.com/rust-lang/stacker) | 0.1.25 | MIT OR Apache-2.0 |
| [static_assertions](https://github.com/nvzqz/static-assertions-rs) | 1.1.0 | MIT OR Apache-2.0 |
| [static_vcruntime](https://github.com/ChrisDenton/static_vcruntime) | 3.0.0 | MIT OR Apache-2.0 OR Zlib |
| [strict-num](https://github.com/RazrFalcon/strict-num) | 0.1.1 | MIT |
| [string_cache](https://github.com/servo/string-cache) | 0.11.0 | MIT OR Apache-2.0 |
| [string_cache](https://github.com/servo/string-cache) | 0.9.0 | MIT OR Apache-2.0 |
| [string_cache_codegen](https://github.com/servo/string-cache) | 0.11.2 | MIT OR Apache-2.0 |
| [string_cache_codegen](https://github.com/servo/string-cache) | 0.6.1 | MIT OR Apache-2.0 |
| [stringprep](https://github.com/sfackler/rust-stringprep) | 0.1.5 | MIT/Apache-2.0 |
| [strsim](https://github.com/rapidfuzz/strsim-rs) | 0.11.1 | MIT |
| [strum](https://github.com/Peternator7/strum) | 0.27.2 | MIT |
| [strum_macros](https://github.com/Peternator7/strum) | 0.27.2 | MIT |
| [subsetter](https://github.com/typst/subsetter) | 0.2.6 | MIT OR Apache-2.0 |
| [subtle](https://github.com/dalek-cryptography/subtle) | 2.6.1 | BSD-3-Clause |
| [svgtypes](https://github.com/linebender/svgtypes) | 0.16.2 | Apache-2.0 OR MIT |
| [syn](https://github.com/dtolnay/syn) | 1.0.109 | MIT OR Apache-2.0 |
| [syn](https://github.com/dtolnay/syn) | 2.0.119 | MIT OR Apache-2.0 |
| [syn](https://github.com/dtolnay/syn) | 3.0.9 | MIT OR Apache-2.0 |
| [sync_wrapper](https://github.com/Actyx/sync_wrapper) | 1.0.2 | Apache-2.0 |
| [synstructure](https://github.com/mystor/synstructure) | 0.14.0 | MIT |
| [syntect](https://github.com/trishume/syntect) | 5.3.0 | MIT |
| [system-deps](https://github.com/gdesmott/system-deps) | 6.2.2 | MIT OR Apache-2.0 |
| [tao](https://github.com/tauri-apps/tao) | 0.37.2 | Apache-2.0 |
| [target-lexicon](https://github.com/bytecodealliance/target-lexicon) | 0.12.16 | Apache-2.0 WITH LLVM-exception |
| [tauri](https://github.com/tauri-apps/tauri) | 2.12.5 | Apache-2.0 OR MIT |
| [tauri-build](https://github.com/tauri-apps/tauri) | 2.7.4 | Apache-2.0 OR MIT |
| [tauri-codegen](https://github.com/tauri-apps/tauri) | 2.7.4 | Apache-2.0 OR MIT |
| [tauri-macros](https://github.com/tauri-apps/tauri) | 2.7.4 | Apache-2.0 OR MIT |
| [tauri-plugin](https://github.com/tauri-apps/tauri) | 2.7.2 | Apache-2.0 OR MIT |
| [tauri-plugin-notification](https://github.com/tauri-apps/plugins-workspace) | 2.8.0 | Apache-2.0 OR MIT |
| [tauri-plugin-opener](https://github.com/tauri-apps/plugins-workspace) | 2.7.2 | Apache-2.0 OR MIT |
| [tauri-runtime](https://github.com/tauri-apps/tauri) | 2.12.2 | Apache-2.0 OR MIT |
| [tauri-runtime-wry](https://github.com/tauri-apps/tauri) | 2.12.3 | Apache-2.0 OR MIT |
| [tauri-utils](https://github.com/tauri-apps/tauri) | 2.10.2 | Apache-2.0 OR MIT |
| [tauri-winres](https://github.com/tauri-apps/winres) | 0.3.7 | MIT |
| [tauri-winrt-notification](https://github.com/tauri-apps/winrt-notification) | 0.8.1 | MIT OR Apache-2.0 |
| [tendril](https://github.com/servo/html5ever) | 0.5.1 | MIT OR Apache-2.0 |
| [thin-vec](https://github.com/mozilla/thin-vec) | 0.2.21 | MIT OR Apache-2.0 |
| [thiserror](https://github.com/dtolnay/thiserror) | 1.0.69 | MIT OR Apache-2.0 |
| [thiserror](https://github.com/dtolnay/thiserror) | 2.0.21 | MIT OR Apache-2.0 |
| [thiserror-impl](https://github.com/dtolnay/thiserror) | 1.0.69 | MIT OR Apache-2.0 |
| [thiserror-impl](https://github.com/dtolnay/thiserror) | 2.0.21 | MIT OR Apache-2.0 |
| [tiff](https://github.com/image-rs/image-tiff) | 0.11.3 | MIT |
| [time](https://github.com/time-rs/time) | 0.3.55 | MIT OR Apache-2.0 |
| [time-core](https://github.com/time-rs/time) | 0.1.9 | MIT OR Apache-2.0 |
| [time-macros](https://github.com/time-rs/time) | 0.2.32 | MIT OR Apache-2.0 |
| [tiny-skia](https://github.com/linebender/tiny-skia) | 0.12.0 | BSD-3-Clause |
| [tiny-skia-path](https://github.com/linebender/tiny-skia/tree/master/path) | 0.12.0 | BSD-3-Clause |
| [tinystr](https://github.com/unicode-org/icu4x) | 0.8.5 | Unicode-3.0 |
| [tinyvec](https://github.com/Lokathor/tinyvec) | 1.13.3 | Zlib OR Apache-2.0 OR MIT |
| [tokio](https://github.com/tokio-rs/tokio) | 1.54.1 | MIT |
| [tokio-macros](https://github.com/tokio-rs/tokio) | 2.7.2 | MIT |
| [tokio-rustls](https://github.com/rustls/tokio-rustls) | 0.26.6 | MIT OR Apache-2.0 |
| [tokio-stream](https://github.com/tokio-rs/tokio) | 0.1.19 | MIT |
| [tokio-util](https://github.com/tokio-rs/tokio) | 0.7.20 | MIT |
| [toml](https://github.com/toml-rs/toml) | 0.8.2 | MIT OR Apache-2.0 |
| [toml](https://github.com/toml-rs/toml) | 1.2.0+spec-1.1.0 | MIT OR Apache-2.0 |
| [toml_datetime](https://github.com/toml-rs/toml) | 0.6.3 | MIT OR Apache-2.0 |
| [toml_datetime](https://github.com/toml-rs/toml) | 1.1.2+spec-1.1.0 | MIT OR Apache-2.0 |
| [toml_edit](https://github.com/toml-rs/toml) | 0.19.15 | MIT OR Apache-2.0 |
| [toml_edit](https://github.com/toml-rs/toml) | 0.20.2 | MIT OR Apache-2.0 |
| [toml_edit](https://github.com/toml-rs/toml) | 0.25.18+spec-1.1.0 | MIT OR Apache-2.0 |
| [toml_parser](https://github.com/toml-rs/toml) | 1.1.5+spec-1.1.0 | MIT OR Apache-2.0 |
| [toml_writer](https://github.com/toml-rs/toml) | 1.2.0+spec-1.1.0 | MIT OR Apache-2.0 |
| [tower](https://github.com/tower-rs/tower) | 0.5.3 | MIT |
| [tower-http](https://github.com/tower-rs/tower-http) | 0.6.11 | MIT |
| [tower-layer](https://github.com/tower-rs/tower) | 0.3.3 | MIT |
| [tower-service](https://github.com/tower-rs/tower) | 0.3.3 | MIT |
| [tracing](https://github.com/tokio-rs/tracing) | 0.1.44 | MIT |
| [tracing-attributes](https://github.com/tokio-rs/tracing) | 0.1.31 | MIT |
| [tracing-core](https://github.com/tokio-rs/tracing) | 0.1.36 | MIT |
| [tray-icon](https://github.com/tauri-apps/tray-icon) | 0.25.1 | MIT OR Apache-2.0 |
| [try-lock](https://github.com/seanmonstar/try-lock) | 0.2.5 | MIT |
| [ttf-parser](https://github.com/harfbuzz/ttf-parser) | 0.25.1 | MIT OR Apache-2.0 |
| [two-face](https://github.com/CosmicHorrorDev/two-face) | 0.4.5 | MIT OR Apache-2.0 |
| [type1-encoding-parser](https://github.com/jrmuizel/type1-encoding-parser) | 0.1.1 | MIT |
| [typed-arena](https://github.com/SimonSapin/rust-typed-arena) | 2.0.2 | MIT |
| [typed-path](https://github.com/chipsenkbeil/typed-path) | 0.12.4 | MIT OR Apache-2.0 |
| [typed-path](https://github.com/chipsenkbeil/typed-path) | 0.13.0 | MIT OR Apache-2.0 |
| [typeid](https://github.com/dtolnay/typeid) | 1.0.3 | MIT OR Apache-2.0 |
| [typenum](https://github.com/paholg/typenum) | 1.20.1 | MIT OR Apache-2.0 |
| [typst](https://github.com/typst/typst) | 0.15.1 | Apache-2.0 |
| [typst-as-lib](https://github.com/Relacibo/typst-as-lib) | 0.17.0 | MIT |
| [typst-assets](https://github.com/typst/typst-assets) | 0.15.1 | Apache-2.0 |
| [typst-eval](https://github.com/typst/typst) | 0.15.1 | Apache-2.0 |
| [typst-html](https://github.com/typst/typst) | 0.15.1 | Apache-2.0 |
| [typst-layout](https://github.com/typst/typst) | 0.15.1 | Apache-2.0 |
| [typst-library](https://github.com/typst/typst) | 0.15.1 | Apache-2.0 |
| [typst-macros](https://github.com/typst/typst) | 0.15.1 | Apache-2.0 |
| [typst-pdf](https://github.com/typst/typst) | 0.15.1 | Apache-2.0 |
| [typst-realize](https://github.com/typst/typst) | 0.15.1 | Apache-2.0 |
| [typst-svg](https://github.com/typst/typst) | 0.15.1 | Apache-2.0 |
| [typst-syntax](https://github.com/typst/typst) | 0.15.1 | Apache-2.0 |
| [typst-timing](https://github.com/typst/typst) | 0.15.1 | Apache-2.0 |
| [typst-utils](https://github.com/typst/typst) | 0.15.1 | Apache-2.0 |
| [unic-langid](https://github.com/zbraniecki/unic-locale) | 0.9.6 | MIT OR Apache-2.0 |
| [unic-langid-impl](https://github.com/zbraniecki/unic-locale) | 0.9.6 | MIT OR Apache-2.0 |
| [unic-langid-macros](https://github.com/zbraniecki/unic-locale) | 0.9.6 | MIT OR Apache-2.0 |
| [unic-langid-macros-impl](https://github.com/zbraniecki/unic-locale) | 0.9.6 | MIT OR Apache-2.0 |
| [unicode-bidi](https://github.com/servo/unicode-bidi) | 0.3.18 | MIT OR Apache-2.0 |
| [unicode-bidi-mirroring](https://github.com/RazrFalcon/unicode-bidi-mirroring) | 0.4.0 | MIT/Apache-2.0 |
| [unicode-ccc](https://github.com/RazrFalcon/unicode-ccc) | 0.4.0 | MIT/Apache-2.0 |
| [unicode-ident](https://github.com/dtolnay/unicode-ident) | 1.0.27 | (MIT OR Apache-2.0) AND Unicode-3.0 |
| [unicode-math-class](https://github.com/typst/unicode-math-class) | 0.1.0 | MIT OR Apache-2.0 |
| [unicode-normalization](https://github.com/unicode-rs/unicode-normalization) | 0.1.26 | (MIT OR Apache-2.0) AND Unicode-3.0 |
| [unicode-properties](https://github.com/unicode-rs/unicode-properties) | 0.1.5 | (MIT OR Apache-2.0) AND Unicode-3.0 |
| [unicode-script](https://github.com/unicode-rs/unicode-script) | 0.5.9 | MIT OR Apache-2.0 |
| [unicode-segmentation](https://github.com/unicode-rs/unicode-segmentation) | 1.13.3 | MIT OR Apache-2.0 |
| [unicode-vo](https://github.com/RazrFalcon/unicode-vo) | 0.1.0 | MIT/Apache-2.0 |
| [unicode-width](https://github.com/unicode-rs/unicode-width) | 0.2.3 | MIT OR Apache-2.0 |
| [unsafe-libyaml](https://github.com/dtolnay/unsafe-libyaml) | 0.2.11 | MIT |
| [unscanny](https://github.com/typst/unscanny) | 0.1.0 | MIT OR Apache-2.0 |
| [untrusted](https://github.com/briansmith/untrusted) | 0.9.0 | ISC |
| [url](https://github.com/servo/rust-url) | 2.5.8 | MIT OR Apache-2.0 |
| [urlpattern](https://github.com/denoland/rust-urlpattern) | 0.6.0 | MIT |
| [usvg](https://github.com/linebender/resvg) | 0.47.0 | Apache-2.0 OR MIT |
| [utf16_iter](https://github.com/hsivonen/utf16_iter) | 1.1.0 | Apache-2.0 OR MIT |
| [utf8_iter](https://github.com/hsivonen/utf8_iter) | 1.1.0 | Apache-2.0 OR MIT |
| [utf_types](https://github.com/hsivonen/utf_types) | 1.0.0 | Apache-2.0 OR MIT |
| [uuid](https://github.com/uuid-rs/uuid) | 1.29.0 | Apache-2.0 OR MIT |
| [vcpkg](https://github.com/mcgoo/vcpkg-rs) | 0.2.15 | MIT/Apache-2.0 |
| [vello_common](https://github.com/linebender/vello) | 0.0.8 | Apache-2.0 OR MIT |
| [vello_common](https://github.com/linebender/vello) | 0.0.9 | Apache-2.0 OR MIT |
| [vello_cpu](https://github.com/linebender/vello) | 0.0.8 | Apache-2.0 OR MIT |
| [version-compare](https://gitlab.com/timvisee/version-compare) | 0.2.1 | MIT |
| [version_check](https://github.com/SergioBenitez/version_check) | 0.9.5 | MIT/Apache-2.0 |
| [vswhom](https://github.com/nabijaczleweli/vswhom.rs) | 0.1.0 | MIT |
| [vswhom-sys](https://github.com/nabijaczleweli/vswhom-sys.rs) | 0.1.3 | MIT |
| [walkdir](https://github.com/BurntSushi/walkdir) | 2.5.0 | Unlicense/MIT |
| [want](https://github.com/seanmonstar/want) | 0.3.2 | MIT |
| [wasmi](https://github.com/wasmi-labs/wasmi) | 1.1.0 | MIT/Apache-2.0 |
| [wasmi_collections](https://github.com/wasmi-labs/wasmi) | 1.1.0 | MIT/Apache-2.0 |
| [wasmi_core](https://github.com/wasmi-labs/wasmi) | 1.1.0 | MIT/Apache-2.0 |
| [wasmi_ir](https://github.com/wasmi-labs/wasmi) | 1.1.0 | MIT/Apache-2.0 |
| [wasmparser](https://github.com/bytecodealliance/wasm-tools/tree/main/crates/wasmparser) | 0.239.0 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT |
| [web-time](https://github.com/daxpedda/web-time) | 1.1.0 | MIT OR Apache-2.0 |
| [web_atoms](https://github.com/servo/html5ever) | 0.2.6 | MIT OR Apache-2.0 |
| [web_atoms](https://github.com/servo/html5ever) | 0.3.0 | MIT OR Apache-2.0 |
| [webkit2gtk](https://github.com/tauri-apps/webkit2gtk-rs) | 2.0.2 | MIT |
| [webkit2gtk-sys](https://github.com/tauri-apps/webkit2gtk-rs) | 2.0.2 | MIT |
| [webview2-com](https://github.com/wravery/webview2-rs) | 0.39.1 | MIT |
| [webview2-com-macros](https://github.com/wravery/webview2-rs) | 0.8.1 | MIT |
| [webview2-com-sys](https://github.com/wravery/webview2-rs) | 0.39.1 | MIT |
| [weezl](https://github.com/image-rs/weezl) | 0.1.12 | MIT OR Apache-2.0 |
| [whoami](https://github.com/ardaku/whoami) | 2.1.3 | Apache-2.0 OR BSL-1.0 OR MIT |
| [winapi-util](https://github.com/BurntSushi/winapi-util) | 0.1.11 | Unlicense OR MIT |
| [window-vibrancy](https://github.com/tauri-apps/tauri-plugin-vibrancy) | 0.8.1 | Apache-2.0 OR MIT |
| [windows](https://github.com/microsoft/windows-rs) | 0.62.2 | MIT OR Apache-2.0 |
| [windows-collections](https://github.com/microsoft/windows-rs) | 0.3.2 | MIT OR Apache-2.0 |
| [windows-core](https://github.com/microsoft/windows-rs) | 0.62.2 | MIT OR Apache-2.0 |
| [windows-future](https://github.com/microsoft/windows-rs) | 0.3.2 | MIT OR Apache-2.0 |
| [windows-implement](https://github.com/microsoft/windows-rs) | 0.60.2 | MIT OR Apache-2.0 |
| [windows-interface](https://github.com/microsoft/windows-rs) | 0.59.3 | MIT OR Apache-2.0 |
| [windows-link](https://github.com/microsoft/windows-rs) | 0.2.1 | MIT OR Apache-2.0 |
| [windows-numerics](https://github.com/microsoft/windows-rs) | 0.3.1 | MIT OR Apache-2.0 |
| [windows-registry](https://github.com/microsoft/windows-rs) | 0.6.1 | MIT OR Apache-2.0 |
| [windows-result](https://github.com/microsoft/windows-rs) | 0.4.1 | MIT OR Apache-2.0 |
| [windows-strings](https://github.com/microsoft/windows-rs) | 0.5.1 | MIT OR Apache-2.0 |
| [windows-sys](https://github.com/microsoft/windows-rs) | 0.60.2 | MIT OR Apache-2.0 |
| [windows-sys](https://github.com/microsoft/windows-rs) | 0.61.2 | MIT OR Apache-2.0 |
| [windows-targets](https://github.com/microsoft/windows-rs) | 0.53.5 | MIT OR Apache-2.0 |
| [windows-threading](https://github.com/microsoft/windows-rs) | 0.2.1 | MIT OR Apache-2.0 |
| [windows-version](https://github.com/microsoft/windows-rs) | 0.1.7 | MIT OR Apache-2.0 |
| [windows_x86_64_msvc](https://github.com/microsoft/windows-rs) | 0.53.1 | MIT OR Apache-2.0 |
| [winnow](https://github.com/winnow-rs/winnow) | 0.5.40 | MIT |
| [winnow](https://github.com/winnow-rs/winnow) | 1.0.5 | MIT |
| [winreg](https://github.com/gentoo90/winreg-rs) | 0.56.0 | MIT |
| [write-fonts](https://github.com/googlefonts/fontations) | 0.48.1 | MIT OR Apache-2.0 |
| [write16](https://github.com/hsivonen/write16) | 1.0.0 | Apache-2.0 OR MIT |
| [writeable](https://github.com/unicode-org/icu4x) | 0.6.4 | Unicode-3.0 |
| [wry](https://github.com/tauri-apps/wry) | 0.57.1 | Apache-2.0 OR MIT |
| [x11](https://github.com/AltF02/x11-rs.git) | 2.21.0 | MIT |
| [x11-dl](https://github.com/AltF02/x11-rs.git) | 2.21.0 | MIT |
| [xdg-home](https://github.com/zeenix/xdg-home) | 1.3.0 | MIT |
| [xmlwriter](https://github.com/RazrFalcon/xmlwriter) | 0.1.0 | MIT |
| [xmp-writer](https://github.com/typst/xmp-writer) | 0.3.3 | MIT OR Apache-2.0 |
| [yaml-rust](https://github.com/chyh1990/yaml-rust) | 0.4.5 | MIT/Apache-2.0 |
| [yoke](https://github.com/unicode-org/icu4x) | 0.8.4 | Unicode-3.0 |
| [yoke-derive](https://github.com/unicode-org/icu4x) | 0.8.4 | Unicode-3.0 |
| [zbus](https://github.com/dbus2/zbus/) | 4.4.0 | MIT |
| [zbus](https://github.com/z-galaxy/zbus/) | 5.19.0 | MIT |
| [zbus_macros](https://github.com/dbus2/zbus/) | 4.4.0 | MIT |
| [zbus_macros](https://github.com/z-galaxy/zbus/) | 5.19.0 | MIT |
| [zbus_names](https://github.com/dbus2/zbus/) | 3.0.0 | MIT |
| [zbus_names](https://github.com/z-galaxy/zbus/) | 4.3.5 | MIT |
| [zcheapstr](https://github.com/z-galaxy/zcheapstr/) | 1.2.0 | MIT |
| [zerocopy](https://github.com/google/zerocopy) | 0.8.63 | BSD-2-Clause OR Apache-2.0 OR MIT |
| [zerocopy-derive](https://github.com/google/zerocopy) | 0.8.63 | BSD-2-Clause OR Apache-2.0 OR MIT |
| [zerofrom](https://github.com/unicode-org/icu4x) | 0.1.8 | Unicode-3.0 |
| [zerofrom-derive](https://github.com/unicode-org/icu4x) | 0.1.8 | Unicode-3.0 |
| [zeroize](https://github.com/RustCrypto/utils) | 1.9.1 | Apache-2.0 OR MIT |
| [zeroize_derive](https://github.com/RustCrypto/utils) | 1.5.0 | Apache-2.0 OR MIT |
| [zerotrie](https://github.com/unicode-org/icu4x) | 0.2.5 | Unicode-3.0 |
| [zerovec](https://github.com/unicode-org/icu4x) | 0.11.9 | Unicode-3.0 |
| [zerovec-derive](https://github.com/unicode-org/icu4x) | 0.11.7 | Unicode-3.0 |
| [zip](https://github.com/zip-rs/zip2) | 8.6.0 | MIT |
| [zip](https://github.com/zip-rs/zip2) | 9.4.2 | MIT |
| [zlib-rs](https://github.com/trifectatechfoundation/zlib-rs) | 0.6.8 | Zlib |
| [zmij](https://github.com/dtolnay/zmij) | 1.0.23 | MIT |
| [zopfli](https://github.com/zopfli-rs/zopfli) | 0.8.4 | Apache-2.0 |
| [zune-core](https://github.com/etemesi254/zune-image) | 0.5.3 | MIT OR Apache-2.0 OR Zlib |
| [zune-jpeg](https://github.com/etemesi254/zune-image/tree/dev/crates/zune-jpeg) | 0.5.15 | MIT OR Apache-2.0 OR Zlib |
| [zvariant](https://github.com/dbus2/zbus/) | 4.2.0 | MIT |
| [zvariant](https://github.com/z-galaxy/zbus/) | 5.15.1 | MIT |
| [zvariant_derive](https://github.com/dbus2/zbus/) | 4.2.0 | MIT |
| [zvariant_derive](https://github.com/z-galaxy/zbus/) | 5.15.1 | MIT |
| [zvariant_utils](https://github.com/dbus2/zbus/) | 2.1.0 | MIT |
| [zvariant_utils](https://github.com/z-galaxy/zbus/) | 4.2.0 | MIT |

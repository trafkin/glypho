# Documentación del proyecto Glypho

## 1. CLI (Interfaz de línea de comandos)
Esta parte del programa se encarga de leer los argumentos del usuario.  
Ejemplo: cuando se ejecuta `cargo run -- README.md`, el CLI detecta el archivo a convertir.

## 2. Servidor HTTP
El servidor inicia cuando el programa corre.  
Sirve el archivo HTML generado y muestra una dirección como `http://0.0.0.0:41427`.

## 3. Conversión de Markdown a HTML
El programa transforma archivos `.md` en HTML usando plantillas.  
Así, el contenido puede verse como una página web en el navegador.

# Documentacion del proyecto Glypho

### Estructura principal
`struct Args`  
Guarda los datos de entrada:
 `input`: el archivo Markdown que se va a convertir.  
 `port`: número de puerto opcional del servidor.  

### Funciona como:
 Usa la librería `clap` para interpretar los comandos del usuario.  
 Permite ejecutar comandos como:  

 ### Estructura principal
`enum GlyphoError`
Define los distintos errores posibles:
 Archivo no encontrado.
 Error de lectura.
 Error de servidor.

### Funciona como:
 Usa `impl Display` para mostrar mensajes claros al usuario.
 Mejora la comprensión de errores en el programa.

### Función: main()
Este es el punto de entrada del programa.
1. Llama a `Args::parse()` para obtener los argumentos.
2. Lee el archivo Markdown.
3. Convierte el contenido a HTML.
4. Inicia el servidor HTTP y muestra la dirección local.

### Función: logger()
Configura el registro de eventos para mostrar información y errores en el programa.

### Estructura principal
`struct InnerState`
 Guarda el contenido HTML que se está permitiendo que fucione el programa.
 Permite actualizar o consultar el contenido actual.

### Funcionalidad
Administra los datos que el servidor necesita para mantenerse activo.

# Documentación de dependencias - Proyecto Glypho

## Funcionalidad  
Administra los datos que el servidor necesita para mantenerse activo.

# Documentación de dependencias - Proyecto Glypho

## Funcionalidad  
Administra los datos que el servidor necesita para mantenerse activo.

---

**1. clap = { version = "4.5.40", features = ["derive"] }**  
**Función principal:** Crear interfaces de línea de comandos (CLI).  
**Por qué funciona:** Usa macros derivadas (`#[derive(Parser)]`) que generan automáticamente el código necesario para leer y procesar argumentos.

---

**2. eyre = "0.6.12"**  
**Función principal:** Manejar errores de manera flexible.  
**Por qué funciona:** Proporciona un sistema de reportes de errores basado en el tipo `Result`, con soporte para contextos y conversiones automáticas de errores.

---

**3. handlebars = "6.3.2"**  
**Función principal:** Renderizar plantillas HTML dinámicas.  
**Por qué funciona:** Usa plantillas con expresiones y variables para combinar datos con contenido HTML de forma eficiente.

---

**4. markdown = { version = "1.0.0", features = ["serde"] }**  
**Función principal:** Convertir texto Markdown en HTML.  
**Por qué funciona:** Interpreta texto con formato Markdown y lo transforma en contenido estructurado compatible con navegadores.

---

**5. tokio = { version = "1.46.1", features = ["full"] }**  
**Función principal:** Ejecutar tareas asincrónicas y manejar concurrencia.  
**Por qué funciona:** Proporciona un runtime asincrónico optimizado para operaciones de entrada/salida y servidores de red.

---

**6. tokio-stream = "0.1.17"**  
**Función principal:** Trabajar con flujos asincrónicos de datos.  
**Por qué funciona:** Permite combinar y transformar flujos (`Stream`) usando las herramientas de Tokio.

---

**7. tracing-subscriber = { version = "0.3.19", features = ["env-filter", "fmt", "registry", "std"] }**  
**Función principal:** Registrar y filtrar eventos de trazas y logs.  
**Por qué funciona:** Gestiona la salida de registros con formato, niveles de detalle y control por entorno.

---

**8. tracing = "0.1.41"**  
**Función principal:** Trazar eventos y registrar información en tiempo de ejecución.  
**Por qué funciona:** Proporciona macros como `trace!`, `info!`, `warn!` y `error!` para seguimiento detallado del programa.

---

**9. async-watcher = "0.3.0"**  
**Función principal:** Detectar cambios en archivos o directorios de forma asincrónica.  
**Por qué funciona:** Emplea tareas asincrónicas para monitorear el sistema de archivos sin bloquear la ejecución principal.

---

**10. futures-util = "0.3.31"**  
**Función principal:** Manipular tareas y flujos asincrónicos.  
**Por qué funciona:** Proporciona combinadores y utilidades para manejar `Future` y `Stream` de forma funcional.

---

**11. futures-core = "0.3.31"**  
**Función principal:** Definir las interfaces base de `Future` y `Stream`.  
**Por qué funciona:** Establece los tipos esenciales usados por todas las bibliotecas asincrónicas en Rust.

---

**12. axum = "0.8.4"**  
**Función principal:** Crear servidores web modernos y asincrónicos.  
**Por qué funciona:** Se basa en Tokio y Tower para construir APIs REST, manejar rutas y peticiones HTTP de forma eficiente.

---

**13. futures = "0.3.31"**  
**Función principal:** Implementar programación asincrónica en Rust.  
**Por qué funciona:** Proporciona los rasgos y utilidades necesarios para ejecutar tareas concurrentes y trabajar con `Future`.

---

**14. bytes = "1.10.1"**  
**Función principal:** Manipular datos binarios de forma eficiente.  
**Por qué funciona:** Define estructuras optimizadas para gestionar buffers de bytes en operaciones de red o archivos.

---

**15. tower = "0.5.2"**  
**Función principal:** Componer servicios y middleware.  
**Por qué funciona:** Permite construir capas (middlewares) que procesan peticiones y respuestas de manera modular.

---

**16. tower-http = { version = "0.6.6", features = ["fs"] }**  
**Función principal:** Extender Tower con utilidades HTTP.  
**Por qué funciona:** Ofrece componentes para servir archivos estáticos, manejar cabeceras, compresión y más.

---

**17. thiserror = "2.0.12"**  
**Función principal:** Crear tipos de error personalizados.  
**Por qué funciona:** Usa macros derivadas (`#[derive(Error)]`) que generan automáticamente implementaciones para el manejo de errores.

---

**18. clap-stdin = "0.7.0"**  
**Función principal:** Leer argumentos desde la entrada estándar (stdin).  
**Por qué funciona:** Extiende `clap` para aceptar datos interactivos o canalizados en la línea de comandos.

---

**19. open = "5.3.2"**  
**Función principal:** Abrir archivos o URLs con las aplicaciones predeterminadas del sistema.  
**Por qué funciona:** Detecta el sistema operativo y usa las herramientas nativas (`xdg-open`, `start`, `open`) para ejecutar la acción.

---

**20. color-eyre = "0.6.5"**  
**Función principal:** Mostrar errores con reportes detallados y coloreados.  
**Por qué funciona:** Amplía `eyre` para presentar errores con contexto, backtrace y colores que facilitan la depuración.

---

### Dependencias específicas para compilación en MUSL

**mimalloc = { version = "*", features = ["secure"] }**  
**Función principal:** Usar un asignador de memoria rápido y seguro.  
**Por qué funciona:** Optimiza el rendimiento y la seguridad en entornos compilados con `musl`, reduciendo fragmentación de memoria.

---

### Dependencias de compilación (`build-dependencies`)

**walkdir = "2.5.0"**  
**Función principal:** Recorrer directorios y archivos de forma recursiva.  
**Por qué funciona:** Permite acceder a todos los archivos de un árbol de directorios fácilmente durante la compilación.

**eyre = "0.6.12"**  
**Función principal:** Manejo de errores en scripts de compilación.  
**Por qué funciona:** Facilita el reporte de errores cuando se generan recursos o configuraciones previas al build.

**handlebars = "6.3.2"**  
**Función principal:** Generar archivos o plantillas durante la compilación.  
**Por qué funciona:** Permite crear contenido dinámico basado en plantillas antes de empaquetar el proyecto.

---

### Perfil de compilación (`profile.release`)

**opt-level = "z"** — Minimiza el tamaño del binario.  
**debug = "none"** — Desactiva la información de depuración.  
**strip = "symbols"** — Elimina símbolos innecesarios para reducir el tamaño.  
**debug-assertions = false** — Evita comprobaciones de depuración.  
**overflow-checks = false** — No revisa desbordamientos aritméticos.  
**lto = "fat"** — Aplica optimización de enlace global (Link-Time Optimization).  
**panic = "abort"** — Minimiza el binario haciendo que los fallos aborten el programa.  
**incremental = false** — Desactiva compilación incremental para máxima optimización.  
**codegen-units = 1** — Usa una sola unidad de generación de código para optimización completa.

---


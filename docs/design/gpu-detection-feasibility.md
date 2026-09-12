# Detección de GPUs/backends sin el juego instalado — estudio de factibilidad

Fecha: 2026-09-12. Repo: `xindeler-updater`. Estado: investigación, **no implementado**.

## Veredicto

**Sí, es viable, y es barato.** El launcher ya tiene `wgpu` 0.19.4 compilado dentro
(transitivo vía `iced` 0.12.1 → `iced_renderer` → `iced_wgpu` 0.12.1). Se puede enumerar
las GPUs reales de la máquina sin ventana, sin superficie y sin el juego.

Verificado empíricamente en esta máquina (macOS, Apple M5 Pro), con un ejemplo temporal
compilado contra el `Cargo.lock` real del repo:

```
Instance::new           268 µs
enumerate_adapters       13.6 ms
name="Apple M5 Pro" backend=Metal device_type=IntegratedGpu
```

Agregar `wgpu = "0.19.4"` a `client/Cargo.toml` produce **una sola línea de diff en
`Cargo.lock`** (`wgpu` en la lista de deps del paquete) — cero crates nuevos, cero
recompilación extra, cero impacto en el tamaño del binario. `iced` **no** reexporta
`wgpu` de forma accesible (el `pub use wgpu;` vive en `iced_wgpu`, que no es alcanzable
desde `iced::*`), así que la dependencia directa es la vía correcta.

## Estado actual del código

- `client/src/profiles.rs:112` `query_wgpu_backends()` y `:144` `query_wgpu_devices()`
  ejecutan el binario del juego (`voxygen list-wgpu-backends` / `list-wgpu-devices`,
  subcomandos, sin `--`) y parsean stdout.
- `client/src/profiles.rs:420` `reload_wgpu_backends()` / `:449` `reload_wgpu_devices()`
  hacen `block_on` **en el hilo de UI** (llamados desde `client/src/gui/mod.rs:110-111`).
- Sin juego instalado: backends → lista estática `WGPU_BACKENDS` (`:92-104`),
  devices → `vec![WgpuDevice::Auto]`.
- Lo elegido se pasa al juego como env vars en `Profile::start` (`:339`):
  `WGPU_BACKEND` (string lowercase mapeado a mano, `:369-382`) y `WGPU_ADAPTER`
  (`profile.wgpu_device.to_string()`, `:384-389`).

## Compatibilidad con el juego — RESUELTO

El repo del juego **sí** está local (`~/Workspace/RustroverProjects/xindeler-new-horizon`),
así que esto no queda como pregunta abierta.

### 1. Devices: el formato coincide

`voxygen/src/main.rs:60` — `ListWgpuDevices` hace exactamente:

```rust
let adapters = Instance::new(&wgpu::InstanceDescriptor::from_env_or_default())
    .enumerate_adapters(Backends::default());
for adapter in adapters { println!("{}", adapter.get_info().name); }
```

Es literalmente la misma llamada que haría el launcher. `Backends::default()` en wgpu 27
es `Backends::all()`, igual que lo que usaríamos. El nombre sale del driver, no de wgpu,
así que la string es la misma.

El consumo tampoco es estricto: `voxygen/src/render/renderer/mod.rs:266` matchea
**por substring** contra `format!("#{i} {name} {device_type:?}")`, o sea que mandar sólo
el nombre (lo que el launcher ya hace) matchea bien.

**Único riesgo residual:** el launcher usa wgpu **0.19.4** y el juego wgpu **27.0.1**.
Si algún driver reporta el nombre distinto entre versiones de wgpu, no matchearía. Bajo,
pero real; y hay un modo de falla feo: si `WGPU_ADAPTER` no matchea nada, el juego devuelve
`RenderError::CouldNotFindAdapter` — **no** cae a auto. Ver mitigación abajo.

### 2. Backends: enumerar con wgpu sería *peor* que lo que hay

`voxygen/src/main.rs:44-58` — `ListWgpuBackends` **no consulta nada**, imprime un array
hardcodeado por OS:

| OS      | Lista del juego            |
|---------|----------------------------|
| Windows | `opengl`, `dx12`, `vulkan` |
| Linux   | `opengl`, `vulkan`         |
| macOS   | `metal`                    |

Comparado con el fallback estático del launcher (`profiles.rs:92-104`):

| OS      | Lista del launcher                | Diferencia                            |
|---------|-----------------------------------|---------------------------------------|
| Windows | `Auto, DX11, DX12, Vulkan`        | **falta OpenGl; DX11 es fantasma**    |
| Linux   | `Auto, Vulkan`                    | **falta OpenGl**                      |
| macOS   | `Auto, Metal`                     | coincide                              |

**DX11 no existe.** wgpu eliminó el backend DX11 en 0.19; no está ni en el wgpu del
launcher ni en el del juego. Y el parser del juego
(`renderer/mod.rs:222-233`) no acepta `"dx11"`: cae en `_ => None` y silenciosamente usa
el default. O sea: hoy, elegir "DX11" en Settings no hace nada.

Conclusión: para backends **no hay que llamar a wgpu**, hay que **sincronizar la lista
estática con la del juego** (sacar DX11, agregar OpenGl en Windows/Linux). Enumerar con
wgpu incluso podría ofrecer un backend que el juego tiene hardcodeado fuera.

Nota aparte: `query_wgpu_backends` parsea `"opengl"` pero `Backend::to_str()` de wgpu
devuelve `"gl"` — otra razón para no mezclar las dos fuentes. (El mapeo de salida en
`Profile::start` sí usa `"gl"`, que el juego acepta como alias de `"opengl"`.)

## Conflicto con iced_wgpu

`iced_wgpu` 0.12.1 crea su propia `wgpu::Instance` y **ya llama a `enumerate_adapters()`**
al arrancar (`iced_wgpu-0.12.1/src/window/compositor.rs:38-43`, loguea "Available adapters").
Dos `Instance` coexistiendo es soportado por wgpu y no comparten estado.

Pero hay un problema conocido y concreto:

- [gfx-rs/wgpu#5930](https://github.com/gfx-rs/wgpu/issues/5930) — crear `wgpu::Instance`
  **desde múltiples hilos** en Linux + NVIDIA produce data race y SIGSEGV frecuente.
  Es específicamente `Instance::new`, no `Adapter`/`Device`.

Como iced crea la suya en el hilo de render, crear la nuestra en paralelo cae justo en
ese caso. Hay que enumerar **secuencialmente**, no concurrentemente con el arranque de iced.

Reusar lo que iced ya obtuvo **no alcanza**: `iced::system::fetch_information()` (feature
`system`, hoy no habilitada) devuelve `Information { adapter: String, backend: String }`
— el adapter **seleccionado**, uno solo. No sirve para poblar un dropdown.

## Riesgos concretos

1. **Segfault/hang de driver mata el launcher.** Además de #5930:
   [#2632](https://github.com/gfx-rs/wgpu/issues/2632) (segfault en `vkEnumeratePhysicalDevices`),
   [#2692](https://github.com/gfx-rs/wgpu/issues/2692) (falla de init con NVIDIA viejas),
   [#5349](https://github.com/gfx-rs/wgpu/issues/5349) (crash del backend GL en
   Wayland/surfaceless). El juego crasheando es malo; el *launcher* crasheando es peor,
   porque es la única puerta para reinstalar.
2. **Bloqueo del hilo de UI.** `reload_wgpu_*` ya usan `block_on` en `update()`. 14 ms acá,
   pero con loader Vulkan lento / llvmpipe / GPU híbrida en Windows puede ser segundos.
3. **Skew de versión wgpu 0.19 vs 27** (ver arriba), con modo de falla duro
   (`CouldNotFindAdapter`) si el nombre no matchea.
4. **`Backends::all()` inicializa GL**, que es el backend más frágil.

## Plan de implementación recomendado

**Enumerar en un proceso hijo, no in-process.** El launcher se re-ejecuta a sí mismo con
un subcomando oculto. Aísla segfaults, permite timeout, elimina por completo el problema
de #5930, y reusa el patrón de parseo de stdout que el código ya tiene.

1. `client/src/cli/parse.rs:52` — agregar variante `ListGpus` (oculta) al enum `Action`
   (ya es un `clap::Subcommand`, es trivial).
2. `client/Cargo.toml` — `wgpu = "0.19.4"` (1 línea; verificado: 1 línea en `Cargo.lock`).
3. `client/src/profiles.rs` — nueva `enumerate_local_gpus()`:
   `wgpu::Instance::new(InstanceDescriptor::default())` +
   `.enumerate_adapters(wgpu::Backends::all())`, imprimir `get_info().name` por línea
   (idéntico al juego). Handler del subcomando la llama y sale.
4. `reload_wgpu_devices()` (`profiles.rs:449`), rama `else` (no instalado): en vez de
   `vec![WgpuDevice::Auto]`, spawnear `current_exe() list-gpus` con **timeout de ~5 s**
   y parsear stdout con el mismo código que `query_wgpu_devices`. Si falla o timeoutea →
   `vec![WgpuDevice::Auto]` (el fallback actual, que se mantiene).
5. **Independiente y más urgente:** arreglar `WGPU_BACKENDS` (`profiles.rs:92-104`) para
   que coincida con `voxygen/src/main.rs:44-58`: sacar `DX11`, agregar `OpenGl` en
   Windows y Linux.
6. Mover las llamadas fuera del hilo de UI (`Command::perform`) en vez del `block_on`
   actual, y cachear el resultado.

**Esfuerzo: chico** (~100-150 líneas, 1 dep de una línea). El paso 5 solo es ~5 líneas.

## Qué confirmar con `xindeler-new-horizon`

- Si el juego sube de wgpu 27 a otra major, reverificar que `get_info().name` no cambió.
  Lo ideal a mediano plazo es alinear el wgpu del launcher con el del juego — pero eso
  requiere subir `iced` 0.12 → 0.13+, que es un refactor grande de la GUI, no de este scope.
- Considerar pedir al juego que `WGPU_ADAPTER` sin match caiga a auto con un warning en
  vez de `CouldNotFindAdapter` (`voxygen/src/render/renderer/mod.rs:266-283`). Eso vuelve
  todo este riesgo de mismatch inofensivo.
- Mejor aún: que `ListWgpuBackends` deje de ser hardcodeado, o que el launcher deje de
  llamarlo y ambos lean una lista única compartida.

## Recomendación

Hacer **ya** el paso 5 (lista estática sincronizada con el juego): es de 5 líneas y
arregla un bug real — hoy "DX11" es una opción que no hace nada y "OpenGl" falta.

La enumeración real de GPUs (pasos 1-4, 6) es viable y de bajo costo, pero el beneficio
es acotado: sólo aplica **antes** de la primera instalación, y casi siempre `Auto` es la
respuesta correcta. Yo lo haría después de que el flujo de instalación esté estable, y
**sólo** con la arquitectura de proceso hijo + timeout — nunca in-process en el hilo de UI.

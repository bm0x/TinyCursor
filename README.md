# TinyCursor 🚀
> Utilidad nativa de escritorio para **Smooth Cursor** con físicas de resortes (*spring dynamics*), rotación reactiva hacia el vector de velocidad, deformación elástica (*squash & stretch*) y diseño **Modern White**.

Inspirado en la especificación técnica de [Magic UI / Framer Motion](https://magicui.design/docs/components/smooth-cursor) y los cursores [Modern White](https://cdn.custom-cursor.com/cursors/modern_white.png), implementado en **Rust puro de ultra-bajo nivel** sin dependencias externas.

---

## ✨ Características Principales

* **Diseño Modern White (Flecha y Mano):**
  * **Cursor Normal (Flecha):** Silueta nítida blanca con borde oscuro redondeado y sombra suave (*drop shadow*), con rotación dinámica según el vector de movimiento y deformación elástica (*squash & stretch*).
  * **Cursor en Clic (Mano):** Se transforma automáticamente en la mano indicadora Modern White al hacer clic o interactuar, con micro-animación táctil de pulsación.
* **100% Visible sobre la Barra de Tareas e Íconos:**
  * Refuerzo continuo de banda Z-Order (`HWND_TOPMOST`), evitando que la barra de tareas de Windows (`Shell_TrayWnd`), el menú de inicio o los iconos del escritorio oculten el cursor.
* **Aplicación en Segundo Plano (Sin ventana de terminal):**
  * Compilado con `#![windows_subsystem = "windows"]`. Se ejecuta silenciosamente en segundo plano como Discord, Epic Games o Spotify.
* **Ícono en la Bandeja del Sistema (Taskbar System Tray):**
  * Muestra el ícono personalizado Modern White en la bandeja de notificaciones (junto al reloj).
  * **Menú contextual con clic derecho:**
    * *⏸ Pausar / ▶ Reanudar:* Restaura temporalmente el cursor normal de Windows para tareas que requieran el puntero nativo.
    * *👆 Mano en Clic:* Activa o desactiva la transformación en mano durante clics.
    * *✕ Salir:* Restaura inmediatamente todos los cursores y finaliza el proceso de forma segura.
  * **Clic izquierdo:** Alterna rápidamente entre pausar y reanudar.
* **Rendimiento Extremo:**
  * Tamaño del binario: **~310 KB**.
  * Consumo de CPU: **0.0% en reposo**, < 0.5% en movimiento activo.
  * Tasa de refresco: Compatible con monitores de 60 Hz, 144 Hz, 240 Hz y 360 Hz.
  * Click-through total sin latencia de entrada (`WS_EX_TRANSPARENT | WS_EX_LAYERED`).
* **Protección Fail-Safe:**
  * Oculta los cursores nativos del sistema y garantiza su restauración inmediata mediante RAII, manejador de pánicos o el atajo de emergencia **`Ctrl + Shift + Esc`**.

---

## 🛠️ Arquitectura

```
TinyCursor/
├── Cargo.toml                                 # Configuración sin dependencias externas
├── assets/
│   ├── arrow.png                              # Sprite Modern White Arrow
│   ├── hand.png                               # Sprite Modern White Hand
│   └── app_icon.ico                           # Ícono de aplicación para Windows
└── src/
    ├── main.rs                                # Loop de 240 Hz, GUI background y tray integration
    ├── core/                                  # CORE MATEMÁTICO (Multiplataforma)
    │   ├── mod.rs
    │   ├── math.rs                            # Vectores 2D, rotación angular y squash & stretch
    │   ├── physics.rs                         # Integración de Euler semi-implícita + sub-stepping
    │   ├── config.rs                          # Calibración de rigidez, amortiguación y elasticidad
    │   └── state.rs                           # Estados visuales del cursor
    └── platform/windows/                      # PLATAFORMA WINDOWS (Nativo Puro)
        ├── mod.rs
        ├── assets.rs                          # Texturas compiladas en memoria (0 lecturas a disco)
        ├── sys.rs                             # Enlace directo FFI (user32, gdi32, shell32, winmm)
        ├── overlay.rs                         # Ventana transparente WS_EX_TRANSPARENT + Topmost
        ├── tray.rs                            # Gestor de ícono en bandeja y menú contextual
        ├── cursor_guard.rs                    # Ocultación + Fail-Safe de cursor
        ├── input.rs                           # Polling de alta velocidad y detección de clic
        └── renderer.rs                        # Muestreo bilineal con drop shadow y DWM composition
```

---

## ⚡ Ejecución

Para iniciar TinyCursor en segundo plano:

```powershell
cargo run --release
```

O haciendo doble clic directamente sobre:
`target\release\tiny-cursor.exe`

Aparecerá el ícono de TinyCursor en la bandeja del sistema en la barra de tareas.

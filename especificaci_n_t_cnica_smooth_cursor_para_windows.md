# Especificación de Arquitectura y Prompt de Implementación: Smooth Cursor para Windows

Este documento contiene la especificación funcional, matemática y arquitectónica para replicar el componente **Smooth Cursor** (inspirado en [Magic UI / Framer Motion](https://magicui.design/docs/components/smooth-cursor)) como una aplicación nativa de escritorio para **Windows 10/11**.

---

## 1. Prompt Maestro para LLM de Programación

> **Copia y pega el siguiente bloque como instrucción inicial para el modelo de lenguaje o agente de código:**

```markdown
Eres un ingeniero de sistemas sénior especializado en desarrollo nativo para Windows (Win32 API, C#/.NET 8+ AOT, C++20 o Rust) y programación gráfica en tiempo real (Direct2D / Skia / DirectX).

Tu objetivo es construir una utilidad de escritorio ligera y de alto rendimiento para Windows que reemplace el puntero visual del sistema por un "Smooth Cursor" con físicas de resortes (spring physics), rotación reactiva hacia el vector de velocidad y deformación dinámica (squash & stretch), emulando el comportamiento del componente Smooth Cursor de Magic UI.

### Requisitos no negociables:
1. Rendimiento y latencia: Debe ejecutarse a la tasa de refresco nativa del monitor (60 Hz, 120 Hz, 144 Hz, 240 Hz) sin bloquear el hilo de entrada (input thread). Consumo de CPU < 1% en reposo.
2. Click-Through total: La ventana de overlay jamás debe interceptar clics, scrolls ni eventos de teclado (`WS_EX_TRANSPARENT` y `WS_EX_LAYERED`).
3. Ocultación del cursor del sistema: El cursor estándar de Windows debe volverse invisible mientras el programa esté activo y restaurarse de manera segura al cerrar la aplicación (incluso ante cierres inesperados o crash handling).
4. Físicas de segundo orden: Implementar un integrador numérico explícito (Semi-implicit Euler o Verlet) para la ecuación de resorte amortiguado (damping ratio $\zeta$, frecuencia angular $\omega_n$).
5. Geometría y transformaciones: La punta del cursor debe rotar hacia la dirección del vector de aceleración/velocidad y estirarse proporcionalmente a la rapidez instantánea.

Entrega el código modular, comentado, con manejo de DPI alto (Per-Monitor DPI Aware v2) y con instrucciones de compilación.
```

---

## 2. Visión General del Sistema

En Windows, los archivos `.cur` y `.ani` no admiten transformaciones afines en tiempo real ni simulación de fuerzas físicas. Por lo tanto, el sistema desacopla la **posición lógica del cursor** (hardware input) de su **representación gráfica visual**:

```
[Ratón Hardware] 
       │
       ▼
[Windows Raw Input / GetCursorPos] ───(Posición Real X, Y)
                                             │
                                             ▼
                             [Simulador de Físicas (Springs)]
                                             │
                                             ▼
                             [Matriz de Transformación 2D]
                                (Posición Suavizada, Rotación, Escala)
                                             │
                                             ▼
                             [Direct2D / Skia Graphics Pipeline]
                                             │
                                             ▼
                             [Overlay Topmost Transparente]
```

---

## 3. Componentes de la Arquitectura

### 3.1. Overlay de Ventana (Window Layer)
La ventana actúa como un lienzo transparente a pantalla completa sobre todos los escritorios y monitores:

* **Estilos extendidos de Win32:**
  * `WS_EX_TOPMOST`: Permite que la ventana esté siempre por encima de las demás ventanas estándar.
  * `WS_EX_TRANSPARENT`: Hace que cualquier prueba de impacto (`HitTest`) falle automáticamente, pasando los clics del ratón a la ventana que se encuentre debajo.
  * `WS_EX_LAYERED`: Habilita la composición alfa por píxel.
  * `WS_EX_TOOLWINDOW`: Oculta la ventana de la barra de tareas y del selector Alt+Tab.
  * `WS_EX_NOACTIVATE`: Previene que la ventana tome el foco al interactuar.

* **Soporte Multi-Monitor y Alta Resolución (DPI):**
  * Invocar `SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2)`.
  * La ventana debe cubrir el área virtual completa del escritorio obtenida mediante `GetSystemMetrics(SM_XVIRTUALSCREEN)`, `SM_YVIRTUALSCREEN`, `SM_CXVIRTUALSCREEN`, y `SM_CYVIRTUALSCREEN`.

### 3.2. Gestión del Cursor Nativo de Windows
Para evitar ver dos punteros simultáneos:
1. **Ocultación:** Crear un cursor transparente de $1 \times 1$ píxel en memoria usando `CreateCursor` con máscaras vacías y asignarlo a todos los identificadores del sistema (`OCR_NORMAL`, `OCR_IBEAM`, `OCR_HAND`, etc.) mediante `SetSystemCursor`.
2. **Restauración en salida:** Llamar a `SystemParametersInfo(SPI_SETCURSORS, 0, NULL, 0)` para restablecer los punteros del sistema a sus valores predeterminados al cerrar o en caso de excepción no controlada (`AppDomain.ProcessExit` / `SetUnhandledExceptionFilter`).

---

## 4. Modelo Físico y Matemático

El movimiento utiliza un sistema dinámico amortiguado de segundo orden expresado en diferencias finitas:

### 4.1. Ecuación del Resorte Amortiguado

$$\vec{a}(t) = \omega_n^2 (\vec{p}_{\text{real}}(t) - \vec{p}_{\text{visual}}(t)) - 2\zeta\omega_n \vec{v}(t)$$

Donde:
* $\vec{p}_{\text{real}}$: Posición real del ratón según el hardware.
* $\vec{p}_{\text{visual}}$: Posición interpolada donde se dibuja el puntero visual.
* $\vec{v}$: Vector de velocidad del puntero visual.
* $\omega_n$ (*Frecuencia natural* / *Stiffness*): Controla la rapidez con la que el cursor busca la posición real (valor recomendado: $25 - 45$).
* $\zeta$ (*Factor de amortiguamiento* / *Damping ratio*):
  * $\zeta < 1$: Subamortiguado (rebote elástico visible, estilo Magic UI).
  * $\zeta = 1$: Críticamente amortiguado (suavidad sin rebote).
  * Valor recomendado: $0.65 - 0.8$.

### 4.2. Integración Numérica (Semi-implicit Euler)
Para cada frame con paso de tiempo $\Delta t$:

$$\vec{v}_{t+\Delta t} = \vec{v}_t + \vec{a}_t \cdot \Delta t$$
$$\vec{p}_{\text{visual}, t+\Delta t} = \vec{p}_{\text{visual}, t} + \vec{v}_{t+\Delta t} \cdot \Delta t$$

### 4.3. Rotación Dinámica (Angular Alignment)
El ángulo de orientación $\theta$ del cursor se calcula a partir del vector velocidad:

$$\theta = \text{atan2}(v_y, v_x)$$

*Para evitar giros bruscos cuando el ratón se detiene ($||\vec{v}|| \to 0$), se debe aplicar una zona muerta (*deadzone*) o congelar el último ángulo registrado.*

### 4.4. Squash & Stretch (Deformación Elástica)
A medida que la magnitud de la velocidad $||\vec{v}||$ aumenta, el cursor se alarga a lo largo del eje local de movimiento y se comprime en el eje transversal para conservar masa/área percibida:

$$s_x = 1.0 + \min\left(\frac{||\vec{v}||}{v_{\max}}, 1.0\right) \cdot \lambda_{\text{stretch}}$$
$$s_y = \frac{1.0}{\sqrt{s_x}}$$

Donde:
* $\lambda_{\text{stretch}}$: Coeficiente de elasticidad (e.g., $0.3 - 0.5$).
* $v_{\max}$: Velocidad límite de referencia para la saturación de escala.

---

## 5. Implementación de Referencia (C# / .NET 8 con Win32 Interop)

### 5.1. Bucle de Simulación de Física
```csharp
public class SmoothCursorPhysics
{
    public float X { get; private set; }
    public float Y { get; private set; }
    public float VelocityX { get; private set; }
    public float VelocityY { get; private set; }
    public float Angle { get; private set; }
    public float ScaleX { get; private set; } = 1f;
    public float ScaleY { get; private set; } = 1f;

    // Parámetros de resorte configurables
    public float Stiffness { get; set; } = 350.0f; // Constante elástica
    public float Damping { get; set; } = 28.0f;    // Fricción/amortiguación
    public float MaxVelocity { get; set; } = 4000.0f;
    public float StretchFactor { get; set; } = 0.35f;

    public void Update(float targetX, float targetY, float deltaTime)
    {
        if (deltaTime <= 0f || deltaTime > 0.1f) deltaTime = 0.016f; // Clamp delta

        // Fuerza del resorte: F = k * (target - current) - c * v
        float forceX = (targetX - X) * Stiffness - VelocityX * Damping;
        float forceY = (targetY - Y) * Stiffness - VelocityY * Damping;

        VelocityX += forceX * deltaTime;
        VelocityY += forceY * deltaTime;

        X += VelocityX * deltaTime;
        Y += VelocityY * deltaTime;

        float speed = MathF.Sqrt(VelocityX * VelocityX + VelocityY * VelocityY);

        // Actualizar rotación solo si la velocidad supera un umbral
        if (speed > 15.0f)
        {
            Angle = MathF.Atan2(VelocityY, VelocityX);
        }

        // Deformación (Squash & Stretch)
        float normalizedSpeed = Math.Clamp(speed / MaxVelocity, 0f, 1f);
        ScaleX = 1.0f + (normalizedSpeed * StretchFactor);
        ScaleY = 1.0f / MathF.Sqrt(ScaleX);
    }
}
```

### 5.2. Configuración de la Ventana Win32 (Click-Through)
```csharp
[DllImport("user32.dll")]
private static extern int SetWindowLong(IntPtr hWnd, int nIndex, int dwNewLong);

[DllImport("user32.dll")]
private static extern int GetWindowLong(IntPtr hWnd, int nIndex);

private const int GWL_EXSTYLE = -20;
private const int WS_EX_TRANSPARENT = 0x00000020;
private const int WS_EX_LAYERED     = 0x00080000;
private const int WS_EX_TOPMOST     = 0x00000008;
private const int WS_EX_TOOLWINDOW  = 0x00000080;

public static void ApplyTransparentOverlayStyles(IntPtr windowHandle)
{
    int currentStyle = GetWindowLong(windowHandle, GWL_EXSTYLE);
    SetWindowLong(windowHandle, GWL_EXSTYLE, 
        currentStyle | WS_EX_TRANSPARENT | WS_EX_LAYERED | WS_EX_TOPMOST | WS_EX_TOOLWINDOW);
}
```

---

## 6. Consideraciones de Rendimiento y Edge Cases

1. **Juegos a Pantalla Completa (Exclusive Fullscreen):**
   * El overlay debe detectar cuándo una ventana entra en modo pantalla completa exclusiva (`GetForegroundWindow` + comprobar bordes de monitor) y deshabilitar temporalmente el dibujo para no interferir ni perder rendimiento.
2. **Latencia de VSync:**
   * Utilizar sincronización mediante `DXGI Present` con buffer `DXGI_SWAP_EFFECT_FLIP_DISCARD` para evitar latencia de fotogramas encolados.
3. **Pausa por Inactividad:**
   * Cuando $||\vec{v}|| < 0.001$ y $|\vec{p}_{\text{real}} - \vec{p}_{\text{visual}}| < 0.1$, detener los redibujados continuos para reducir el uso de GPU/CPU a 0%. Reactivar inmediatamente con el próximo evento de entrada.
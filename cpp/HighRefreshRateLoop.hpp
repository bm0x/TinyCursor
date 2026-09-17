#pragma once

#include <windows.h>
#include <d3d11.h>
#include <dxgi1_2.h>
#include <algorithm>

#include "TinyPhysics.hpp"
#include "CanvasRenderer.hpp"

namespace TinyCursor {

class HighRefreshRateLoop {
private:
    IDXGISwapChain1* pSwapChain = nullptr;
    ID2D1DeviceContext* pD2DContext = nullptr;
    LARGE_INTEGER perfFreq{};
    LARGE_INTEGER prevCounter{};
    HMONITOR currentMonitor = nullptr;
    DWORD currentDisplayHz = 144;

public:
    void Initialize(IDXGISwapChain1* swapChain, ID2D1DeviceContext* d2dContext) {
        this->pSwapChain = swapChain;
        this->pD2DContext = d2dContext;
        QueryPerformanceFrequency(&perfFreq);
        QueryPerformanceCounter(&prevCounter);
    }

    void RunTick(TinyPhysics& physics, CanvasRenderer& renderer) {
        LARGE_INTEGER currentCounter;
        QueryPerformanceCounter(&currentCounter);

        // Sub-microsecond delta time calculation
        double deltaTime = static_cast<double>(currentCounter.QuadPart - prevCounter.QuadPart) 
                           / static_cast<double>(perfFreq.QuadPart);
        prevCounter = currentCounter;

        // Clamp anomalies (e.g. during window dragging or OS suspension)
        if (deltaTime <= 0.0 || deltaTime > 0.05) {
            deltaTime = 1.0 / static_cast<double>(currentDisplayHz);
        }

        // 1. Query raw hardware mouse position
        POINT targetPos;
        GetCursorPos(&targetPos);

        // 2. Handle Multi-Monitor mixed frequencies (e.g. 144 Hz primary + 60 Hz secondary)
        HMONITOR hMon = MonitorFromPoint(targetPos, MONITOR_DEFAULTTONEAREST);
        if (hMon != currentMonitor) {
            currentMonitor = hMon;
            MONITORINFOEXW mi{};
            mi.cbSize = sizeof(mi);
            if (GetMonitorInfoW(hMon, &mi)) {
                DEVMODEW dm{};
                dm.dmSize = sizeof(dm);
                if (EnumDisplaySettingsExW(mi.szDevice, ENUM_CURRENT_SETTINGS, &dm, 0)) {
                    if (dm.dmDisplayFrequency >= 30) {
                        currentDisplayHz = dm.dmDisplayFrequency;
                    }
                }
            }
        }

        // 3. Sample background luminance under real mouse coordinates
        HDC hdcScreen = GetDC(nullptr);
        if (hdcScreen) {
            COLORREF pixel = GetPixel(hdcScreen, targetPos.x, targetPos.y);
            ReleaseDC(nullptr, hdcScreen);

            if (pixel != CLR_INVALID) {
                const float r = static_cast<float>(GetRValue(pixel)) / 255.0f;
                const float g = static_cast<float>(GetGValue(pixel)) / 255.0f;
                const float b = static_cast<float>(GetBValue(pixel)) / 255.0f;
                renderer.UpdatePerceptualLuminance(r, g, b, static_cast<float>(deltaTime));
            }
        }

        // 4. Solve Spring Physics (Semi-implicit Euler + Squash & Stretch)
        physics.Update(
            static_cast<float>(targetPos.x),
            static_cast<float>(targetPos.y),
            static_cast<float>(deltaTime)
        );

        // 5. Draw cursor into Direct2D target surface
        if (pD2DContext) {
            pD2DContext->BeginDraw();
            pD2DContext->Clear(D2D1::ColorF(0.0f, 0.0f, 0.0f, 0.0f));

            const bool isClicking = (GetAsyncKeyState(VK_LBUTTON) & 0x8000) != 0;
            renderer.DrawCursor(physics.GetRenderState(), CursorShape::Arrow, isClicking);

            pD2DContext->EndDraw();
        }

        // 6. Hardware VBLANK Lock via Modern Flip Model
        // STRICT: Zero Thread::sleep, zero Task::delay, zero SetTimer.
        // SyncInterval = 1 locks directly to the monitor's VBLANK interrupt:
        // 144 Hz display -> blocks for ~6.94 ms
        // 240 Hz display -> blocks for ~4.16 ms
        // 60 Hz display  -> blocks for ~16.66 ms
        pSwapChain->Present(1, 0);
    }
};

} // namespace TinyCursor

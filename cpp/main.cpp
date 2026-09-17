#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <d3d11_1.h>
#include <dxgi1_2.h>
#include <d2d1_1.h>
#include <dcomp.h>
#include <wrl/client.h>
#include <shellapi.h>

#include "TinyPhysics.hpp"
#include "CanvasRenderer.hpp"
#include "HighRefreshRateLoop.hpp"

#pragma comment(lib, "d3d11.lib")
#pragma comment(lib, "dxgi.lib")
#pragma comment(lib, "d2d1.lib")
#pragma comment(lib, "dcomp.lib")
#pragma comment(lib, "user32.lib")
#pragma comment(lib, "gdi32.lib")
#pragma comment(lib, "shell32.lib")

using Microsoft::WRL::ComPtr;

// Dynamically resolved internal window band function signatures
typedef HWND(WINAPI* PfnCreateWindowInBand)(
    DWORD dwExStyle,
    LPCWSTR lpClassName,
    LPCWSTR lpWindowName,
    DWORD dwStyle,
    int X, int Y, int nWidth, int nHeight,
    HWND hWndParent,
    HMENU hMenu,
    HINSTANCE hInstance,
    LPVOID lpParam,
    DWORD dwBand
);

constexpr DWORD ZBID_UIACCESS = 12;
constexpr DWORD ZBID_SYSTEM_TOOLS = 11;
constexpr LPCWSTR OVERLAY_CLASS = L"TinyCursorOverlayDirectX";

// Global RAII guard to ensure system cursors are always restored
class CursorGuard {
public:
    CursorGuard() {
        HideSystemCursors();
    }
    ~CursorGuard() {
        RestoreSystemCursors();
    }

    static void HideSystemCursors() {
        BYTE andMask[1] = { 0xFF };
        BYTE xorMask[1] = { 0x00 };
        HCURSOR blank = CreateCursor(nullptr, 0, 0, 1, 1, andMask, xorMask);
        if (blank) {
            SetSystemCursor(blank, 32512); // OCR_NORMAL
        }
    }

    static void RestoreSystemCursors() {
        SystemParametersInfoW(SPI_SETCURSORS, 0, nullptr, 0);
    }
};

LRESULT CALLBACK WndProc(HWND hwnd, UINT msg, WPARAM wParam, LPARAM lParam) {
    if (msg == WM_DESTROY) {
        PostQuitMessage(0);
        return 0;
    }
    return DefWindowProcW(hwnd, msg, wParam, lParam);
}

int WINAPI wWinMain(HINSTANCE hInstance, HINSTANCE, PWSTR, int) {
    SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);

    WNDCLASSEXW wc{};
    wc.cbSize = sizeof(wc);
    wc.lpfnWndProc = WndProc;
    wc.hInstance = hInstance;
    wc.lpszClassName = OVERLAY_CLASS;
    RegisterClassExW(&wc);

    const DWORD exStyle = WS_EX_TOPMOST
        | WS_EX_TRANSPARENT
        | WS_EX_NOREDIRECTIONBITMAP
        | WS_EX_TOOLWINDOW
        | WS_EX_NOACTIVATE;

    const int vx = GetSystemMetrics(SM_XVIRTUALSCREEN);
    const int vy = GetSystemMetrics(SM_YVIRTUALSCREEN);
    const int vw = (std::max)(GetSystemMetrics(SM_CXVIRTUALSCREEN), 1920);
    const int vh = (std::max)(GetSystemMetrics(SM_CYVIRTUALSCREEN), 1080);

    // 1. First priority: Elevated Z-Bands via CreateWindowInBand
    HMODULE hUser32 = GetModuleHandleW(L"user32.dll");
    auto pCreateWindowInBand = (PfnCreateWindowInBand)GetProcAddress(hUser32, "CreateWindowInBand");

    HWND hwnd = nullptr;
    if (pCreateWindowInBand) {
        hwnd = pCreateWindowInBand(
            exStyle, OVERLAY_CLASS, OVERLAY_CLASS, WS_POPUP,
            vx, vy, vw, vh,
            nullptr, nullptr, hInstance, nullptr,
            ZBID_UIACCESS
        );
    }

    // 2. Fallback to standard topmost popup
    if (!hwnd) {
        hwnd = CreateWindowExW(
            exStyle, OVERLAY_CLASS, OVERLAY_CLASS, WS_POPUP,
            vx, vy, vw, vh,
            nullptr, nullptr, hInstance, nullptr
        );
    }

    if (!hwnd) return 1;

    ShowWindow(hwnd, SW_SHOW);
    SetWindowPos(hwnd, HWND_TOPMOST, vx, vy, vw, vh, SWP_NOACTIVATE | SWP_SHOWWINDOW);

    // 3. Initialize Direct3D 11.1 Device and Modern DXGI Flip Model SwapChain
    ComPtr<ID3D11Device> pD3DDevice;
    ComPtr<ID3D11DeviceContext> pD3DContext;
    D3D_FEATURE_LEVEL featureLevel;

    HRESULT hr = D3D11CreateDevice(
        nullptr,
        D3D_DRIVER_TYPE_HARDWARE,
        nullptr,
        D3D11_CREATE_DEVICE_BGRA_SUPPORT | D3D11_CREATE_DEVICE_SINGLETHREADED,
        nullptr, 0,
        D3D11_SDK_VERSION,
        &pD3DDevice,
        &featureLevel,
        &pD3DContext
    );
    if (FAILED(hr)) return 1;

    ComPtr<IDXGIDevice> pDXGIDevice;
    pD3DDevice.As(&pDXGIDevice);

    ComPtr<IDXGIAdapter> pAdapter;
    pDXGIDevice->GetAdapter(&pAdapter);

    ComPtr<IDXGIFactory2> pDXGIFactory;
    pAdapter->GetParent(IID_PPV_ARGS(&pDXGIFactory));

    DXGI_SWAP_CHAIN_DESC1 swapDesc{};
    swapDesc.Width = static_cast<UINT>(vw);
    swapDesc.Height = static_cast<UINT>(vh);
    swapDesc.Format = DXGI_FORMAT_B8G8R8A8_UNORM;
    swapDesc.SampleDesc.Count = 1;
    swapDesc.BufferUsage = DXGI_USAGE_RENDER_TARGET_OUTPUT;
    swapDesc.BufferCount = 2;
    swapDesc.Scaling = DXGI_SCALING_STRETCH;
    swapDesc.SwapEffect = DXGI_SWAP_EFFECT_FLIP_DISCARD;
    swapDesc.AlphaMode = DXGI_ALPHA_MODE_PREMULTIPLIED;

    ComPtr<IDXGISwapChain1> pSwapChain;
    hr = pDXGIFactory->CreateSwapChainForComposition(pD3DDevice.Get(), &swapDesc, nullptr, &pSwapChain);
    if (FAILED(hr)) return 1;

    // 4. Connect to DirectComposition Visual Tree
    ComPtr<IDCompositionDevice> pDCompDevice;
    DCompositionCreateDevice(pDXGIDevice.Get(), IID_PPV_ARGS(&pDCompDevice));

    ComPtr<IDCompositionTarget> pDCompTarget;
    pDCompDevice->CreateTargetForHwnd(hwnd, TRUE, &pDCompTarget);

    ComPtr<IDCompositionVisual> pDCompVisual;
    pDCompDevice->CreateVisual(&pDCompVisual);
    pDCompVisual->SetContent(pSwapChain.Get());
    pDCompTarget->SetRoot(pDCompVisual.Get());
    pDCompDevice->Commit();

    // 5. Initialize Direct2D Factory and Device Context
    D2D1_FACTORY_OPTIONS d2dOptions{};
    ComPtr<ID2D1Factory1> pD2DFactory;
    D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, __uuidof(ID2D1Factory1), &d2dOptions, &pD2DFactory);

    ComPtr<ID2D1Device> pD2DDevice;
    pD2DFactory->CreateDevice(pDXGIDevice.Get(), &pD2DDevice);

    ComPtr<ID2D1DeviceContext> pD2DContext;
    pD2DDevice->CreateDeviceContext(D2D1_DEVICE_CONTEXT_OPTIONS_NONE, &pD2DContext);

    ComPtr<IDXGISurface> pBackBuffer;
    pSwapChain->GetBuffer(0, IID_PPV_ARGS(&pBackBuffer));

    D2D1_BITMAP_PROPERTIES1 bitmapProps = D2D1::BitmapProperties1(
        D2D1_BITMAP_OPTIONS_TARGET | D2D1_BITMAP_OPTIONS_CANNOT_DRAW,
        D2D1::PixelFormat(DXGI_FORMAT_B8G8R8A8_UNORM, D2D1_ALPHA_MODE_PREMULTIPLIED),
        96.0f, 96.0f
    );

    ComPtr<ID2D1Bitmap1> pTargetBitmap;
    pD2DContext->CreateBitmapFromDxgiSurface(pBackBuffer.Get(), &bitmapProps, &pTargetBitmap);
    pD2DContext->SetTarget(pTargetBitmap.Get());
    pD2DContext->SetAntialiasMode(D2D1_ANTIALIAS_MODE_PER_PRIMITIVE);

    // 6. Initialize Physics, Renderer, and Hardware VBLANK Loop
    POINT startPos;
    GetCursorPos(&startPos);

    TinyCursor::TinyPhysics physics(static_cast<float>(startPos.x), static_cast<float>(startPos.y));
    TinyCursor::CanvasRenderer renderer;
    renderer.Initialize(pD2DFactory.Get(), pD2DContext.Get());

    TinyCursor::HighRefreshRateLoop loop;
    loop.Initialize(pSwapChain.Get(), pD2DContext.Get());

    // Hide native cursor with RAII restoration
    CursorGuard guard;

    MSG msg{};
    while (msg.message != WM_QUIT) {
        // Poll OS window messages without blocking
        while (PeekMessageW(&msg, nullptr, 0, 0, PM_REMOVE)) {
            if (msg.message == WM_QUIT) break;
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        // Emergency exit hotkey: Ctrl + Alt + Shift + Esc
        const bool ctrl = (GetAsyncKeyState(VK_CONTROL) & 0x8000) != 0;
        const bool alt = (GetAsyncKeyState(VK_MENU) & 0x8000) != 0;
        const bool shift = (GetAsyncKeyState(VK_SHIFT) & 0x8000) != 0;
        const bool esc = (GetAsyncKeyState(VK_ESCAPE) & 0x8000) != 0;
        if (ctrl && alt && shift && esc) {
            break;
        }

        // Hardware VBLANK synchronized tick (144 Hz = 144 FPS)
        loop.RunTick(physics, renderer);
    }

    return 0;
}

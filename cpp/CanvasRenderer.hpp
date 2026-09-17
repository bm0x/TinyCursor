#pragma once

#include <d2d1_1.h>
#include <d2d1_1helper.h>
#include <dwrite.h>
#include <wrl/client.h>
#include <algorithm>
#include <cmath>

#include "TinyPhysics.hpp"

namespace TinyCursor {

using Microsoft::WRL::ComPtr;

enum class CursorShape {
    Arrow,
    Hand,
    IBeam,
    ResizeNS,
    ResizeWE,
    Zoom,
};

class CanvasRenderer {
private:
    ComPtr<ID2D1Factory1> pFactory;
    ComPtr<ID2D1DeviceContext> pContext;

    // Cached Path Geometries
    ComPtr<ID2D1PathGeometry> pGeoArrow;
    ComPtr<ID2D1PathGeometry> pGeoHand;
    ComPtr<ID2D1PathGeometry> pGeoIBeam;
    ComPtr<ID2D1PathGeometry> pGeoResizeNS;
    ComPtr<ID2D1PathGeometry> pGeoResizeWE;
    ComPtr<ID2D1PathGeometry> pGeoZoom;

    // Brushes
    ComPtr<ID2D1SolidColorBrush> pFillBrush;
    ComPtr<ID2D1SolidColorBrush> pBevelBrush;
    ComPtr<ID2D1SolidColorBrush> pShadowBrush;

    float currentThemeBlend = 0.0f; // 0.0 = White/Pearl, 1.0 = Black/Obsidian
    float targetThemeBlend = 0.0f;

public:
    HRESULT Initialize(ID2D1Factory1* factory, ID2D1DeviceContext* context) {
        pFactory = factory;
        pContext = context;

        HRESULT hr = S_OK;

        // 1. Create Solid Color Brushes
        hr = pContext->CreateSolidColorBrush(D2D1::ColorF(0.98f, 0.98f, 1.00f, 0.98f), &pFillBrush);
        if (FAILED(hr)) return hr;
        hr = pContext->CreateSolidColorBrush(D2D1::ColorF(0.85f, 0.88f, 0.92f, 0.95f), &pBevelBrush);
        if (FAILED(hr)) return hr;
        hr = pContext->CreateSolidColorBrush(D2D1::ColorF(0.0f, 0.0f, 0.0f, 0.35f), &pShadowBrush);
        if (FAILED(hr)) return hr;

        // 2. Build Cached 3D Volumetric Cursor Geometries (Googlebook / Magic Pointer style)
        hr = BuildArrowGeometry();
        if (FAILED(hr)) return hr;
        hr = BuildHandGeometry();
        if (FAILED(hr)) return hr;
        hr = BuildIBeamGeometry();
        if (FAILED(hr)) return hr;

        return S_OK;
    }

    /// Adapts contrast based on desktop pixel luminance under hardware mouse coordinates:
    /// L = 0.2126 * R + 0.7152 * G + 0.0722 * B
    void UpdatePerceptualLuminance(float r, float g, float b, float dt) {
        const float lum = 0.2126f * r + 0.7152f * g + 0.0722f * b;

        // Hysteresis deadband: > 0.55 light background (Black cursor), < 0.45 dark (White cursor)
        if (lum > 0.55f) {
            targetThemeBlend = 1.0f;
        } else if (lum < 0.45f) {
            targetThemeBlend = 0.0f;
        }

        const float blendDiff = targetThemeBlend - currentThemeBlend;
        if (std::abs(blendDiff) > 0.001f) {
            currentThemeBlend += blendDiff * (1.0f - std::exp(-8.5f * dt));
        } else {
            currentThemeBlend = targetThemeBlend;
        }

        // Interpolate colors between Pearl White and Obsidian Black
        const float t = std::clamp(currentThemeBlend, 0.0f, 1.0f);
        const float fillR = 0.98f + (0.08f - 0.98f) * t;
        const float fillG = 0.98f + (0.08f - 0.98f) * t;
        const float fillB = 1.00f + (0.10f - 1.00f) * t;
        pFillBrush->SetColor(D2D1::ColorF(fillR, fillG, fillB, 0.98f));

        const float bevR = 0.85f + (0.38f - 0.85f) * t;
        const float bevG = 0.88f + (0.38f - 0.88f) * t;
        const float bevB = 0.92f + (0.42f - 0.92f) * t;
        pBevelBrush->SetColor(D2D1::ColorF(bevR, bevG, bevB, 0.95f));
    }

    void DrawCursor(const RenderState& state, CursorShape shape = CursorShape::Arrow, bool isClicking = false) {
        ID2D1PathGeometry* pGeo = pGeoArrow.Get();
        float hotspotX = 3.0f;
        float hotspotY = 2.0f;

        if (shape == CursorShape::Hand && pGeoHand) {
            pGeo = pGeoHand.Get();
            hotspotX = 10.0f;
            hotspotY = 2.0f;
        } else if (shape == CursorShape::IBeam && pGeoIBeam) {
            pGeo = pGeoIBeam.Get();
            hotspotX = 8.0f;
            hotspotY = 13.0f;
        }

        const float clickScale = isClicking ? 0.88f : 1.0f;
        const float sx = state.scaleX * clickScale;
        const float sy = state.scaleY * clickScale;

        // 1. Draw Soft Drop Projection Shadow (3px offset along Y, 6px blur equivalent)
        const auto shadowMatrix = MakeAffineMatrix(
            state.x + 2.5f,
            state.y + 3.5f,
            state.angle,
            sx * 1.04f,
            sy * 1.04f,
            hotspotX,
            hotspotY
        );
        pContext->SetTransform(shadowMatrix);
        pContext->FillGeometry(pGeo, pShadowBrush.Get());

        // 2. Draw Main Volumetric 3D Body
        const auto bodyMatrix = MakeAffineMatrix(
            state.x,
            state.y,
            state.angle,
            sx,
            sy,
            hotspotX,
            hotspotY
        );
        pContext->SetTransform(bodyMatrix);
        pContext->FillGeometry(pGeo, pFillBrush.Get());

        // 3. Draw Pronounced 3D Bevel Perimeter
        pContext->DrawGeometry(pGeo, pBevelBrush.Get(), 1.6f);

        // Reset Transform to Identity
        pContext->SetTransform(D2D1::Matrix3x2F::Identity());
    }

private:
    [[nodiscard]] static D2D1::Matrix3x2F MakeAffineMatrix(
        float posX, float posY, float angleRad,
        float scaleX, float scaleY,
        float originX, float originY) noexcept
    {
        const float cosA = std::cos(angleRad);
        const float sinA = std::sin(angleRad);

        const float m11 = scaleX * cosA;
        const float m12 = scaleX * sinA;
        const float m21 = -scaleY * sinA;
        const float m22 = scaleY * cosA;

        const float dx = posX - (m11 * originX + m21 * originY);
        const float dy = posY - (m12 * originX + m22 * originY);

        return D2D1::Matrix3x2F(m11, m12, m21, m22, dx, dy);
    }

    HRESULT BuildArrowGeometry() {
        HRESULT hr = pFactory->CreatePathGeometry(&pGeoArrow);
        if (FAILED(hr)) return hr;

        ComPtr<ID2D1GeometrySink> sink;
        hr = pGeoArrow->Open(&sink);
        if (FAILED(hr)) return hr;

        sink->BeginFigure(D2D1::Point2F(3.0f, 2.0f), D2D1_FIGURE_BEGIN_FILLED);
        sink->AddBezier(D2D1::BezierSegment(
            D2D1::Point2F(4.0f, 10.0f),
            D2D1::Point2F(3.5f, 20.0f),
            D2D1::Point2F(3.0f, 28.0f)
        ));
        sink->AddBezier(D2D1::BezierSegment(
            D2D1::Point2F(4.5f, 28.5f),
            D2D1::Point2F(6.5f, 25.0f),
            D2D1::Point2F(9.0f, 22.0f)
        ));
        sink->AddLine(D2D1::Point2F(21.0f, 22.0f));
        sink->AddBezier(D2D1::BezierSegment(
            D2D1::Point2F(21.5f, 20.5f),
            D2D1::Point2F(14.0f, 12.0f),
            D2D1::Point2F(3.0f, 2.0f)
        ));
        sink->EndFigure(D2D1_FIGURE_END_CLOSED);
        return sink->Close();
    }

    HRESULT BuildHandGeometry() {
        HRESULT hr = pFactory->CreatePathGeometry(&pGeoHand);
        if (FAILED(hr)) return hr;

        ComPtr<ID2D1GeometrySink> sink;
        hr = pGeoHand->Open(&sink);
        if (FAILED(hr)) return hr;

        sink->BeginFigure(D2D1::Point2F(10.0f, 2.0f), D2D1_FIGURE_BEGIN_FILLED);
        sink->AddLine(D2D1::Point2F(8.0f, 14.0f));
        sink->AddBezier(D2D1::BezierSegment(
            D2D1::Point2F(4.0f, 16.0f),
            D2D1::Point2F(2.0f, 19.0f),
            D2D1::Point2F(4.0f, 23.0f)
        ));
        sink->AddLine(D2D1::Point2F(7.0f, 28.0f));
        sink->AddLine(D2D1::Point2F(18.0f, 28.0f));
        sink->AddBezier(D2D1::BezierSegment(
            D2D1::Point2F(20.0f, 23.0f),
            D2D1::Point2F(19.0f, 14.0f),
            D2D1::Point2F(13.0f, 14.0f)
        ));
        sink->AddLine(D2D1::Point2F(13.0f, 2.0f));
        sink->AddBezier(D2D1::BezierSegment(
            D2D1::Point2F(13.0f, 0.5f),
            D2D1::Point2F(10.0f, 0.5f),
            D2D1::Point2F(10.0f, 2.0f)
        ));
        sink->EndFigure(D2D1_FIGURE_END_CLOSED);
        return sink->Close();
    }

    HRESULT BuildIBeamGeometry() {
        HRESULT hr = pFactory->CreatePathGeometry(&pGeoIBeam);
        if (FAILED(hr)) return hr;

        ComPtr<ID2D1GeometrySink> sink;
        hr = pGeoIBeam->Open(&sink);
        if (FAILED(hr)) return hr;

        sink->BeginFigure(D2D1::Point2F(3.0f, 3.0f), D2D1_FIGURE_BEGIN_FILLED);
        sink->AddLine(D2D1::Point2F(13.0f, 3.0f));
        sink->AddLine(D2D1::Point2F(13.0f, 6.0f));
        sink->AddLine(D2D1::Point2F(9.5f, 6.0f));
        sink->AddLine(D2D1::Point2F(9.5f, 20.0f));
        sink->AddLine(D2D1::Point2F(13.0f, 20.0f));
        sink->AddLine(D2D1::Point2F(13.0f, 23.0f));
        sink->AddLine(D2D1::Point2F(3.0f, 23.0f));
        sink->AddLine(D2D1::Point2F(3.0f, 20.0f));
        sink->AddLine(D2D1::Point2F(6.5f, 20.0f));
        sink->AddLine(D2D1::Point2F(6.5f, 6.0f));
        sink->AddLine(D2D1::Point2F(3.0f, 6.0f));
        sink->EndFigure(D2D1_FIGURE_END_CLOSED);
        return sink->Close();
    }
};

} // namespace TinyCursor

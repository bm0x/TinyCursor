#pragma once

#include <cmath>
#include <numbers>
#include <algorithm>

namespace TinyCursor {

struct RenderState {
    float x;
    float y;
    float angle;
    float scaleX;
    float scaleY;
    bool isSettled;
};

class TinyPhysics {
public:
    // Physical spring constants matching specification
    // Angular frequency: omega_n = 32.0 rad/s -> stiffness k = omega_n^2 = 1024.0
    // Damping ratio: zeta = 0.72 -> damping coefficient c = 2 * zeta * omega_n = 46.08
    float stiffness = 1024.0f;
    float damping = 46.08f;
    float maxVelocity = 3200.0f;
    float stretchFactor = 0.35f;
    float minRotationSpeed = 20.0f;

    // Resting angle: ~ -118 degrees (top-left)
    static constexpr float RESTING_ANGLE = -2.06f;

private:
    float posX = 0.0f;
    float posY = 0.0f;
    float velX = 0.0f;
    float velY = 0.0f;
    float currentAngle = RESTING_ANGLE;
    float currentScaleX = 1.0f;
    float currentScaleY = 1.0f;
    bool isSettled = true;

public:
    TinyPhysics(float startX = 0.0f, float startY = 0.0f)
        : posX(startX), posY(startY) {}

    void Teleport(float x, float y) {
        posX = x;
        posY = y;
        velX = 0.0f;
        velY = 0.0f;
        currentScaleX = 1.0f;
        currentScaleY = 1.0f;
        isSettled = true;
    }

    void Update(float targetX, float targetY, float deltaTime) {
        // Clamp delta to avoid divergence on lag spikes
        const float dt = std::clamp(deltaTime, 0.0005f, 0.05f);

        // Sub-stepping at 500 Hz for high numerical precision
        constexpr float MAX_SUB_STEP = 0.002f;
        const int subSteps = static_cast<int>(std::ceil(dt / MAX_SUB_STEP));
        const float subDt = dt / static_cast<float>(subSteps);

        for (int i = 0; i < subSteps; ++i) {
            // Semi-implicit Euler integration:
            // a = omega_n^2 * (x_target - x) - 2 * zeta * omega_n * v
            const float dispX = targetX - posX;
            const float dispY = targetY - posY;

            const float forceX = dispX * stiffness - velX * damping;
            const float forceY = dispY * stiffness - velY * damping;

            velX += forceX * subDt;
            velY += forceY * subDt;

            posX += velX * subDt;
            posY += velY * subDt;
        }

        const float speed = std::sqrt(velX * velX + velY * velY);
        const float distToTarget = std::hypot(targetX - posX, targetY - posY);

        // Equilibrium check for 0% CPU consumption in idle mode
        if (speed < 0.2f && distToTarget < 0.3f) {
            posX = targetX;
            posY = targetY;
            velX = 0.0f;
            velY = 0.0f;
            currentScaleX += (1.0f - currentScaleX) * (1.0f - std::exp(-12.0f * dt));
            currentScaleY = 1.0f / std::sqrt(currentScaleX);
            isSettled = true;
            return;
        }

        isSettled = false;

        // 1. Angular orientation based on velocity direction
        float targetAngle = currentAngle;
        if (speed > minRotationSpeed) {
            targetAngle = std::atan2(velY, velX);
        }

        // 2. Shortest-arc angular interpolation (Slerp)
        constexpr float PI = std::numbers::pi_v<float>;
        float angleDiff = std::fmod(targetAngle - currentAngle + PI, 2.0f * PI);
        if (angleDiff < 0.0f) angleDiff += 2.0f * PI;
        angleDiff -= PI;

        const float angularSpeed = 24.0f + std::min(speed / 300.0f, 16.0f);
        const float angularBlend = std::clamp(1.0f - std::exp(-angularSpeed * dt), 0.0f, 1.0f);
        currentAngle += angleDiff * angularBlend;

        // 3. Area/volume-conserving Squash & Stretch:
        // s_x = 1.0 + min(||v|| / v_max, 1.0) * lambda
        // s_y = 1.0 / sqrt(s_x)
        const float velRatio = std::min(speed / maxVelocity, 1.0f);
        const float targetSx = 1.0f + velRatio * stretchFactor;

        const float scaleBlend = std::clamp(1.0f - std::exp(-20.0f * dt), 0.0f, 1.0f);
        currentScaleX += (targetSx - currentScaleX) * scaleBlend;
        currentScaleY = 1.0f / std::sqrt(currentScaleX);
    }

    [[nodiscard]] RenderState GetRenderState() const noexcept {
        return RenderState{
            .x = posX,
            .y = posY,
            .angle = currentAngle,
            .scaleX = currentScaleX,
            .scaleY = currentScaleY,
            .isSettled = isSettled,
        };
    }
};

} // namespace TinyCursor

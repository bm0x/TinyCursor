// =====================================================================
// TinyCursor - Magic UI Physics-Based Smooth Cursor Engine
// =====================================================================

(function () {
  // Only initialize on devices with a mouse/fine pointer
  if (window.matchMedia && window.matchMedia('(pointer: coarse)').matches) {
    return;
  }

  const cursorEl = document.getElementById('smooth-cursor');
  const cursorImg = document.getElementById('cursor-img');
  const hudRawPos = document.getElementById('hud-raw-pos');
  const hudSmoothPos = document.getElementById('hud-smooth-pos');
  const hudSpeed = document.getElementById('hud-speed');
  const hudState = document.getElementById('hud-state');

  // Real Hardware Pointer Target
  let targetX = -100;
  let targetY = -100;
  let isVisible = false;

  // Smooth Cursor Physics State
  let posX = -100;
  let posY = -100;
  let velX = 0;
  let velY = 0;
  let currentAngle = 0;
  let isMouseDown = false;
  let isHoveringInteractive = false;
  let isOverLightSurface = false;

  // Physics Spring Coefficients (Magic UI / Framer Motion specification)
  const stiffness = 320.0;
  const damping = 26.0;
  const mass = 0.5;

  let lastTime = performance.now();

  // Track global pointer across entire window
  window.addEventListener(
    'pointermove',
    (e) => {
      targetX = e.clientX;
      targetY = e.clientY;

      if (!isVisible) {
        isVisible = true;
        posX = targetX;
        posY = targetY;
        cursorEl.style.display = 'block';
      }

      // Detect interactive elements and surface luminance passively
      const target = e.target;
      isHoveringInteractive = !!(
        target &&
        (target.closest('a') ||
          target.closest('button') ||
          target.closest('input') ||
          target.closest('.btn') ||
          target.closest('.test-btn') ||
          target.getAttribute('role') === 'button')
      );
      isOverLightSurface = !!(target && target.closest('.test-light'));
    },
    { passive: true }
  );

  // Click squash & stretch
  window.addEventListener(
    'pointerdown',
    () => {
      isMouseDown = true;
    },
    { passive: true }
  );

  window.addEventListener(
    'pointerup',
    () => {
      isMouseDown = false;
    },
    { passive: true }
  );

  window.addEventListener('mouseleave', () => {
    isVisible = false;
    cursorEl.style.display = 'none';
  });

  // Main Spring Physics Loop (RAF)
  function updateCursor(now) {
    const dt = Math.min((now - lastTime) / 1000, 0.032);
    lastTime = now;

    if (isVisible) {
      // 1. Spring differential equation (semi-implicit Euler)
      const forceX = (targetX - posX) * stiffness - velX * damping;
      const forceY = (targetY - posY) * stiffness - velY * damping;

      const accX = forceX / mass;
      const accY = forceY / mass;

      velX += accX * dt;
      velY += accY * dt;

      posX += velX * dt;
      posY += velY * dt;

      // 2. Velocity rotation (Magic UI signature motion)
      const speed = Math.hypot(velX, velY);
      let targetAngle = 0;
      if (speed > 35 && !isHoveringInteractive) {
        const rawAngle = Math.atan2(velY, velX) * (180 / Math.PI);
        const delta = ((rawAngle - 45 + 180) % 360) - 180;
        targetAngle = Math.max(Math.min(delta * 0.22, 22), -22);
      }
      currentAngle += (targetAngle - currentAngle) * Math.min(dt * 12, 1.0);

      // 3. Elastic Squash & Stretch scale
      let scale = isMouseDown ? 0.82 : isHoveringInteractive ? 1.08 : 1.0;

      // Hotspot offset: Arrow tip is (0,0), Hand tip is (10,0)
      const offsetX = isHoveringInteractive ? -8 : 0;
      const offsetY = 0;

      // Apply GPU-accelerated 3D Transform
      cursorEl.style.transform = `translate3d(${posX + offsetX}px, ${posY + offsetY}px, 0) rotate(${currentAngle}deg) scale(${scale})`;

      // 4. Update sprite source based on contrast and interactive state
      const targetSrc = isOverLightSurface
        ? isHoveringInteractive
          ? 'assets/cursor_hand_black.png'
          : 'assets/cursor_arrow_black.png'
        : isHoveringInteractive
        ? 'assets/cursor_hand.png'
        : 'assets/cursor_arrow.png';

      if (!cursorImg.src.endsWith(targetSrc)) {
        cursorImg.src = targetSrc;
      }

      // 6. Update Real-Time HUD Metrics if visible
      if (hudRawPos) {
        hudRawPos.textContent = `X: ${Math.round(targetX)} | Y: ${Math.round(targetY)}`;
        hudSmoothPos.textContent = `X: ${Math.round(posX)} | Y: ${Math.round(posY)}`;
        hudSpeed.textContent = `${Math.round(speed)} px/s`;
        hudState.textContent = `${isHoveringInteractive ? 'Hand' : 'Arrow'} (${isOverLightSurface ? 'Black' : 'White'})`;
      }
    }

    requestAnimationFrame(updateCursor);
  }

  requestAnimationFrame((time) => {
    lastTime = time;
    updateCursor(time);
  });
})();

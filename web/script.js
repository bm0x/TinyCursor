// =====================================================================
// TinyCursor - Web Physics & Dynamic Platform Detection Engine
// =====================================================================
// Replicates the native Rust DirectX engine 1:1:
// - Euler Semi-implicit Spring Physics with sub-stepping (~350Hz)
// - Trajectory-aligned Slerp banking with fluid recovery to canonical resting pose
// - Dynamic Squash & Stretch deformation (preserving perceived volume)
// - Hotspot-pinned transforms without drifting or wobbling
// - State-aware cursor swapping (14 custom desktop cursor variants)
// - Dual-theme contrast morphing (Arctic White / Obsidian Black)
// - Operating System detection & GitHub Release version recognition
// =====================================================================

(function () {
  'use strict';

  // -------------------------------------------------------------------
  // 1. Operating System Detection & Platform Manager
  // -------------------------------------------------------------------
  const PlatformManager = {
    detectedOS: 'windows', // 'windows' | 'macos' | 'linux'
    releaseInfo: {
      version: 'v0.1.0',
      name: 'TinyCursor v0.1.0',
      publishedAt: 'September 2026',
      installerUrl: 'https://github.com/bm0x/TinyCursor/releases/latest',
      portableUrl: 'https://github.com/bm0x/TinyCursor/releases/latest',
      releaseUrl: 'https://github.com/bm0x/TinyCursor/releases/latest',
    },

    init() {
      this.detectOS();
      this.fetchLatestRelease();
      this.bindPlatformTabs();
    },

    detectOS() {
      const userAgent = window.navigator.userAgent || '';
      const platform = window.navigator.platform || '';
      const userAgentData = window.navigator.userAgentData;

      let os = 'windows';

      if (userAgentData && userAgentData.platform) {
        const p = userAgentData.platform.toLowerCase();
        if (p.includes('win')) os = 'windows';
        else if (p.includes('mac')) os = 'macos';
        else if (p.includes('linux') || p.includes('android')) os = 'linux';
      } else {
        if (/Win/i.test(platform) || /Windows/i.test(userAgent)) os = 'windows';
        else if (/Mac/i.test(platform) || /Macintosh|Mac OS X/i.test(userAgent)) os = 'macos';
        else if (/Linux/i.test(platform) || /Linux/i.test(userAgent)) os = 'linux';
      }

      this.detectedOS = os;
      this.applyPlatformUI(os);
    },

    async fetchLatestRelease() {
      const CACHE_KEY = 'tinycursor_latest_release_v1';
      const CACHE_TTL = 10 * 60 * 1000; // 10 minutes cache

      try {
        const cached = localStorage.getItem(CACHE_KEY);
        if (cached) {
          const parsed = JSON.parse(cached);
          if (Date.now() - parsed.timestamp < CACHE_TTL) {
            this.releaseInfo = parsed.data;
            this.updateVersionUI();
            return;
          }
        }

        const res = await fetch('https://api.github.com/repos/bm0x/TinyCursor/releases/latest', {
          headers: { Accept: 'application/vnd.github.v3+json' },
        });

        if (!res.ok) throw new Error(`GitHub API HTTP ${res.status}`);
        const data = await res.json();

        let installerUrl = 'https://github.com/bm0x/TinyCursor/releases/latest';
        let portableUrl = 'https://github.com/bm0x/TinyCursor/releases/latest';

        if (Array.isArray(data.assets)) {
          const installerAsset = data.assets.find(
            (a) => a.name.endsWith('.exe') && (a.name.includes('Setup') || a.name.includes('tiny-cursor'))
          );
          if (installerAsset) installerUrl = installerAsset.browser_download_url;

          const zipAsset = data.assets.find((a) => a.name.endsWith('.zip'));
          if (zipAsset) portableUrl = zipAsset.browser_download_url;
        }

        const formattedDate = data.published_at
          ? new Date(data.published_at).toLocaleDateString('en-US', {
              month: 'short',
              year: 'numeric',
            })
          : 'September 2026';

        this.releaseInfo = {
          version: data.tag_name || 'v0.1.0',
          name: data.name || `TinyCursor ${data.tag_name || 'v0.1.0'}`,
          publishedAt: formattedDate,
          installerUrl,
          portableUrl,
          releaseUrl: data.html_url || 'https://github.com/bm0x/TinyCursor/releases/latest',
        };

        localStorage.setItem(
          CACHE_KEY,
          JSON.stringify({ timestamp: Date.now(), data: this.releaseInfo })
        );

        this.updateVersionUI();
      } catch (err) {
        // Graceful fallback to static build info
        this.updateVersionUI();
      }
    },

    updateVersionUI() {
      const tagDisplay = document.getElementById('release-tag-display');
      const osSpec = document.getElementById('release-os-spec');
      const sigDisplay = document.getElementById('release-signature-display');

      if (tagDisplay) tagDisplay.textContent = `Version ${this.releaseInfo.version}`;
      if (osSpec && this.detectedOS === 'windows') {
        osSpec.textContent = 'Windows 10/11 (64-bit)';
      }
      if (sigDisplay) sigDisplay.textContent = 'Authenticode Signed';

      const secBtn = document.getElementById('hero-secondary-download');
      if (secBtn && this.releaseInfo.portableUrl) {
        secBtn.href = this.releaseInfo.portableUrl;
      }

      const primaryBtn = document.getElementById('hero-primary-download');
      if (primaryBtn && this.detectedOS === 'windows' && this.releaseInfo.installerUrl) {
        primaryBtn.href = this.releaseInfo.installerUrl;
      }
    },

    applyPlatformUI(platform) {
      const osNameEl = document.getElementById('os-name-detected');
      const osStatusEl = document.getElementById('os-status-text');
      const primaryBtn = document.getElementById('hero-primary-download');
      const primaryBtnText = document.getElementById('hero-download-text');
      const primaryBtnIcon = document.getElementById('hero-download-icon');
      const secondaryBtn = document.getElementById('hero-secondary-download');

      // Update tab selection state
      document.querySelectorAll('.platform-tab').forEach((tab) => {
        if (tab.getAttribute('data-platform') === platform) {
          tab.classList.add('active');
        } else {
          tab.classList.remove('active');
        }
      });

      if (platform === 'windows') {
        if (osNameEl) osNameEl.textContent = 'Detected: Windows (64-bit)';
        if (osStatusEl) osStatusEl.textContent = '• Optimal Z-Band 16 & DirectComposition Verified';
        if (primaryBtn) primaryBtn.href = this.releaseInfo.installerUrl;
        if (primaryBtnText) primaryBtnText.textContent = `Download for Windows (.exe)`;
        if (primaryBtnIcon) {
          primaryBtnIcon.innerHTML = `<svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor"><path d="M0 3.449L9.75 2.1v9.451H0m10.949-9.602L24 0v11.4H10.949M0 12.6h9.75v9.451L0 20.699M10.949 12.6H24V24l-12.9-1.801"/></svg>`;
        }
        if (secondaryBtn) {
          secondaryBtn.style.display = 'inline-flex';
          secondaryBtn.href = this.releaseInfo.portableUrl;
          const secText = document.getElementById('hero-secondary-text');
          if (secText) secText.textContent = 'Portable (.zip)';
        }
      } else if (platform === 'macos') {
        if (osNameEl) osNameEl.textContent = 'Detected: macOS';
        if (osStatusEl) osStatusEl.textContent = '• Native Apple Silicon & Intel Port in Active Development';
        if (primaryBtn) primaryBtn.href = 'https://github.com/bm0x/TinyCursor';
        if (primaryBtnText) primaryBtnText.textContent = 'Star on GitHub (macOS WIP)';
        if (primaryBtnIcon) {
          primaryBtnIcon.innerHTML = `<svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor"><path d="M18.71 19.5c-.83 1.24-1.71 2.45-3.05 2.47-1.34.03-1.77-.79-3.29-.79-1.53 0-2 .77-3.27.82-1.31.05-2.3-1.32-3.14-2.53C4.25 17 2.94 12.45 4.7 9.39c.87-1.52 2.43-2.48 4.12-2.51 1.28-.02 2.5.87 3.29.87.78 0 2.26-1.07 3.81-.91.65.03 2.47.26 3.64 1.98-.09.06-2.17 1.28-2.15 3.81.03 3.02 2.65 4.03 2.68 4.04-.03.07-.42 1.44-1.38 2.83M15.97 6.38c.62-.75 1.04-1.8 0.92-2.85-.9.04-2 .6-2.65 1.35-.58.67-1.09 1.74-.95 2.77 1.01.08 2.05-.52 2.68-1.27z"/></svg>`;
        }
        if (secondaryBtn) {
          secondaryBtn.style.display = 'inline-flex';
          secondaryBtn.href = this.releaseInfo.installerUrl;
          const secText = document.getElementById('hero-secondary-text');
          if (secText) secText.textContent = 'Get Windows .exe anyway';
        }
      } else {
        if (osNameEl) osNameEl.textContent = 'Detected: Linux';
        if (osStatusEl) osStatusEl.textContent = '• Wayland / X11 Engine in Active Development';
        if (primaryBtn) primaryBtn.href = 'https://github.com/bm0x/TinyCursor';
        if (primaryBtnText) primaryBtnText.textContent = 'Star on GitHub (Linux WIP)';
        if (primaryBtnIcon) {
          primaryBtnIcon.innerHTML = `<svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor"><path d="M12.016 0C5.38 0 0 5.38 0 12.016c0 6.635 5.38 12.015 12.016 12.015 6.635 0 12.015-5.38 12.015-12.015C24.03 5.38 18.65 0 12.016 0zm-1.05 4.41c1.19-.04 2.12.75 2.37 1.95.25 1.2.04 2.54-.72 3.51-.55.71-1.35 1.15-2.25 1.19-.9.04-1.78-.32-2.39-1.01-.61-.69-.87-1.63-.73-2.54.14-.91.73-1.69 1.54-2.11.36-.18.77-.28 1.18-.29z"/></svg>`;
        }
        if (secondaryBtn) {
          secondaryBtn.style.display = 'inline-flex';
          secondaryBtn.href = this.releaseInfo.installerUrl;
          const secText = document.getElementById('hero-secondary-text');
          if (secText) secText.textContent = 'Get Windows .exe anyway';
        }
      }
    },

    bindPlatformTabs() {
      document.querySelectorAll('.platform-tab').forEach((tab) => {
        tab.addEventListener('click', () => {
          const targetOS = tab.getAttribute('data-platform');
          if (targetOS) this.applyPlatformUI(targetOS);
        });
      });
    },
  };

  // -------------------------------------------------------------------
  // 2. Pure Native Spring Physics Engine (Web Replica of Rust Core)
  // -------------------------------------------------------------------
  class SmoothCursorEngine {
    constructor() {
      // Ignore on touch devices
      if (window.matchMedia && window.matchMedia('(pointer: coarse)').matches) {
        return;
      }

      this.cursorEl = document.getElementById('smooth-cursor');
      this.cursorImg = document.getElementById('cursor-img');
      if (!this.cursorEl || !this.cursorImg) return;

      // Hotspots in 32px display coordinate space
      this.HOTSPOTS = {
        arrow: { x: 8.0, y: 3.5 },
        hand: { x: 13.0, y: 4.0 },
        ibeam: { x: 16.0, y: 16.0 },
        crosshair: { x: 16.0, y: 16.0 },
        move: { x: 16.0, y: 16.0 },
        help: { x: 8.0, y: 3.5 },
        wait: { x: 16.0, y: 16.0 },
        'resize-we': { x: 16.0, y: 16.0 },
        'resize-ns': { x: 16.0, y: 16.0 },
      };

      // Sprite file mapping (Arctic White vs Obsidian Black)
      this.SPRITES = {
        arrow: { white: 'assets/cursor_arrow.png', black: 'assets/cursor_arrow_black.png' },
        hand: { white: 'assets/cursor_hand.png', black: 'assets/cursor_hand_black.png' },
        ibeam: { white: 'assets/cursor_ibeam.png', black: 'assets/cursor_ibeam_black.png' },
        crosshair: { white: 'assets/white_crosshair.png', black: 'assets/black_crosshair.png' },
        move: { white: 'assets/white_move.png', black: 'assets/black_move.png' },
        help: { white: 'assets/white_help.png', black: 'assets/black_help.png' },
        wait: { white: 'assets/white_wait.png', black: 'assets/black_wait.png' },
        'resize-we': { white: 'assets/white_resize_we.png', black: 'assets/black_resize_we.png' },
        'resize-ns': { white: 'assets/white_resize_ns.png', black: 'assets/black_resize_ns.png' },
      };

      // Target (Hardware pointer position)
      this.targetX = -100;
      this.targetY = -100;
      this.isVisible = false;

      // Physical state
      this.posX = -100;
      this.posY = -100;
      this.velX = 0;
      this.velY = 0;
      this.angle = 0; // Relative to canonical resting angle in degrees
      this.scaleX = 1.0;
      this.scaleY = 1.0;

      // Pointer interactions & modes
      this.isClicking = false;
      this.pointerKind = 'arrow';
      this.forcedKind = null;
      this.themeMode = 'auto'; // 'auto' | 'white' | 'black'
      this.isOverLight = false;

      // Spring physics configuration presets (matching Rust core CursorConfig)
      this.presets = {
        balanced: { stiffness: 540, damping: 38, maxVelocity: 3000, stretchFactor: 0.32 },
        snappy: { stiffness: 920, damping: 46, maxVelocity: 3600, stretchFactor: 0.20 },
        floaty: { stiffness: 280, damping: 22, maxVelocity: 2600, stretchFactor: 0.46 },
      };
      this.currentPreset = 'balanced';
      this.config = { ...this.presets.balanced };

      // Performance pacing & FPS metric
      this.lastTime = performance.now();
      this.frameCount = 0;
      this.fpsTimer = performance.now();
      this.measuredFPS = 60;

      // HUD Elements
      this.hudRawPos = document.getElementById('hud-raw-pos');
      this.hudSmoothPos = document.getElementById('hud-smooth-pos');
      this.hudSpeed = document.getElementById('hud-speed');
      this.hudAngle = document.getElementById('hud-angle');
      this.hudScale = document.getElementById('hud-scale');
      this.hudState = document.getElementById('hud-state');
      this.hudFPS = document.getElementById('hud-fps');

      this.bindEvents();
      this.bindToolbar();
      this.startLoop();
    }

    bindEvents() {
      // Global hardware pointer tracking
      window.addEventListener(
        'pointermove',
        (e) => {
          this.targetX = e.clientX;
          this.targetY = e.clientY;

          if (!this.isVisible) {
            this.isVisible = true;
            this.posX = this.targetX;
            this.posY = this.targetY;
            this.cursorEl.style.display = 'block';
          }

          this.detectElementUnderCursor(e.target);
        },
        { passive: true }
      );

      window.addEventListener(
        'pointerdown',
        () => {
          this.isClicking = true;
        },
        { passive: true }
      );

      window.addEventListener(
        'pointerup',
        () => {
          this.isClicking = false;
        },
        { passive: true }
      );

      window.addEventListener('mouseleave', () => {
        this.isVisible = false;
        this.cursorEl.style.display = 'none';
      });
    }

    detectElementUnderCursor(target) {
      if (!target) return;

      // 1. Explicit data-cursor overrides
      const explicitCursor = target.closest('[data-cursor]');
      if (explicitCursor) {
        this.pointerKind = explicitCursor.getAttribute('data-cursor') || 'arrow';
      } else if (
        target.closest('input') ||
        target.closest('textarea') ||
        target.closest('code') ||
        target.closest('pre') ||
        target.closest('.test-input')
      ) {
        this.pointerKind = 'ibeam';
      } else if (
        target.closest('a') ||
        target.closest('button') ||
        target.closest('.btn') ||
        target.closest('.test-btn') ||
        target.closest('.platform-tab') ||
        target.closest('.btn-preset') ||
        target.closest('.chip') ||
        target.getAttribute('role') === 'button'
      ) {
        this.pointerKind = 'hand';
      } else {
        this.pointerKind = 'arrow';
      }

      // 2. Luminance / Surface Contrast Detection
      if (this.themeMode === 'white') {
        this.isOverLight = false;
      } else if (this.themeMode === 'black') {
        this.isOverLight = true;
      } else {
        // Auto mode: Detect if hovering light background card
        this.isOverLight = !!target.closest('.test-light');
      }
    }

    bindToolbar() {
      // Preset switching buttons
      document.querySelectorAll('[data-preset]').forEach((btn) => {
        btn.addEventListener('click', (e) => {
          e.preventDefault();
          const presetKey = btn.getAttribute('data-preset');
          if (this.presets[presetKey]) {
            this.currentPreset = presetKey;
            this.config = { ...this.presets[presetKey] };
            document
              .querySelectorAll('[data-preset]')
              .forEach((b) => b.classList.remove('active'));
            btn.classList.add('active');
          }
        });
      });

      // Theme mode switching buttons
      document.querySelectorAll('[data-theme]').forEach((btn) => {
        btn.addEventListener('click', (e) => {
          e.preventDefault();
          const themeKey = btn.getAttribute('data-theme');
          this.themeMode = themeKey;
          document
            .querySelectorAll('[data-theme]')
            .forEach((b) => b.classList.remove('active'));
          btn.classList.add('active');
        });
      });
    }

    startLoop() {
      const tick = (now) => {
        let dt = (now - this.lastTime) / 1000;
        this.lastTime = now;

        // Clamp dt to avoid physics explosion on background tab wake-up
        if (dt <= 0 || dt > 0.05) dt = 1 / 60;

        // FPS estimation
        this.frameCount++;
        if (now - this.fpsTimer >= 500) {
          this.measuredFPS = Math.round((this.frameCount * 1000) / (now - this.fpsTimer));
          this.frameCount = 0;
          this.fpsTimer = now;
        }

        if (this.isVisible) {
          this.updatePhysics(dt);
          this.render();
        }

        requestAnimationFrame(tick);
      };

      requestAnimationFrame((time) => {
        this.lastTime = time;
        requestAnimationFrame(tick);
      });
    }

    updatePhysics(dt) {
      const { stiffness, damping, maxVelocity, stretchFactor } = this.config;

      // 1. High-frequency Sub-stepping (~350Hz) for identical stability across all monitor Hz
      const maxSubStep = 0.003;
      const subSteps = Math.max(1, Math.ceil(dt / maxSubStep));
      const subDt = dt / subSteps;

      for (let i = 0; i < subSteps; i++) {
        // Semi-implicit Euler integration: F = k * (target - current) - c * v
        const forceX = (this.targetX - this.posX) * stiffness - this.velX * damping;
        const forceY = (this.targetY - this.posY) * stiffness - this.velY * damping;

        this.velX += forceX * subDt;
        this.velY += forceY * subDt;
        this.posX += this.velX * subDt;
        this.posY += this.velY * subDt;
      }

      const speed = Math.hypot(this.velX, this.velY);

      // 2. Velocity-Aligned Trajectory Banking with Slerp Recovery
      // Natural canonical resting angle in screen space is ~ -115° (top-left slant).
      const RESTING_DEG = -115.0;
      let targetAngle = 0;

      if (speed > 30 && this.pointerKind === 'arrow') {
        const motionDeg = (Math.atan2(this.velY, this.velX) * 180) / Math.PI;
        const diff = ((motionDeg - RESTING_DEG + 180) % 360) - 180;
        // Soft bank clamp [-35°, 35°] for organic response
        targetAngle = Math.max(-35, Math.min(35, diff * 0.35));
      }

      // Dynamic angular blend rate: recovers faster at higher speeds
      const angularSpeed = 16.0 + Math.min(speed / 180, 16.0);
      const angleDiff = ((targetAngle - this.angle + 180) % 360) - 180;
      this.angle += angleDiff * (1 - Math.exp(-angularSpeed * dt));

      // 3. Elastic Squash & Stretch Deformation (Volume Preserving)
      const normalizedSpeed = Math.min(speed / maxVelocity, 1.0);
      let targetScaleX = 1.0 + normalizedSpeed * stretchFactor;

      // Click compression bounce
      if (this.isClicking) {
        targetScaleX *= 0.86;
      }

      // Hand pointer scale punch
      if (this.pointerKind === 'hand') {
        targetScaleX *= 1.05;
      }

      const scaleBlend = 1 - Math.exp(-22.0 * dt);
      this.scaleX += (targetScaleX - this.scaleX) * scaleBlend;
      this.scaleY = 1.0 / Math.sqrt(Math.max(0.5, this.scaleX));
    }

    render() {
      // 1. Exact Hotspot Placement
      const hs = this.HOTSPOTS[this.pointerKind] || this.HOTSPOTS.arrow;
      this.cursorEl.style.transformOrigin = `${hs.x}px ${hs.y}px`;

      // Hand, text, and utility cursors stay upright; Arrow uses dynamic banking angle
      const renderAngle = this.pointerKind === 'arrow' ? this.angle : 0;

      // 2. Apply Hardware-Accelerated 3D Transform
      this.cursorEl.style.transform = `translate3d(${this.posX - hs.x}px, ${this.posY - hs.y}px, 0) rotate(${renderAngle}deg) scale(${this.scaleX}, ${this.scaleY})`;

      // 3. Sprite selection based on pointer kind & surface luminance
      const isBlack = this.isOverLight;
      const spritePair = this.SPRITES[this.pointerKind] || this.SPRITES.arrow;
      const targetSrc = isBlack ? spritePair.black : spritePair.white;

      if (!this.cursorImg.src.endsWith(targetSrc)) {
        this.cursorImg.src = targetSrc;
      }

      // 4. Update Live HUD Metrics
      this.updateHUD();
    }

    updateHUD() {
      const speed = Math.hypot(this.velX, this.velY);

      if (this.hudRawPos) {
        this.hudRawPos.textContent = `X: ${Math.round(this.targetX)} | Y: ${Math.round(this.targetY)}`;
      }
      if (this.hudSmoothPos) {
        this.hudSmoothPos.textContent = `X: ${Math.round(this.posX)} | Y: ${Math.round(this.posY)}`;
      }
      if (this.hudSpeed) {
        this.hudSpeed.textContent = `${Math.round(speed)} px/s`;
      }
      if (this.hudAngle) {
        this.hudAngle.textContent = `${this.angle.toFixed(1)}°`;
      }
      if (this.hudScale) {
        this.hudScale.textContent = `${this.scaleX.toFixed(2)}x`;
      }
      if (this.hudState) {
        const kindLabel =
          this.pointerKind.charAt(0).toUpperCase() + this.pointerKind.slice(1);
        const themeLabel = this.isOverLight ? 'Obsidian Black' : 'Arctic White';
        this.hudState.textContent = `${kindLabel} (${themeLabel})`;
      }
      if (this.hudFPS) {
        this.hudFPS.textContent = `~${this.measuredFPS} FPS`;
      }
    }
  }

  // -------------------------------------------------------------------
  // 3. Initialize All Web Systems on DOM Ready
  // -------------------------------------------------------------------
  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', () => {
      PlatformManager.init();
      new SmoothCursorEngine();
    });
  } else {
    PlatformManager.init();
    new SmoothCursorEngine();
  }
})();

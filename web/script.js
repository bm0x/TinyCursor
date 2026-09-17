// TinyCursor Web Interactive Spring Physics Simulation
// Simulates the exact Euler semi-implicit critically damped spring physics implemented in Rust

const canvas = document.getElementById('interactive-canvas');
const cursor = document.getElementById('virtual-cursor');
const cursorBody = document.getElementById('cursor-body');

// Physics state
let mouseX = 0;
let mouseY = 0;
let posX = 0;
let posY = 0;
let velX = 0;
let velY = 0;
let isHovering = false;

// Spring parameters matching TinyCursor's src/core/config.rs
const stiffness = 750.0;
const damping = 54.0;
let lastTime = performance.now();

// Track mouse within canvas
canvas.addEventListener('mouseenter', (e) => {
  isHovering = true;
  const rect = canvas.getBoundingClientRect();
  mouseX = e.clientX - rect.left;
  mouseY = e.clientY - rect.top;
  posX = mouseX;
  posY = mouseY;
  velX = 0;
  velY = 0;
});

canvas.addEventListener('mouseleave', () => {
  isHovering = false;
});

canvas.addEventListener('mousemove', (e) => {
  const rect = canvas.getBoundingClientRect();
  mouseX = e.clientX - rect.left;
  mouseY = e.clientY - rect.top;
});

// Physics loop
function updatePhysics(now) {
  const dt = Math.min((now - lastTime) / 1000, 0.032);
  lastTime = now;

  if (isHovering) {
    // Spring physics step (Euler semi-implicit)
    const dispX = posX - mouseX;
    const dispY = posY - mouseY;

    const forceX = -stiffness * dispX - damping * velX;
    const forceY = -stiffness * dispY - damping * velY;

    velX += forceX * dt;
    velY += forceY * dt;

    posX += velX * dt;
    posY += velY * dt;

    // Velocity angle calculation for smooth rotation
    const speed = Math.sqrt(velX * velX + velY * velY);
    let rotation = 0;
    if (speed > 40.0) {
      const angle = Math.atan2(velY, velX) * (180 / Math.PI);
      // Lead angle interpolation
      rotation = Math.min(Math.max((angle - 45) * 0.15, -25), 25);
    }

    cursor.style.transform = `translate(${posX - 4}px, ${posY - 4}px) rotate(${rotation}deg)`;

    // Detect light vs dark side of the canvas
    const rect = canvas.getBoundingClientRect();
    const isLightSide = posX < (rect.width / 2);

    if (isLightSide) {
      // Over light surface: morph to Obsidian Black with white stroke
      cursorBody.setAttribute('fill', '#10131A');
      cursorBody.setAttribute('stroke', '#FFFFFF');
    } else {
      // Over dark surface: morph to Arctic White with dark stroke
      cursorBody.setAttribute('fill', '#FFFFFF');
      cursorBody.setAttribute('stroke', '#10131A');
    }
  }

  requestAnimationFrame(updatePhysics);
}

requestAnimationFrame((time) => {
  lastTime = time;
  updatePhysics(time);
});

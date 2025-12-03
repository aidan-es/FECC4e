// Copyright (C) 2025 aidan-es. Licensed under the GNU AGPLv3.
const canvas = document.getElementById('rainCanvas');
const ctx = canvas.getContext('2d');

// Configuration
const SPEED = 2; // Falling speed
const DENSITY = 1; // Probability of a new character spawning per frame (0-1)
const SPRITE_SIZE = 96; // Size of each character in the sprite sheet
const SCALE = 0.5; // Scale of characters on screen
const TOTAL_CHARACTERS = 404;
const COLUMNS = 20; // Number of columns in the sprite sheet

// Load sprite sheet
const spriteSheet = new Image();
spriteSheet.src = '404-sprites.png';

let particles = [];

class Particle {
    constructor() {
        this.x = Math.random() * canvas.width;
        this.y = -SPRITE_SIZE * SCALE;
        this.speed = SPEED + Math.random() * 2; // Add some variation
        this.index = Math.floor(Math.random() * TOTAL_CHARACTERS);
    }

    update() {
        this.y += this.speed;
    }

    draw() {
        const col = this.index % COLUMNS;
        const row = Math.floor(this.index / COLUMNS);
        const sx = col * SPRITE_SIZE;
        const sy = row * SPRITE_SIZE;

        ctx.drawImage(
            spriteSheet,
            sx, sy, SPRITE_SIZE, SPRITE_SIZE,
            Math.floor(this.x), Math.floor(this.y), SPRITE_SIZE * SCALE, SPRITE_SIZE * SCALE
        );
    }
}

function resize() {
    canvas.width = window.innerWidth;
    canvas.height = window.innerHeight;
}

window.addEventListener('resize', resize);
resize();

function animate() {
    ctx.clearRect(0, 0, canvas.width, canvas.height);

    // Spawn new particles
    if (Math.random() < DENSITY) {
        particles.push(new Particle());
    }

    // Update and draw particles
    for (let i = particles.length - 1; i >= 0; i--) {
        particles[i].update();
        particles[i].draw();

        // Remove particles that are off screen
        if (particles[i].y > canvas.height) {
            particles.splice(i, 1);
        }
    }

    requestAnimationFrame(animate);
}

spriteSheet.onload = () => {
    animate();
};

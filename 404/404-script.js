// Copyright (C) 2025-2026 aidan-es. Licensed under the GNU AGPLv3.
import { fetch_gzip_bytes, read_art_bundle } from './art_bundle.js';

const canvas = document.getElementById('rainCanvas');
const ctx = canvas.getContext('2d');

// Configuration
const SPEED = 2; // Falling speed
const DENSITY = 1; // Probability of a new character spawning per frame (0-1)
const SCALE = 0.5; // Scale of characters on screen

// The characters, made by fecc_cli
const sprites = await Promise.all(
    read_art_bundle(await fetch_gzip_bytes('404-characters.gz', 'auto'))
        .map((file) => createImageBitmap(new Blob([file.data], { type: 'image/png' })))
);

let particles = [];

class Particle {
    constructor() {
        this.sprite = sprites[Math.floor(Math.random() * sprites.length)];
        this.x = Math.random() * canvas.width;
        this.y = -this.sprite.height * SCALE;
        this.speed = SPEED + Math.random() * 2; // Add some variation
    }

    update() {
        this.y += this.speed;
    }

    draw() {
        ctx.drawImage(
            this.sprite,
            Math.floor(this.x), Math.floor(this.y), this.sprite.width * SCALE, this.sprite.height * SCALE
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

        // Remove particles that are off-screen
        if (particles[i].y > canvas.height) {
            particles.splice(i, 1);
        }
    }

    requestAnimationFrame(animate);
}

animate();

// @ts-check
import { defineConfig } from 'astro/config';
import tailwindcss from '@tailwindcss/vite';

export default defineConfig({
    compressHTML: true,
    server: {
        host: 'localhost',
        port: 4321,
    },
    vite: {
        plugins: [tailwindcss()],
    },
});

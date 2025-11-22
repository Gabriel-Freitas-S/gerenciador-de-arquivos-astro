import { defineConfig } from "drizzle-kit";

export default defineConfig({
    schema: "./src/db/schema.ts",
    dialect: "sqlite",
    // Note: dbCredentials are not set because the DB file location is dynamic in Tauri (AppData).
    // This config is primarily for generating SQL migrations from the schema.
});

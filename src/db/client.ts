import { drizzle } from "drizzle-orm/sqlite-proxy";
import Database from "@tauri-apps/plugin-sql";
import * as schema from "./schema";

// Initialize the database connection to the same file used by the backend
const dbPromise = Database.load("sqlite:archive.sqlite");

export const db = drizzle(async (sql, params, method) => {
    const sqlite = await dbPromise;
    try {
        if (method === "run") {
            await sqlite.execute(sql, params);
            return { rows: [] };
        }

        // For select queries
        const rows = await sqlite.select<any[]>(sql, params);

        // If method is 'values', Drizzle expects array of arrays
        if (method === "values") {
            const values = rows.map(r => Object.values(r));
            return { rows: values };
        }

        // For 'all', 'get'
        return { rows };
    } catch (e) {
        console.error("SQL Error:", e);
        throw e;
    }
}, { schema });

import { sqliteTable, text, integer } from "drizzle-orm/sqlite-core";
import { sql } from "drizzle-orm";

export const users = sqliteTable("users", {
  id: integer("id").primaryKey({ autoIncrement: true }),
  name: text("name").notNull(),
  login: text("login").notNull().unique(),
  passwordHash: text("password_hash").notNull(),
  role: text("role").notNull().default("admin"),
  createdAt: text("created_at").notNull().default(sql`CURRENT_TIMESTAMP`),
});

export const storageUnits = sqliteTable("storage_units", {
  id: integer("id").primaryKey({ autoIncrement: true }),
  label: text("label").notNull(),
  type: text("type").notNull(),
  section: text("section"),
  capacity: integer("capacity").default(0),
  occupancy: integer("occupancy").default(0),
  metadata: text("metadata"),
  createdAt: text("created_at").notNull().default(sql`CURRENT_TIMESTAMP`),
  updatedAt: text("updated_at").notNull().default(sql`CURRENT_TIMESTAMP`),
});

export const movements = sqliteTable("movements", {
  id: integer("id").primaryKey({ autoIncrement: true }),
  reference: text("reference"),
  itemLabel: text("item_label"),
  fromUnit: text("from_unit"),
  toUnit: text("to_unit"),
  action: text("action").notNull(),
  note: text("note"),
  actor: text("actor").notNull(),
  createdAt: text("created_at").notNull().default(sql`CURRENT_TIMESTAMP`),
});

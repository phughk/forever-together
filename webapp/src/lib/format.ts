import type { Faction, Role } from "../api/types";

export function formatAge(seconds: number): string {
  const s = Math.max(0, Math.floor(seconds));
  if (s < 60) return `${s} s`;
  const m = Math.floor(s / 60);
  if (m < 60) return `${m} min`;
  const h = Math.floor(m / 60);
  return `${h} h ${m % 60} min`;
}

export function formatClock(seconds: number): string {
  const s = Math.max(0, Math.ceil(seconds));
  return `${Math.floor(s / 60)}:${String(s % 60).padStart(2, "0")}`;
}

export const ROLE_LABEL: Record<Role, string> = {
  tank: "Tank",
  healer: "Healer",
  damage: "Damage",
  unknown: "Role unknown",
};

export const FACTION_LABEL: Record<Faction, string> = {
  alliance: "Alliance",
  horde: "Horde",
};

export function fullName(m: { firstName: string | null; surname: string | null }): string {
  return [m.firstName, m.surname].filter(Boolean).join(" ");
}

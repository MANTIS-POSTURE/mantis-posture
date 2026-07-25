import { invoke } from '@tauri-apps/api/core';

export interface Folder {
  id: number;
  name: string;
  description: string;
}

export interface Incident {
  id: number;
  title: string;
  severity: string;
  status: string;
  description: string;
}

export interface Action {
  id: number;
  title: string;
  priority: string;
  status: string;
  incident_id: number | null;
}

export interface Identity {
  id: number;
  folder_id: number | null;
  identity_type: string;
  value: string;
  label: string | null;
}

export interface Exposure {
  id: number;
  identity_id: number | null;
  source: string;
  severity: string;
  description: string;
  detected_at: string;
}

export interface RgpdRequest {
  id: number;
  target_entity: string;
  request_type: string;
  status: string;
  created_at: string;
  notes: string | null;
}

export interface TimelineEntry {
  id: number;
  event_type: string;
  description: string;
  created_at: string;
}

export async function listFolders(): Promise<Folder[]> {
  return await invoke<Folder[]>('list_folders');
}

export async function listIncidents(): Promise<Incident[]> {
  return await invoke<Incident[]>('list_incidents');
}

export async function listActions(): Promise<Action[]> {
  return await invoke<Action[]>('list_actions');
}

export async function listIdentities(): Promise<Identity[]> {
  return await invoke<Identity[]>('list_identities');
}

export async function listExposures(): Promise<Exposure[]> {
  return await invoke<Exposure[]>('list_exposures');
}

export async function listRgpdRequests(): Promise<RgpdRequest[]> {
  return await invoke<RgpdRequest[]>('list_rgpd_requests');
}

export async function listTimelineEntries(): Promise<TimelineEntry[]> {
  return await invoke<TimelineEntry[]>('list_timeline_entries');
}

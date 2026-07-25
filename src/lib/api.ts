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

export async function listFolders(): Promise<Folder[]> {
  return await invoke<Folder[]>('list_folders');
}

export async function listIncidents(): Promise<Incident[]> {
  return await invoke<Incident[]>('list_incidents');
}

export async function listActions(): Promise<Action[]> {
  return await invoke<Action[]>('list_actions');
}

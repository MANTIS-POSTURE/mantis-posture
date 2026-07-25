import { invoke } from '@tauri-apps/api/core';

export interface Folder {
  id: string;
  name: string;
  context: string;
}

export interface Incident {
  id: string;
  title: string;
  severity: string;
  discovered_at: string;
  what: string;
  why: string;
  impact: string;
  confidence: string;
  next_step: string;
  folder_id: string | null;
}

export interface Action {
  id: string;
  title: string;
  priority_id: string;
  difficulty_id: string;
  deadline: string;
  status: string;
  guidance: string;
  proof_expected: string;
  folder_id: string | null;
  incident_id: string | null;
}

export interface Identity {
  id: string;
  label: string;
  kind: string;
  value: string;
  folder_id: string | null;
  notes: string | null;
  address_line1: string | null;
  address_line2: string | null;
  city: string | null;
  postal_code: string | null;
  country: string | null;
}

export interface Exposure {
  id: string;
  title: string;
  kind: string;
  severity: string;
  status: string;
  discovered_at: string;
  source: string;
  what: string;
  why: string;
  folder_id: string | null;
}

export interface RgpdRequest {
  id: string;
  type_id: string;
  target: string;
  dpo_contact: string;
  status_id: string;
  data_summary: string;
  draft_preview: string;
}

export interface TimelineEntry {
  id: string;
  event_type: string;
  description: string;
  created_at: string;
}

export interface OsintModule {
  id: string;
  name: string;
  description: string;
  target_kind: string;
  frequency: string;
  status: string;
  last_run: string | null;
  next_run: string | null;
}

export interface PostureScore {
  score: number;
  open_incidents: number;
  high_exposures: number;
  completed_actions: number;
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

export async function listOsintModules(): Promise<OsintModule[]> {
  return await invoke<OsintModule[]>('list_osint_modules');
}

export async function getPostureScore(): Promise<PostureScore> {
  return await invoke<PostureScore>('get_posture_score');
}

export async function updateActionStatus(id: string, status: string): Promise<void> {
  await invoke('update_action_status', { actionId: id, status });
}

export async function updateRgpdRequestStatus(id: string, statusId: string): Promise<void> {
  await invoke('update_rgpd_request_status', { requestId: id, statusId });
}

export async function createIdentity(
  label: string, 
  kind: string, 
  value: string, 
  folderId: string | null, 
  notes: string | null,
  addressLine1: string | null,
  addressLine2: string | null,
  city: string | null,
  postalCode: string | null,
  country: string | null
): Promise<Identity> {
  return await invoke<Identity>('create_identity', { 
    label, 
    kind, 
    value, 
    folderId, 
    notes,
    addressLine1,
    addressLine2,
    city,
    postalCode,
    country
  });
}

export async function updateIdentity(
  id: string, 
  label: string, 
  kind: string, 
  value: string, 
  folderId: string | null, 
  notes: string | null,
  addressLine1: string | null,
  addressLine2: string | null,
  city: string | null,
  postalCode: string | null,
  country: string | null
): Promise<void> {
  await invoke('update_identity', { 
    id, 
    label, 
    kind, 
    value, 
    folderId, 
    notes,
    addressLine1,
    addressLine2,
    city,
    postalCode,
    country
  });
}

export async function deleteIdentity(id: string): Promise<void> {
  await invoke('delete_identity', { id });
}

export async function runOsintModule(moduleId: string): Promise<string> {
  return await invoke<string>('run_osint_module', { moduleId });
}

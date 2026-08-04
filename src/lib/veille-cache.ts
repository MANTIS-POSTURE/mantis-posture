import type {
  OsintAnalysisReport,
  OsintReviewWorkspace,
  OsintScanSessionListItem,
  OsintScanSessionSummary
} from '$lib/api';

export type VeilleViewCache = {
  sessions: OsintScanSessionListItem[];
  result: OsintScanSessionSummary | null;
  reviewWorkspace: OsintReviewWorkspace;
  analysis: OsintAnalysisReport | null;
  historyHasMore: boolean;
};

const views = new Map<string, VeilleViewCache>();

export function getVeilleViewCache(identityId: string): VeilleViewCache | null {
  return views.get(identityId) ?? null;
}

export function setVeilleViewCache(identityId: string, view: VeilleViewCache): void {
  if (identityId) views.set(identityId, view);
}

import { writable } from 'svelte/store';

const storageKey = 'mantis.activeIdentityId';

function initialIdentityId(): string | null {
	if (typeof localStorage === 'undefined') return null;
	return localStorage.getItem(storageKey);
}

export const activeIdentityId = writable<string | null>(initialIdentityId());

export function setActiveIdentityId(identityId: string | null) {
	activeIdentityId.set(identityId);
	if (typeof localStorage === 'undefined') return;
	if (identityId) localStorage.setItem(storageKey, identityId);
	else localStorage.removeItem(storageKey);
}

import { writable } from 'svelte/store';

export type ContentOperation = {
  id: string; title: string; stage: string; detail: string; percent: number;
  current: number; total: number; bytes_done: number; bytes_total: number;
  status: 'running' | 'success' | 'error';
};
export const contentOperation = writable<ContentOperation | null>(null);

export interface ExportResult {
  path: string;
  format: string;
  included_files: number;
  referenced_files: number;
  omitted: { name: string; version: string; filename: string; reason: string; project_url?: string | null }[];
}

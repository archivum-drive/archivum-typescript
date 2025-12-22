export interface File {
  filename: string;
  size: number;
}

export interface Bookmark {
  url: string;
  title?: string;
}

export type NodeType = { File: File } | { Bookmark: Bookmark };

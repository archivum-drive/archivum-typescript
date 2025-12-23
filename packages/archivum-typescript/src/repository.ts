/** biome-ignore-all lint/suspicious/noExplicitAny: wasm shenanigans */
import { Repository as CoreRepository } from "archivum-core-wasm";
import { Node } from "./types/node";
import type { NodeType } from "./types/nodeTypes";
import type { TagColor } from "./types/tag";
import { Tag } from "./types/tag";

type CoreTag = {
  id: number;
  path: string[];
};

type CoreNode = {
  id: number;
  data?: unknown;
  tag_ids?: number[];
  date_created: string;
  date_updated: string;
};

const DEFAULT_TAG_COLOR: TagColor = "gray";
const EMPTY_NODE_DATA: NodeType = {
  File: { filename: "", size: 0 },
  type: "file",
};

// todo: better solution ??
function normalizeNodeType(data: unknown): NodeType {
  if (!data || typeof data !== "object") return EMPTY_NODE_DATA;

  const record = data as Record<string, unknown>;

  // Preferred: already has the discriminant.
  if (
    record.type === "file" &&
    record.File &&
    typeof record.File === "object"
  ) {
    return { File: record.File as any, type: "file" };
  }
  if (
    record.type === "bookmark" &&
    record.Bookmark &&
    typeof record.Bookmark === "object"
  ) {
    return { Bookmark: record.Bookmark as any, type: "bookmark" };
  }

  // Backward-compatible: infer from the enum-like shape produced by Rust/serde.
  if (record.File && typeof record.File === "object") {
    return { File: record.File as any, type: "file" };
  }
  if (record.Bookmark && typeof record.Bookmark === "object") {
    return { Bookmark: record.Bookmark as any, type: "bookmark" };
  }

  return EMPTY_NODE_DATA;
}

function toCoreNodeData(nodeType: NodeType): unknown {
  // Rust expects an enum-like shape: { File: {...} } or { Bookmark: {...} }
  // The TS-only discriminant `type` must not be sent to core.
  if (nodeType.type === "file") return { File: nodeType.File };
  return { Bookmark: nodeType.Bookmark };
}

export class Repository {
  private repo: CoreRepository;

  constructor() {
    this.repo = new CoreRepository();
  }

  static fromJson(json: string): Repository {
    const coreRepo = CoreRepository.loadFromJson(json);
    const repository = new Repository();
    repository.repo = coreRepo;
    return repository;
  }

  toJson(): string {
    return this.repo.saveToJson();
  }

  // Tags
  upsertTag(tag: Tag): void {
    this.repo.upsertTag(tag.id, tag.path);
  }

  deleteTag(tagId: number): void {
    this.repo.deleteTag(tagId);
  }

  getTag(tagId: number): Tag | undefined {
    const coreTag = this.repo.getTag(tagId) as CoreTag | undefined;
    if (!coreTag) return undefined;
    // Color is currently not persisted in core; default to gray.
    return new Tag(coreTag.id, coreTag.path, DEFAULT_TAG_COLOR);
  }

  getAllTags(): Tag[] {
    const coreTags = this.repo.getAllTags() as CoreTag[];
    return coreTags.map((t) => new Tag(t.id, t.path, DEFAULT_TAG_COLOR));
  }

  getNextTagId(): number {
    return this.repo.getNextTagId();
  }

  getTagByPath(pathSegments: string[]): Tag | undefined {
    try {
      const tagId = this.repo.getTagByPath(pathSegments);
      return this.getTag(tagId);
    } catch (e) {
      console.warn(`Tag not found for path: ${pathSegments}:, ${e}`);
      return undefined;
    }
  }

  getChildTags(parentTagId: number): Tag[] {
    const coreTags = this.repo.getChildTags(parentTagId) as CoreTag[];
    return coreTags.map((t) => new Tag(t.id, t.path, DEFAULT_TAG_COLOR));
  }

  // Nodes
  upsertNode(node: Node): void {
    this.repo.upsertNode(
      node.id,
      toCoreNodeData(node.data),
      node.date_created,
      node.date_updated,
    );
  }

  deleteNode(nodeId: number): void {
    this.repo.deleteNode(nodeId);
  }

  getNode(nodeId: number): Node | undefined {
    const coreNode = this.repo.getNode(nodeId) as CoreNode | undefined;
    if (!coreNode) return undefined;

    const tagMap = new Map(this.getAllTags().map((t) => [t.id, t] as const));
    const tags = (coreNode.tag_ids ?? [])
      .map((id) => tagMap.get(id))
      .filter((t): t is Tag => Boolean(t));

    const data = normalizeNodeType(coreNode.data);

    return new Node(
      coreNode.id,
      data,
      tags,
      coreNode.date_created,
      coreNode.date_updated,
    );
  }

  getAllNodes(): Node[] {
    const coreNodes = this.repo.getAllNodes() as CoreNode[];
    const tagMap = new Map(this.getAllTags().map((t) => [t.id, t] as const));

    return coreNodes.map((n) => {
      const tags = (n.tag_ids ?? [])
        .map((id) => tagMap.get(id))
        .filter((t): t is Tag => Boolean(t));

      const data = normalizeNodeType(n.data);

      return new Node(n.id, data, tags, n.date_created, n.date_updated);
    });
  }

  getNextNodeId(): number {
    return this.repo.getNextNodeId();
  }

  tagNode(nodeId: number, tagId: number): void {
    this.repo.tagNode(nodeId, tagId);
  }

  untagNode(nodeId: number, tagId: number): void {
    this.repo.untagNode(nodeId, tagId);
  }

  getNodesWithTag(tagId: number): Node[] {
    try {
      const coreNodes = this.repo.getNodesWithTag(tagId) as CoreNode[];
      const tagMap = new Map(this.getAllTags().map((t) => [t.id, t] as const));

      return coreNodes.map((n) => {
        const tags = (n.tag_ids ?? [])
          .map((id) => tagMap.get(id))
          .filter((t): t is Tag => Boolean(t));

        const data = normalizeNodeType(n.data);

        return new Node(n.id, data, tags, n.date_created, n.date_updated);
      });
    } catch {
      return [];
    }
  }
}

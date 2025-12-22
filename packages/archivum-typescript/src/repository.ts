import { Repository as CoreRepository } from "archivum-core-wasm";
import { Tag } from "./types/tag";
import { Node } from "./types/node";
import type { TagColor } from "./types/tag";
import type { NodeType } from "./types/nodeTypes";

type CoreTag = {
  id: number;
  path: string[];
};

type CoreNode = {
  id: number;
  data?: NodeType;
  tag_ids?: number[];
  date_created: string;
  date_updated: string;
};

const DEFAULT_TAG_COLOR: TagColor = "gray";
const EMPTY_NODE_DATA: NodeType = { File: { filename: "", size: 0 } };

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

  getTagByPath(path: string): Tag | undefined {
    const parts = path.split("/").filter(Boolean);
    const tagId = this.repo.getTagByPath(parts);
    if (!tagId) return undefined;
    return this.getTag(tagId);
  }

  // Nodes
  upsertNode(node: Node): void {
    this.repo.upsertNode(
      node.id,
      node.data,
      node.date_created,
      node.date_updated
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

    return new Node(
      coreNode.id,
      coreNode.data ?? EMPTY_NODE_DATA,
      tags,
      coreNode.date_created,
      coreNode.date_updated
    );
  }

  getAllNodes(): Node[] {
    const coreNodes = this.repo.getAllNodes() as CoreNode[];
    const tagMap = new Map(this.getAllTags().map((t) => [t.id, t] as const));

    return coreNodes.map((n) => {
      const tags = (n.tag_ids ?? [])
        .map((id) => tagMap.get(id))
        .filter((t): t is Tag => Boolean(t));

      return new Node(
        n.id,
        n.data ?? EMPTY_NODE_DATA,
        tags,
        n.date_created,
        n.date_updated
      );
    });
  }

  tagNode(nodeId: number, tagId: number): void {
    this.repo.tagNode(nodeId, tagId);
  }

  untagNode(nodeId: number, tagId: number): void {
    this.repo.untagNode(nodeId, tagId);
  }

  getNodesWithTag(tagId: number): Node[] {
    const coreNodes = this.repo.getNodesWithTag(tagId) as CoreNode[];
    const tagMap = new Map(this.getAllTags().map((t) => [t.id, t] as const));

    return coreNodes.map((n) => {
      const tags = (n.tag_ids ?? [])
        .map((id) => tagMap.get(id))
        .filter((t): t is Tag => Boolean(t));

      return new Node(
        n.id,
        n.data ?? EMPTY_NODE_DATA,
        tags,
        n.date_created,
        n.date_updated
      );
    });
  }
}

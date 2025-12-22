import type { NodeType } from "./nodeTypes";
import type { Tag } from "./tag";

export class Node {
  id: number;

  deleted: boolean = false;

  data: NodeType;
  tags: Tag[];

  date_created: string;
  date_updated: string;

  constructor(
    id: number,
    data: NodeType,
    tags: Tag[] = [],
    date_created: string,
    date_updated: string
  ) {
    this.id = id;
    this.data = data;
    this.tags = tags;
    this.date_created = date_created;
    this.date_updated = date_updated;
  }
}

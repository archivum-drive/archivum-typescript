export type TagColor = "red" | "blue" | "green" | "yellow" | "purple" | "gray";

export class Tag {
  id: number;

  deleted: boolean = false;

  path: string[];

  color: TagColor;

  constructor(id: number, path: string[], color: TagColor) {
    this.id = id;
    this.path = path;
    this.color = color;
  }
}

import { Repository } from "archivum-core-wasm";

const repo = new Repository();

console.log("Repository node IDs:", repo.listNodeIds());

repo.upsertNode(
	"9b75e340-6afd-4a57-a442-899bcd8417f8",
	"2025-01-01",
	"2025-01-01",
);

console.log("Updated Repository node IDs:", repo.listNodeIds());

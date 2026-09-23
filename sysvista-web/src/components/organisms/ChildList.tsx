import { useState } from "react";
import type { VisibleItem } from "../../lib/projection/types";

interface ChildListProps {
  items: VisibleItem[];
  selectedId: string | null;
  onSelect: (id: string) => void;
  onDescend: (key: string) => void;
}

export function ChildList({ items, selectedId, onSelect, onDescend }: ChildListProps) {
  const [query, setQuery] = useState("");
  const visible = items.filter((item) => item.name.toLowerCase().includes(query.toLowerCase()));
  return <div className="h-full overflow-auto bg-[var(--canvas)] p-4">
    <label className="block text-sm font-medium">Filter children
      <input value={query} onChange={(event) => setQuery(event.target.value)}
        className="mt-1 block w-full rounded border border-[var(--border)] bg-[var(--surface)] p-2" />
    </label>
    <p className="my-3 text-sm">{items.length} children; showing {visible.length}</p>
    <ul className="space-y-1">{visible.map((item) => <li key={item.id}>
      <button type="button" aria-current={selectedId === item.id}
        onClick={() => onSelect(item.id)}
        onDoubleClick={() => item.scopeId && onDescend(`scope:${item.scopeId}`)}
        onKeyDown={(event) => {
          if (event.key === "Enter" && item.scopeId) { event.preventDefault(); onDescend(`scope:${item.scopeId}`); }
        }}
        className="w-full rounded border border-[var(--border)] bg-[var(--surface)] p-2 text-left focus:outline-2 focus:outline-sky-500">
        <span className="mr-2 text-xs uppercase">{item.kind}</span>{item.name}
      </button>
    </li>)}</ul>
  </div>;
}

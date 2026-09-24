import { Search } from "lucide-react";
import { useState, useRef, useEffect } from "react";
import type { EntitySearchHit } from "../../lib/search";

interface SearchBarProps {
  query: string;
  results: EntitySearchHit[];
  onSearch: (query: string) => void;
  onSelect: (hit: EntitySearchHit) => void;
}

export function SearchBar({
  query,
  results,
  onSearch,
  onSelect,
}: SearchBarProps) {
  const [isOpen, setIsOpen] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const handleClickOutside = (e: MouseEvent) => {
      if (
        containerRef.current &&
        !containerRef.current.contains(e.target as HTMLElement)
      ) {
        setIsOpen(false);
      }
    };
    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, []);

  return (
    <div ref={containerRef} className="relative">
      <div className="flex items-center gap-2">
        {/* Search input */}
        <div className="relative">
          <Search className="absolute left-2.5 top-1/2 -translate-y-1/2 h-4 w-4 text-gray-500" />
          <input
            ref={inputRef}
            type="text"
            value={query}
            onChange={(e) => {
              onSearch(e.target.value);
              setIsOpen(true);
            }}
            onFocus={() => query && setIsOpen(true)}
            placeholder="Search symbols..."
            className="w-72 pl-9 pr-3 py-1.5 text-sm bg-[var(--surface)] border border-[var(--border)] rounded-lg placeholder:text-[var(--muted)] focus:outline-none focus:border-sky-500"
          />
        </div>
      </div>

      {/* Search results dropdown */}
      {isOpen && results.length > 0 && (
        <div className="absolute top-full left-0 mt-1 w-96 bg-[var(--surface)] border border-[var(--border)] rounded-lg shadow-xl z-50 max-h-64 overflow-y-auto">
          {results.map((hit) => (
            <button className="block w-full border-b border-[var(--border)] px-3 py-2 text-left hover:bg-[var(--surface-raised)]" key={hit.entityId}
              onClick={() => {
                onSelect(hit);
                setIsOpen(false);
              }}><span className="block text-sm font-medium">{hit.name}</span><span className="block text-xs text-[var(--muted)]">{hit.kind} · {hit.owningScopeId}</span></button>
          ))}
        </div>
      )}
    </div>
  );
}

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
  const [active, setActive] = useState(0);
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
            role="combobox"
            aria-label="Search symbols"
            aria-expanded={isOpen && results.length > 0}
            aria-controls="symbol-search-results"
            aria-autocomplete="list"
            aria-activedescendant={isOpen && results[active] ? `symbol-option-${active}` : undefined}
            value={query}
            onChange={(e) => {
              onSearch(e.target.value);
              setIsOpen(true);
              setActive(0);
            }}
            onKeyDown={(event) => {
              if (event.key === "Escape") { setIsOpen(false); return; }
              if (event.key === "ArrowDown" || event.key === "ArrowUp") {
                event.preventDefault(); setIsOpen(true);
                setActive((value) => (value + (event.key === "ArrowDown" ? 1 : -1) + results.length) % Math.max(results.length, 1));
              }
              if (event.key === "Enter" && isOpen && results[active]) {
                event.preventDefault(); onSelect(results[active]); setIsOpen(false);
              }
            }}
            onFocus={() => query && setIsOpen(true)}
            placeholder="Search symbols..."
            className="w-72 pl-9 pr-3 py-1.5 text-sm bg-[var(--surface)] border border-[var(--border)] rounded-lg placeholder:text-[var(--muted)] focus:outline-none focus:border-sky-500"
          />
        </div>
      </div>

      {/* Search results dropdown */}
      {isOpen && results.length > 0 && (
        <div id="symbol-search-results" role="listbox" className="absolute top-full left-0 mt-1 w-96 bg-[var(--surface)] border border-[var(--border)] rounded-lg shadow-xl z-50 max-h-64 overflow-y-auto">
          {results.map((hit, index) => (
            <button id={`symbol-option-${index}`} role="option" aria-selected={active === index} className="block w-full border-b border-[var(--border)] px-3 py-2 text-left hover:bg-[var(--surface-raised)]" key={hit.entityId}
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

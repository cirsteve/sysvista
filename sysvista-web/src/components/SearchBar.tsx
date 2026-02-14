import { Search } from "lucide-react";
import { useState, useRef, useEffect } from "react";
import type { DetectedComponent, ComponentKind } from "../types/schema";

const KIND_COLORS: Record<string, string> = {
  model: "bg-blue-500",
  service: "bg-green-500",
  transport: "bg-orange-500",
  transform: "bg-purple-500",
};

interface SearchBarProps {
  query: string;
  results: DetectedComponent[];
  activeKinds: Set<ComponentKind>;
  onSearch: (query: string) => void;
  onSelect: (component: DetectedComponent) => void;
  onToggleKind: (kind: ComponentKind) => void;
}

export function SearchBar({
  query,
  results,
  activeKinds,
  onSearch,
  onSelect,
  onToggleKind,
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

  const kinds: ComponentKind[] = ["model", "service", "transport", "transform"];

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
            placeholder="Search components..."
            className="w-64 pl-9 pr-3 py-1.5 text-sm bg-gray-800 border border-gray-700 rounded-lg text-gray-200 placeholder-gray-500 focus:outline-none focus:border-gray-500"
          />
        </div>

        {/* Filter chips */}
        <div className="flex gap-1">
          {kinds.map((kind) => (
            <button
              key={kind}
              onClick={() => onToggleKind(kind)}
              className={`px-2 py-1 text-xs rounded-full border transition-colors ${
                activeKinds.has(kind)
                  ? `${KIND_COLORS[kind]} text-white border-transparent`
                  : "bg-gray-800 text-gray-400 border-gray-700 hover:border-gray-500"
              }`}
            >
              {kind}
            </button>
          ))}
        </div>
      </div>

      {/* Search results dropdown */}
      {isOpen && results.length > 0 && (
        <div className="absolute top-full left-0 mt-1 w-80 bg-gray-800 border border-gray-700 rounded-lg shadow-xl z-50 max-h-64 overflow-y-auto">
          {results.map((comp) => (
            <button
              key={comp.id}
              onClick={() => {
                onSelect(comp);
                setIsOpen(false);
              }}
              className="w-full text-left px-3 py-2 hover:bg-gray-700 transition-colors flex items-center gap-2"
            >
              <span
                className={`w-2 h-2 rounded-full shrink-0 ${KIND_COLORS[comp.kind]}`}
              />
              <div className="min-w-0">
                <div className="text-sm text-gray-200 truncate">
                  {comp.name}
                </div>
                <div className="text-xs text-gray-500 truncate">
                  {comp.source.file}
                </div>
              </div>
            </button>
          ))}
        </div>
      )}
    </div>
  );
}

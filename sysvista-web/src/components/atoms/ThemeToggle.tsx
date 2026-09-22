import { Moon, Sun } from "lucide-react";

interface ThemeToggleProps { theme: "light" | "dark"; onToggle: () => void }

export function ThemeToggle({ theme, onToggle }: ThemeToggleProps) {
  const Icon = theme === "light" ? Moon : Sun;
  return <button type="button" onClick={onToggle} className="rounded border border-[var(--border)] p-2 hover:bg-[var(--surface-raised)]" aria-label={`Use ${theme === "light" ? "dark" : "light"} theme`}><Icon className="h-4 w-4" /></button>;
}

import type { LucideIcon } from "lucide-react";

interface IconButtonProps {
  icon: LucideIcon;
  label?: string;
  badge?: string | number;
  onClick: () => void;
  variant?: "default" | "active" | "close";
  badgeColorClass?: string;
}

export function IconButton({
  icon: Icon,
  label,
  badge,
  onClick,
  variant = "default",
  badgeColorClass,
}: IconButtonProps) {
  const styles = {
    default: "bg-gray-800 hover:bg-gray-700 text-gray-300 border-gray-700",
    active: "bg-cyan-900/60 text-cyan-200 border-cyan-600 hover:bg-cyan-800/60",
    close: "hover:bg-gray-700 text-gray-400 hover:text-gray-200 border-transparent",
  };

  return (
    <button
      onClick={onClick}
      className={`flex items-center gap-1.5 ${variant === "close" ? "p-1" : "px-3 py-1.5 text-xs"} rounded-lg border transition-colors ${styles[variant]}`}
    >
      <Icon className="h-3.5 w-3.5" />
      {label && <span>{label}</span>}
      {badge !== undefined && (
        <span className={`text-xs rounded-full px-1.5 py-0.5 min-w-[1.25rem] text-center ${badgeColorClass ?? "bg-gray-700 text-gray-300"}`}>
          {badge}
        </span>
      )}
    </button>
  );
}

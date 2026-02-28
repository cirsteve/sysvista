import type { LucideIcon } from "lucide-react";

interface SectionHeaderProps {
  icon?: LucideIcon;
  label: string;
  count?: number;
}

export function SectionHeader({ icon: Icon, label, count }: SectionHeaderProps) {
  return (
    <div className="flex items-center gap-1.5 text-xs font-medium text-gray-400 mb-1">
      {Icon && <Icon className="h-3.5 w-3.5" />}
      {label}
      {count !== undefined && <span>({count})</span>}
    </div>
  );
}

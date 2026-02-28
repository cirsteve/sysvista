import type { LucideIcon } from "lucide-react";
import type { ReactNode } from "react";

interface FieldGroupProps {
  label: string;
  icon?: LucideIcon;
  children: ReactNode;
}

export function FieldGroup({ label, icon: Icon, children }: FieldGroupProps) {
  return (
    <div className="mb-4">
      <div className="flex items-center gap-1.5 text-xs font-medium text-gray-400 mb-1">
        {Icon && <Icon className="h-3.5 w-3.5" />}
        {label}
      </div>
      {children}
    </div>
  );
}

export interface ClusterLabelData {
  label: string;
  count: number;
}

export function ClusterLabelNode({ data }: { data: ClusterLabelData }) {
  return (
    <div className="flex items-center gap-2 pointer-events-none select-none">
      <div className="w-1 h-6 rounded-full bg-cyan-400" />
      <span className="text-sm font-semibold text-cyan-200">
        {data.label}
      </span>
      <span className="text-xs text-gray-500">({data.count} components)</span>
    </div>
  );
}

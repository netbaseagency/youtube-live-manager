import { Plus, Tv, Radio } from "lucide-react";
import { Button } from "./ui/button";
import { Badge } from "./ui/badge";

interface HeaderProps {
  instanceId: string;
  onAddClick: () => void;
  streamCount: number;
  liveCount: number;
}

export function Header({ instanceId, onAddClick, streamCount, liveCount }: HeaderProps) {
  return (
    <header className="sticky top-0 z-50 w-full border-b border-slate-200 bg-white">
      <div className="max-w-7xl mx-auto flex h-12 items-center justify-between px-4">
        {/* Logo & Title */}
        <div className="flex items-center gap-2.5">
          <div className="flex h-7 w-7 items-center justify-center rounded-lg bg-gradient-to-br from-red-500 to-red-600 shadow-sm">
            <Tv className="h-3.5 w-3.5 text-white" />
          </div>
          <div>
            <h1 className="text-sm font-semibold text-slate-900">Quản Lý Live YouTube</h1>
            <p className="text-[10px] text-slate-400">
              Phiên: {instanceId.slice(0, 8)}
            </p>
          </div>
        </div>

        {/* Stats & Actions */}
        <div className="flex items-center gap-3">
          {/* Stats */}
          <div className="hidden sm:flex items-center gap-2">
            <Badge variant="outline">
              {streamCount} luồng
            </Badge>
            {liveCount > 0 && (
              <Badge variant="live" className="gap-1">
                <Radio className="h-2.5 w-2.5 animate-pulse-live" />
                {liveCount} LIVE
              </Badge>
            )}
          </div>

          {/* Add Button */}
          <Button onClick={onAddClick} size="sm">
            <Plus className="mr-1 h-3.5 w-3.5" />
            Thêm Luồng
          </Button>
        </div>
      </div>
    </header>
  );
}
